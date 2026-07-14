use crate::errors::AppError;
use crate::state::AppState;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use uuid::Uuid;
use sqlx::Row;
use serde::{Deserialize, Serialize};
use validator::Validate;
use crate::utils::crypto::hash_password;
use crate::utils::sanitize::{limits, sanitize_str};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/vendor/register", post(register_vendor))
        .route("/subscription/tiers", get(list_subscription_tiers))
}

#[derive(Deserialize, Validate)]
pub struct RegisterVendorRequest {
    #[validate(length(min = 2, max = 150))]
    pub official_name: String,
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: String,
    #[validate(length(min = 10, message = "Phone number is required"))]
    pub phone: String,
    pub city_id: Option<Uuid>,
}

async fn register_vendor(
    State(state): State<AppState>,
    Json(payload): Json<RegisterVendorRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    payload.validate()?;
    tracing::info!("Registering vendor corporate email: {}", payload.email);

    // ── Centralized adaptive password validation ─────────────────────────────
    crate::utils::validation::validate_password(&payload.password)?;

    // 1. Check if user already exists in the Vendor domain
    let existing_user = sqlx::query(
        "SELECT id FROM global_users WHERE email = $1 AND domain_type = 'Vendor'::user_domain_enum",
    )
    .bind(&payload.email)
    .fetch_optional(&state.db)
    .await?;

    if existing_user.is_some() {
        return Err(AppError::BadRequest(
            "Email is already registered".to_string(),
        ));
    }

    // 2. Hash password securely using bcrypt on blocking thread pool
    let hashed_password = hash_password(payload.password.clone()).await?;

    // 3. Generate unique vendor slug based on official_name
    let base_slug = payload
        .official_name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join("-");

    let slug = format!("{}-{}", base_slug, &Uuid::new_v4().to_string()[..8]);

    // 4. Insert user, vendor profile, and vendor record inside an atomic database Transaction
    let mut tx = state.db.begin().await?;

    // Sanitize all free-text corporate fields before multi-table INSERT.
    let clean_official_name = sanitize_str(&payload.official_name, limits::NAME_LONG);
    let clean_email = sanitize_str(&payload.email, limits::EMAIL);
    let clean_phone = sanitize_str(&payload.phone, limits::PHONE);

    let user_id: Uuid = sqlx::query_scalar(
        "INSERT INTO global_users (email, password_hash, domain_type, scopes) VALUES ($1, $2, $3, $4) RETURNING id"
    )
    .bind(&clean_email)
    .bind(&hashed_password)
    .bind(crate::models::user::DomainType::Vendor)
    .bind(vec!["owner".to_string()])
    .fetch_one(&mut *tx)
    .await?;

    // NOTE: vendor_profiles table no longer exists in the clean schema.
    // All brand identity is stored directly in the `vendors` table (see INSERT below).

    // Fetch the 'Free' subscription tier ID to explicitly assign it during vendor registration
    let free_tier_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM subscription_tiers WHERE name = 'Free' LIMIT 1",
    )
    .fetch_optional(&mut *tx)
    .await?;

    // category is intentionally omitted — they belong to Listings, not the Account.
    // status = 'active': vendor accounts are always active; Listings go through moderation.
    sqlx::query(
        "INSERT INTO vendors (
            user_id, name_ar, name_en, slug, status, phone, email, city_id,
            coordinator_name_ar, coordinator_name_en, coordinator_phone, subscription_tier_id
         ) VALUES ($1, $2, $3, $4, 'active', $5, $6, $7, $2, $3, $5, $8)",
    )
    .bind(user_id)
    .bind(&clean_official_name)
    .bind(&clean_official_name)
    .bind(&slug)
    .bind(&clean_phone)
    .bind(&clean_email)
    .bind(payload.city_id)
    .bind(free_tier_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    // 5. Issue JWT immediately — frictionless registration auto-logs the vendor in.
    //    This eliminates the "register → go to login" friction from the old flow.
    let token = crate::routes::identity::auth::generate_jwt(
        &user_id.to_string(),
        &clean_email,
        crate::models::user::DomainType::Vendor,
        vec![crate::models::user::UserScope::Vendor(
            crate::models::user::VendorScope::Owner,
        )],
        &state.jwt_secret,
    )?;

    tracing::info!(
        target: "audit",
        event    = "vendor_registered",
        user_id  = %user_id,
        email    = %clean_email,
        slug     = %slug,
        "New vendor account created and auto-session issued"
    );

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Vendor account created. Welcome to ZafafWorld!",
        "slug": slug,
        "token": token,
        "user": {
            "id": user_id.to_string(),
            "email": clean_email,
            "role": "Vendor",
            "first_name": clean_official_name,
            "last_name": ""
        }
    })))
}




async fn list_subscription_tiers(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let tiers_db = sqlx::query!(
        r#"
        SELECT id, name, priority_score, policy_limits, price, billing_cycle, features
        FROM subscription_tiers
        ORDER BY priority_score ASC
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let tiers: Vec<serde_json::Value> = tiers_db
        .into_iter()
        .map(|r| {
            json!({
                "id": r.id,
                "name": r.name,
                "priority_score": r.priority_score,
                "policy_limits": r.policy_limits,
                "price": r.price.to_string(),
                "billing_cycle": r.billing_cycle,
                "features": r.features
            })
        })
        .collect();

    Ok(Json(json!({ "status": "success", "tiers": tiers })))
}


