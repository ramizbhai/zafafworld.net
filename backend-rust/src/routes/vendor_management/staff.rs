use crate::errors::AppError;
use crate::middleware::auth::{RequireVendor, RequireVendorOwner, RlsTx};
use crate::state::AppState;
use crate::models::inquiry::VendorStaff;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use uuid::Uuid;
use sqlx::Row;
use serde::Deserialize;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/staff", get(list_staff).post(create_staff))
        .route("/staff/:id", axum::routing::put(update_staff).delete(delete_staff))
        .route("/staff/:id/status", axum::routing::patch(update_staff_status))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateStaffRequest {
    name: String,
    email: String,
    role: String,
    status: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateStaffStatusRequest {
    status: String,
}

async fn list_staff(
    State(state): State<AppState>,
    auth: RequireVendorOwner,
) -> Result<Json<Value>, AppError> {
    let user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid user ID format in token".to_string()))?;

    let vendor_id = sqlx::query_scalar::<_, Uuid>("SELECT id FROM vendors WHERE user_id = $1")
        .bind(user_uuid)
        .fetch_optional(&state.db)
        .await?;

    let vendor_id = match vendor_id {
        Some(id) => id,
        None => return Err(AppError::NotFound("Vendor profile not found".to_string())),
    };

    let staff: Vec<VendorStaff> = sqlx::query_as::<_, VendorStaff>(
        "SELECT id, vendor_id, name, email, role, status, created_at
         FROM vendor_staff
         WHERE vendor_id = $1
         ORDER BY created_at DESC",
    )
    .bind(vendor_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(json!({
        "status": "success",
        "staff": staff
    })))
}

async fn create_staff(
    State(state): State<AppState>,
    auth: RequireVendorOwner,
    Json(payload): Json<CreateStaffRequest>,
) -> Result<Json<Value>, AppError> {
    let user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid user ID format in token".to_string()))?;

    let vendor_id = sqlx::query_scalar::<_, Uuid>("SELECT id FROM vendors WHERE user_id = $1")
        .bind(user_uuid)
        .fetch_optional(&state.db)
        .await?;

    let vendor_id = match vendor_id {
        Some(id) => id,
        None => return Err(AppError::NotFound("Vendor profile not found".to_string())),
    };

    if payload.name.trim().is_empty() {
        return Err(AppError::BadRequest("Staff name is required".to_string()));
    }
    if payload.email.trim().is_empty() || !payload.email.contains('@') {
        return Err(AppError::BadRequest(
            "A valid email address is required".to_string(),
        ));
    }
    if payload.role != "admin" && payload.role != "editor" && payload.role != "viewer" {
        return Err(AppError::BadRequest("Invalid staff role".to_string()));
    }
    if payload.status != "active" && payload.status != "inactive" {
        return Err(AppError::BadRequest("Invalid staff status".to_string()));
    }

    let new_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO vendor_staff (id, vendor_id, name, email, role, status, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, NOW())",
    )
    .bind(new_id)
    .bind(vendor_id)
    .bind(payload.name.trim())
    .bind(payload.email.trim())
    .bind(payload.role)
    .bind(payload.status)
    .execute(&state.db)
    .await
    ?;

    Ok(Json(json!({
        "status": "success",
        "message": "Staff member created successfully",
        "id": new_id.to_string()
    })))
}

async fn update_staff(
    _auth: RequireVendor,
    mut rls_tx: RlsTx,
    axum::extract::Path(staff_id): axum::extract::Path<Uuid>,
    Json(payload): Json<CreateStaffRequest>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    if payload.name.trim().is_empty() {
        return Err(AppError::BadRequest("Staff name is required".to_string()));
    }
    if payload.email.trim().is_empty() || !payload.email.contains('@') {
        return Err(AppError::BadRequest(
            "A valid email address is required".to_string(),
        ));
    }
    if payload.role != "admin" && payload.role != "editor" && payload.role != "viewer" {
        return Err(AppError::BadRequest("Invalid staff role".to_string()));
    }
    if payload.status != "active" && payload.status != "inactive" {
        return Err(AppError::BadRequest("Invalid staff status".to_string()));
    }

    let rows_affected = sqlx::query(
        "UPDATE vendor_staff
         SET name = $1, email = $2, role = $3, status = $4
         WHERE id = $5 AND vendor_id = $6",
    )
    .bind(payload.name.trim())
    .bind(payload.email.trim())
    .bind(payload.role)
    .bind(payload.status)
    .bind(staff_id)
    .bind(vendor_id)
    .execute(&mut *rls_tx.tx)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound(
            "Staff member not found or access denied".to_string(),
        ));
    }

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Staff member updated successfully"
    })))
}

async fn delete_staff(
    _auth: RequireVendor,
    mut rls_tx: RlsTx,
    axum::extract::Path(staff_id): axum::extract::Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    let rows_affected = sqlx::query("DELETE FROM vendor_staff WHERE id = $1 AND vendor_id = $2")
        .bind(staff_id)
        .bind(vendor_id)
        .execute(&mut *rls_tx.tx)
        .await?
        .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound(
            "Staff member not found or access denied".to_string(),
        ));
    }

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Staff member deleted successfully"
    })))
}

async fn update_staff_status(
    _auth: RequireVendor,
    mut rls_tx: RlsTx,
    axum::extract::Path(staff_id): axum::extract::Path<Uuid>,
    Json(payload): Json<UpdateStaffStatusRequest>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    if payload.status != "active" && payload.status != "inactive" {
        return Err(AppError::BadRequest("Invalid staff status".to_string()));
    }

    let rows_affected = sqlx::query(
        "UPDATE vendor_staff
         SET status = $1
         WHERE id = $2 AND vendor_id = $3",
    )
    .bind(&payload.status)
    .bind(staff_id)
    .bind(vendor_id)
    .execute(&mut *rls_tx.tx)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound(
            "Staff member not found or access denied".to_string(),
        ));
    }

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Staff member status updated successfully"
    })))
}


