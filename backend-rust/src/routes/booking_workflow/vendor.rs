use crate::errors::AppError;
use crate::middleware::auth::{RequireVendor, RlsTx};
use crate::state::AppState;
use crate::utils::sanitize::{limits, sanitize_str};
use axum::{
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use uuid::Uuid;
use sqlx::Row;
use serde::Deserialize;
use rust_decimal::Decimal;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/bookings", get(list_vendor_bookings))
        .route("/bookings/calendar", get(get_vendor_booking_calendar))
        .route(
            "/bookings/check-availability",
            post(check_vendor_availability),
        )
        .route("/bookings/:id", get(get_vendor_booking_detail))
        .route("/quotations", post(create_vendor_quotation))
        .route(
            "/bookings/:id/transition",
            post(transition_vendor_booking).layer(axum::middleware::from_fn_with_state(
                state.clone(),
                crate::middleware::idempotency::idempotent_gate_middleware,
            )),
        )
}

#[derive(Deserialize)]
struct CheckAvailabilityRequest {
    product_id: Option<Uuid>,
    wedding_date: String,
}

async fn list_vendor_bookings(
    auth: RequireVendor,
    mut rls_tx: RlsTx,
) -> Result<Json<Value>, AppError> {
    let vendor_user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid vendor ID in session".to_string()))?;

    let rows = sqlx::query(
        "SELECT 
            b.id, b.booking_number, b.status, b.wedding_date::text AS wedding_date,
            b.event_type, b.guest_count, b.total_price::float8 AS total_price,
            b.deposit_paid::float8 AS deposit_paid, b.customer_first_name,
            b.customer_last_name, b.customer_phone, b.customer_email,
            b.special_requests, b.created_at, p.title AS product_title
         FROM core_bookings b
         LEFT JOIN vendor_products p ON b.product_id = p.id
         WHERE b.vendor_id = $1
         ORDER BY b.wedding_date DESC, b.created_at DESC",
    )
    .bind(vendor_user_uuid)
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let bookings: Vec<Value> = rows
        .iter()
        .map(|row| {
            let id: Uuid = row.get("id");
            let booking_number: String = row.get("booking_number");
            let status: String = row.get("status");
            let wedding_date: String = row.get("wedding_date");
            let event_type: Option<String> = row.get("event_type");
            let guest_count: Option<i32> = row.get("guest_count");
            let total_price: f64 = row.get("total_price");
            let deposit_paid: f64 = row.get("deposit_paid");
            let first_name: Option<String> = row.get("customer_first_name");
            let last_name: Option<String> = row.get("customer_last_name");
            let phone: Option<String> = row.get("customer_phone");
            let email: Option<String> = row.get("customer_email");
            let special_requests: Option<String> = row.get("special_requests");
            let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
            let product_title: Option<String> = row.get("product_title");

            let customer_name = match (first_name, last_name) {
                (Some(f), Some(l)) => format!("{} {}", f, l),
                (Some(f), None) => f,
                _ => "Customer".to_string(),
            };

            json!({
                "id": id.to_string(),
                "bookingNumber": booking_number,
                "status": status,
                "weddingDate": wedding_date,
                "eventType": event_type.unwrap_or_default(),
                "guestCount": guest_count.unwrap_or(0),
                "totalPrice": total_price,
                "depositPaid": deposit_paid,
                "customerName": customer_name,
                "customerPhone": phone.unwrap_or_default(),
                "customerEmail": email.unwrap_or_default(),
                "specialRequests": special_requests.unwrap_or_default(),
                "productTitle": product_title.unwrap_or_default(),
                "createdAt": created_at.to_rfc3339(),
            })
        })
        .collect();

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "bookings": bookings
    })))
}

