use crate::errors::AppError;
use crate::middleware::auth::{RequireAdmin, RlsTx};
use crate::state::AppState;
use axum::{
    extract::Path,
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};
use uuid::Uuid;
use sqlx::Row;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AdminTransitionRequest {
    #[serde(rename = "toStatus")]
    pub to_status: String,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/bookings", get(list_bookings))
        .route("/bookings/:id/transition", post(transition_admin_booking))
}

async fn transition_admin_booking(
    auth: RequireAdmin,
    mut rls_tx: RlsTx,
    Path(booking_id): Path<Uuid>,
    Json(payload): Json<AdminTransitionRequest>,
) -> Result<Json<Value>, AppError> {
    // 1. Fetch current booking status
    let row = sqlx::query("SELECT status, vendor_id, client_id FROM core_bookings WHERE id = $1")
        .bind(booking_id)
        .fetch_optional(&mut *rls_tx.tx)
        .await?;

    let booking = match row {
        Some(r) => r,
        None => return Err(AppError::NotFound("Booking not found".to_string())),
    };

    let current_status: String = booking.get("status");
    let vendor_id: Uuid = booking.get("vendor_id");
    let client_id: Option<Uuid> = booking.get("client_id");
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
        "Booking_Active" => {
            if current_status != "Escrow_Verified" {
                return Err(AppError::BadRequest(format!(
                    "Invalid transition: Cannot move from {} to Booking_Active",
                    current_status
                )));
            }
        }
        "confirmed" => {
            if current_status != "pending" {
                return Err(AppError::BadRequest(format!(
                    "Invalid transition: Cannot move from {} to confirmed",
                    current_status
                )));
            }
        }
        "cancelled" => {
            // Admins can cancel any non-cancelled booking at any time
        }
        _ => {
            return Err(AppError::BadRequest(format!(
                "Invalid transition: Admins are not authorized to transition to {}",
                target
            )));
        }
    }

    // 3. Update status
    sqlx::query("UPDATE core_bookings SET status = $1, updated_at = NOW() WHERE id = $2")
        .bind(target)
        .bind(booking_id)
        .execute(&mut *rls_tx.tx)
        .await?;

    let admin_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid admin user ID".to_string()))?;

    // Status transition history
    sqlx::query(
        "INSERT INTO status_history (entity_type, entity_id, old_status, new_status, changed_by, reason, vendor_id, client_id)
         VALUES ('booking', $1, $2, $3, $4, 'Booking transitioned by admin', $5, $6)",
    )
    .bind(booking_id)
    .bind(&current_status)
    .bind(target)
    .bind(admin_uuid)
    .bind(vendor_id)
    .bind(client_id)
    .execute(&mut *rls_tx.tx)
    .await?;

    // Admin audit log
    sqlx::query(
        "INSERT INTO admin_audit_logs (entity_type, entity_id, actor_id, action, before_state, after_state)
         VALUES ('booking', $1, $2, 'transition_booking', $3, $4)",
    )
    .bind(booking_id)
    .bind(admin_uuid)
    .bind(json!({ "status": current_status }))
    .bind(json!({ "status": target }))
    .execute(&mut *rls_tx.tx)
    .await?;

    // Commit transaction
    rls_tx
        .tx
        .commit()
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    tracing::info!(
        target: "audit",
        actor_id  = %auth.user_id,
        event     = "booking_status_transitioned",
        target_id = %booking_id,
        prev      = %current_status,
        next      = %target,
        "Admin booking status mutation committed"
    );

    Ok(Json(json!({
        "status": "success",
        "message": format!("Booking status transitioned successfully by Admin to {}", target),
        "bookingId": booking_id,
        "newStatus": target
    })))
}


#[derive(serde::Deserialize)]
pub struct AdminBookingsQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub status: Option<String>,
    pub search: Option<String>,
}

