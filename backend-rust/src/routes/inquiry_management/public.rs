use crate::errors::AppError;
use crate::middleware::auth::{RequireAuth, RlsTx};
use crate::state::AppState;
use crate::utils::sanitize::{limits, sanitize_opt, sanitize_str};
use axum::{
    extract::{State},
    routing::post,
    Json, Router,
};
use serde::{Deserialize};
use serde_json::{json, Value};
use uuid::Uuid;
use sqlx::Row;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/inquiries", post(submit_vendor_inquiry))
        .route("/inquiries/guest", post(submit_guest_inquiry))
        .route("/afrah/inquiry", post(submit_afrah_inquiry))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitVendorInquiryRequest {
    pub vendor_id: Option<Uuid>, // vendors.id (accepted for backward compat or direct vendor inquiries)
    pub listing_id: Option<Uuid>, // vendor_products.id — the specific listing the client viewed
    pub event_date: String,
    pub guest_count: i32,
    pub message: String,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}

/// POST /api/v1/public/inquiries
/// Submits an inquiry to a vendor, optionally scoped to a specific listing.
/// Requires client authentication.
async fn submit_vendor_inquiry(
    auth: RequireAuth,
    State(state): State<AppState>,
    mut rls_tx: RlsTx,
    Json(payload): Json<SubmitVendorInquiryRequest>,
) -> Result<Json<Value>, AppError> {
    let client_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid client ID".to_string()))?;

    // 1. Structural Validation
    let date =
        chrono::NaiveDate::parse_from_str(&payload.event_date, "%Y-%m-%d").map_err(|_| {
            AppError::BadRequest("Invalid event date format. Use YYYY-MM-DD".to_string())
        })?;

    if date <= chrono::Utc::now().date_naive() {
        return Err(AppError::BadRequest(
            "Event date must be in the future".to_string(),
        ));
    }

    if payload.guest_count <= 0 {
        return Err(AppError::BadRequest(
            "Guest count must be greater than zero".to_string(),
        ));
    }

    // 2. Resolve target vendor & product (listing) details
    // We support both direct vendor inquiry (legacy) and listing-centric inquiry.
    let (vendor_id, vendor_user_id, listing_title) = match (payload.listing_id, payload.vendor_id) {
        (Some(listing_id), _) => {
            let row = sqlx::query(
                "SELECT p.vendor_id, v.user_id, p.title_en \
                 FROM vendor_products p \
                 JOIN vendors v ON p.vendor_id = v.id \
                 WHERE p.id = $1"
            )
            .bind(listing_id)
            .fetch_optional(&mut *rls_tx.tx)
            .await?;

            match row {
                Some(r) => (
                    r.get::<Uuid, _>("vendor_id"),
                    r.get::<Uuid, _>("user_id"),
                    r.get::<String, _>("title_en"),
                ),
                None => return Err(AppError::NotFound("Listing not found".to_string())),
            }
        }
        (None, Some(vendor_id)) => {
            let row = sqlx::query("SELECT user_id, name_en FROM vendors WHERE id = $1")
                .bind(vendor_id)
                .fetch_optional(&mut *rls_tx.tx)
                .await?;

            match row {
                Some(r) => (
                    vendor_id,
                    r.get::<Uuid, _>("user_id"),
                    "Direct Vendor Inquiry".to_string(),
                ),
                None => return Err(AppError::NotFound("Vendor not found".to_string())),
            }
        }
        (None, None) => {
            return Err(AppError::BadRequest(
                "Either listingId or vendorId is required to submit an inquiry".to_string(),
            ))
        }
    };

    // Get customer contact info from profile if not provided in payload
    let customer_name = match payload.name {
        Some(n) => sanitize_str(&n, limits::NAME_SHORT),
        None => {
            let name_row = sqlx::query("SELECT first_name, last_name FROM client_profiles WHERE client_id = $1")
                .bind(client_uuid)
                .fetch_optional(&mut *rls_tx.tx)
                .await?;
            match name_row {
                Some(r) => format!(
                    "{} {}",
                    r.get::<Option<String>, _>("first_name").unwrap_or_default(),
                    r.get::<Option<String>, _>("last_name").unwrap_or_default()
                ).trim().to_string(),
                None => "Client".to_string(),
            }
        }
    };

    let customer_phone = match payload.phone {
        Some(p) => sanitize_str(&p, limits::PHONE),
        None => {
            sqlx::query_scalar("SELECT phone FROM client_profiles WHERE client_id = $1")
                .bind(client_uuid)
                .fetch_optional(&mut *rls_tx.tx)
                .await?
                .unwrap_or_default()
        }
    };

    let customer_email = match payload.email {
        Some(e) => sanitize_str(&e, limits::EMAIL),
        None => {
            sqlx::query_scalar("SELECT email FROM users WHERE id = $1")
                .bind(client_uuid)
                .fetch_optional(&mut *rls_tx.tx)
                .await?
                .unwrap_or_default()
        }
    };

    let clean_message = sanitize_str(&payload.message, limits::MESSAGE);
    let new_id = Uuid::new_v4();

    // 3. Create core inquiry row
    sqlx::query(
        "INSERT INTO inquiries (id, client_id, vendor_id, product_id, event_date, guest_count, message, status) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending')"
    )
    .bind(new_id)
    .bind(client_uuid)
    .bind(vendor_id)
    .bind(payload.listing_id)
    .bind(date)
    .bind(payload.guest_count)
    .bind(&clean_message)
    .execute(&mut *rls_tx.tx)
    .await?;

    // 4. Resolve vendor email & configuration parameters
    let vendor_email: String = sqlx::query_scalar("SELECT email FROM users WHERE id = $1")
        .bind(vendor_user_id)
        .fetch_one(&mut *rls_tx.tx)
        .await?;

    let vendor_phone: String = sqlx::query_scalar("SELECT phone FROM vendors WHERE id = $1")
        .bind(vendor_id)
        .fetch_one(&mut *rls_tx.tx)
        .await?;

    // 5. Build transactional message outbox payload
    let outbox_payload = serde_json::json!({
        "vendor_email": vendor_email,
        "vendor_phone": vendor_phone,
        "customer_name": customer_name,
        "customer_phone": customer_phone,
        "customer_email": customer_email,
        "event_date": payload.event_date,
        "guest_count": payload.guest_count,
        "message": clean_message,
        "listing_title": listing_title,
        "is_whatsapp": true
    });

    sqlx::query(
        "INSERT INTO notification_outbox ( \
            event_type, aggregate_type, aggregate_id, payload, status, attempt_count, next_retry_at \
         ) VALUES ('new_inquiry', 'inquiry', $1, $2, 'PENDING', 0, NOW())"
    )
    .bind(new_id)
    .bind(outbox_payload)
    .execute(&mut *rls_tx.tx)
    .await?;

    // 6. Automatically provision Chat Thread between client & vendor user
    // First check if a conversation already exists
    let existing_conv: Option<Uuid> = sqlx::query_scalar(
        "SELECT c.id FROM conversations c \
         JOIN conversation_participants p1 ON c.id = p1.conversation_id \
         JOIN conversation_participants p2 ON c.id = p2.conversation_id \
         WHERE p1.user_id = $1 AND p2.user_id = $2"
    )
    .bind(client_uuid)
    .bind(vendor_user_id)
    .fetch_optional(&mut *rls_tx.tx)
    .await?;

    let conversation_id = match existing_conv {
        Some(cid) => cid,
        None => {
            let cid = Uuid::new_v4();
            sqlx::query("INSERT INTO conversations (id) VALUES ($1)")
                .bind(cid)
                .execute(&mut *rls_tx.tx)
                .await?;

            sqlx::query("INSERT INTO conversation_participants (conversation_id, user_id) VALUES ($1, $2)")
                .bind(cid)
                .bind(client_uuid)
                .execute(&mut *rls_tx.tx)
                .await?;

            sqlx::query("INSERT INTO conversation_participants (conversation_id, user_id) VALUES ($1, $2)")
                .bind(cid)
                .bind(vendor_user_id)
                .execute(&mut *rls_tx.tx)
                .await?;

            cid
        }
    };

    // Post system-generated initial context message detailing inquiry scope
    let initial_msg_body = format!(
        "System: New inquiry received for '{}' on {}. Guest Count: {}. Message: '{}'",
        listing_title, payload.event_date, payload.guest_count, clean_message
    );

    let msg_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO messages (id, conversation_id, sender_id, body) \
         VALUES ($1, $2, $3, $4)"
    )
    .bind(msg_id)
    .bind(conversation_id)
    .bind(client_uuid)
    .bind(&initial_msg_body)
    .execute(&mut *rls_tx.tx)
    .await?;

    rls_tx.tx.commit().await?;

    // Publish event asynchronously to telemetry outbox
    let _ = crate::routes::telemetry_diagnostics::events::publish_telemetry_event(
        &state.db,
        "inquiry_created",
        serde_json::json!({
            "inquiry_id": new_id.to_string(),
            "client_id": client_uuid.to_string(),
            "vendor_id": vendor_id.to_string(),
            "body": initial_msg_body.to_string(),
            "temp_id": None::<String>,
            "attachments": vec![String::new(); 0],
            "participant_ids": vec![client_uuid, vendor_user_id],
            "created_at": chrono::Utc::now(),
        })
    ).await;

    tracing::info!(
        target: "audit",
        action = "submit_inquiry",
        client_id = %client_uuid,
        vendor_id = %vendor_id,
        "Auth inquiry submitted"
    );

    Ok(Json(json!({
        "status": "success",
        "message": "Lead inquiry submitted successfully",
        "id": new_id.to_string(),
        "conversationId": conversation_id.to_string(),
        "listingId": payload.listing_id
    })))
}

