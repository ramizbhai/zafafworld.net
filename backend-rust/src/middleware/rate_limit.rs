use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::time::Instant;

use crate::errors::AppError;
use crate::state::AppState;

#[derive(Clone, Copy, Debug)]
pub struct TokenBucket {
    pub tokens: f64,
    pub last_refreshed: Instant,
}

/// Custom high-performance, in-memory rate-limiter middleware.
/// Rejects high-velocity brute force or DDoS attempts before serialization or database overhead occurs.
/// Works in both UDS mode (ConnectInfo is None) and TCP mode (ConnectInfo is Some).
pub async fn rate_limiter_middleware(
    State(state): State<AppState>,
    crate::utils::ip::SecureClientIp(client_ip): crate::utils::ip::SecureClientIp,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    // Copy path to an owned String so the immutable borrow on `request` is
    // released before we call `request.extensions_mut()` below.
    let path = request.uri().path().to_owned();
    let path = path.as_str();

    // Determine the rate limiting tier
    let (max_capacity, refill_rate) = if (path.starts_with("/api/v1/auth")
        && path != "/api/v1/auth/me")
        || path == "/api/v1/public/vendor/register"
    {
        (5.0, 5.0 / 60.0) // Strict Tier: max 5 tokens, refills 5 per 60s
    } else if path.starts_with("/api/v1/public") {
        (100.0, 100.0 / 60.0) // Public Tier: max 100 tokens, refills 100 per 60s
    } else {
        (200.0, 200.0 / 60.0) // Authenticated Tier
    };

    // Rate-limit key: use the authenticated user's subject UUID when available
    // (more accurate than IP for shared-NAT clients), falling back to client IP.
    // As a side effect, cache the decoded Claims in request extensions so that
    // domain_segregation_middleware (which runs after this layer) can read the
    // Claims directly from extensions instead of performing a second JWT decode.
    // See middleware/mod.rs for the ordering invariant this relies on.
    let mut key = client_ip.to_string();

    if let Some(token) = crate::middleware::auth::extract_session_token(request.headers(), path) {
        if let Ok(token_data) = jsonwebtoken::decode::<crate::models::user::Claims>(
            &token,
            &jsonwebtoken::DecodingKey::from_secret(state.jwt_secret.as_bytes()),
            &jsonwebtoken::Validation::default(),
        ) {
            key = token_data.claims.sub.clone();
            // Cache for domain_segregation (runs later in the middleware stack)
            request.extensions_mut().insert(token_data.claims);
        }
    } else if let Some(auth_header) = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
    {
        if let Some(t) = auth_header.strip_prefix("Bearer ") {
            if let Ok(token_data) = jsonwebtoken::decode::<crate::models::user::Claims>(
                t.trim(),
                &jsonwebtoken::DecodingKey::from_secret(state.jwt_secret.as_bytes()),
                &jsonwebtoken::Validation::default(),
            ) {
                key = token_data.claims.sub.clone();
                // Cache for domain_segregation (runs later in the middleware stack)
                request.extensions_mut().insert(token_data.claims);
            }
        }
    }

    if key == "127.0.0.1" || client_ip.is_loopback() || state.config.app_environment == "development" || state.config.app_environment == "test" {
        return Ok(next.run(request).await);
    }

    let now = Instant::now();

    // Access the concurrent sharded map with a write entry lock
    let mut entry = state
        .rate_limit_store
        .entry(key.clone())
        .or_insert(TokenBucket {
            tokens: max_capacity,
            last_refreshed: now,
        });

    let elapsed = now.duration_since(entry.last_refreshed).as_secs_f64();

    // Replenishment calculation
    let mut current_tokens = entry.tokens + elapsed * refill_rate;

    // Dynamic ceiling clamping based on the active path's security tier
    if current_tokens > max_capacity {
        current_tokens = max_capacity;
    }

    if current_tokens < 1.0 {
        // Log the security event
        tracing::warn!(
            target: "security",
            client_key = %key,
            request_path = %path,
            available_tokens = %current_tokens,
            "Rate limit triggered - blocking inbound request"
        );

        // Zero-allocation early exit: short circuit before upstream routing
        return Err(AppError::Status(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many requests. Please try again later.".to_string(),
        ));
    }

    // Decrement token allocation and update internal tracking states
    entry.tokens = current_tokens - 1.0;
    entry.last_refreshed = now;

    // CRITICAL: Release DashMap write lock by dropping the entry guard explicitly
    // before awaiting the next layer. DashMap locks are !Send and holding them across
    // the await boundary would make the returned Future !Send, breaking Axum's compile contracts.
    drop(entry);

    Ok(next.run(request).await)
}