async fn list_bookings(
    _auth: RequireAdmin,
    mut rls_tx: RlsTx,
    axum::extract::Query(query): axum::extract::Query<AdminBookingsQuery>,
) -> Result<Json<Value>, AppError> {
    tracing::info!("Admin listing escrow bookings...");

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(10).clamp(1, 100);
    let offset = (page - 1) * limit;

    let mut query_builder = String::from(
        "FROM core_bookings b
                                          LEFT JOIN vendor_products p ON b.product_id = p.id
                                          LEFT JOIN vendors v ON b.vendor_id = v.user_id
                                          WHERE 1 = 1",
    );
    let mut param_idx = 1;
    let mut bindings = Vec::new();

    if let Some(ref status_filter) = query.status {
        if status_filter != "all" && !status_filter.trim().is_empty() {
            query_builder.push_str(&format!(" AND b.status = ${}", param_idx));
            bindings.push(status_filter.clone());
            param_idx += 1;
        }
    }

    if let Some(ref search_term) = query.search {
        let clean_search = search_term.trim();
        if !clean_search.is_empty() {
            query_builder.push_str(&format!(
                " AND (b.booking_number ILIKE ${} OR b.customer_first_name ILIKE ${} OR b.customer_last_name ILIKE ${} OR v.name_en ILIKE ${} OR v.name_ar ILIKE ${})",
                param_idx, param_idx, param_idx, param_idx, param_idx
            ));
            bindings.push(format!("%{}%", clean_search));
            param_idx += 1;
        }
    }

    // 1. Query total count
    let count_query = format!("SELECT COUNT(*)::bigint {}", query_builder);
    let mut sql_count = sqlx::query_scalar(&count_query);
    for b in &bindings {
        sql_count = sql_count.bind(b);
    }
    let total_count: i64 = sql_count.fetch_one(&mut *rls_tx.tx).await?;

    // 2. Query paginated records
    let select_query = format!(
        "SELECT 
            b.id, b.booking_number, b.status, b.wedding_date::text, b.event_type, b.guest_count,
            b.total_price::float8, b.deposit_paid::float8, b.created_at::text,
            b.customer_first_name, b.customer_last_name, b.customer_email, b.customer_phone,
            p.title AS product_title,
            v.name_en AS vendor_name_en, v.name_ar AS vendor_name_ar
         {}
         ORDER BY b.created_at DESC
         LIMIT ${} OFFSET ${}",
        query_builder,
        param_idx,
        param_idx + 1
    );

    let mut sql_select = sqlx::query(&select_query);
    for b in &bindings {
        sql_select = sql_select.bind(b);
    }
    sql_select = sql_select.bind(limit).bind(offset);
    let rows = sql_select.fetch_all(&mut *rls_tx.tx).await?;

    let mut bookings_list = Vec::new();
    for row in rows {
        let id: Uuid = row.get("id");
        let booking_number: String = row.get("booking_number");
        let status: String = row.get("status");
        let wedding_date: String = row.get("wedding_date");
        let event_type: String = row.get("event_type");
        let guest_count: i32 = row.get("guest_count");
        let total_price: f64 = row.get("total_price");
        let deposit_paid: f64 = row.get("deposit_paid");
        let created_at: String = row.get("created_at");
        let customer_first_name: String = row.get("customer_first_name");
        let customer_last_name: String = row.get("customer_last_name");
        let customer_email: String = row.get("customer_email");
        let customer_phone: String = row.get("customer_phone");
        let product_title: Option<String> = row.get("product_title");
        let vendor_name_en: Option<String> = row.get("vendor_name_en");
        let vendor_name_ar: Option<String> = row.get("vendor_name_ar");

        bookings_list.push(json!({
            "id": id.to_string(),
            "bookingNumber": booking_number,
            "status": status,
            "weddingDate": wedding_date,
            "eventType": event_type,
            "guestCount": guest_count,
            "totalPrice": total_price,
            "depositPaid": deposit_paid,
            "createdAt": created_at,
            "customerFirstName": customer_first_name,
            "customerLastName": customer_last_name,
            "customerEmail": customer_email,
            "customerPhone": customer_phone,
            "productTitle": product_title.unwrap_or_default(),
            "vendorNameEn": vendor_name_en.unwrap_or_default(),
            "vendorNameAr": vendor_name_ar.unwrap_or_default(),
        }));
    }

    rls_tx.tx.commit().await?;

    let total_pages = ((total_count + limit - 1) / limit).max(1);

    Ok(Json(json!({
        "status": "success",
        "bookings": bookings_list,
        "total": total_count,
        "page": page,
        "totalPages": total_pages
    })))
}



// ─── ADMIN AUDIT LOGS (TASK 2) ────────────────────────────────────────────────