async fn get_vendor_booking_detail(
    auth: RequireVendor,
    mut rls_tx: RlsTx,
    axum::extract::Path(booking_id): axum::extract::Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let vendor_user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid vendor ID in session".to_string()))?;

    let row = sqlx::query(
        "SELECT 
            b.id, b.booking_number, b.status, b.wedding_date::text AS wedding_date,
            b.event_type, b.guest_count, b.total_price::float8 AS total_price,
            b.deposit_paid::float8 AS deposit_paid, b.customer_first_name,
            b.customer_last_name, b.customer_phone, b.customer_email,
            b.special_requests, b.created_at, p.title AS product_title
         FROM core_bookings b
         LEFT JOIN vendor_products p ON b.product_id = p.id
         WHERE b.id = $1 AND b.vendor_id = $2",
    )
    .bind(booking_id)
    .bind(vendor_user_uuid)
    .fetch_optional(&mut *rls_tx.tx)
    .await?;

    let row =
        row.ok_or_else(|| AppError::NotFound("Booking not found or access denied".to_string()))?;

    let id: Uuid = row.get("id");
    let booking_number: String = row.get("booking_number");
    let status: String = row.get("status");
    let wedding_date: String = row.get("wedding_date");
    let event_type: Option<String> = row.get("event_type");
    let guest_count: Option<i32> = row.get("guest_count");
    let total_price: f64 = row.get("total_price");
    let deposit_paid: f64 = row.get("deposit_paid");
    let first_name: Option<String> = row.get("customer_first_name");
    let last_name: Option<String> = row.get("customer_last_name");
    let phone: Option<String> = row.get("customer_phone");
    let email: Option<String> = row.get("customer_email");
    let special_requests: Option<String> = row.get("special_requests");
    let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
    let product_title: Option<String> = row.get("product_title");

    let customer_name = match (first_name, last_name) {
        (Some(f), Some(l)) => format!("{} {}", f, l),
        (Some(f), None) => f,
        _ => "Customer".to_string(),
    };

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "booking": {
            "id": id.to_string(),
            "bookingNumber": booking_number,
            "status": status,
            "weddingDate": wedding_date,
            "eventType": event_type.unwrap_or_default(),
            "guestCount": guest_count.unwrap_or(0),
            "totalPrice": total_price,
            "depositPaid": deposit_paid,
            "customerName": customer_name,
            "customerPhone": phone.unwrap_or_default(),
            "customerEmail": email.unwrap_or_default(),
            "specialRequests": special_requests.unwrap_or_default(),
            "productTitle": product_title.unwrap_or_default(),
            "createdAt": created_at.to_rfc3339(),
        }
    })))
}

async fn get_vendor_booking_calendar(
    auth: RequireVendor,
    mut rls_tx: RlsTx,
) -> Result<Json<Value>, AppError> {
    let vendor_user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid vendor ID in session".to_string()))?;

    let rows = sqlx::query(
        "SELECT 
            id, booking_number, status, wedding_date::text AS wedding_date,
            customer_first_name, customer_last_name
         FROM core_bookings
         WHERE vendor_id = $1 AND status != 'cancelled'
         ORDER BY wedding_date ASC",
    )
    .bind(vendor_user_uuid)
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let events: Vec<Value> = rows
        .iter()
        .map(|row| {
            let id: Uuid = row.get("id");
            let booking_number: String = row.get("booking_number");
            let status: String = row.get("status");
            let wedding_date: String = row.get("wedding_date");
            let first_name: Option<String> = row.get("customer_first_name");
            let title = format!(
                "{} ({})",
                first_name.as_deref().unwrap_or("Booking"),
                booking_number
            );

            json!({
                "id": id.to_string(),
                "title": title,
                "date": wedding_date,
                "status": status,
            })
        })
        .collect();

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "events": events
    })))
}

async fn check_vendor_availability(
    auth: RequireVendor,
    mut rls_tx: RlsTx,
    Json(payload): Json<CheckAvailabilityRequest>,
) -> Result<Json<Value>, AppError> {
    let vendor_user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid vendor ID in session".to_string()))?;

    let parsed_date = chrono::NaiveDate::parse_from_str(&payload.wedding_date, "%Y-%m-%d")
        .map_err(|_| {
            AppError::BadRequest("Invalid wedding_date format. Expected YYYY-MM-DD".to_string())
        })?;

    let existing_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM core_bookings 
         WHERE vendor_id = $1 
           AND wedding_date = $2 
           AND ($3::uuid IS NULL OR product_id = $3)
           AND status IN ('confirmed', 'Confirmed', 'Escrow_Verified', 'Booking_Active')",
    )
    .bind(vendor_user_uuid)
    .bind(parsed_date)
    .bind(payload.product_id)
    .fetch_one(&mut *rls_tx.tx)
    .await?;

    let is_available = existing_count == 0;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "isAvailable": is_available,
        "conflictCount": existing_count
    })))
}

