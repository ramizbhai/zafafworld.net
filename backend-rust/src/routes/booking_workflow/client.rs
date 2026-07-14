use crate::errors::AppError;
use crate::middleware::auth::{RequireAuth, RlsTx};
use crate::models::user::DomainType;
use crate::state::AppState;
use axum::{
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use uuid::Uuid;
use sqlx::Row;
use serde::Deserialize;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/bookings", get(get_client_bookings))
        .route(
            "/bookings/:id/transition",
            post(transition_client_booking).layer(axum::middleware::from_fn_with_state(
                state.clone(),
                crate::middleware::idempotency::idempotent_gate_middleware,
            )),
        )
        .route("/quotations", get(list_client_quotations))
        .route("/quotations/:id", get(get_quotation_details))
        .route("/quotations/:id/accept", post(accept_quotation))
        .route("/quotations/:id/reject", post(reject_quotation))
        .route(
            "/quotations/:id/request-revision",
            post(request_quotation_revision),
        )
}

#[derive(Deserialize)]
struct TransitionRequest {
    #[serde(rename = "toStatus")]
    to_status: String,
}

async fn transition_client_booking(
    auth: RequireAuth,
    mut rls_tx: RlsTx,
    axum::extract::Path(booking_id): axum::extract::Path<Uuid>,
    Json(payload): Json<TransitionRequest>,
) -> Result<Json<Value>, AppError> {
    if auth.role != DomainType::Client {
        return Err(AppError::Forbidden(
            "Client credentials required".to_string(),
        ));
    }
    let client_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid client ID in session".to_string()))?;

    // 1. Fetch current booking under client RLS context
    let row = sqlx::query("SELECT status, vendor_id, booking_number FROM core_bookings WHERE id = $1 AND client_id = $2")
        .bind(booking_id)
        .bind(client_uuid)
        .fetch_optional(&mut *rls_tx.tx)
        .await?;

    let booking = match row {
        Some(r) => r,
        None => {
            return Err(AppError::NotFound(
                "Booking not found or not owned by you".to_string(),
            ))
        }
    };

    let current_status: String = booking.get("status");
    let vendor_user_id: Uuid = booking.get("vendor_id");
    let booking_number: String = booking.get("booking_number");
    let target = payload.to_status.as_str();

    // 2. State Machine Validation
    if current_status == target {
        return Err(AppError::BadRequest(format!(
            "Booking is already in {} state",
            target
        )));
    }

    if current_status == "cancelled" {
        return Err(AppError::BadRequest(
            "Cannot transition a cancelled booking".to_string(),
        ));
    }

    match target {
        "Pending_Vendor_Acceptance" => {
            if current_status != "Draft_Inquiry" && current_status != "pending" {
                return Err(AppError::BadRequest(format!(
                    "Invalid transition: Cannot move from {} to Pending_Vendor_Acceptance",
                    current_status
                )));
            }
        }
        "cancelled" => {
            if current_status == "Booking_Active" {
                return Err(AppError::BadRequest(
                    "Cannot cancel an active/confirmed booking directly. Please contact support."
                        .to_string(),
                ));
            }
        }
        _ => {
            return Err(AppError::BadRequest(format!(
                "Invalid transition: Clients are not authorized to transition to {}",
                target
            )));
        }
    }

    // 3. Update the booking status
    sqlx::query("UPDATE core_bookings SET status = $1, updated_at = NOW() WHERE id = $2")
        .bind(target)
        .bind(booking_id)
        .execute(&mut *rls_tx.tx)
        .await?;

    if target == "cancelled" {
        // Resolve vendor profile ID
        let target_vendor_profile_id: Option<Uuid> =
            sqlx::query_scalar("SELECT id FROM vendors WHERE user_id = $1")
                .bind(vendor_user_id)
                .fetch_optional(&mut *rls_tx.tx)
                .await?;

        let _ = sqlx::query(
            "INSERT INTO system_events (user_id, target_vendor_id, event_type, message_ar, message_en)
             VALUES ($1, $2, 'booking_cancelled', $3, $4)"
        )
        .bind(client_uuid)
        .bind(target_vendor_profile_id)
        .bind(format!("تم إلغاء الحجز رقم «{}» من قبل العميل", booking_number))
        .bind(format!("Booking '{}' has been cancelled by the client", booking_number))
        .execute(&mut *rls_tx.tx)
        .await;
    }

    // Commit transaction
    rls_tx
        .tx
        .commit()
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(json!({
        "status": "success",
        "message": format!("Booking status transitioned successfully to {}", target),
        "bookingId": booking_id,
        "newStatus": target
    })))
}


