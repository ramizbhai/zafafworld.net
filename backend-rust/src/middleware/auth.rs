use crate::errors::AppError;
use crate::models::user::{Claims, DomainType};
use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts, State},
    http::{request::Parts, Request},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};

pub fn extract_session_token(headers: &axum::http::HeaderMap, path: &str) -> Option<String> {
    headers
        .get("Cookie")
        .and_then(|value| value.to_str().ok())
        .and_then(|cookie_str| {
            // First pass: try path-specific cookie
            let is_client = path.starts_with("/api/v1/client/");
            let is_vendor = path.starts_with("/api/v1/vendor/");
            let is_admin = path.starts_with("/api/v1/admin/");

            let specific_prefix = if is_client {
                Some("zafaf_client_session=")
            } else if is_vendor {
                Some("zafaf_vendor_session=")
            } else if is_admin {
                Some("zafaf_admin_session=")
            } else {
                None
            };

            if let Some(prefix) = specific_prefix {
                if let Some(token) = cookie_str.split(';').find_map(|cookie| {
                    let cookie = cookie.trim();
                    cookie.strip_prefix(prefix).map(|val| val.to_string())
                }) {
                    return Some(token);
                }
            }

            // Fallback: try any of the session cookies in order
            cookie_str.split(';').find_map(|cookie| {
                let cookie = cookie.trim();
                if let Some(val) = cookie.strip_prefix("zafaf_vendor_session=") {
                    return Some(val.to_string());
                }
                if let Some(val) = cookie.strip_prefix("zafaf_client_session=") {
                    return Some(val.to_string());
                }
                if let Some(val) = cookie.strip_prefix("zafaf_admin_session=") {
                    return Some(val.to_string());
                }
                if let Some(val) = cookie.strip_prefix("zafaf_session=") {
                    return Some(val.to_string());
                }
                None
            })
        })
}

#[derive(Clone, Debug)]
pub struct RequireAuth {
    pub user_id: String,
    pub email: String,
    pub role: DomainType,
    pub scopes: Vec<crate::models::user::UserScope>,
}

pub async fn verify_token(
    token: &str,
    jwt_secret: &str,
    db: &sqlx::PgPool,
) -> Result<Claims, AppError> {
    use sqlx::Row;

    let decoding_key = DecodingKey::from_secret(jwt_secret.as_bytes());
    let validation = Validation::default();

    let token_data = decode::<Claims>(token, &decoding_key, &validation).map_err(|err| {
        tracing::warn!("JWT signature decode failure: {}", err);
        AppError::Unauthorized("Invalid or expired authentication token".to_string())
    })?;

    let claims = token_data.claims;

    let user_uuid = uuid::Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::BadRequest("Malformed user ID in session".to_string()))?;

    let row = sqlx::query("SELECT token_valid_after FROM global_users WHERE id = $1")
        .bind(user_uuid)
        .fetch_optional(db)
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let user_row = match row {
        Some(r) => r,
        None => return Err(AppError::Unauthorized("User not found".to_string())),
    };

    let token_valid_after: chrono::DateTime<chrono::Utc> = user_row
        .try_get("token_valid_after")
        .map_err(|err| AppError::Database(err.to_string()))?;

    if claims.iat < token_valid_after.timestamp() {
        return Err(AppError::Unauthorized("Token has been revoked".to_string()));
    }

    Ok(claims)
}

#[async_trait]
impl<S> FromRequestParts<S> for RequireAuth
where
    S: Send + Sync,
    crate::state::AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // First try to get claims from extensions (inserted by the segregation middleware)
        if let Some(claims) = parts.extensions.get::<Claims>() {
            return Ok(RequireAuth {
                user_id: claims.sub.clone(),
                email: claims.email.clone(),
                role: claims.role,
                scopes: claims.scopes.clone(),
            });
        }

        // Fallback to manual extraction and decoding if not in extensions
        let app_state = crate::state::AppState::from_ref(state);
        let jwt_secret = app_state.jwt_secret;

        // Extract Authorization header or Cookie
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|value| value.to_str().ok());

        let mut token = None;
        if let Some(header) = auth_header {
            if let Some(t) = header.strip_prefix("Bearer ") {
                token = Some(t.trim().to_string());
            }
        }
        if token.is_none() {
            token = extract_session_token(&parts.headers, parts.uri.path());
        }

        let token = match token {
            Some(t) => t,
            None => {
                return Err(AppError::Unauthorized(
                    "Missing authentication token".to_string(),
                ))
            }
        };

        let claims = verify_token(&token, &jwt_secret, &app_state.db).await?;

        // Cache in parts extensions for downstream extractors
        parts.extensions.insert(claims.clone());

        Ok(RequireAuth {
            user_id: claims.sub,
            email: claims.email,
            role: claims.role,
            scopes: claims.scopes,
        })
    }
}

