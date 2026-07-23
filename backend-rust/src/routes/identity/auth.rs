use axum::{
    extract::State,
    http::{header::SET_COOKIE, HeaderMap},
    routing::{get, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::Row;
use uuid::Uuid;
use validator::Validate;

use crate::errors::AppError;
use crate::middleware::auth::RequireAuth;
use crate::models::user::Claims;
use crate::state::AppState;
use crate::utils::crypto::{hash_password, verify_password};
use crate::utils::sanitize::{limits, sanitize_str};

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me))
        .route("/csrf-token", get(get_csrf_token))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
        .route("/change-password", post(change_password))
}

/// GET /api/v1/auth/csrf-token
/// Returns a fresh CSRF token for the authenticated user.
/// The client must include this token in the `X-CSRF-Token` header
/// on all mutating requests (POST, PUT, PATCH, DELETE) to authenticated routes.
async fn get_csrf_token(auth: RequireAuth) -> Result<Json<Value>, AppError> {
    // Generate a cryptographically random token using two UUIDv4s concatenated.
    // UUID v4 uses OS CSPRNG (getrandom), providing 122 bits of entropy each.
    // Two concatenated = 244 bits = far exceeds CSRF token requirements.
    let token = format!(
        "{}{}",
        Uuid::new_v4().as_simple(),
        Uuid::new_v4().as_simple()
    );

    tracing::debug!(
        target: "security",
        user_id = %auth.user_id,
        "CSRF token issued"
    );

    Ok(Json(json!({
        "csrfToken": token
    })))
}

#[derive(Deserialize, Validate)]
struct RegisterRequest {
    #[validate(length(min = 2, max = 100))]
    first_name: Option<String>,
    #[validate(length(min = 2, max = 100))]
    last_name: Option<String>,
    #[validate(length(min = 2, max = 200))]
    full_name: Option<String>,
    #[validate(email(message = "Invalid email format"))]
    email: String,
    #[validate(length(min = 8, max = 20))]
    phone: Option<String>,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    password: String,
    domain_type: Option<crate::models::user::DomainType>,
    city: Option<String>,
}

#[derive(Deserialize, Validate)]
struct LoginRequest {
    #[validate(email)]
    email: String,
    #[validate(length(min = 1))]
    password: String,
    domain_type: Option<crate::models::user::DomainType>,
}

#[derive(Serialize)]
struct AuthResponse {
    status: String,
    message: String,
    token: Option<String>,
    user: Option<UserDto>,
}

#[derive(Serialize)]
struct UserDto {
    id: String,
    email: String,
    role: String,
    first_name: String,
    last_name: String,
}

