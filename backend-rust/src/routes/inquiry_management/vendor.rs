use crate::errors::AppError;
use crate::middleware::auth::{RequireVendor, RlsTx};
use crate::state::AppState;
use crate::utils::sanitize::{limits, sanitize_str};
use crate::models::inquiry::{Inquiry, VendorInquiryDto};
use axum::{
    routing::{get, patch, post},
    Json, Router,
};
use serde_json::{json, Value};
use uuid::Uuid;
use sqlx::Row;
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/inquiries", get(list_inquiries).post(create_inquiry))
        .route("/inquiries/:id/read", patch(mark_inquiry_read))
        .route("/inquiries/:id/close", patch(close_inquiry))
        .route("/inquiries/:id/status", patch(update_inquiry_status))
        .route("/inquiries/:id/convert-to-booking", post(convert_inquiry_to_booking))
}

#[derive(Deserialize)]
pub struct InquiryQuery {
    pub status: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

async fn list_inquiries(
    auth: RequireVendor,
    mut rls_tx: RlsTx,
    axum::extract::Query(query): axum::extract::Query<InquiryQuery>,
) -> Result<Json<Value>, AppError> {
    let user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid user ID format in token".to_string()))?;

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = (page - 1) * limit;

    let mut query_str = "SELECT id, vendor_id, customer_name, phone, wedding_date, message, status, created_at, resolution_note FROM lead_inquiries WHERE vendor_id = $1".to_string();
    let mut param_idx = 2;
    let mut status_val = None;

    if let Some(ref status) = query.status {
        query_str.push_str(&format!(" AND status = ${}", param_idx));
        param_idx += 1;
        status_val = Some(status);
    }

    query_str.push_str(&format!(
        " ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
        param_idx,
        param_idx + 1
    ));

    let mut db_query = sqlx::query_as::<_, Inquiry>(&query_str).bind(user_uuid);
    if let Some(status) = status_val {
        db_query = db_query.bind(status);
    }
    db_query = db_query.bind(limit as i64).bind(offset as i64);

    let inquiries = db_query.fetch_all(&mut *rls_tx.tx).await?;

    // Also get total count
    let mut count_query_str =
        "SELECT COUNT(*) FROM lead_inquiries WHERE vendor_id = $1".to_string();
    if query.status.is_some() {
        count_query_str.push_str(" AND status = $2");
    }
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_query_str).bind(user_uuid);
    if let Some(ref status) = query.status {
        count_query = count_query.bind(status);
    }
    let total: i64 = count_query.fetch_one(&mut *rls_tx.tx).await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "inquiries": inquiries,
        "pagination": {
            "total": total,
            "page": page,
            "limit": limit,
            "totalPages": (total as f64 / limit as f64).ceil() as i64
        }
    })))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateInquiryRequest {
    pub customer_name: String,
    pub phone: String,
    pub wedding_date: String,
    pub message: String,
}

async fn create_inquiry(
    auth: RequireVendor,
    mut rls_tx: RlsTx,
    Json(payload): Json<CreateInquiryRequest>,
) -> Result<Json<Value>, AppError> {
    let user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid user ID format in token".to_string()))?;

    if payload.customer_name.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Customer name is required".to_string(),
        ));
    }
    if payload.phone.trim().is_empty() {
        return Err(AppError::BadRequest("Phone number is required".to_string()));
    }

    let parsed_date = chrono::NaiveDate::parse_from_str(&payload.wedding_date, "%Y-%m-%d")
        .map_err(|_| {
            AppError::BadRequest("Invalid wedding date. Must be YYYY-MM-DD".to_string())
        })?;

    let new_id = Uuid::new_v4();
    // Sanitize free-text inquiry fields before binding to prevent stored-XSS.
    let clean_customer_name = sanitize_str(&payload.customer_name, limits::NAME_SHORT);
    let clean_phone = sanitize_str(&payload.phone, limits::PHONE);
    let clean_message = sanitize_str(&payload.message, limits::MESSAGE);

    let city_id: Option<Uuid> =
        sqlx::query_scalar("SELECT city_id FROM vendors WHERE user_id = $1")
            .bind(user_uuid)
            .fetch_optional(&mut *rls_tx.tx)
            .await?
            .flatten();

    sqlx::query(
        "INSERT INTO lead_inquiries ( \
            id, vendor_id, customer_name, phone, wedding_date, message, status, created_at, city_id \
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(new_id)
    .bind(user_uuid)
    .bind(&clean_customer_name)
    .bind(&clean_phone)
    .bind(parsed_date)
    .bind(&clean_message)
    .bind("new")
    .bind(chrono::Utc::now())
    .bind(city_id)
    .execute(&mut *rls_tx.tx)
    .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Inquiry created successfully",
        "id": new_id.to_string()
    })))
}