async fn get_client_bookings(
    auth: RequireAuth,
    mut rls_tx: RlsTx,
) -> Result<Json<Value>, AppError> {
    if auth.role != DomainType::Client {
        return Err(AppError::Forbidden(
            "Client credentials required".to_string(),
        ));
    }
    let client_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid client ID in session".to_string()))?;

    let bookings_rows = sqlx::query(
        r#"SELECT 
            b.id, b.booking_number, b.status, b.wedding_date::text, b.event_type, b.guest_count, 
            b.total_price::float8, b.deposit_paid::float8, b.created_at::text,
            p.title AS product_name_ar, p.title AS product_name_en,
            b.booked_gender_section::text
         FROM core_bookings b
         LEFT JOIN vendor_products p ON b.product_id = p.id
         WHERE b.client_id = $1
         ORDER BY b.created_at DESC"#,
    )
    .bind(client_uuid)
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let mut bookings = Vec::new();
    for row in bookings_rows {
        bookings.push(json!({
            "id": row.get::<Uuid, _>("id"),
            "bookingNumber": row.get::<String, _>("booking_number"),
            "status": row.get::<String, _>("status"),
            "weddingDate": row.get::<String, _>("wedding_date"),
            "eventType": row.get::<String, _>("event_type"),
            "guestCount": row.get::<i32, _>("guest_count"),
            "totalPrice": row.get::<f64, _>("total_price"),
            "depositPaid": row.get::<f64, _>("deposit_paid"),
            "createdAt": row.get::<String, _>("created_at"),
            "productNameAr": row.get::<Option<String>, _>("product_name_ar"),
            "productNameEn": row.get::<Option<String>, _>("product_name_en"),
            "bookedGenderSection": row.get::<Option<String>, _>("booked_gender_section"),
        }));
    }

    rls_tx
        .tx
        .commit()
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(json!({
        "status": "success",
        "data": bookings
    })))
}


#[derive(Deserialize)]
struct RequestRevisionPayload {
    notes: String,
}

async fn list_client_quotations(
    auth: RequireAuth,
    mut rls_tx: RlsTx,
) -> Result<Json<Value>, AppError> {
    if auth.role != DomainType::Client {
        return Err(AppError::Forbidden(
            "Client credentials required".to_string(),
        ));
    }
    let client_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid client session".to_string()))?;

    let rows = sqlx::query(
        "SELECT q.id, q.booking_id, q.vendor_id, q.current_version, q.total_price::float8, q.deposit_amount::float8,
                q.status, q.expires_at::text, q.notes, q.created_at::text, v.name_ar AS vendor_name_ar, v.name_en AS vendor_name_en
         FROM booking_quotations q
         JOIN vendors v ON q.vendor_id = v.id
         WHERE q.client_id = $1
         ORDER BY q.created_at DESC"
    )
    .bind(client_uuid)
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let quotations: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "id": r.get::<Uuid, _>("id").to_string(),
                "bookingId": r.get::<Uuid, _>("booking_id").to_string(),
                "vendorId": r.get::<Uuid, _>("vendor_id").to_string(),
                "vendorNameAr": r.get::<String, _>("vendor_name_ar"),
                "vendorNameEn": r.get::<String, _>("vendor_name_en"),
                "version": r.get::<i32, _>("current_version"),
                "totalPrice": r.get::<f64, _>("total_price"),
                "depositAmount": r.get::<f64, _>("deposit_amount"),
                "status": r.get::<String, _>("status"),
                "expiresAt": r.get::<Option<String>, _>("expires_at"),
                "notes": r.get::<Option<String>, _>("notes"),
                "createdAt": r.get::<String, _>("created_at"),
            })
        })
        .collect();

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "data": quotations
    })))
}

async fn get_quotation_details(
    auth: RequireAuth,
    mut rls_tx: RlsTx,
    axum::extract::Path(quotation_id): axum::extract::Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    if auth.role != DomainType::Client {
        return Err(AppError::Forbidden(
            "Client credentials required".to_string(),
        ));
    }
    let client_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid client session".to_string()))?;

    let row = sqlx::query(
        "SELECT q.id, q.booking_id, q.vendor_id, q.current_version, q.total_price::float8, q.deposit_amount::float8,
                q.status, q.expires_at::text, q.notes, q.created_at::text, v.name_ar AS vendor_name_ar, v.name_en AS vendor_name_en
         FROM booking_quotations q
         JOIN vendors v ON q.vendor_id = v.id
         WHERE q.id = $1 AND q.client_id = $2"
    )
    .bind(quotation_id)
    .bind(client_uuid)
    .fetch_optional(&mut *rls_tx.tx)
    .await?;

    let row =
        row.ok_or_else(|| AppError::NotFound("Quotation not found or access denied".to_string()))?;

    let revisions_rows = sqlx::query(
        "SELECT version, total_price::float8, deposit_amount::float8, sender_role, notes, created_at::text
         FROM booking_quotation_revisions
         WHERE quotation_id = $1
         ORDER BY version ASC"
    )
    .bind(quotation_id)
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let revisions: Vec<Value> = revisions_rows
        .iter()
        .map(|r| {
            json!({
                "version": r.get::<i32, _>("version"),
                "totalPrice": r.get::<f64, _>("total_price"),
                "depositAmount": r.get::<f64, _>("deposit_amount"),
                "senderRole": r.get::<String, _>("sender_role"),
                "notes": r.get::<Option<String>, _>("notes"),
                "createdAt": r.get::<String, _>("created_at"),
            })
        })
        .collect();

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "data": {
            "id": row.get::<Uuid, _>("id").to_string(),
            "bookingId": row.get::<Uuid, _>("booking_id").to_string(),
            "vendorId": row.get::<Uuid, _>("vendor_id").to_string(),
            "vendorNameAr": row.get::<String, _>("vendor_name_ar"),
            "vendorNameEn": row.get::<String, _>("vendor_name_en"),
            "version": row.get::<i32, _>("current_version"),
            "totalPrice": row.get::<f64, _>("total_price"),
            "depositAmount": row.get::<f64, _>("deposit_amount"),
            "status": row.get::<String, _>("status"),
            "expiresAt": row.get::<Option<String>, _>("expires_at"),
            "notes": row.get::<Option<String>, _>("notes"),
            "createdAt": row.get::<String, _>("created_at"),
            "revisions": revisions
        }
    })))
}