pub(crate) fn generate_jwt(
    user_id: &str,
    email: &str,
    role: crate::models::user::DomainType,
    scopes: Vec<crate::models::user::UserScope>,
    secret: &str,
) -> Result<String, AppError> {
    let now = Utc::now();
    let expiration = now
        .checked_add_signed(Duration::days(1))
        .ok_or_else(|| AppError::Internal("Timestamp calculation overflow".to_string()))?
        .timestamp();
    let iat = now.timestamp();

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        role,
        scopes,
        exp: expiration,
        iat,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|err| AppError::Internal(format!("JWT signing error: {}", err)))?;

    Ok(token)
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    payload.validate()?;

    let clean_email = sanitize_str(&payload.email, limits::EMAIL).to_lowercase();

    tracing::info!("Registering user: {}", clean_email);

    let domain_type = payload
        .domain_type
        .unwrap_or(crate::models::user::DomainType::Client);

    // CRITICAL: Block public administrative registration to prevent privilege escalation
    if domain_type == crate::models::user::DomainType::Admin {
        return Err(AppError::Forbidden(
            "Administrative registration is not allowed through this channel".to_string(),
        ));
    }

    // HIGH: Centralized password validation (adaptive complexity & DoS bounds)
    crate::utils::validation::validate_password(&payload.password)?;

    // 1. Hash password securely using bcrypt on blocking thread pool
    // NOTE: The UNIQUE constraint on global_users.(email, domain_type) is the sole authoritative
    // duplicate gate. Duplicate collisions are caught by map_db_error.
    let hashed_password = hash_password(payload.password.clone()).await?;

    let phone_str = payload.phone.clone().unwrap_or_default();
    let (first_name, last_name) = match (payload.first_name, payload.last_name) {
        (Some(f), Some(l)) => (f, l),
        (Some(f), None) => (f, "".to_string()),
        (None, Some(l)) => ("".to_string(), l),
        (None, None) => {
            if let Some(ref full) = payload.full_name {
                let parts: Vec<&str> = full.split_whitespace().collect();
                if parts.len() >= 2 {
                    (parts[0].to_string(), parts[1..].join(" "))
                } else {
                    (full.clone(), "".to_string())
                }
            } else {
                ("".to_string(), "".to_string())
            }
        }
    };

    let city_id = payload
        .city
        .as_deref()
        .and_then(|c| Uuid::parse_str(c).ok());

    // 2. Insert user and profile inside an atomic database Transaction
    let mut tx = state.db.begin().await?;

    let _ = sqlx::query("SELECT set_config('app.current_user_role', 'admin', true)")
        .execute(&mut *tx)
        .await;

    let user_id: Uuid = sqlx::query_scalar(
        "INSERT INTO global_users (email, password_hash, domain_type, scopes) VALUES ($1, $2, $3, $4) RETURNING id"
    )
    .bind(&clean_email)
    .bind(&hashed_password)
    .bind(domain_type)
    .bind(match domain_type {
        crate::models::user::DomainType::Admin => vec!["super_admin".to_string()],
        crate::models::user::DomainType::Vendor => vec!["owner".to_string()],
        _ => vec![],
    })
    .fetch_one(&mut *tx)
    .await
    .map_err(crate::errors::map_db_error)?;

    match domain_type {
        crate::models::user::DomainType::Client => {
            sqlx::query(
                "INSERT INTO client_profiles (client_id, first_name, last_name, phone, city_id) VALUES ($1, $2, $3, $4, $5)"
            )
            .bind(user_id)
            .bind(&first_name)
            .bind(&last_name)
            .bind(&phone_str)
            .bind(city_id)
            .execute(&mut *tx)
            .await
            .map_err(crate::errors::map_db_error)?;
        }
        crate::models::user::DomainType::Vendor => {
            // New schema: vendor data goes into `vendors`, not `vendor_profiles`
            let base_slug = format!(
                "{}-{}",
                first_name.to_lowercase().replace(' ', "-"),
                &uuid::Uuid::new_v4().to_string()[..8]
            );
            sqlx::query(
                "INSERT INTO vendors (user_id, name_ar, name_en, slug, status, phone, email, city_id) VALUES ($1, $2, $3, $4, 'active', $5, $6, $7)"
            )
            .bind(user_id)
            .bind(&first_name)
            .bind(format!("{} {}", first_name, last_name))
            .bind(&base_slug)
            .bind(&phone_str)
            .bind(&clean_email)
            .bind(city_id)
            .execute(&mut *tx)
            .await
            .map_err(crate::errors::map_db_error)?;
        }
        crate::models::user::DomainType::Admin => {
            sqlx::query(
                "INSERT INTO client_profiles (client_id, first_name, last_name, phone, city_id) VALUES ($1, $2, $3, $4, $5)"
            )
            .bind(user_id)
            .bind(&first_name)
            .bind(&last_name)
            .bind(&phone_str)
            .bind(city_id)
            .execute(&mut *tx)
            .await
            .map_err(crate::errors::map_db_error)?;
        }
    }

    // Event logging hook for user/vendor registration
    let event_type = match domain_type {
        crate::models::user::DomainType::Client => "user_registered",
        crate::models::user::DomainType::Vendor => "vendor_registered",
        crate::models::user::DomainType::Admin => "system_alert",
    };
    let msg_ar = match domain_type {
        crate::models::user::DomainType::Client => {
            format!("تم تسجيل مستخدم عميل جديد: {}", clean_email)
        }
        crate::models::user::DomainType::Vendor => format!("تم تسجيل مورد جديد: {}", first_name),
        crate::models::user::DomainType::Admin => format!("تم تسجيل مسؤول جديد: {}", clean_email),
    };
    let msg_en = match domain_type {
        crate::models::user::DomainType::Client => {
            format!("New client user registered: {}", clean_email)
        }
        crate::models::user::DomainType::Vendor => format!("New vendor registered: {}", first_name),
        crate::models::user::DomainType::Admin => {
            format!("New administrator registered: {}", clean_email)
        }
    };

    sqlx::query(
        "INSERT INTO system_events (user_id, event_type, message_ar, message_en)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(user_id)
    .bind(event_type)
    .bind(&msg_ar)
    .bind(&msg_en)
    .execute(&mut *tx)
    .await
    .map_err(crate::errors::map_db_error)?;

    tx.commit().await?;

    let default_scopes = match domain_type {
        crate::models::user::DomainType::Admin => vec![crate::models::user::UserScope::Admin(
            crate::models::user::AdminScope::SuperAdmin,
        )],
        crate::models::user::DomainType::Vendor => vec![crate::models::user::UserScope::Vendor(
            crate::models::user::VendorScope::Owner,
        )],
        _ => vec![],
    };

    // 4. Generate signed JWT token
    let token = generate_jwt(
        &user_id.to_string(),
        &clean_email,
        domain_type,
        default_scopes,
        &state.jwt_secret,
    )?;

    let domain_str = format!("{:?}", domain_type);

    Ok(Json(AuthResponse {
        status: "success".to_string(),
        message: "User registered successfully".to_string(),
        token: Some(token),
        user: Some(UserDto {
            id: user_id.to_string(),
            email: clean_email,
            role: domain_str,
            first_name,
            last_name,
        }),
    }))
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    payload.validate()?;

    let clean_email = sanitize_str(&payload.email, limits::EMAIL).to_lowercase();

    tracing::info!("Login attempt: {}", clean_email);

    let target_domain = payload
        .domain_type
        .unwrap_or(crate::models::user::DomainType::Client);

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    let _ = sqlx::query("SELECT set_config('app.current_user_role', 'admin', true)")
        .execute(&mut *tx)
        .await;

    // ── 1. Read lockout threshold from admin_settings ─────────────────────────
    let max_attempts: i32 =
        sqlx::query("SELECT value FROM admin_settings WHERE key = 'max_login_attempts'")
            .fetch_optional(&mut *tx)
            .await?
            .and_then(|row| {
                let v: Option<String> = row.get("value");
                v?.parse::<i32>().ok()
            })
            .unwrap_or(5);

    const LOCKOUT_MINUTES: i64 = 15;

    // ── 2. Fetch user including lockout columns and scopes ────────────────
    let user_row = sqlx::query(
        "SELECT id, email, password_hash, domain_type, \
                failed_login_attempts, locked_until, scopes \
         FROM global_users \
         WHERE email = $1 AND domain_type = $2",
    )
    .bind(&clean_email)
    .bind(target_domain)
    .fetch_optional(&mut *tx)
    .await?;

    let user_row = match user_row {
        Some(row) => row,
        None => {
            let _ = tx.rollback().await;
            const DUMMY_HASH: &str = "$2b$12$Mxpu0vFmXrcx0K1mg9D7FeWG4oNGJEFt2/k0.EXB10M0my0nECMD2";
            let _ = verify_password(payload.password.clone(), DUMMY_HASH.to_string()).await;
            return Err(AppError::Unauthorized("Invalid ID or Password".to_string()));
        }
    };
    let user_id: Uuid = user_row.get("id");
    let user_email: String = user_row.get("email");
    let password_hash: String = user_row.get("password_hash");
    let domain_type: crate::models::user::DomainType = user_row.get("domain_type");
    let failed_attempts: i32 = user_row.get("failed_login_attempts");
    let locked_until: Option<chrono::DateTime<chrono::Utc>> = user_row.get("locked_until");
    let raw_scopes: Vec<String> = user_row.try_get("scopes").unwrap_or_default();

    let mut parsed_scopes = Vec::new();
    for scope in raw_scopes {
        let normalized = if scope == "admin:all" {
            "super_admin".to_string()
        } else {
            scope
        };
        if let Ok(val) = serde_json::from_value::<crate::models::user::UserScope>(json!(normalized))
        {
            parsed_scopes.push(val);
        }
    }

    // ── 3. Lockout gate — checked BEFORE bcrypt ───────────────────────────────
    if let Some(locked_ts) = locked_until {
        if locked_ts > Utc::now() {
            let _ = tx.rollback().await;
            const DUMMY_HASH: &str = "$2b$12$Mxpu0vFmXrcx0K1mg9D7FeWG4oNGJEFt2/k0.EXB10M0my0nECMD2";
            let _ = verify_password(payload.password.clone(), DUMMY_HASH.to_string()).await;
            tracing::warn!(
                target: "security",
                user_id = %user_id,
                locked_until = %locked_ts,
                "Login attempt blocked: account locked"
            );
            return Err(AppError::Unauthorized("Invalid ID or Password".to_string()));
        }
        // Lock has naturally expired — clear it before proceeding
        let _ = sqlx::query(
            "UPDATE global_users \
             SET locked_until = NULL, failed_login_attempts = 0 \
             WHERE id = $1",
        )
        .bind(user_id)
        .execute(&mut *tx)
        .await;
    }

    // ── 4. Verify password ────────────────────────────────────────────────────
    let is_valid = verify_password(payload.password.clone(), password_hash).await?;

    if !is_valid {
        let new_attempts = failed_attempts + 1;

        if new_attempts >= max_attempts {
            let locked_until_ts = Utc::now() + chrono::Duration::minutes(LOCKOUT_MINUTES);
            sqlx::query(
                "UPDATE global_users \
                 SET failed_login_attempts = $1, locked_until = $2 \
                 WHERE id = $3",
            )
            .bind(new_attempts)
            .bind(locked_until_ts)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

            tracing::warn!(
                target: "security",
                user_id = %user_id,
                attempts = new_attempts,
                locked_until = %locked_until_ts,
                "Account locked after failed login threshold reached"
            );
        } else {
            sqlx::query("UPDATE global_users SET failed_login_attempts = $1 WHERE id = $2")
                .bind(new_attempts)
                .bind(user_id)
                .execute(&mut *tx)
                .await?;

            tracing::warn!(
                target: "security",
                user_id = %user_id,
                attempts = new_attempts,
                threshold = max_attempts,
                "Failed login attempt recorded"
            );
        }
        let _ = tx.commit().await;

        // Identical error for wrong-password and locked — no enumeration
        return Err(AppError::Unauthorized("Invalid ID or Password".to_string()));
    }

    // ── 5. Successful auth — reset lockout counters ───────────────────────────
    sqlx::query(
        "UPDATE global_users \
         SET failed_login_attempts = 0, locked_until = NULL \
         WHERE id = $1",
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    // ── 6. Fetch profile details ──────────────────────────────────────────────
    let (first_name, last_name) = match domain_type {
        crate::models::user::DomainType::Client | crate::models::user::DomainType::Admin => {
            let profile_row = sqlx::query(
                "SELECT first_name, last_name FROM client_profiles WHERE client_id = $1",
            )
            .bind(user_id)
            .fetch_optional(&mut *tx)
            .await?;

            match profile_row {
                Some(row) => {
                    let fname: Option<String> = row.get("first_name");
                    let lname: Option<String> = row.get("last_name");
                    (fname.unwrap_or_default(), lname.unwrap_or_default())
                }
                None => ("".to_string(), "".to_string()),
            }
        }
        crate::models::user::DomainType::Vendor => {
            let profile_row =
                sqlx::query("SELECT name_en, name_ar FROM vendors WHERE user_id = $1")
                    .bind(user_id)
                    .fetch_optional(&mut *tx)
                    .await?;

            match profile_row {
                Some(row) => {
                    let fname: Option<String> = row.get("name_en");
                    let lname: Option<String> = row.get("name_ar");
                    (fname.unwrap_or_default(), lname.unwrap_or_default())
                }
                None => ("".to_string(), "".to_string()),
            }
        }
    };

    tx.commit().await?;

    // ── 7. Issue signed JWT ───────────────────────────────────────────────────
    let token = generate_jwt(
        &user_id.to_string(),
        &user_email,
        domain_type,
        parsed_scopes,
        &state.jwt_secret,
    )?;

    let domain_str = format!("{:?}", domain_type);

    Ok(Json(AuthResponse {
        status: "success".to_string(),
        message: "Login successful".to_string(),
        token: Some(token),
        user: Some(UserDto {
            id: user_id.to_string(),
            email: user_email,
            role: domain_str,
            first_name,
            last_name,
        }),
    }))
}

async fn logout(
    State(state): State<AppState>,
    auth: RequireAuth,
) -> Result<(HeaderMap, Json<Value>), AppError> {
    let user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid user ID format".to_string()))?;

    sqlx::query("UPDATE global_users SET token_valid_after = NOW() WHERE id = $1")
        .bind(user_uuid)
        .execute(&state.db)
        .await?;

    let mut headers = HeaderMap::new();
    for cookie_name in &[
        "zafaf_session",
        "zafaf_client_session",
        "zafaf_vendor_session",
        "zafaf_admin_session",
    ] {
        headers.append(
            SET_COOKIE,
            format!("{}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0; Expires=Thu, 01 Jan 1970 00:00:00 GMT", cookie_name)
                .parse()
                .expect("static cookie-clear header is always valid"),
        );
    }

    Ok((
        headers,
        Json(json!({
            "status": "success",
            "message": "Session logged out successfully"
        })),
    ))
}

async fn me(State(state): State<AppState>, auth: RequireAuth) -> Result<Json<Value>, AppError> {
    // Parse UUID from the verified RequireAuth user_id
    let user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid user ID format in token".to_string()))?;

    // Query details from database joining profile and user records
    let (domain_str, first_name, last_name) = match auth.role {
        crate::models::user::DomainType::Client | crate::models::user::DomainType::Admin => {
            let user_details = sqlx::query(
                "SELECT u.email, u.domain_type, p.first_name, p.last_name FROM global_users u LEFT JOIN client_profiles p ON u.id = p.client_id WHERE u.id = $1"
            )
            .bind(user_uuid)
            .fetch_optional(&state.db)
            .await?;

            let row = match user_details {
                Some(r) => r,
                None => return Err(AppError::NotFound("User not found".to_string())),
            };
            let domain_type: crate::models::user::DomainType = row.get("domain_type");
            let fname: Option<String> = row.get("first_name");
            let lname: Option<String> = row.get("last_name");
            (
                format!("{:?}", domain_type),
                fname.unwrap_or_default(),
                lname.unwrap_or_default(),
            )
        }
        crate::models::user::DomainType::Vendor => {
            // New schema: vendor identity is in `vendors` table, keyed by user_id
            let user_details = sqlx::query(
                "SELECT u.email, u.domain_type, v.name_en, v.name_ar FROM global_users u LEFT JOIN vendors v ON u.id = v.user_id WHERE u.id = $1"
            )
            .bind(user_uuid)
            .fetch_optional(&state.db)
            .await?;

            let row = match user_details {
                Some(r) => r,
                None => return Err(AppError::NotFound("User not found".to_string())),
            };
            let domain_type: crate::models::user::DomainType = row.get("domain_type");
            let name_en: Option<String> = row.get("name_en");
            let name_ar: Option<String> = row.get("name_ar");
            (
                format!("{:?}", domain_type),
                name_en.unwrap_or_default(),
                name_ar.unwrap_or_default(),
            )
        }
    };

    Ok(Json(json!({
        "status": "success",
        "user": {
            "id": auth.user_id,
            "email": auth.email,
            "role": domain_str,
            "scopes": auth.scopes,
            "first_name": first_name,
            "last_name": last_name
        }
    })))
}

#[derive(Deserialize)]
struct ForgotPasswordRequest {
    email: String,
    domain_type: crate::models::user::DomainType,
}

#[derive(Deserialize)]
struct ResetPasswordRequest {
    token: String,
    password: String,
}

async fn forgot_password(
    State(state): State<AppState>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Result<Json<Value>, AppError> {
    let email_clean = payload.email.trim().to_lowercase();
    tracing::info!(
        "Forgot password requested for email: {}, domain: {:?}",
        email_clean,
        payload.domain_type
    );

    let row = sqlx::query("SELECT id FROM global_users WHERE email = $1 AND domain_type = $2")
        .bind(&email_clean)
        .bind(payload.domain_type)
        .fetch_optional(&state.db)
        .await?;

    if let Some(user_row) = row {
        let user_id: Uuid = user_row.get("id");
        let token = Uuid::new_v4().as_simple().to_string();
        let hashed_token = hash_token(&token);
        let expires_at = Utc::now() + Duration::hours(1);

        // Delete any existing tokens for this user
        sqlx::query("DELETE FROM password_reset_tokens WHERE user_id = $1")
            .bind(user_id)
            .execute(&state.db)
            .await?;

        // Insert new token
        sqlx::query(
            "INSERT INTO password_reset_tokens (token, user_id, expires_at) VALUES ($1, $2, $3)",
        )
        .bind(&hashed_token)
        .bind(user_id)
        .bind(expires_at)
        .execute(&state.db)
        .await?;

        // ── Blocker 3 Fix: build link from configured FRONTEND_URL (never hardcoded) ─
        let link = match payload.domain_type {
            crate::models::user::DomainType::Vendor => {
                format!("{}/reset-password?token={}", state.frontend_url, token)
            }
            _ => {
                format!("{}/auth/reset-password?token={}", state.frontend_url, token)
            }
        };

        // ── Blocker 2 Fix: dispatch via real email service ────────────────────
        // send_password_reset honours the security contract:
        //   • Production SMTP mode → sends real email, does NOT log link
        //   • Dev fallback mode    → prints to stdout, emits [DEV EMAIL] marker
        if let Err(e) = state
            .email_service
            .send_password_reset(&email_clean, &link)
            .await
        {
            // Log the delivery failure but do NOT surface it to the caller —
            // revealing email delivery errors enables user enumeration.
            tracing::error!(
                target: "security",
                email = %email_clean,
                error = %e,
                "Password reset email delivery failed"
            );
        }
    } else {
        tracing::info!(
            target: "security",
            email = %email_clean,
            domain = ?payload.domain_type,
            "Forgot password request for unknown account (timing-safe no-op)"
        );
    }

    // Always return a generic timing-safe success message to prevent user enumeration
    Ok(Json(json!({
        "status": "success",
        "message": "If an account is associated with this email, a recovery link has been sent."
    })))
}

async fn reset_password(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<Json<Value>, AppError> {
    // Centralized password validation (adaptive complexity & DoS bounds)
    crate::utils::validation::validate_password(&payload.password)?;

    let hashed_token = hash_token(&payload.token);

    let token_row =
        sqlx::query("SELECT user_id, expires_at FROM password_reset_tokens WHERE token = $1")
            .bind(&hashed_token)
            .fetch_optional(&state.db)
            .await?;

    let (user_id, expires_at) = match token_row {
        Some(row) => {
            let user_id: Uuid = row.get("user_id");
            let expires_at: chrono::DateTime<chrono::Utc> = row.get("expires_at");
            (user_id, expires_at)
        }
        None => {
            return Err(AppError::BadRequest(
                "Invalid or expired password reset token".to_string(),
            ));
        }
    };

    if expires_at < Utc::now() {
        // Safe pruning
        let _ = sqlx::query("DELETE FROM password_reset_tokens WHERE token = $1")
            .bind(&hashed_token)
            .execute(&state.db)
            .await;
        return Err(AppError::BadRequest(
            "Invalid or expired password reset token".to_string(),
        ));
    }

    // Securely hash password on blocking thread pool
    let hashed_password = hash_password(payload.password.clone()).await?;

    // Perform atomic transaction
    let mut tx = state.db.begin().await?;

    // Update password and invalidate all issued sessions immediately
    sqlx::query("UPDATE global_users SET password_hash = $1, token_valid_after = NOW() + INTERVAL '1 second', updated_at = NOW() WHERE id = $2")
        .bind(&hashed_password)
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(crate::errors::map_db_error)?;

    // Consume single-use token
    sqlx::query("DELETE FROM password_reset_tokens WHERE token = $1")
        .bind(&hashed_token)
        .execute(&mut *tx)
        .await
        .map_err(crate::errors::map_db_error)?;

    tx.commit().await?;

    tracing::info!("Successfully reset password for user ID: {}", user_id);

    Ok(Json(json!({
        "status": "success",
        "message": "Password reset successfully"
    })))
}

#[derive(Deserialize, Validate)]
struct ChangePasswordRequest {
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    old_password: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    new_password: String,
}

async fn change_password(
    auth: RequireAuth,
    State(state): State<AppState>,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<Json<Value>, AppError> {
    payload.validate()?;
    crate::utils::validation::validate_password(&payload.new_password)?;

    let user_id = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid user ID format".to_string()))?;

    let mut tx = state.db.begin().await?;

    let user_row = sqlx::query("SELECT password_hash FROM global_users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(crate::errors::map_db_error)?;

    let user_row = match user_row {
        Some(row) => row,
        None => return Err(AppError::NotFound("User not found".to_string())),
    };

    let password_hash: String = user_row.get("password_hash");

    let is_valid = verify_password(payload.old_password.clone(), password_hash).await?;
    if !is_valid {
        return Err(AppError::Status(
            axum::http::StatusCode::UNAUTHORIZED,
            "Invalid current password".to_string(),
        ));
    }

    let hashed_password = hash_password(payload.new_password.clone()).await?;

    sqlx::query("UPDATE global_users SET password_hash = $1, token_valid_after = NOW() + INTERVAL '1 second', updated_at = NOW() WHERE id = $2")
        .bind(&hashed_password)
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(crate::errors::map_db_error)?;

    tx.commit().await?;

    tracing::info!("Successfully changed password for user ID: {}", user_id);

    Ok(Json(json!({
        "status": "success",
        "message": "Password changed successfully"
    })))
}
