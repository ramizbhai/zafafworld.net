use crate::errors::AppError;
use crate::middleware::auth::{RequireAuth, RlsTx};
use crate::state::AppState;
use crate::utils::sanitize::{limits, sanitize_opt, sanitize_str};
use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use serde_json::{json, Value};
use uuid::Uuid;
use sqlx::Row;
use serde::Deserialize;
use rust_decimal::Decimal;
use rust_decimal::RoundingStrategy;
use rust_decimal_macros::dec;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/bookings",
            post(create_booking).layer(axum::middleware::from_fn_with_state(
                state.clone(),
                crate::middleware::idempotency::idempotent_gate_middleware,
            )),
        )
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateBookingInput {
    /// New listing-centric field — the vendor_products.id of the specific hall/listing.
    pub listing_id: Uuid,
    pub event_date: String,
    pub event_type: String,
    pub guest_count: i32,
    pub special_requests: Option<String>,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
}

async fn create_booking(
    State(state): State<AppState>,
    auth: RequireAuth,
    mut rls_tx: RlsTx,
    Json(input): Json<CreateBookingInput>,
) -> Result<Json<Value>, AppError> {
    tracing::info!(
        "Received listing-centric booking request for listing_id: {}",
        input.listing_id
    );

    let client_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid client ID in session".to_string()))?;

    // 1. Structural Validation
    let date = chrono::NaiveDate::parse_from_str(&input.event_date, "%Y-%m-%d").map_err(|_| {
        AppError::BadRequest("Invalid event date format. Use YYYY-MM-DD".to_string())
    })?;

    let today = chrono::Utc::now().date_naive();
    if date <= today {
        return Err(AppError::BadRequest(
            "Wedding date must be in the future".to_string(),
        ));
    }

    if input.guest_count <= 0 {
        return Err(AppError::BadRequest(
            "Guest count must be greater than zero".to_string(),
        ));
    }

    if input.first_name.trim().is_empty() || input.last_name.trim().is_empty() {
        return Err(AppError::BadRequest(
            "First name and last name are required".to_string(),
        ));
    }

    if input.email.trim().is_empty() || !input.email.contains('@') {
        return Err(AppError::BadRequest(
            "A valid email address is required".to_string(),
        ));
    }

    if input.phone.trim().is_empty() {
        return Err(AppError::BadRequest("Phone number is required".to_string()));
    }

    // 2. Resolve listing → vendor. This is the core of the new listing-centric flow.
    // We look up the vendor_product (listing) and join to the vendor account to get
    // the vendor_user_id needed for core_bookings.vendor_id.
    let listing_row = sqlx::query(
        "SELECT 
            vp.id AS listing_id,
            vp.vendor_id,
            vp.title,
            vp.base_price_sar,
            vp.deposit_percentage,
            COALESCE(vp.attributes->>'genderSection', vp.attributes->>'gender_section') AS gender_section,
            CASE 
                WHEN (vp.attributes->'menCapacity') IS NULL AND (vp.attributes->'men_capacity') IS NULL 
                 AND (vp.attributes->'womenCapacity') IS NULL AND (vp.attributes->'women_capacity') IS NULL THEN NULL 
                ELSE GREATEST(
                    COALESCE((vp.attributes->>'menCapacity')::int, (vp.attributes->>'men_capacity')::int, 0),
                    COALESCE((vp.attributes->>'womenCapacity')::int, (vp.attributes->>'women_capacity')::int, 0),
                    COALESCE((vp.attributes->>'menCapacity')::int, (vp.attributes->>'men_capacity')::int, 0) + 
                    COALESCE((vp.attributes->>'womenCapacity')::int, (vp.attributes->>'women_capacity')::int, 0)
                ) 
            END AS effective_capacity_max,
            LEAST(
                NULLIF(COALESCE((vp.attributes->>'menCapacity')::int, (vp.attributes->>'men_capacity')::int, 0), 0),
                NULLIF(COALESCE((vp.attributes->>'womenCapacity')::int, (vp.attributes->>'women_capacity')::int, 0), 0)
            ) AS effective_capacity_min,
            v.user_id AS vendor_user_id,
            v.status AS vendor_status,
            COALESCE(v.city_id, vp.city_id) AS city_id
         FROM vendor_products vp
         JOIN vendors v ON vp.vendor_id = v.id
         WHERE vp.id = $1 AND vp.status = 'active' AND vp.is_available = TRUE AND v.status = 'active'"
    )
    .bind(input.listing_id)
    .fetch_optional(&mut *rls_tx.tx)
    .await?;

    let listing = match listing_row {
        Some(row) => row,
        None => {
            return Err(AppError::NotFound(
                "Listing not found, unavailable, or the vendor account is inactive.".to_string(),
            ))
        }
    };

    let vendor_user_id: Uuid = listing
        .try_get::<Option<Uuid>, &str>("vendor_user_id")
        .unwrap_or(None)
        .ok_or_else(|| AppError::Internal("Vendor account has no associated user.".to_string()))?;
    let vendor_id: Uuid = listing.get("vendor_id");
    let base_price_sar: Option<Decimal> = listing.try_get("base_price_sar").ok().flatten();
    let deposit_pct: i32 = listing.get("deposit_percentage");
    let gender_section: Option<String> = listing.get("gender_section");
    let cap_max: Option<i32> = listing.try_get("effective_capacity_max").ok().flatten();
    let cap_min: Option<i32> = listing.try_get("effective_capacity_min").ok().flatten();
    let city_id: Option<Uuid> = listing.try_get("city_id").ok().flatten();

    // Capacity guard — only enforced if the listing has capacity data
    if let Some(max) = cap_max {
        if input.guest_count > max {
            return Err(AppError::BadRequest(format!(
                "Guest count {} exceeds this listing's maximum capacity of {}.",
                input.guest_count, max
            )));
        }
    }
    if let Some(min) = cap_min {
        if min > 0 && input.guest_count < min {
            return Err(AppError::BadRequest(format!(
                "Guest count {} is below this listing's minimum capacity of {}.",
                input.guest_count, min
            )));
        }
    }

    // 3. Compute pricing — use listing's base_price_sar; fall back to packages, then default.
    let mut starting_price = base_price_sar.unwrap_or(Decimal::from(25000));

    if base_price_sar.is_none() {
        // No listing price set — look up the vendor's active packages as a fallback.
        let pkg_rows = sqlx::query(
            "SELECT discounted_price FROM packages 
             WHERE vendor_id = (SELECT vendor_id FROM vendors WHERE user_id = $1) 
             AND expiry_date >= CURRENT_DATE",
        )
        .bind(vendor_user_id)
        .fetch_all(&mut *rls_tx.tx)
        .await?;

        if !pkg_rows.is_empty() {
            let mut min_disc = Decimal::MAX;
            for row in &pkg_rows {
                let d: Decimal = row.get(0);
                if d > Decimal::ZERO && d < min_disc {
                    min_disc = d;
                }
            }
            if min_disc < Decimal::MAX {
                starting_price = min_disc;
            }
        }
    }

    let tax_rate = dec!(0.15);
    let tax = (starting_price * tax_rate)
        .round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero);
    let total_price = starting_price + tax;
    let deposit_rate = Decimal::new(deposit_pct as i64, 2); // e.g. 25 → 0.25
    let deposit_paid = (total_price * deposit_rate)
        .round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero);

    // 4. Generate booking number
    let booking_uuid = Uuid::new_v4();
    let booking_number = format!("ZF-{}", &booking_uuid.to_string()[0..8].to_uppercase());

    // 5. Sanitize inputs
    let clean_event_type = sanitize_str(&input.event_type, limits::CATEGORY);
    let clean_first_name = sanitize_str(&input.first_name, limits::NAME_SHORT);
    let clean_last_name = sanitize_str(&input.last_name, limits::NAME_SHORT);
    let clean_phone = sanitize_str(&input.phone, limits::PHONE);
    let clean_email = sanitize_str(&input.email, limits::EMAIL);
    let clean_special_requests = sanitize_opt(&input.special_requests, limits::MESSAGE);

    // Convert gender_section string to the enum for core_bookings
    let _gender_section_val = gender_section.as_deref();

    // 6. Persist
    let insert_result = sqlx::query(
        "INSERT INTO core_bookings (
            booking_number, vendor_id, product_id,
            status, wedding_date, event_type, guest_count,
            total_price, deposit_paid,
            customer_first_name, customer_last_name,
            customer_phone, customer_email,
            special_requests, client_id, city_id
        ) VALUES (
            $1, $2, $3,
            'pending', $4, $5, $6,
            $7::numeric, $8::numeric,
            $9, $10,
            $11, $12,
            $13, $14, $15
        )",
    )
    .bind(&booking_number)
    .bind(vendor_user_id)
    .bind(input.listing_id)
    .bind(date)
    .bind(&clean_event_type)
    .bind(input.guest_count)
    .bind(total_price)
    .bind(deposit_paid)
    .bind(&clean_first_name)
    .bind(&clean_last_name)
    .bind(&clean_phone)
    .bind(&clean_email)
    .bind(&clean_special_requests)
    .bind(client_uuid)
    .bind(city_id)
    .execute(&mut *rls_tx.tx)
    .await;

    match insert_result {
        Ok(_) => {
            // Event logging hook for booking creation
            let _ = sqlx::query(
                "INSERT INTO system_events (user_id, target_vendor_id, event_type, message_ar, message_en)
                 VALUES ($1, $2, 'booking_received', $3, $4)"
            )
            .bind(client_uuid)
            .bind(vendor_id)
            .bind(format!("تم إنشاء حجز جديد برقم «{}» بقيمة {} ريال", booking_number, total_price))
            .bind(format!("New booking '{}' created with total price SAR {}", booking_number, total_price))
            .execute(&mut *rls_tx.tx)
            .await;

            rls_tx
                .tx
                .commit()
                .await
                .map_err(|err| AppError::Database(err.to_string()))?;
        }
        Err(err) => {
            if let Some(db_err) = err.as_database_error() {
                if db_err.code() == Some(std::borrow::Cow::Borrowed("23505")) {
                    return Err(AppError::BadRequest(
                        "This listing is already booked for the selected date. Please choose another date.".to_string()
                    ));
                }
            }
            return Err(AppError::Database(err.to_string()));
        }
    }

    // 7. Broadcast event asynchronously
    let _ = state.booking_event_tx.send(crate::state::BookingEvent {
        booking_number: booking_number.clone(),
        client_id: client_uuid,
        vendor_id: vendor_user_id,
        total_price,
        timestamp: chrono::Utc::now(),
    });

    tracing::info!(
        "Successfully created listing-centric booking: {} for listing: {}",
        booking_number,
        input.listing_id
    );

    Ok(Json(json!({
        "status": "success",
        "bookingNumber": booking_number,
        "totalPrice": total_price,
        "depositPaid": deposit_paid,
        "weddingDate": input.event_date,
        "listingId": input.listing_id
    })))
}