#[derive(Deserialize)]
struct CreateVendorQuotationRequest {
    #[serde(rename = "bookingId")]
    booking_id: Uuid,
    #[serde(rename = "totalPrice")]
    total_price: f64,
    #[serde(rename = "depositAmount")]
    deposit_amount: f64,
    #[serde(rename = "expiresAt")]
    expires_at: Option<String>,
    notes: Option<String>,
}

async fn create_vendor_quotation(
    auth: RequireVendor,
    mut rls_tx: RlsTx,
    Json(payload): Json<CreateVendorQuotationRequest>,
) -> Result<Json<Value>, AppError> {
    let vendor_user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid vendor ID format".to_string()))?;

    let row =
        sqlx::query("SELECT client_id FROM core_bookings WHERE id = $1 AND vendor_id = $2")
            .bind(payload.booking_id)
            .bind(vendor_user_uuid)
            .fetch_optional(&mut *rls_tx.tx)
            .await?;

    let row =
        row.ok_or_else(|| AppError::NotFound("Booking not found or access denied".to_string()))?;
    let client_id: Uuid = row.get("client_id");

    let parsed_expires = match payload.expires_at {
        Some(d) => {
            chrono::NaiveDateTime::parse_from_str(&format!("{} 23:59:59", d), "%Y-%m-%d %H:%M:%S")
                .map(|dt| {
                    chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc)
                })
                .ok()
        }
        None => None,
    };

    let quotation_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO booking_quotations (id, booking_id, client_id, vendor_id, current_version, total_price, deposit_amount, status, expires_at, notes)
         VALUES ($1, $2, $3, $4, 1, $5, $6, 'Pending_Client_Review', $7, $8)"
    )
    .bind(quotation_id)
    .bind(payload.booking_id)
    .bind(client_id)
    .bind(vendor_user_uuid)
    .bind(payload.total_price)
    .bind(payload.deposit_amount)
    .bind(parsed_expires)
    .bind(&payload.notes)
    .execute(&mut *rls_tx.tx)
    .await?;

    sqlx::query(
        "INSERT INTO booking_quotation_revisions (quotation_id, version, total_price, deposit_amount, sender_role, notes)
         VALUES ($1, 1, $2, $3, 'Vendor', $4)"
    )
    .bind(quotation_id)
    .bind(payload.total_price)
    .bind(payload.deposit_amount)
    .bind(&payload.notes)
    .execute(&mut *rls_tx.tx)
    .await?;

    sqlx::query(
        "UPDATE core_bookings SET status = 'Quote_Sent', updated_at = NOW() WHERE id = $1",
    )
    .bind(payload.booking_id)
    .execute(&mut *rls_tx.tx)
    .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Quotation created and sent to client successfully",
        "quotationId": quotation_id.to_string()
    })))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TransitionBookingRequest {
    #[serde(rename = "toStatus")]
    to_status: String,
}

async fn transition_vendor_booking(
    auth: RequireVendor,
    mut rls_tx: RlsTx,
    axum::extract::Path(booking_id): axum::extract::Path<Uuid>,
    Json(payload): Json<TransitionBookingRequest>,
) -> Result<Json<Value>, AppError> {
    let vendor_user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid vendor ID in session".to_string()))?;

    // 1. Fetch current booking under vendor RLS context
    let row = sqlx::query("SELECT status, client_id, booking_number FROM core_bookings WHERE id = $1 AND vendor_id = $2")
        .bind(booking_id)
        .bind(vendor_user_uuid)
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
    let client_id: Uuid = booking.get("client_id");
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
        "Escrow_Verified" => {
            if current_status != "Pending_Vendor_Acceptance" && current_status != "pending" {
                return Err(AppError::BadRequest(format!(
                    "Invalid transition: Cannot move from {} to Escrow_Verified",
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
                "Invalid transition: Vendors are not authorized to transition to {}",
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
                .bind(vendor_user_uuid)
                .fetch_optional(&mut *rls_tx.tx)
                .await?;

        let _ = sqlx::query(
            "INSERT INTO system_events (user_id, target_vendor_id, event_type, message_ar, message_en)
             VALUES ($1, $2, 'booking_cancelled', $3, $4)"
        )
        .bind(client_id)
        .bind(target_vendor_profile_id)
        .bind(format!("تم إلغاء الحجز رقم «{}» من قبل المورد", booking_number))
        .bind(format!("Booking '{}' has been cancelled by the vendor", booking_number))
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