async fn mark_inquiry_read(
    auth: RequireVendor,
    mut rls_tx: RlsTx,
    axum::extract::Path(inquiry_id): axum::extract::Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid user ID format in token".to_string()))?;

    let rows_affected = sqlx::query(
        "UPDATE lead_inquiries \
         SET status = 'read' \
         WHERE id = $1 AND vendor_id = $2",
    )
    .bind(inquiry_id)
    .bind(user_uuid)
    .execute(&mut *rls_tx.tx)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound(
            "Inquiry not found or access denied".to_string(),
        ));
    }

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Inquiry marked as read"
    })))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CloseInquiryRequest {
    resolution_note: Option<String>,
}

async fn close_inquiry(
    auth: RequireVendor,
    axum::extract::Path(inquiry_id): axum::extract::Path<Uuid>,
    mut rls_tx: RlsTx,
    Json(payload): Json<CloseInquiryRequest>,
) -> Result<Json<Value>, AppError> {
    let user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid user ID format in token".to_string()))?;

    let rows_affected = sqlx::query(
        "UPDATE lead_inquiries \
         SET status = 'done', \
             resolution_note = $3 \
         WHERE id = $1 AND vendor_id = $2 AND status IN ('pending', 'read')",
    )
    .bind(inquiry_id)
    .bind(user_uuid)
    .bind(payload.resolution_note.as_deref().unwrap_or(""))
    .execute(&mut *rls_tx.tx)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound(
            "Inquiry not found, already closed, or access denied".to_string(),
        ));
    }

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Inquiry closed successfully"
    })))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateInquiryStatusRequest {
    status: String,
}

async fn update_inquiry_status(
    auth: RequireVendor,
    axum::extract::Path(inquiry_id): axum::extract::Path<Uuid>,
    mut rls_tx: RlsTx,
    Json(payload): Json<UpdateInquiryStatusRequest>,
) -> Result<Json<Value>, AppError> {
    let user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid user ID format in token".to_string()))?;

    let valid_statuses = [
        "new",
        "read",
        "done",
        "expired",
        "rejected",
        "negotiation",
        "unreachable",
        "paid",
    ];
    let mapped_status = match payload.status.as_str() {
        "negot" => "negotiation",
        "unreach" => "unreachable",
        other => other,
    };

    if !valid_statuses.contains(&mapped_status) {
        return Err(AppError::BadRequest(format!(
            "Invalid status literal: {}. Must be one of {:?}",
            payload.status, valid_statuses
        )));
    }

    let rows_affected = sqlx::query(
        "UPDATE lead_inquiries \
         SET status = $3 \
         WHERE id = $1 AND vendor_id = $2",
    )
    .bind(inquiry_id)
    .bind(user_uuid)
    .bind(mapped_status)
    .execute(&mut *rls_tx.tx)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound(
            "Inquiry not found or access denied".to_string(),
        ));
    }

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Inquiry status updated successfully"
    })))
}