// Role-based admin route guard
#[allow(dead_code)]
pub struct RequireAdmin {
    pub user_id: String,
    pub email: String,
    pub role: DomainType,
    pub scopes: Vec<crate::models::user::UserScope>,
}

#[async_trait]
impl<S> FromRequestParts<S> for RequireAdmin
where
    S: Send + Sync,
    crate::state::AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth = RequireAuth::from_request_parts(parts, state).await?;
        if auth.role == DomainType::Admin {
            Ok(RequireAdmin {
                user_id: auth.user_id,
                email: auth.email,
                role: auth.role,
                scopes: auth.scopes,
            })
        } else {
            Err(AppError::Forbidden(
                "Administrator privileges required".to_string(),
            ))
        }
    }
}

#[allow(dead_code)]
pub struct RequireSuperAdmin {
    pub user_id: String,
    pub email: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for RequireSuperAdmin
where
    S: Send + Sync,
    crate::state::AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth = RequireAuth::from_request_parts(parts, state).await?;

        if auth.role == DomainType::Admin
            && auth.scopes.contains(&crate::models::user::UserScope::Admin(
                crate::models::user::AdminScope::SuperAdmin,
            ))
        {
            Ok(RequireSuperAdmin {
                user_id: auth.user_id,
                email: auth.email,
            })
        } else {
            Err(AppError::Forbidden(
                "Super Administrator privileges required".to_string(),
            ))
        }
    }
}

// ─── VENDOR ROLE GUARD ────────────────────────────────────────────────────────
/// Type-safe vendor role guard. Any handler declaring this extractor as a
/// parameter is **compile-enforced** to require DomainType::Vendor — regardless
/// of middleware configuration. Provides the handler-level second layer of
/// defense-in-depth that mirrors RequireAdmin for the vendor surface.
#[allow(dead_code)]
pub struct RequireVendor {
    pub user_id: String,
    pub email: String,
    pub role: DomainType,
    pub scopes: Vec<crate::models::user::UserScope>,
}

#[async_trait]
impl<S> FromRequestParts<S> for RequireVendor
where
    S: Send + Sync,
    crate::state::AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Delegate all cryptographic token validation and DB token_valid_after
        // checks to RequireAuth — avoids duplicating verification logic.
        let auth = RequireAuth::from_request_parts(parts, state).await?;

        // Explicit role assertion gate: short-circuit immediately on any
        // non-Vendor domain. Returns HTTP 403 before handler body is reached.
        if auth.role != DomainType::Vendor {
            return Err(AppError::Forbidden(
                "Vendor account credentials are required to access this resource.".to_string(),
            ));
        }

        Ok(RequireVendor {
            user_id: auth.user_id,
            email: auth.email,
            role: auth.role,
            scopes: auth.scopes,
        })
    }
}

#[allow(dead_code)]
pub struct RequireVendorOwner {
    pub user_id: String,
    pub email: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for RequireVendorOwner
where
    S: Send + Sync,
    crate::state::AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth = RequireAuth::from_request_parts(parts, state).await?;

        if auth.role == DomainType::Vendor
            && auth
                .scopes
                .contains(&crate::models::user::UserScope::Vendor(
                    crate::models::user::VendorScope::Owner,
                ))
        {
            Ok(RequireVendorOwner {
                user_id: auth.user_id,
                email: auth.email,
            })
        } else {
            Err(AppError::Forbidden(
                "Vendor Owner privileges required".to_string(),
            ))
        }
    }
}

// ─── DOMAIN SEGREGATION MIDDLEWARE (PHASE 3) ─────────────────────────────────
pub async fn domain_segregation_middleware(
    State(state): State<crate::state::AppState>,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    let path = req.uri().path();

    if req.method() == axum::http::Method::OPTIONS {
        return Ok(next.run(req).await);
    }

    let is_client = path.starts_with("/api/v1/client/");
    let is_vendor = path.starts_with("/api/v1/vendor/");
    let is_admin = path.starts_with("/api/v1/admin/");

    if is_client || is_vendor || is_admin {
        // Extract Authorization header or Cookie
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok());
        let mut token = None;
        if let Some(header) = auth_header {
            if let Some(t) = header.strip_prefix("Bearer ") {
                token = Some(t.trim().to_string());
            }
        }
        if token.is_none() {
            token = extract_session_token(req.headers(), path);
        }

        let token = match token {
            Some(t) => t,
            None => {
                return Err(AppError::Unauthorized(
                    "Missing authentication token".to_string(),
                ))
            }
        };

        // ── Claims resolution: extensions cache → full verify_token fallback ────
        // rate_limiter_middleware (which runs before us in the stack) decodes the
        // JWT and inserts `Claims` into request extensions. Read from there first
        // to avoid a second cryptographic JWT decode on every authenticated request.
        // If extensions are empty (e.g. in tests or if rate_limit is bypassed),
        // fall back to the full verify_token path which also validates DB state.
        let claims = if let Some(cached) = req.extensions().get::<Claims>() {
            cached.clone()
        } else {
            verify_token(&token, &state.jwt_secret, &state.db).await?
        };

        let user_domain = claims.role;

        // Strict segregation checks
        if is_client && user_domain != DomainType::Client {
            return Err(AppError::Forbidden(
                "Client credentials required".to_string(),
            ));
        }
        if is_vendor && user_domain != DomainType::Vendor {
            return Err(AppError::Forbidden(
                "Vendor credentials required".to_string(),
            ));
        }
        if is_admin && user_domain != DomainType::Admin {
            return Err(AppError::Forbidden("Admin privileges required".to_string()));
        }

        // Refresh in request extensions (ensures downstream extractors always have it,
        // even when we read it from the cache above)
        req.extensions_mut().insert(claims);
    }

    Ok(next.run(req).await)
}