async fn accept_quotation(
    auth: RequireAuth,
    mut rls_tx: RlsTx,
    axum::extract::Path(quotation_id): axum::extract::Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    if auth.role != DomainType::Client {
        return Err(AppError::Forbidden(
            "Client credentials required".to_string(),
        ));
    }
    let client_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid client session".to_string()))?;

    let row = sqlx::query(
        "SELECT booking_id, total_price::float8, deposit_amount::float8 FROM booking_quotations WHERE id = $1 AND client_id = $2"
    )
    .bind(quotation_id)
    .bind(client_uuid)
    .fetch_optional(&mut *rls_tx.tx)
    .await?;

    let row =
        row.ok_or_else(|| AppError::NotFound("Quotation not found or access denied".to_string()))?;
    let booking_id: Uuid = row.get("booking_id");
    let total_price: f64 = row.get("total_price");
    let deposit_amount: f64 = row.get("deposit_amount");

    sqlx::query(
        "UPDATE booking_quotations SET status = 'Accepted', updated_at = NOW() WHERE id = $1",
    )
    .bind(quotation_id)
    .execute(&mut *rls_tx.tx)
    .await?;

    sqlx::query("UPDATE core_bookings SET status = 'confirmed', total_price = $1, deposit_paid = $2, updated_at = NOW() WHERE id = $3")
        .bind(total_price)
        .bind(deposit_amount)
        .bind(booking_id)
        .execute(&mut *rls_tx.tx)
        .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Quotation accepted and booking confirmed successfully"
    })))
}

async fn reject_quotation(
    auth: RequireAuth,
    mut rls_tx: RlsTx,
    axum::extract::Path(quotation_id): axum::extract::Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    if auth.role != DomainType::Client {
        return Err(AppError::Forbidden(
            "Client credentials required".to_string(),
        ));
    }
    let client_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid client session".to_string()))?;

    sqlx::query("UPDATE booking_quotations SET status = 'Rejected', updated_at = NOW() WHERE id = $1 AND client_id = $2")
        .bind(quotation_id)
        .bind(client_uuid)
        .execute(&mut *rls_tx.tx)
        .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Quotation rejected"
    })))
}

async fn request_quotation_revision(
    auth: RequireAuth,
    mut rls_tx: RlsTx,
    axum::extract::Path(quotation_id): axum::extract::Path<Uuid>,
    Json(payload): Json<RequestRevisionPayload>,
) -> Result<Json<Value>, AppError> {
    if auth.role != DomainType::Client {
        return Err(AppError::Forbidden(
            "Client credentials required".to_string(),
        ));
    }
    let client_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid client session".to_string()))?;

    let row = sqlx::query(
        "SELECT current_version, total_price::float8, deposit_amount::float8 FROM booking_quotations WHERE id = $1 AND client_id = $2"
    )
    .bind(quotation_id)
    .bind(client_uuid)
    .fetch_optional(&mut *rls_tx.tx)
    .await?;

    let row =
        row.ok_or_else(|| AppError::NotFound("Quotation not found or access denied".to_string()))?;
    let current_version: i32 = row.get("current_version");
    let total_price: f64 = row.get("total_price");
    let deposit_amount: f64 = row.get("deposit_amount");

    sqlx::query("UPDATE booking_quotations SET status = 'Revision_Requested', updated_at = NOW() WHERE id = $1")
        .bind(quotation_id)
        .execute(&mut *rls_tx.tx)
        .await?;

    sqlx::query(
        "INSERT INTO booking_quotation_revisions (quotation_id, version, total_price, deposit_amount, sender_role, notes)
         VALUES ($1, $2, $3, $4, 'Client', $5)"
    )
    .bind(quotation_id)
    .bind(current_version)
    .bind(total_price)
    .bind(deposit_amount)
    .bind(&payload.notes)
    .execute(&mut *rls_tx.tx)
    .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Revision requested successfully"
    })))
}


