use crate::errors::AppError;
use crate::middleware::auth::{RequireVendor, RlsTx};
use crate::state::AppState;
use crate::utils::sanitize::{limits, sanitize_opt, sanitize_str};
use axum::{
    extract::State,
    routing::{get, put},
    Json, Router,
};
use serde_json::{json, Value};
use uuid::Uuid;
use sqlx::Row;
use serde::Deserialize;
use validator::Validate;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/analytics", get(get_vendor_analytics))
        .route("/profile", put(update_vendor_profile))
        .route(
            "/subscription-requests",
            get(list_subscription_requests).post(create_subscription_request),
        )
}


#[derive(Deserialize, Validate)]
pub struct UpdateProfileRequest {
    // ── Brand-level corporate identity fields ─────────────────────────────────
    #[validate(length(min = 2, max = 150))]
    pub name_ar: String,
    #[validate(length(min = 2, max = 150))]
    pub name_en: String,
    pub description_ar: Option<String>,
    pub description_en: Option<String>,
    pub address_ar: Option<String>,
    pub address_en: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub crm_venue_id: Option<String>,
    pub star_rating: Option<rust_decimal::Decimal>,
    pub website: Option<String>,
    pub maps_url: Option<String>,
    pub video_url_1: Option<String>,
    pub instagram_url: Option<String>,
    pub cr_number: Option<String>,
    pub coordinator_name_ar: Option<String>,
    pub coordinator_name_en: Option<String>,
    pub coordinator_phone: Option<String>,
    pub coordinator_whatsapp: Option<String>,
    pub version: i32,

    // City (UUID string)
    pub city_id: Option<String>,

    // Phase 3 Saudi Filters
    pub has_partition: Option<bool>,
    pub capacity_min: Option<i32>,
    pub capacity_max: Option<i32>,
    pub amenities: Option<Vec<String>>,
}


async fn get_vendor_analytics(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    _auth: RequireVendor,
    mut rls_tx: RlsTx,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    let repo = crate::repositories::vendor_repository::PgVendorRepository::new(state.db.clone());
    let chart_data = repo.get_vendor_analytics_chart(&mut rls_tx.tx, vendor_id).await?;
    let top_products = repo.get_top_performing_products(&mut rls_tx.tx, vendor_id).await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "data": {
            "monthlyAnalyticsChart": chart_data,
            "topPerformingProducts": top_products
        }
    })))
}