// ─── POSTGRESQL TRANSACTION RLS SESSION GUARD (PHASE 3) ──────────────────────
/// A PostgreSQL transaction with RLS session context already applied.
///
/// The `user_uuid` is stored at construction time (we know it from JWT Claims)
/// so `get_vendor_id()` only performs the `vendors WHERE user_id = ?` lookup,
/// without an extra `SELECT current_setting(...)` round-trip.
pub struct RlsTx {
    pub tx: sqlx::Transaction<'static, sqlx::Postgres>,
    user_uuid: uuid::Uuid,
}

/// Backward-compatible field accessor — existing callers write `rls_tx.0`.
/// We keep this by exposing `tx` publicly; callers that used `rls_tx.0` must
/// be updated to `rls_tx.tx`. This is a mechanical rename.
impl std::ops::Deref for RlsTx {
    type Target = sqlx::Transaction<'static, sqlx::Postgres>;
    fn deref(&self) -> &Self::Target { &self.tx }
}
impl std::ops::DerefMut for RlsTx {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.tx }
}

#[async_trait]
impl<S> FromRequestParts<S> for RlsTx
where
    S: Send + Sync,
    crate::state::AppState: FromRef<S>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 1. Get authenticated claims from extensions, or fallback to manual extraction.
        //    Claims are inserted by rate_limiter_middleware before this extractor runs.
        let claims = match parts.extensions.get::<Claims>() {
            Some(c) => c.clone(),
            None => {
                let auth = RequireAuth::from_request_parts(parts, state).await?;
                Claims {
                    sub: auth.user_id,
                    email: auth.email,
                    role: auth.role,
                    scopes: auth.scopes,
                    exp: 0,
                    iat: 0,
                }
            }
        };

        // 2. Parse UUID once — stored on the struct to avoid re-reading it later.
        let user_uuid = uuid::Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::BadRequest("Malformed user ID in session".to_string()))?;

        // 3. Extract AppState, begin transaction
        let app_state = crate::state::AppState::from_ref(state);
        let mut tx = app_state
            .db
            .begin()
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;

        // 4. Set local (transaction-scoped) RLS session variables.
        //    Two queries are unavoidable here (PostgreSQL does not support
        //    setting multiple config values in one call).
        sqlx::query("SELECT set_config('app.current_user_id', $1, true)")
            .bind(user_uuid.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;

        let role_str = match claims.role {
            DomainType::Client => "client",
            DomainType::Vendor => "vendor",
            DomainType::Admin  => "admin",
        };
        sqlx::query("SELECT set_config('app.current_user_role', $1, true)")
            .bind(role_str)
            .execute(&mut *tx)
            .await
            .map_err(|err| AppError::Database(err.to_string()))?;

        Ok(RlsTx { tx, user_uuid })
    }
}

impl RlsTx {
    /// Return the vendor's `vendors.id` for the authenticated user.
    ///
    /// The `user_uuid` was stored at construction time from the JWT `sub` claim,
    /// so this method does **not** re-read `current_setting('app.current_user_id')`
    /// from PostgreSQL — that eliminates one extra round-trip per vendor handler.
    pub async fn get_vendor_id(&mut self) -> Result<uuid::Uuid, AppError> {
        let vendor_id =
            sqlx::query_scalar::<_, uuid::Uuid>("SELECT id FROM vendors WHERE user_id = $1")
                .bind(self.user_uuid)
                .fetch_optional(&mut *self.tx)
                .await
                .map_err(|err| AppError::Database(err.to_string()))?;

        match vendor_id {
            Some(id) => Ok(id),
            None => Err(AppError::NotFound("Vendor profile not found".to_string())),
        }
    }

    /// Return the user UUID stored at construction time from the JWT claim.
    /// Provided so callers that need the raw user UUID don't have to call `get_vendor_id()`.
    /// Not yet called from production code — published as future API for client/admin handlers.
    #[allow(dead_code)]
    pub fn user_uuid(&self) -> uuid::Uuid {
        self.user_uuid
    }
}