/// POST /api/v1/public/inquiries/guest
/// Submits an inquiry to a vendor as a guest (auth-less).
async fn submit_guest_inquiry(
    State(state): State<AppState>,
    crate::utils::ip::SecureClientIp(client_ip): crate::utils::ip::SecureClientIp,
    Json(payload): Json<SubmitVendorInquiryRequest>,
) -> Result<Json<Value>, AppError> {
    // Strict Rate Limiting via DashMap — 5 inquiries per guest IP per hour
    let ip = client_ip.to_string();
    let rl_key = format!("guest_inquiry_{}", ip);
    let mut allowed = false;
    let refill_rate: f64 = 5.0 / 3600.0; // 5 tokens per hour (in tokens/second)
    let max_tokens: f64 = 5.0;

    if ip == "127.0.0.1"
        || client_ip.is_loopback()
        || state.config.app_environment == "development"
    {
        allowed = true;
    } else if let Some(mut bucket) = state.rate_limit_store.get_mut(&rl_key) {
        let elapsed = bucket.last_refreshed.elapsed().as_secs_f64();
        let mut current_tokens = bucket.tokens + elapsed * refill_rate;
        if current_tokens > max_tokens {
            current_tokens = max_tokens;
        }
        if current_tokens >= 1.0 {
            bucket.tokens = current_tokens - 1.0;
            bucket.last_refreshed = std::time::Instant::now();
            allowed = true;
        } else {
            bucket.tokens = current_tokens;
            bucket.last_refreshed = std::time::Instant::now();
        }
    } else {
        state.rate_limit_store.insert(
            rl_key.clone(),
            crate::middleware::rate_limit::TokenBucket {
                tokens: max_tokens - 1.0,
                last_refreshed: std::time::Instant::now(),
            },
        );
        allowed = true;
    }

    if !allowed {
        tracing::warn!(target: "security", ip = %ip, "Guest inquiry rate limit exceeded");
        return Err(AppError::TooManyRequests(
            "Rate limit exceeded for guest inquiries".to_string(),
        ));
    }

    // Structural Validation
    let date =
        chrono::NaiveDate::parse_from_str(&payload.event_date, "%Y-%m-%d").map_err(|_| {
            AppError::BadRequest("Invalid event date format. Use YYYY-MM-DD".to_string())
        })?;

    if date <= chrono::Utc::now().date_naive() {
        return Err(AppError::BadRequest(
            "Event date must be in the future".to_string(),
        ));
    }

    if payload.guest_count <= 0 {
        return Err(AppError::BadRequest(
            "Guest count must be greater than zero".to_string(),
        ));
    }

    // Guest inquiries require explicit contact details
    let raw_name = payload.name.ok_or_else(|| {
        AppError::BadRequest("Name is required for guest inquiries".to_string())
    })?;
    let raw_phone = payload.phone.ok_or_else(|| {
        AppError::BadRequest("Phone is required for guest inquiries".to_string())
    })?;
    let raw_email = payload.email.ok_or_else(|| {
        AppError::BadRequest("Email is required for guest inquiries".to_string())
    })?;

    let clean_name = sanitize_str(&raw_name, limits::NAME_SHORT);
    let clean_phone = sanitize_str(&raw_phone, limits::PHONE);
    let clean_email = sanitize_str(&raw_email, limits::EMAIL);

    if clean_name.len() < 3 {
        return Err(AppError::BadRequest(
            "Name must be at least 3 characters".to_string(),
        ));
    }

    // Resolve target vendor & product (listing) details
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let (vendor_id, vendor_user_id, listing_title) = match (payload.listing_id, payload.vendor_id) {
        (Some(listing_id), _) => {
            let row = sqlx::query(
                "SELECT p.vendor_id, v.user_id, p.title_en \
                 FROM vendor_products p \
                 JOIN vendors v ON p.vendor_id = v.id \
                 WHERE p.id = $1"
            )
            .bind(listing_id)
            .fetch_optional(&mut *tx)
            .await?;

            match row {
                Some(r) => (
                    r.get::<Uuid, _>("vendor_id"),
                    r.get::<Uuid, _>("user_id"),
                    r.get::<String, _>("title_en"),
                ),
                None => return Err(AppError::NotFound("Listing not found".to_string())),
            }
        }
        (None, Some(vendor_id)) => {
            let row = sqlx::query("SELECT user_id, name_en FROM vendors WHERE id = $1")
                .bind(vendor_id)
                .fetch_optional(&mut *tx)
                .await?;

            match row {
                Some(r) => (
                    vendor_id,
                    r.get::<Uuid, _>("user_id"),
                    "Direct Vendor Inquiry".to_string(),
                ),
                None => return Err(AppError::NotFound("Vendor not found".to_string())),
            }
        }
        (None, None) => {
            return Err(AppError::BadRequest(
                "Either listingId or vendorId is required to submit an inquiry".to_string(),
            ))
        }
    };

    let clean_message = sanitize_str(&payload.message, limits::MESSAGE);
    let new_id = Uuid::new_v4();

    // Create guest inquiry row (no client_id, status = 'pending')
    sqlx::query(
        "INSERT INTO inquiries (id, vendor_id, product_id, event_date, guest_count, message, status, guest_name, guest_phone, guest_email) \
         VALUES ($1, $2, $3, $4, $5, $6, 'pending', $7, $8, $9)"
    )
    .bind(new_id)
    .bind(vendor_id)
    .bind(payload.listing_id)
    .bind(date)
    .bind(payload.guest_count)
    .bind(&clean_message)
    .bind(&clean_name)
    .bind(&clean_phone)
    .bind(&clean_email)
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // Resolve vendor email & phone
    let vendor_email: String = sqlx::query_scalar("SELECT email FROM users WHERE id = $1")
        .bind(vendor_user_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let vendor_phone: String = sqlx::query_scalar("SELECT phone FROM vendors WHERE id = $1")
        .bind(vendor_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Build transactional message outbox payload
    let outbox_payload = serde_json::json!({
        "vendor_email": vendor_email,
        "vendor_phone": vendor_phone,
        "customer_name": format!("{} (Guest)", clean_name),
        "customer_phone": clean_phone,
        "customer_email": clean_email,
        "event_date": payload.event_date,
        "guest_count": payload.guest_count,
        "message": clean_message,
        "listing_title": listing_title,
        "is_whatsapp": true
    });

    sqlx::query(
        "INSERT INTO notification_outbox ( \
            event_type, aggregate_type, aggregate_id, payload, status, attempt_count, next_retry_at \
         ) VALUES ('new_inquiry', 'inquiry', $1, $2, 'PENDING', 0, NOW())"
    )
    .bind(new_id)
    .bind(outbox_payload)
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    tracing::info!(
        target: "audit",
        action = "submit_guest_inquiry",
        vendor_id = %vendor_user_id,
        ip = %ip,
        "Guest inquiry submitted securely"
    );

    Ok(Json(json!({
        "status": "success",
        "message": "Guest inquiry submitted successfully"
    })))
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitAfrahInquiryRequest {
    pub name: String,
    pub phone: String,
    pub is_whatsapp: bool,
    pub event_date: String,
    #[serde(default)]
    pub message: String,
    pub email: Option<String>,
}

/// POST /api/v1/public/afrah/inquiry
/// Submits an inquiry to the Afrah concierge service. No authentication required.
pub(crate) async fn submit_afrah_inquiry(
    State(state): State<AppState>,
    crate::utils::ip::SecureClientIp(client_ip): crate::utils::ip::SecureClientIp,
    Json(payload): Json<SubmitAfrahInquiryRequest>,
) -> Result<Json<Value>, AppError> {
    // Rate Limiting — 5 afrah inquiries per IP per hour
    let ip = client_ip.to_string();
    let rl_key = format!("afrah_inquiry_{}", ip);
    let mut allowed = false;
    let refill_rate: f64 = 5.0 / 3600.0;
    let max_tokens: f64 = 5.0;

    if ip == "127.0.0.1"
        || client_ip.is_loopback()
        || state.config.app_environment == "development"
    {
        allowed = true;
    } else if let Some(mut bucket) = state.rate_limit_store.get_mut(&rl_key) {
        let elapsed = bucket.last_refreshed.elapsed().as_secs_f64();
        let mut current_tokens = bucket.tokens + elapsed * refill_rate;
        if current_tokens > max_tokens {
            current_tokens = max_tokens;
        }
        if current_tokens >= 1.0 {
            bucket.tokens = current_tokens - 1.0;
            bucket.last_refreshed = std::time::Instant::now();
            allowed = true;
        } else {
            bucket.tokens = current_tokens;
            bucket.last_refreshed = std::time::Instant::now();
        }
    } else {
        state.rate_limit_store.insert(
            rl_key.clone(),
            crate::middleware::rate_limit::TokenBucket {
                tokens: max_tokens - 1.0,
                last_refreshed: std::time::Instant::now(),
            },
        );
        allowed = true;
    }

    if !allowed {
        tracing::warn!(target: "security", ip = %ip, "Afrah inquiry rate limit exceeded");
        return Err(AppError::TooManyRequests(
            "Rate limit exceeded for Afrah inquiries".to_string(),
        ));
    }

    // Validation
    let clean_name = sanitize_str(&payload.name, limits::NAME_SHORT);
    if clean_name.len() < 3 {
        return Err(AppError::BadRequest(
            "Name must be at least 3 characters".to_string(),
        ));
    }

    let clean_phone = sanitize_str(&payload.phone, limits::PHONE);
    if clean_phone.is_empty() {
        return Err(AppError::BadRequest(
            "Phone number is required".to_string(),
        ));
    }

    let event_date =
        chrono::NaiveDate::parse_from_str(&payload.event_date, "%Y-%m-%d").map_err(|_| {
            AppError::BadRequest("Invalid event date format. Use YYYY-MM-DD".to_string())
        })?;

    if event_date <= chrono::Utc::now().date_naive() {
        return Err(AppError::BadRequest(
            "Event date must be in the future".to_string(),
        ));
    }

    let clean_message = if payload.message.is_empty() {
        "Wedding planning request via Afrah concierge".to_string()
    } else {
        sanitize_str(&payload.message, limits::MESSAGE)
    };
    let clean_email = sanitize_opt(&payload.email, limits::EMAIL);

    let new_id = Uuid::new_v4();

    // Insert into afrah_inquiries table
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    sqlx::query(
        "INSERT INTO afrah_inquiries (id, name, phone, is_whatsapp, event_date, message, email, status, ip_address) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending', $8)",
    )
    .bind(new_id)
    .bind(&clean_name)
    .bind(&clean_phone)
    .bind(payload.is_whatsapp)
    .bind(event_date)
    .bind(&clean_message)
    .bind(&clean_email)
    .bind(&ip)
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // Queue notification via outbox — email to afrah@zafafworld.net + WhatsApp
    let afrah_email = "afrah@zafafworld.net";
    let outbox_payload = serde_json::json!({
        "vendor_email": afrah_email,
        "vendor_phone": state.config.afrah_notification_phone.clone(),
        "customer_name": clean_name,
        "customer_phone": clean_phone,
        "customer_email": clean_email.as_deref().unwrap_or(""),
        "event_date": payload.event_date,
        "guest_count": 0,
        "message": clean_message,
        "listing_title": "Afrah Concierge Request",
        "is_whatsapp": payload.is_whatsapp,
    });

    sqlx::query(
        "INSERT INTO notification_outbox ( \
            event_type, aggregate_type, aggregate_id, payload, status, attempt_count, next_retry_at \
         ) VALUES ('new_inquiry', 'afrah_inquiry', $1, $2, 'PENDING', 0, NOW())",
    )
    .bind(new_id)
    .bind(outbox_payload)
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    tracing::info!(
        target: "audit",
        action = "submit_afrah_inquiry",
        inquiry_id = %new_id,
        ip = %ip,
        "Afrah concierge inquiry submitted"
    );

    Ok(Json(json!({
        "status": "success",
        "message": "Afrah inquiry submitted successfully",
        "tracking_id": format!("AFR-{}", &new_id.to_string()[..8].to_uppercase())
    })))
}