#[derive(Deserialize)]
struct ConvertInquiryRequest {
    total_price: Option<f64>,
    deposit_paid: Option<f64>,
}

async fn convert_inquiry_to_booking(
    auth: RequireVendor,
    axum::extract::Path(inquiry_id): axum::extract::Path<Uuid>,
    mut rls_tx: RlsTx,
    Json(payload): Json<ConvertInquiryRequest>,
) -> Result<Json<Value>, AppError> {
    let user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid user ID format in token".to_string()))?;

    let row = sqlx::query(
        "SELECT id, vendor_id, client_id, product_id, city_id, customer_name, phone, wedding_date::text AS wedding_date, message, status \
         FROM lead_inquiries \
         WHERE id = $1 AND vendor_id = $2"
    )
    .bind(inquiry_id)
    .bind(user_uuid)
    .fetch_optional(&mut *rls_tx.tx)
    .await?;

    let row =
        row.ok_or_else(|| AppError::NotFound("Inquiry not found or access denied".to_string()))?;

    let current_status: String = row.get("status");
    if current_status == "converted" || current_status == "paid" {
        return Err(AppError::BadRequest(
            "Inquiry has already been converted into a booking".to_string(),
        ));
    }

    let customer_name: String = row.get("customer_name");
    let phone: String = row.get::<Option<String>, _>("phone").unwrap_or_default();
    let wedding_date_str: String = row.get("wedding_date");
    let special_requests: Option<String> = row.get("message");
    let product_id: Option<Uuid> = row.get("product_id");
    let client_id: Option<Uuid> = row.get("client_id");
    let city_id: Option<Uuid> = row.get("city_id");

    let parsed_date = chrono::NaiveDate::parse_from_str(&wedding_date_str, "%Y-%m-%d")
        .unwrap_or_else(|_| chrono::Utc::now().date_naive());

    let booking_number = format!(
        "BK-{:06X}",
        ((Uuid::new_v4().as_u128() & 0xFFFFFF) as u32) % 1_000_000
    );
    let new_booking_id = Uuid::new_v4();
    let total_price = payload.total_price.unwrap_or(0.0);
    let deposit_paid = payload.deposit_paid.unwrap_or(0.0);

    let name_parts: Vec<&str> = customer_name.split_whitespace().collect();
    let first_name = name_parts.first().copied().unwrap_or("Customer");
    let last_name = if name_parts.len() > 1 {
        name_parts[1..].join(" ")
    } else {
        "".to_string()
    };

    sqlx::query(
        "INSERT INTO core_bookings ( \
            id, booking_number, vendor_id, product_id, \
            status, wedding_date, event_type, guest_count, \
            total_price, deposit_paid, \
            customer_first_name, customer_last_name, \
            customer_phone, customer_email, \
            special_requests, client_id, city_id \
        ) VALUES ( \
            $1, $2, $3, $4, \
            'confirmed', $5, 'wedding', 100, \
            $6::numeric, $7::numeric, \
            $8, $9, \
            $10, '', \
            $11, $12, $13 \
        )",
    )
    .bind(new_booking_id)
    .bind(&booking_number)
    .bind(user_uuid)
    .bind(product_id)
    .bind(parsed_date)
    .bind(total_price)
    .bind(deposit_paid)
    .bind(first_name)
    .bind(&last_name)
    .bind(&phone)
    .bind(&special_requests)
    .bind(client_id)
    .bind(city_id)
    .execute(&mut *rls_tx.tx)
    .await?;

    sqlx::query(
        "UPDATE lead_inquiries \
         SET status = 'converted', resolution_note = 'Converted into booking' \
         WHERE id = $1 AND vendor_id = $2",
    )
    .bind(inquiry_id)
    .bind(user_uuid)
    .execute(&mut *rls_tx.tx)
    .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Lead successfully converted into a booking",
        "bookingId": new_booking_id.to_string(),
        "bookingNumber": booking_number
    })))
}

