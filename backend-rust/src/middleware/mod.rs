//! Middleware stack for the ZafafWorld backend.
//!
//! ## Middleware Execution Order (outermost → innermost, request direction)
//!
//! Axum processes `.layer()` calls in **reverse declaration order** — the last
//! `.layer()` in `main.rs` wraps all others and runs first on incoming requests.
//! Reading `main.rs` bottom-up, the effective execution order is:
//!
//! | # | Middleware               | Responsibility                                   |
//! |---|--------------------------|--------------------------------------------------|
//! | 1 | `request_logger`         | logs every request/response with timing          |
//! | 2 | `rate_limiter_middleware` | token-bucket; **decodes JWT, caches `Claims`**   |
//! | 3 | `TimeoutLayer` (120 s)   | hard request cut-off                             |
//! | 4 | `CatchPanicLayer`        | converts panics to 500 responses                 |
//! | 5 | `CorsLayer`              | CORS preflight + header injection                |
//! | 6 | `domain_segregation`     | **reads `Claims` from extensions** set by #2;    |
//! |   |                          | enforces client/vendor/admin domain isolation    |
//! | 7 | `csrf_protection`        | CSRF header validation for mutating requests     |
//! | 8 | `inject_security_headers`| HSTS, X-Frame-Options, etc.                      |
//! |   | → handler                | per-handler extractors: RequireAuth, RlsTx, etc. |
//!
//! ## INVARIANT
//!
//! `rate_limiter_middleware` (#2) **MUST** run before `domain_segregation` (#6).
//!
//! `rate_limiter` decodes the JWT and inserts the `Claims` into request extensions.
//! `domain_segregation` reads `Claims` from extensions instead of decoding the JWT a
//! second time. If this ordering changes, the `Claims` cache is empty and
//! `domain_segregation` falls back to a second full JWT decode, silently regressing
//! performance and potentially breaking the token-expiry enforcement order.
//!
//! When adding new layers in `main.rs`, always re-read this comment and verify the
//! new layer does not violate the ordering invariant above.

pub mod auth;
pub mod csrf;
pub mod idempotency;
pub mod logging;
pub mod rate_limit;
pub mod security;
