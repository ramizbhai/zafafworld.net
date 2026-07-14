use crate::errors::AppError;
use crate::state::AppState;
use axum::{
    body::{Body, Bytes},
    extract::State,
    http::{HeaderMap, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use dashmap::mapref::entry::Entry;
use tokio::time::Instant;

#[derive(Clone, Debug)]
pub enum IdempotentState {
    Started {
        created_at: Instant,
    },
    Completed {
        status: StatusCode,
        headers: HeaderMap,
        body: Bytes,
        created_at: Instant,
    },
}

pub async fn idempotent_gate_middleware(
    headers: HeaderMap,
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let key = match headers.get("Idempotency-Key") {
        Some(val) => match val.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                return Err(AppError::Status(
                    StatusCode::BAD_REQUEST,
                    "Invalid Idempotency-Key header encoding.".to_string(),
                ))
            }
        },
        None => return Ok(next.run(req).await),
    };

    // Lock-free check on DashMap / Atomic operation inside a block
    let cached_res = {
        let entry = state.idempotency_store.entry(key.clone());
        match entry {
            Entry::Occupied(occupied) => match occupied.get() {
                IdempotentState::Started { .. } => {
                    return Err(AppError::Status(
                        StatusCode::CONFLICT,
                        "A request with this idempotency key is already in progress.".to_string(),
                    ));
                }
                IdempotentState::Completed {
                    status,
                    headers: cached_headers,
                    body,
                    ..
                } => {
                    let mut response = Response::new(Body::from(body.clone()));
                    *response.status_mut() = *status;
                    *response.headers_mut() = cached_headers.clone();
                    Some(response)
                }
            },
            Entry::Vacant(vacant) => {
                vacant.insert(IdempotentState::Started {
                    created_at: Instant::now(),
                });
                None
            }
        }
    };

    if let Some(res) = cached_res {
        return Ok(res);
    }

    // Execute the downstream transaction pipeline
    let response = next.run(req).await;
    let status = response.status();

    if status.is_success() || status.is_redirection() {
        let (parts, body) = response.into_parts();

        // Use axum::body::to_bytes to read the body up to a limit of 32 KB
        let bytes = match axum::body::to_bytes(body, 32 * 1024).await {
            Ok(b) => b,
            Err(err) => {
                state.idempotency_store.remove(&key);
                return Err(AppError::Internal(format!(
                    "Failed to parse response body bytes: {}",
                    err
                )));
            }
        };

        state.idempotency_store.insert(
            key,
            IdempotentState::Completed {
                status: parts.status,
                headers: parts.headers.clone(),
                body: bytes.clone(),
                created_at: Instant::now(),
            },
        );

        let mut reconstructed = Response::new(Body::from(bytes));
        *reconstructed.status_mut() = parts.status;
        *reconstructed.headers_mut() = parts.headers;
        Ok(reconstructed)
    } else {
        // If execution fails, instantly drop and remove the state token from the map to enable clean retry runs
        state.idempotency_store.remove(&key);
        Ok(response)
    }
}