async fn update_vendor_profile(
    State(state): State<AppState>,
    _auth: RequireVendor,
    mut rls_tx: RlsTx,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<Value>, AppError> {
    if payload.name_ar.trim().is_empty() || payload.name_en.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Vendor name in Arabic and English is required".to_string(),
        ));
    }

    let vendor_id = rls_tx.get_vendor_id().await?;

    // Sanitize all free-text profile fields before the UPDATE write.
    let clean_name_ar = sanitize_str(&payload.name_ar, limits::NAME_LONG);
    let clean_name_en = sanitize_str(&payload.name_en, limits::NAME_LONG);
    let clean_description_ar = sanitize_opt(&payload.description_ar, limits::DESCRIPTION);
    let clean_description_en = sanitize_opt(&payload.description_en, limits::DESCRIPTION);
    let clean_address_ar = sanitize_opt(&payload.address_ar, limits::DESCRIPTION);
    let clean_address_en = sanitize_opt(&payload.address_en, limits::DESCRIPTION);
    let clean_phone = sanitize_opt(&payload.phone, limits::PHONE);
    let clean_email = sanitize_opt(&payload.email, limits::EMAIL);
    let clean_crm_venue_id = sanitize_opt(&payload.crm_venue_id, limits::NAME_SHORT);
    let clean_website = sanitize_opt(&payload.website, limits::NAME_LONG);
    let clean_maps_url = sanitize_opt(&payload.maps_url, limits::DESCRIPTION);
    let clean_video_url_1 = sanitize_opt(&payload.video_url_1, limits::NAME_LONG);
    let clean_instagram_url = sanitize_opt(&payload.instagram_url, limits::NAME_LONG);
    let clean_cr_number = sanitize_opt(&payload.cr_number, limits::NAME_SHORT);
    let clean_coord_name_ar = sanitize_opt(&payload.coordinator_name_ar, limits::NAME_SHORT);
    let clean_coord_name_en = sanitize_opt(&payload.coordinator_name_en, limits::NAME_SHORT);
    let clean_coord_phone = sanitize_opt(&payload.coordinator_phone, limits::PHONE);
    let clean_coord_whatsapp = sanitize_opt(&payload.coordinator_whatsapp, limits::PHONE);

    // Parse city_id from string to UUID if provided
    let parsed_city_id: Option<Uuid> = match &payload.city_id {
        Some(cid) if !cid.trim().is_empty() => Some(
            Uuid::parse_str(cid)
                .map_err(|_| AppError::BadRequest("Invalid city_id UUID format".to_string()))?,
        ),
        _ => None,
    };

    let repo = crate::repositories::vendor_repository::PgVendorRepository::new(state.db.clone());
    let rows_affected = repo.update_vendor_profile(
        &mut rls_tx.tx,
        vendor_id,
        &payload,
        &clean_name_ar,
        &clean_name_en,
        clean_description_ar.as_deref(),
        clean_description_en.as_deref(),
        clean_address_ar.as_deref(),
        clean_address_en.as_deref(),
        clean_phone.as_deref(),
        clean_email.as_deref(),
        clean_crm_venue_id.as_deref(),
        clean_website.as_deref(),
        clean_maps_url.as_deref(),
        clean_video_url_1.as_deref(),
        clean_instagram_url.as_deref(),
        clean_cr_number.as_deref(),
        clean_coord_name_ar.as_deref(),
        clean_coord_name_en.as_deref(),
        clean_coord_phone.as_deref(),
        clean_coord_whatsapp.as_deref(),
        parsed_city_id,
    ).await?;

    if rows_affected == 0 {
        return Err(AppError::Status(
            axum::http::StatusCode::PRECONDITION_FAILED,
            "Operational conflict: The vendor profile was updated by another operator. Please reload and try again.".to_string(),
        ));
    }

    rls_tx.tx.commit().await?;

    tracing::info!("Vendor {} corporate profile updated", vendor_id);

    Ok(Json(json!({
        "status": "success",
        "message": "Corporate profile updated successfully"
    })))
}


// ─── SUBSCRIPTION REQUEST HANDLERS ───────────────────────────────────────────

#[derive(Deserialize)]
struct CreateSubscriptionRequest {
    requested_tier_id: String,
}

async fn list_subscription_requests(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    mut rls_tx: RlsTx,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;
    let repo = crate::repositories::vendor_repository::PgVendorRepository::new(state.db.clone());
    let requests = repo.list_subscription_requests(&mut rls_tx.tx, vendor_id).await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "requests": requests
    })))
}

async fn create_subscription_request(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    mut rls_tx: RlsTx,
    Json(payload): Json<CreateSubscriptionRequest>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;
    let tier_id = Uuid::parse_str(&payload.requested_tier_id)
        .map_err(|_| AppError::BadRequest("Invalid tier ID format".to_string()))?;

    let repo = crate::repositories::vendor_repository::PgVendorRepository::new(state.db.clone());
    
    // Check if there is already a pending request
    let has_pending = repo.check_pending_subscription(&mut rls_tx.tx, vendor_id).await?;

    if has_pending {
        return Err(AppError::BadRequest(
            "You already have a pending subscription request. Please wait for it to be processed."
                .to_string(),
        ));
    }

    let request_id = Uuid::new_v4();

    repo.create_subscription_request(&mut rls_tx.tx, request_id, vendor_id, tier_id).await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Subscription request submitted successfully",
        "id": request_id.to_string()
    })))
}
