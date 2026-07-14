use crate::errors::AppError;
use crate::middleware::auth::{RequireVendor, RlsTx};
use crate::state::AppState;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use uuid::Uuid;
use sqlx::Row;
use serde::Deserialize;
use crate::utils::sanitize::{limits, sanitize_str};

#[derive(Deserialize)]
pub struct CreatePackageRequest {
    pub name_ar: String,
    pub name_en: String,
    pub description_ar: String,
    pub description_en: String,
    pub original_price: f64,
    pub discounted_price: f64,
    pub is_zafaf_exclusive: bool,
    pub expiry_date: String,
    pub product_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct UpdatePackageRequest {
    pub name_ar: String,
    pub name_en: String,
    pub description_ar: String,
    pub description_en: String,
    pub original_price: f64,
    pub discounted_price: f64,
    pub is_zafaf_exclusive: bool,
    pub expiry_date: String,
    pub product_id: Option<Uuid>,
    pub version: i32,
}

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/packages", get(list_packages).post(create_package))
        .route("/packages/:id", axum::routing::put(update_package).delete(delete_package))
        .route("/packages/:id/duplicate", post(duplicate_package))
}

async fn list_packages(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    mut rls_tx: RlsTx,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;
    let repo = crate::repositories::vendor_repository::PgVendorRepository::new(state.db.clone());
    let packages = repo.list_packages(&mut rls_tx.tx, vendor_id).await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "packages": packages
    })))
}

async fn create_package(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    mut rls_tx: RlsTx,
    Json(payload): Json<CreatePackageRequest>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    if payload.name_ar.trim().is_empty() || payload.name_en.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Package name is required in both Arabic and English".to_string(),
        ));
    }

    if payload.discounted_price >= payload.original_price {
        return Err(AppError::BadRequest(
            "Discounted price must be strictly less than the original price".to_string(),
        ));
    }

    let parsed_date =
        chrono::NaiveDate::parse_from_str(&payload.expiry_date, "%Y-%m-%d").map_err(|_| {
            AppError::BadRequest("Invalid expiry date format. Expected YYYY-MM-DD".to_string())
        })?;

    let today = chrono::Utc::now().date_naive();
    if parsed_date < today {
        return Err(AppError::BadRequest(
            "Expiry date must be a future date (on or after today)".to_string(),
        ));
    }

    let new_id = Uuid::new_v4();
    let clean_name_ar = sanitize_str(&payload.name_ar, limits::NAME_LONG);
    let clean_name_en = sanitize_str(&payload.name_en, limits::NAME_LONG);
    let clean_description_ar = sanitize_str(&payload.description_ar, limits::DESCRIPTION);
    let clean_description_en = sanitize_str(&payload.description_en, limits::DESCRIPTION);

    let repo = crate::repositories::vendor_repository::PgVendorRepository::new(state.db.clone());
    repo.create_package(
        &mut rls_tx.tx,
        new_id,
        vendor_id,
        payload.product_id,
        &clean_name_ar,
        &clean_name_en,
        &clean_description_ar,
        &clean_description_en,
        payload.original_price,
        payload.discounted_price,
        payload.is_zafaf_exclusive,
        parsed_date,
    )
    .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Package created successfully",
        "id": new_id.to_string()
    })))
}

async fn update_package(
    _auth: RequireVendor,
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    mut rls_tx: RlsTx,
    axum::extract::Path(package_id): axum::extract::Path<Uuid>,
    Json(payload): Json<UpdatePackageRequest>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    if payload.name_ar.trim().is_empty() || payload.name_en.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Package name is required in both Arabic and English".to_string(),
        ));
    }

    if payload.discounted_price >= payload.original_price {
        return Err(AppError::BadRequest(
            "Discounted price must be strictly less than the original price".to_string(),
        ));
    }

    let parsed_date =
        chrono::NaiveDate::parse_from_str(&payload.expiry_date, "%Y-%m-%d").map_err(|_| {
            AppError::BadRequest("Invalid expiry date format. Expected YYYY-MM-DD".to_string())
        })?;

    let today = chrono::Utc::now().date_naive();
    if parsed_date < today {
        return Err(AppError::BadRequest(
            "Expiry date must be a future date (on or after today)".to_string(),
        ));
    }

    let repo = crate::repositories::vendor_repository::PgVendorRepository::new(state.db.clone());
    repo.update_package(
        &mut rls_tx.tx,
        package_id,
        vendor_id,
        payload.product_id,
        &payload.name_ar,
        &payload.name_en,
        &payload.description_ar,
        &payload.description_en,
        payload.original_price,
        payload.discounted_price,
        payload.is_zafaf_exclusive,
        parsed_date,
        payload.version,
    )
    .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Package updated successfully"
    })))
}

async fn delete_package(
    _auth: RequireVendor,
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    mut rls_tx: RlsTx,
    axum::extract::Path(package_id): axum::extract::Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;
    let repo = crate::repositories::vendor_repository::PgVendorRepository::new(state.db.clone());
    repo.delete_package(&mut rls_tx.tx, package_id, vendor_id)
        .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Package deleted successfully"
    })))
}

async fn duplicate_package(
    _auth: RequireVendor,
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    mut rls_tx: RlsTx,
    axum::extract::Path(package_id): axum::extract::Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;
    let new_id = Uuid::new_v4();
    let repo = crate::repositories::vendor_repository::PgVendorRepository::new(state.db.clone());
    repo.duplicate_package(&mut rls_tx.tx, package_id, vendor_id, new_id)
        .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Package duplicated successfully",
        "id": new_id.to_string()
    })))
}



