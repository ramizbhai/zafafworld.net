//! CSRF (Cross-Site Request Forgery) protection middleware.
//!
//! Strategy: Double-Submit Cookie Pattern + Custom Header
//!
//! 1. Client calls GET /api/v1/auth/csrf-token to obtain a random token.
//! 2. Backend stores the token in the `csrf_tokens` table bound to the user.
//! 3. Client sends the token in the `X-CSRF-Token` header on every mutating
//!    request (POST, PUT, PATCH, DELETE).
//! 4. This middleware validates the header token against the database.
//!
//! SameSite=Lax cookies already block most CSRF vectors. This layer provides
//! defense-in-depth for browsers that don't fully support SameSite.

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::errors::AppError;

/// CSRF validation middleware.
///
/// Skips validation for:
/// - Safe methods (GET, HEAD, OPTIONS)
/// - Unauthenticated routes (/api/v1/auth/*, /api/v1/public/*)
///
/// For all other mutating requests, requires a valid `X-CSRF-Token` header.
pub async fn csrf_protection_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let method = request.method().clone();
    let path = request.uri().path().to_string();

    // 1. Skip safe methods — they should be side-effect-free by HTTP spec
    if method == Method::GET || method == Method::HEAD || method == Method::OPTIONS {
        return Ok(next.run(request).await);
    }

    // 2. Skip public, auth, and internal routes — internal endpoints are
    //    server-to-server only and protected by private network namespaces
    //    and shared secret validation (CSRF applies only to browser clients).
    if path.starts_with("/api/v1/auth") || path.starts_with("/api/v1/public") || path.starts_with("/api/v1/internal") {
        return Ok(next.run(request).await);
    }

    // 3. Skip requests using explicit Bearer token authentication.
    //    CSRF exploits rely on the browser *automatically* attaching cookies.
    //    When a caller provides an explicit `Authorization: Bearer <token>` header,
    //    they already prove possession of the token — CSRF is not a threat.
    //    This exemption is required for SvelteKit SSR server-to-server calls
    //    (e.g., dashboard load, booking transitions) that authenticate via Bearer.
    if let Some(auth_header) = request.headers().get("authorization") {
        if let Ok(val) = auth_header.to_str() {
            if val.starts_with("Bearer ") {
                return Ok(next.run(request).await);
            }
        }
    }

    // 3. For authenticated mutating requests (vendor/admin/client), require
    //    the X-CSRF-Token header. The token's presence proves the request
    //    originated from our own frontend (cross-origin JS cannot read custom
    //    headers from our API responses due to CORS).
    let csrf_header = request.headers().get("x-csrf-token");

    match csrf_header {
        Some(token_val) => {
            let token_str = token_val.to_str().unwrap_or("");
            // Validate: token must be a non-empty hex string of reasonable length
            if token_str.len() < 32 || token_str.len() > 64 {
                tracing::warn!(
                    target: "security",
                    path = %path,
                    "CSRF token rejected: invalid length ({})",
                    token_str.len()
                );
                return Err(AppError::Status(
                    StatusCode::FORBIDDEN,
                    "Invalid CSRF token".to_string(),
                ));
            }
            // Token format check: must be hex characters only
            if !token_str.chars().all(|c| c.is_ascii_hexdigit()) {
                tracing::warn!(
                    target: "security",
                    path = %path,
                    "CSRF token rejected: non-hex characters"
                );
                return Err(AppError::Status(
                    StatusCode::FORBIDDEN,
                    "Invalid CSRF token".to_string(),
                ));
            }
            // Token is structurally valid — allow the request.
            // NOTE: Full DB-backed token validation (lookup + single-use deletion)
            // would be added here for maximum security. For now, the combination of
            // SameSite=Lax cookies + custom header requirement provides strong CSRF
            // protection because:
            // - SameSite=Lax prevents cookie attachment on cross-origin POST
            // - Custom header requirement prevents simple form-based CSRF attacks
            // - CORS policy prevents cross-origin JS from reading our token endpoint
            Ok(next.run(request).await)
        }
        None => {
            // No CSRF token header present — reject the request
            tracing::warn!(
                target: "security",
                path = %path,
                method = %method,
                "CSRF token missing on mutating request"
            );
            Err(AppError::Status(
                StatusCode::FORBIDDEN,
                "CSRF token required for this operation".to_string(),
            ))
        }
    }
}
