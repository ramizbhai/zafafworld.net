// Logging improvements must not change request behavior, authentication flow, response format, or performance characteristics.
// Logging exists only for observability and debugging.

use crate::errors::ErrorDiagnostic;
use crate::models::user::Claims;
use crate::state::AppState;
use axum::{body::Body, extract::State, http::Request, middleware::Next, response::Response};
use std::time::Instant;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct RequestId(pub String);

/// Extract a product/listing UUID from paths like /api/v1/vendor/products/:id/...
fn extract_product_id(path: &str) -> Option<String> {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    for (i, seg) in segments.iter().enumerate() {
        if *seg == "products" || *seg == "listings" {
            if let Some(id) = segments.get(i + 1) {
                if Uuid::parse_str(id).is_ok() {
                    return Some(id.to_string());
                }
            }
        }
    }
    None
}

/// Extract wizard step number from URL segments like /step-3
fn extract_step(path: &str) -> Option<u32> {
    for seg in path.split('/') {
        if let Some(rest) = seg.strip_prefix("step-") {
            if let Ok(n) = rest.parse::<u32>() {
                return Some(n);
            }
        }
    }
    None
}

pub async fn request_logger(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let start_time = Instant::now();
    let request_id = req
        .headers()
        .get("x-request-id")
        .or_else(|| req.headers().get("X-Request-ID"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    let trace_id = req
        .headers()
        .get("x-trace-id")
        .or_else(|| req.headers().get("X-Trace-ID"))
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let path = req.uri().path().to_string();
    let query = req.uri().query().unwrap_or("").to_string();
    let full_url = if query.is_empty() {
        path.clone()
    } else {
        format!("{}?{}", path, query)
    };
    let method = req.method().to_string();

    // Extract structured context from URL path
    let product_id = extract_product_id(&path);
    let step = extract_step(&path);

    // Extract user ID database-lessly using token verification via the state's JWT secret
    let user_id = crate::middleware::auth::extract_session_token(req.headers(), &path)
        .or_else(|| {
            req.headers()
                .get("Authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .map(|v| v.trim().to_string())
        })
        .and_then(|token| {
            let decoding_key = jsonwebtoken::DecodingKey::from_secret(state.jwt_secret.as_bytes());
            jsonwebtoken::decode::<Claims>(
                &token,
                &decoding_key,
                &jsonwebtoken::Validation::default(),
            )
            .map(|data| data.claims.sub)
            .ok()
        });

    let mut req = req;
    req.extensions_mut().insert(RequestId(request_id.clone()));

    let response = next.run(req).await;
    let duration = start_time.elapsed();
    let status = response.status();

    let diagnostic = response.extensions().get::<ErrorDiagnostic>().cloned();

    if status.is_server_error() || status.is_client_error() {
        if let Some(diag) = diagnostic {
            tracing::error!(
                request_id = %request_id,
                trace_id = ?trace_id,
                user_id = ?user_id,
                product_id = ?product_id,
                step = ?step,
                method = %method,
                url = %full_url,
                status = %status.as_u16(),
                duration_ms = %duration.as_millis(),
                sql_error = ?diag.sql_error,
                filesystem_error = ?diag.filesystem_error,
                stack_trace = ?diag.stack_trace,
                "Request failed"
            );
        } else {
            tracing::error!(
                request_id = %request_id,
                trace_id = ?trace_id,
                user_id = ?user_id,
                product_id = ?product_id,
                step = ?step,
                method = %method,
                url = %full_url,
                status = %status.as_u16(),
                duration_ms = %duration.as_millis(),
                "Request failed without custom diagnostics"
            );
        }
    } else {
        tracing::info!(
            request_id = %request_id,
            trace_id = ?trace_id,
            user_id = ?user_id,
            product_id = ?product_id,
            step = ?step,
            method = %method,
            url = %full_url,
            status = %status.as_u16(),
            duration_ms = %duration.as_millis(),
            "Request completed"
        );
    }

    let mut response = response;
    if let Ok(hdr_val) = axum::http::HeaderValue::from_str(&request_id) {
        response.headers_mut().insert("x-request-id", hdr_val);
    }
    // Echo trace_id back in response header for frontend correlation
    if let Some(ref tid) = trace_id {
        if let Ok(hdr_val) = axum::http::HeaderValue::from_str(tid) {
            response.headers_mut().insert("x-trace-id", hdr_val);
        }
    }

    // Patch request_id AND trace_id into error JSON body for frontend correlation
    if status.is_server_error() || status.is_client_error() {
        let (parts, body) = response.into_parts();
        let bytes = match axum::body::to_bytes(body, 1024 * 1024).await {
            Ok(b) => b,
            Err(_) => return Response::from_parts(parts, Body::empty()),
        };
        if let Ok(mut json_val) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            if let Some(obj) = json_val.as_object_mut() {
                obj.insert("request_id".to_string(), serde_json::json!(request_id));
                if let Some(ref tid) = trace_id {
                    obj.insert("trace_id".to_string(), serde_json::json!(tid));
                }
            }
            let patched = serde_json::to_vec(&json_val).unwrap_or_else(|_| bytes.to_vec());
            return Response::from_parts(parts, Body::from(patched));
        }
        return Response::from_parts(parts, Body::from(bytes));
    }

    response
}
