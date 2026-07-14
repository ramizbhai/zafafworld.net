use axum::{
    async_trait,
    extract::{ConnectInfo, FromRequestParts},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub struct SecureClientIp(pub IpAddr);

#[derive(Debug)]
pub enum ClientIpError {
    ResolutionFailed,
}

impl IntoResponse for ClientIpError {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({
                "status": "error",
                "message": "Secure client IP resolution failed. Connection source untrusted."
            })),
        )
            .into_response()
    }
}

use axum::extract::FromRef;

#[async_trait]
impl<S> FromRequestParts<S> for SecureClientIp
where
    S: Send + Sync,
    crate::state::AppState: FromRef<S>,
{
    type Rejection = ClientIpError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = crate::state::AppState::from_ref(state);
        let peer_ip = parts
            .extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ConnectInfo(addr)| addr.ip());

        // UDS connections (peer_ip is None) are implicitly trusted.
        // TCP connections must have their physical source validated against TRUSTED_PROXIES.
        let is_trusted = match peer_ip {
            None => true,
            Some(ip) => app_state.trusted_proxies.contains(&ip),
        };

        if is_trusted {
            // 1. CF-Connecting-IP (injected by Cloudflare, trusted if Nginx rewrote $remote_addr)
            if let Some(ip) = parts
                .headers
                .get("cf-connecting-ip")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| IpAddr::from_str(s.trim()).ok())
            {
                return Ok(SecureClientIp(ip));
            }

            // 2. X-Real-IP (set by Nginx proxy)
            if let Some(ip) = parts
                .headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| IpAddr::from_str(s.trim()).ok())
            {
                return Ok(SecureClientIp(ip));
            }

            // 3. X-Forwarded-For (proxy chain)
            if let Some(ip) = parts
                .headers
                .get("x-forwarded-for")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.split(',').next())
                .and_then(|s| IpAddr::from_str(s.trim()).ok())
            {
                return Ok(SecureClientIp(ip));
            }
        }

        // 4. TCP physical peer fallback (for non-proxied requests)
        if let Some(ip) = peer_ip {
            return Ok(SecureClientIp(ip));
        }

        // 5. Fail-Closed: resolve failure
        Err(ClientIpError::ResolutionFailed)
    }
}
