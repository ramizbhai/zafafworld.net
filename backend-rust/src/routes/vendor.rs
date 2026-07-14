use crate::errors::AppError;
use crate::state::AppState;
use crate::utils::sanitize::{limits, sanitize_opt, sanitize_str};
use axum::{
    extract::State,
    routing::{delete, get, patch, post, put},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;
use validator::Validate;

use crate::middleware::auth::{RequireVendor, RequireVendorOwner, RlsTx};
use crate::models::inquiry::{
    Inquiry, VendorInquiryDto, VendorReview, VendorStaff, VendorTask,
    VendorWhatsappTemplate,
};
use sqlx::Row;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/stats/dashboard", get(get_dashboard))
        .route("/tickets/messages", get(get_vendor_chat_messages))
        .route("/tickets/reply", post(post_vendor_chat_reply))


        .route("/gallery", get(list_gallery).post(add_gallery_image))
        .route("/gallery/:id", delete(delete_gallery_image))
        .route("/gallery/:id/cover", patch(set_cover_image))
        .route("/upload", post(upload_file))
        .route("/upload/status/:id", get(get_upload_status))
        .route("/notifications", get(list_notifications))
        .route("/notifications/read", patch(mark_notifications_read))
        .route("/reviews", get(list_reviews).post(create_review))
        .route("/reviews/:id/status", patch(update_review_status))
        .route("/tasks", get(list_tasks).post(create_task))
        .route("/tasks/:id", put(update_task).delete(delete_task))
        .route("/templates", get(list_templates).post(create_template))
        .route(
            "/templates/:id",
            put(update_template).delete(delete_template),
        )
        .route(
            "/conversations",
            get(crate::routes::conversations::list_conversations)
                .post(crate::routes::conversations::create_conversation),
        )
        .route(
            "/conversations/:id/messages",
            get(crate::routes::conversations::list_messages)
                .post(crate::routes::conversations::send_message),
        )
        .route(
            "/messages/:id/read",
            patch(crate::routes::conversations::mark_message_read),
        )

}

async fn get_dashboard(auth: RequireVendor, mut rls_tx: RlsTx) -> Result<Json<Value>, AppError> {
    let user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid user ID format in token".to_string()))?;

    // 1. Fetch vendor record for this user, including their city details
    let vendor_row = sqlx::query(
        "SELECT v.id, v.name_ar, v.name_en, v.description_ar, v.description_en, v.slug, v.category, v.status,
                v.address_ar, v.address_en, v.phone, v.email, v.capacity_min, v.capacity_max, v.latitude, v.longitude, v.amenities, v.version,
                v.crm_venue_id, v.star_rating, v.event_spaces_available, v.event_type, v.website, v.maps_url, v.video_url_1, v.subscription_status,
                v.subscription_expires_at,
                v.city_id, c.name_en AS city_name_en, c.name_ar AS city_name_ar,
                st.id AS tier_id, st.name AS tier_name, st.policy_limits
         FROM vendors v 
         LEFT JOIN cities c ON v.city_id = c.id 
         LEFT JOIN subscription_tiers st ON v.subscription_tier_id = st.id
         WHERE v.user_id = $1"
    )
    .bind(user_uuid)
    .fetch_optional(&mut *rls_tx.tx)
    .await?;

    let vendor = match vendor_row {
        Some(row) => row,
        None => {
            return Err(AppError::NotFound(
                "Vendor profile not found for this user account".to_string(),
            ))
        }
    };

    let subscription_status: String = vendor.get("subscription_status");
    if subscription_status == "stopped" {
        return Err(AppError::Status(
            axum::http::StatusCode::PAYMENT_REQUIRED,
            "subscription_stopped".to_string(),
        ));
    }

    let vendor_id: Uuid = vendor.get("id");
    let name_ar: String = vendor.get("name_ar");
    let name_en: String = vendor.get("name_en");
    let description_ar: Option<String> = vendor.get("description_ar");
    let description_en: Option<String> = vendor.get("description_en");
    let slug: String = vendor.get("slug");
    let category: Option<String> = vendor.try_get("category").ok().flatten();
    let status: String = vendor.get("status");
    let city_id: Option<Uuid> = vendor.get("city_id");
    let city_name: String = vendor
        .try_get("city_name_en")
        .unwrap_or_else(|_| "".to_string());
    let city_name_ar: String = vendor
        .try_get("city_name_ar")
        .unwrap_or_else(|_| "".to_string());

    let address_ar: Option<String> = vendor.get("address_ar");
    let address_en: Option<String> = vendor.get("address_en");
    let phone: Option<String> = vendor.get("phone");
    let email: Option<String> = vendor.get("email");
    let capacity_min: Option<i32> = vendor.get("capacity_min");
    let capacity_max: Option<i32> = vendor.get("capacity_max");
    let latitude: Option<f64> = vendor.get("latitude");
    let longitude: Option<f64> = vendor.get("longitude");
    let amenities: Option<Vec<String>> = vendor.get("amenities");
    let version: i32 = vendor.get("version");

    let subscription_expires_at: Option<chrono::DateTime<chrono::Utc>> =
        vendor.try_get("subscription_expires_at").ok();
    let (computed_tier_id, computed_limits) =
        crate::utils::policy::PolicyEngine::fetch_limits(vendor_id, &mut rls_tx.tx).await?;
    let tier_id: Option<String> = Some(computed_tier_id.clone());
    let tier_name: Option<String> = if computed_tier_id == "free" {
        Some("Free".to_string())
    } else {
        vendor.try_get("tier_name").ok()
    };
    let policy_limits: Option<Value> = Some(json!(computed_limits));

    let crm_venue_id: Option<String> = vendor.get("crm_venue_id");
    let star_rating: Option<rust_decimal::Decimal> = vendor.get("star_rating");
    let _event_spaces_available: Option<i32> = vendor.get("event_spaces_available"); // now derived from active products count
    let _event_type: Option<String> = vendor.get("event_type"); // superseded by product_category per product

    let website: Option<String> = vendor.get("website");
    let maps_url: Option<String> = vendor.get("maps_url");
    let video_url_1: Option<String> = vendor.get("video_url_1");

    // 2. Query count of packages and gallery items
    let package_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM vendor_packages WHERE vendor_id = $1")
            .bind(vendor_id)
            .fetch_one(&mut *rls_tx.tx)
            .await?;

    let gallery_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM vendor_gallery WHERE vendor_id = $1")
            .bind(vendor_id)
            .fetch_one(&mut *rls_tx.tx)
            .await?;

    // 2b. Query vendor products (halls) for the dashboard
    let product_rows = sqlx::query(
        "SELECT id, COALESCE(title_en, title_ar, title, 'Unnamed Product') AS title, slug, product_category,
                COALESCE(attributes->>'genderSection', attributes->>'gender_section', 'any') AS gender_section,
                COALESCE((attributes->>'menCapacity')::int, (attributes->>'men_capacity')::int) AS men_capacity,
                COALESCE((attributes->>'womenCapacity')::int, (attributes->>'women_capacity')::int) AS women_capacity,
                base_price_sar::float8 AS base_price_sar,
                status, is_available, is_featured
         FROM vendor_products
         WHERE vendor_id = $1 AND status != 'archived'
         ORDER BY created_at ASC",
    )
    .bind(vendor_id)
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let products_list: Vec<Value> = product_rows
        .iter()
        .map(|row| {
            let pid: Uuid = row.get("id");
            let title: String = row.get("title");
            let gender: String = row.get("gender_section");
            let men_cap: Option<i32> = row.get("men_capacity");
            let women_cap: Option<i32> = row.get("women_capacity");
            let total_cap = match gender.as_str() {
                "dual_parallel" => men_cap.unwrap_or(0) + women_cap.unwrap_or(0),
                "men_only" => men_cap.unwrap_or(0),
                _ => women_cap.unwrap_or(0),
            };
            json!({
                "id": pid.to_string(),
                "nameAr": title.clone(),
                "nameEn": title,
                "slug": row.get::<String, _>("slug"),
                "productCategory": row.get::<String, _>("product_category"),
                "genderSection": gender,
                "totalCapacity": total_cap,
                "basePriceSar": row.get::<Option<f64>, _>("base_price_sar"),
                "status": row.get::<String, _>("status"),
                "isAvailable": row.get::<bool, _>("is_available"),
                "isFeatured": row.get::<bool, _>("is_featured"),
            })
        })
        .collect();

    let total_products_count = products_list.len() as i64;
    let active_products_count = products_list
        .iter()
        .filter(|p| p["status"] == "active" || p["status"] == "pending_approval")
        .count() as i64;
    let draft_products_count = products_list
        .iter()
        .filter(|p| p["status"] == "draft")
        .count() as i64;

    // 3. Query pre-aggregated Lead Metrics & Booking Metrics
    let lead_stats = sqlx::query(
        "SELECT 
            COUNT(*)::bigint AS total_leads,
            COUNT(*) FILTER (WHERE status IN ('pending', 'new'))::bigint AS new_leads
         FROM lead_inquiries WHERE vendor_id = $1",
    )
    .bind(user_uuid)
    .fetch_one(&mut *rls_tx.tx)
    .await?;

    let total_leads_count: i64 = lead_stats.get("total_leads");
    let new_leads_count: i64 = lead_stats.get("new_leads");

    let booking_stats = sqlx::query(
        "SELECT 
            COUNT(*)::bigint AS total_bookings,
            COUNT(*) FILTER (WHERE status IN ('pending', 'Pending_Vendor_Acceptance', 'Draft_Inquiry'))::bigint AS pending_bookings,
            COUNT(*) FILTER (WHERE status IN ('confirmed', 'Confirmed', 'completed', 'Completed'))::bigint AS confirmed_bookings,
            COALESCE(SUM(total_price) FILTER (WHERE status IN ('pending', 'Pending_Vendor_Acceptance', 'Draft_Inquiry')), 0.00)::float8 AS pending_pipeline,
            COALESCE(SUM(total_price) FILTER (WHERE status IN ('confirmed', 'Confirmed', 'completed', 'Completed') AND date_trunc('month', created_at) = date_trunc('month', CURRENT_TIMESTAMP)), 0.00)::float8 AS monthly_revenue
         FROM core_bookings WHERE vendor_id = $1"
    )
    .bind(user_uuid)
    .fetch_one(&mut *rls_tx.tx)
    .await?;

    let total_bookings_count: i64 = booking_stats.get("total_bookings");
    let pending_bookings_count: i64 = booking_stats.get("pending_bookings");
    let confirmed_bookings_count: i64 = booking_stats.get("confirmed_bookings");
    let pending_revenue_pipeline: f64 = booking_stats.get("pending_pipeline");
    let monthly_revenue: f64 = booking_stats.get("monthly_revenue");

    let booking_conversion_rate = if total_leads_count > 0 {
        ((confirmed_bookings_count as f64) / (total_leads_count as f64)) * 100.0
    } else {
        0.0
    };

    // 4. Fetch explicit array of lead_inquiries with computed view metrics
    let inquiries_rows = sqlx::query(
        "SELECT id, customer_name, COALESCE(phone, '') AS phone, wedding_date::text, message, status, created_at 
         FROM lead_inquiries 
         WHERE vendor_id = $1 
         ORDER BY created_at DESC",
    )
    .bind(user_uuid)
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let today = chrono::Utc::now().date_naive();
    let mut inquiries_list = Vec::with_capacity(inquiries_rows.len());

    for row in inquiries_rows {
        let inquiry_id: Uuid = row.get("id");
        let customer_name: String = row.get("customer_name");
        let phone: String = row.get("phone");
        let wedding_date_str: String = row.get("wedding_date");
        let message: String = row.get("message");
        let inquiry_status: String = row.get("status");
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");

        // A. lead_tracking_hash: Computed tracking hex string derived from UUID u128 representation
        let lead_tracking_hash = format!("{:x}", inquiry_id.as_u128());

        // B. geographical_zone: Clean pre-aggregated zone
        let geographical_zone = format!("{} Central Zone", city_name);

        // C. urgency_index: CRITICAL, HIGH, MEDIUM, LOW
        let urgency_index =
            if let Ok(w_date) = chrono::NaiveDate::parse_from_str(&wedding_date_str, "%Y-%m-%d") {
                let days = (w_date - today).num_days();
                if days < 60 {
                    "CRITICAL"
                } else if days < 120 {
                    "HIGH"
                } else if days < 180 {
                    "MEDIUM"
                } else {
                    "LOW"
                }
            } else {
                "LOW"
            };

        // D. response_latency_countdown: response time remaining (e.g. "18h remaining")
        let elapsed = (chrono::Utc::now() - created_at).num_hours();
        let hours_left = 24 - elapsed;
        let response_latency_countdown = if hours_left <= 0 {
            "EXPIRED".to_string()
        } else {
            format!("{}h remaining", hours_left)
        };

        inquiries_list.push(json!({
            "id": inquiry_id,
            "customerName": customer_name,
            "phone": phone,
            "weddingDate": wedding_date_str,
            "message": message,
            "status": inquiry_status,
            "createdAt": created_at,
            "leadTrackingHash": lead_tracking_hash,
            "geographicalZone": geographical_zone,
            "urgencyIndex": urgency_index,
            "responseLatencyCountdown": response_latency_countdown,
        }));
    }

    // 5. Query review aggregations (overall and granular 3-axis)
    let review_stats = sqlx::query(
        "SELECT 
            COUNT(*)::bigint as total_reviews,
            COALESCE(AVG(rating_quality::float), 0.0)::float8 as avg_quality,
            COALESCE(AVG(rating_staff::float), 0.0)::float8 as avg_staff,
            COALESCE(AVG(rating_communication::float), 0.0)::float8 as avg_communication
         FROM reviews 
         WHERE vendor_id = $1 AND status = 'approved'",
    )
    .bind(vendor_id)
    .fetch_one(&mut *rls_tx.tx)
    .await?;

    let total_reviews: i64 = review_stats.get("total_reviews");
    let avg_quality: f64 = review_stats.get("avg_quality");
    let avg_staff: f64 = review_stats.get("avg_staff");
    let avg_communication: f64 = review_stats.get("avg_communication");

    // Compute mathematically the overall average
    let avg_overall = if total_reviews > 0 {
        (avg_quality + avg_staff + avg_communication) / 3.0
    } else {
        0.0
    };

    rls_tx.tx.commit().await?;

    // 6. Return complete optimized dashboard telemetry values
    Ok(Json(json!({
        "status": "success",
        "data": {
            "vendor": {
                "id": vendor_id.to_string(),
                "name_ar": name_ar,
                "name_en": name_en,
                "description_ar": description_ar.unwrap_or_default(),
                "description_en": description_en.unwrap_or_default(),
                "slug": slug,
                "category": category.as_deref().unwrap_or(""),
                "status": status,
                "address_ar": address_ar.unwrap_or_default(),
                "address_en": address_en.unwrap_or_default(),
                "phone": phone.unwrap_or_default(),
                "email": email.unwrap_or_default(),
                "capacity_min": capacity_min,
                "capacity_max": capacity_max,
                "latitude": latitude,
                "longitude": longitude,
                "amenities": amenities.map(|a| a.join(", ")).unwrap_or_default(),
                "crm_venue_id": crm_venue_id,
                "star_rating": star_rating,
                "event_spaces_available": active_products_count,
                "event_type": Value::Null,

                "city_id": city_id.map(|id| id.to_string()),
                "city_name_en": city_name,
                "city_name_ar": city_name_ar,
                "website": website,
                "maps_url": maps_url,
                "video_url_1": video_url_1,
                "version": version,
                "tier_id": tier_id,
                "tier_name": tier_name,
                "subscription_expires_at": subscription_expires_at,
                "policy_limits": policy_limits,
            },
            "products": products_list,
            "metrics": {
                "totalProducts": total_products_count,
                "activeProducts": active_products_count,
                "draftProducts": draft_products_count,
                "totalPackages": package_count,
                "totalGalleryItems": gallery_count,
                "totalLeads": total_leads_count,
                "newLeads": new_leads_count,
                "totalBookings": total_bookings_count,
                "pendingBookings": pending_bookings_count,
                "confirmedBookings": confirmed_bookings_count,
                "monthlyRevenue": monthly_revenue,
                "bookingConversionRate": booking_conversion_rate,
                "activeLeadsCount": new_leads_count,
                "totalPendingRevenuePipeline": pending_revenue_pipeline,
                "active_packages": package_count,
                "active_products": active_products_count,
                "total_reviews": total_reviews,
                "avg_overall": avg_overall,
                "avg_quality": avg_quality,
                "avg_staff": avg_staff,
                "avg_communication": avg_communication,
            },
            "leadInquiries": inquiries_list
        }
    })))
}

#[derive(Deserialize)]
pub struct ReviewQuery {
    pub status: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

async fn list_reviews(
    mut rls_tx: RlsTx,
    axum::extract::Query(query): axum::extract::Query<ReviewQuery>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = (page - 1) * limit;

    let mut query_str = "
        SELECT 
            r.id, 
            r.vendor_id, 
            COALESCE(cp.first_name || ' ' || cp.last_name, 'Couple') AS couple_name, 
            r.rating AS rating, 
            r.review_text AS comment, 
            r.status AS status, 
            r.created_at AS created_at
         FROM vendor_reviews r
         LEFT JOIN client_profiles cp ON r.client_id = cp.client_id
         WHERE r.vendor_id = $1"
        .to_string();

    let mut param_idx = 2;
    let mut status_val = None;

    if let Some(ref status) = query.status {
        query_str.push_str(&format!(" AND r.status = ${}", param_idx));
        param_idx += 1;
        status_val = Some(status);
    }

    query_str.push_str(&format!(
        " ORDER BY r.created_at DESC LIMIT ${} OFFSET ${}",
        param_idx,
        param_idx + 1
    ));

    let mut db_query = sqlx::query_as::<_, VendorReview>(&query_str).bind(vendor_id);
    if let Some(status) = status_val {
        db_query = db_query.bind(status);
    }
    db_query = db_query.bind(limit as i64).bind(offset as i64);

    let reviews = db_query.fetch_all(&mut *rls_tx.tx).await?;

    let mut count_query_str =
        "SELECT COUNT(*) FROM vendor_reviews WHERE vendor_id = $1".to_string();
    if query.status.is_some() {
        count_query_str.push_str(" AND status = $2");
    }
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_query_str).bind(vendor_id);
    if let Some(ref status) = query.status {
        count_query = count_query.bind(status);
    }
    let total: i64 = count_query.fetch_one(&mut *rls_tx.tx).await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "reviews": reviews,
        "pagination": {
            "total": total,
            "page": page,
            "limit": limit,
            "totalPages": (total as f64 / limit as f64).ceil() as i64
        }
    })))
}

#[derive(Deserialize)]
pub struct CreateReviewRequest {
    pub rating: i32,
    pub comment: String,
}

async fn create_review(
    mut rls_tx: RlsTx,
    Json(payload): Json<CreateReviewRequest>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;
    let new_id = Uuid::new_v4();

    // Assuming we can insert a review offline. If client_id is required, we use a generic UUID.
    // For now, assume client_id is nullable or we leave it out.
    let _ = sqlx::query(
        "INSERT INTO vendor_reviews (id, vendor_id, rating, review_text, status) VALUES ($1, $2, $3, $4, 'approved')"
    )
    .bind(new_id)
    .bind(vendor_id)
    .bind(payload.rating)
    .bind(&payload.comment)
    .execute(&mut *rls_tx.tx)
    .await
    .map_err(|err| AppError::Database(err.to_string()))?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Review added successfully",
        "id": new_id
    })))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateReviewStatusRequest {
    status: String,
}

async fn update_review_status(
    mut rls_tx: RlsTx,
    axum::extract::Path(review_id): axum::extract::Path<Uuid>,
    Json(payload): Json<UpdateReviewStatusRequest>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    let target_status = if payload.status == "pending" {
        "pending_approval"
    } else {
        &payload.status
    };

    if target_status != "approved"
        && target_status != "rejected"
        && target_status != "pending_approval"
    {
        return Err(AppError::BadRequest("Invalid review status".to_string()));
    }

    let rows_affected = sqlx::query(
        "UPDATE vendor_reviews 
         SET status = $1 
         WHERE id = $2 AND vendor_id = $3",
    )
    .bind(target_status)
    .bind(review_id)
    .bind(vendor_id)
    .execute(&mut *rls_tx.tx)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound(
            "Review not found or access denied".to_string(),
        ));
    }
    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Review status updated successfully"
    })))
}
async fn list_tasks(mut rls_tx: RlsTx) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    let tasks = sqlx::query_as::<_, VendorTask>(
        "SELECT id, vendor_id, title_ar, title_en, is_completed, due_date
         FROM vendor_tasks
         WHERE vendor_id = $1
         ORDER BY due_date ASC NULLS LAST",
    )
    .bind(vendor_id)
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "tasks": tasks
    })))
}

async fn list_templates(
    State(state): State<AppState>,
    auth: RequireVendor,
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

    let templates = sqlx::query_as::<_, VendorWhatsappTemplate>(
        "SELECT id, vendor_id, template_name, body_text_ar, body_text_en, updated_at
         FROM vendor_whatsapp_templates
         WHERE vendor_id = $1
         ORDER BY updated_at DESC",
    )
    .bind(vendor_id)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(json!({
        "status": "success",
        "templates": templates
    })))
}


// ─── NOTIFICATIONS / SYSTEM EVENTS ───────────────────────────────────────────

/// Read vendor in-app notifications from system_events table
async fn list_notifications(
    State(state): State<AppState>,
    auth: RequireVendor,
) -> Result<Json<Value>, AppError> {
    let user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid user ID format in token".to_string()))?;

    let rows = sqlx::query(
        "SELECT id, event_type, message_ar, message_en, is_read, created_at
         FROM system_events
         WHERE user_id = $1
         ORDER BY created_at DESC
         LIMIT 50",
    )
    .bind(user_uuid)
    .fetch_all(&state.db)
    .await?;

    let mut notifications = Vec::new();
    for row in rows {
        let id: Uuid = row.get("id");
        let event_type: String = row.get("event_type");
        let message_ar: String = row.get("message_ar");
        let message_en: String = row.get("message_en");
        let is_read: bool = row.get("is_read");
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");

        notifications.push(json!({
            "id": id.to_string(),
            "event_type": event_type,
            "message_ar": message_ar,
            "message_en": message_en,
            "is_read": is_read,
            "created_at": created_at.to_rfc3339()
        }));
    }

    let unread_count = notifications
        .iter()
        .filter(|n| !n.get("is_read").and_then(|v| v.as_bool()).unwrap_or(true))
        .count();

    Ok(Json(json!({
        "status": "success",
        "notifications": notifications,
        "unread_count": unread_count
    })))
}

/// Mark all notifications as read for the current vendor
async fn mark_notifications_read(
    State(state): State<AppState>,
    auth: RequireVendor,
) -> Result<Json<Value>, AppError> {
    let user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid user ID format in token".to_string()))?;

    sqlx::query("UPDATE system_events SET is_read = TRUE WHERE user_id = $1 AND is_read = FALSE")
        .bind(user_uuid)
        .execute(&state.db)
        .await?;

    Ok(Json(json!({
        "status": "success",
        "message": "All notifications marked as read"
    })))
}

// ─── GALLERY CRUD HANDLERS ───────────────────────────────────────────────────

#[derive(Deserialize)]
struct AddGalleryImageRequest {
    image_url: String,
    #[serde(default)]
    caption: Option<String>,
    is_cover: bool,
    #[serde(default)]
    product_id: Option<Uuid>,
    #[serde(default)]
    file_path: Option<String>,
}

async fn list_gallery(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    mut rls_tx: RlsTx,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;
    let repo = crate::repositories::vendor_repository::PgVendorRepository::new(state.db.clone());
    let gallery = repo.list_gallery(&mut rls_tx.tx, vendor_id).await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "gallery": gallery
    })))
}

async fn add_gallery_image(
    State(state): State<AppState>,
    _auth: RequireVendor,
    mut rls_tx: RlsTx,
    body_str: String,
) -> Result<Json<Value>, AppError> {
    tracing::info!("RECEIVED GALLERY PAYLOAD: {}", body_str);
    let payload: AddGalleryImageRequest = serde_json::from_str(&body_str)
        .map_err(|e| AppError::BadRequest(format!("Failed to parse gallery payload: {}", e)))?;
    let vendor_id = rls_tx.get_vendor_id().await?;

    let url = payload.image_url.trim();

    if url.is_empty() {
        return Err(AppError::BadRequest("Image URL is required".to_string()));
    }

    let is_relative_upload = crate::utils::storage_paths::is_relative_upload(url, &state.config.minio_root_prefix);
    let is_https = url.starts_with("https://");

    if !is_relative_upload && !is_https {
        return Err(AppError::BadRequest(
            format!("Invalid image URL. Must be a relative upload path (/{}/...) or an HTTPS URL.", state.config.minio_root_prefix),
        ));
    }

    // Extra guard: reject any URL that contains a null byte or newline (header injection)
    if url.contains('\0') || url.contains('\n') || url.contains('\r') {
        return Err(AppError::BadRequest(
            "Image URL contains invalid characters".to_string(),
        ));
    }

    let new_id = Uuid::new_v4();
    
    let repo = crate::repositories::vendor_repository::PgVendorRepository::new(state.db.clone());
    repo.add_gallery_image(
        &mut rls_tx.tx,
        vendor_id,
        new_id,
        url,
        payload.is_cover,
        &payload.caption,
        payload.product_id,
    ).await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Image asset added successfully",
        "id": new_id.to_string()
    })))
}

async fn set_cover_image(
    State(state): State<AppState>,
    _auth: RequireVendor,
    mut rls_tx: RlsTx,
    axum::extract::Path(image_id): axum::extract::Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    let repo = crate::repositories::vendor_repository::PgVendorRepository::new(state.db.clone());
    repo.set_cover_image(&mut rls_tx.tx, vendor_id, image_id).await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Cover image updated successfully"
    })))
}

async fn delete_gallery_image(
    State(state): State<AppState>,
    _auth: RequireVendor,
    mut rls_tx: RlsTx,
    axum::extract::Path(image_id): axum::extract::Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    let repo = crate::repositories::vendor_repository::PgVendorRepository::new(state.db.clone());
    let file_path = repo.get_gallery_image_path(&mut rls_tx.tx, vendor_id, image_id).await?;
    repo.delete_gallery_image(&mut rls_tx.tx, vendor_id, image_id).await?;

    rls_tx.tx.commit().await?;

    // Delete physical file from disk (non-blocking, best-effort)
    if let Some(path) = file_path {
        if let Err(e) = tokio::fs::remove_file(&path).await {
            tracing::warn!("Could not delete gallery file '{}': {}", path, e);
        }
    }

    Ok(Json(json!({
        "status": "success",
        "message": "Image asset deleted successfully"
    })))
}

async fn upload_file(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    _auth: RequireVendor,
    mut rls_tx: RlsTx,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<Value>, AppError> {
    let mut product_id: Option<Uuid> = None;
    let mut is_cover = false;
    let mut media_type = String::from("image");

    // Use RlsTx for vendor ID resolution — consistent RLS context for all DB queries.
    let vendor_id = rls_tx.get_vendor_id().await?;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| AppError::BadRequest(err.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();

        if name == "product_id" {
            let text = field.text().await.unwrap_or_default();
            if !text.is_empty() {
                product_id = Uuid::parse_str(&text).ok();
                if product_id.is_none() {
                    tracing::warn!("Failed to parse product_id from multipart form data: {}", text);
                }
            }
        } else if name == "is_cover" {
            let text = field.text().await.unwrap_or_default();
            is_cover = text.trim().to_lowercase() == "true";
            tracing::debug!("Multipart parsed is_cover: {} (raw: {})", is_cover, text);
        } else if name == "media_type" {
            let text = field.text().await.unwrap_or_default();
            if !text.trim().is_empty() {
                media_type = text.trim().to_lowercase();
            }
        } else if name == "file" {
            let is_video = media_type == "video";

            // Policy check runs inside the same RLS-scoped transaction.
            if let Err(e) = crate::utils::policy::PolicyEngine::check_media_limit(
                vendor_id, product_id, is_video, is_cover, &mut rls_tx.tx,
            )
            .await {
                crate::services::metrics::inc_upload_failed();
                return Err(e);
            }
            rls_tx.tx.commit().await?;

            let file_name = field.file_name().unwrap_or("upload.jpg").to_string();

            let target_dir = "assets/uploads/gallery/";
            let url_prefix = "/assets/uploads/gallery/";
            let max_bytes = if is_video {
                200 * 1024 * 1024 // 200 MB for video
            } else {
                10 * 1024 * 1024  // 10 MB for image
            };
            let processed = match crate::services::media::process_and_save_upload(
                field,
                &file_name,
                target_dir,
                url_prefix,
                max_bytes,
                1920,
                &state.minio_client,
            )
            .await {
                Ok(p) => p,
                Err(e) => {
                    crate::services::metrics::inc_upload_failed();
                    return Err(e);
                }
            };

            crate::services::metrics::inc_upload_success();
            return Ok(Json(json!({
                "status": "success",
                "id": processed.id,
                "status_state": processed.status,
                "url":       processed.file_url,
                "file_path": processed.disk_path,
                "media_type": processed.media_type,
                "thumbnail_url": processed.thumbnail_url,
                "file_size": processed.file_size,
                "duration_seconds": processed.duration_seconds
            })));
        }
    }

    Err(AppError::BadRequest(
        "A valid image or video file attachment is required.".to_string(),
    ))
}

async fn get_upload_status(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    _auth: RequireVendor,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let upload = crate::repositories::uploaded_files_repository::get_by_id(&state.db, id)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    if let Some(up) = upload {
        Ok(Json(json!({
            "status": "success",
            "data": {
                "id": up.id,
                "status": up.status,
                "file_name": up.file_name,
                "file_url": format!("/{}", up.object_key),
                "mime_type": up.mime_type,
                "file_size": up.file_size,
                "error_message": up.error_message,
            }
        })))
    } else {
        Err(AppError::NotFound("Upload not found".to_string()))
    }
}



#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateTaskRequest {
    title_ar: String,
    title_en: String,
    due_date: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateTaskRequest {
    title_ar: String,
    title_en: String,
    is_completed: bool,
    due_date: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateTemplateRequest {
    template_name: String,
    body_text_ar: Option<String>,
    body_text_en: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateTemplateRequest {
    template_name: String,
    body_text_ar: Option<String>,
    body_text_en: Option<String>,
}

async fn create_task(
    State(state): State<AppState>,
    auth: RequireVendorOwner,
    Json(payload): Json<CreateTaskRequest>,
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

    if payload.title_ar.trim().is_empty() || payload.title_en.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Task title is required in both Arabic and English".to_string(),
        ));
    }

    let parsed_due_date = if let Some(ref date_str) = payload.due_date {
        if date_str.is_empty() {
            None
        } else if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
            Some(dt.with_timezone(&chrono::Utc))
        } else if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            let dt = date
                .and_hms_opt(12, 0, 0)
                .ok_or_else(|| AppError::Internal("Failed to construct datetime".to_string()))?;
            Some(chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                dt,
                chrono::Utc,
            ))
        } else {
            return Err(AppError::BadRequest("Invalid due_date format".to_string()));
        }
    } else {
        None
    };

    let task = sqlx::query_as::<_, VendorTask>(
        "INSERT INTO vendor_tasks (vendor_id, title_ar, title_en, is_completed, due_date)
         VALUES ($1, $2, $3, FALSE, $4)
         RETURNING id, vendor_id, title_ar, title_en, is_completed, due_date",
    )
    .bind(vendor_id)
    .bind(&payload.title_ar)
    .bind(&payload.title_en)
    .bind(parsed_due_date)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Task created successfully",
        "task": task
    })))
}

async fn update_task(
    State(state): State<AppState>,
    auth: RequireVendorOwner,
    axum::extract::Path(task_id): axum::extract::Path<Uuid>,
    Json(payload): Json<UpdateTaskRequest>,
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

    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM vendor_tasks WHERE id = $1 AND vendor_id = $2)",
    )
    .bind(task_id)
    .bind(vendor_id)
    .fetch_one(&state.db)
    .await?;

    if !exists {
        return Err(AppError::NotFound(
            "Task not found or access denied".to_string(),
        ));
    }

    if payload.title_ar.trim().is_empty() || payload.title_en.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Task title is required in both Arabic and English".to_string(),
        ));
    }

    let parsed_due_date = if let Some(ref date_str) = payload.due_date {
        if date_str.is_empty() {
            None
        } else if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
            Some(dt.with_timezone(&chrono::Utc))
        } else if let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
            let dt = date
                .and_hms_opt(12, 0, 0)
                .ok_or_else(|| AppError::Internal("Failed to construct datetime".to_string()))?;
            Some(chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                dt,
                chrono::Utc,
            ))
        } else {
            return Err(AppError::BadRequest("Invalid due_date format".to_string()));
        }
    } else {
        None
    };

    sqlx::query(
        "UPDATE vendor_tasks
         SET title_ar = $1, title_en = $2, is_completed = $3, due_date = $4
         WHERE id = $5 AND vendor_id = $6",
    )
    .bind(&payload.title_ar)
    .bind(&payload.title_en)
    .bind(payload.is_completed)
    .bind(parsed_due_date)
    .bind(task_id)
    .bind(vendor_id)
    .execute(&state.db)
    .await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Task updated successfully"
    })))
}

async fn delete_task(
    State(state): State<AppState>,
    auth: RequireVendorOwner,
    axum::extract::Path(task_id): axum::extract::Path<Uuid>,
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

    let rows_affected = sqlx::query("DELETE FROM vendor_tasks WHERE id = $1 AND vendor_id = $2")
        .bind(task_id)
        .bind(vendor_id)
        .execute(&state.db)
        .await?
        .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound(
            "Task not found or access denied".to_string(),
        ));
    }

    Ok(Json(json!({
        "status": "success",
        "message": "Task deleted successfully"
    })))
}

async fn create_template(
    State(state): State<AppState>,
    auth: RequireVendorOwner,
    Json(payload): Json<CreateTemplateRequest>,
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

    if payload.template_name.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Template name is required".to_string(),
        ));
    }

    let template = sqlx::query_as::<_, VendorWhatsappTemplate>(
        "INSERT INTO vendor_whatsapp_templates (vendor_id, template_name, body_text_ar, body_text_en)
         VALUES ($1, $2, $3, $4)
         RETURNING id, vendor_id, template_name, body_text_ar, body_text_en, updated_at"
    )
    .bind(vendor_id)
    .bind(&payload.template_name)
    .bind(&payload.body_text_ar)
    .bind(&payload.body_text_en)
    .fetch_one(&state.db)
    .await
    ?;

    Ok(Json(json!({
        "status": "success",
        "message": "Template created successfully",
        "template": template
    })))
}

async fn update_template(
    _auth: RequireVendorOwner,
    mut rls_tx: RlsTx,
    axum::extract::Path(template_id): axum::extract::Path<Uuid>,
    Json(payload): Json<UpdateTemplateRequest>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    if payload.template_name.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Template name is required".to_string(),
        ));
    }

    let rows_affected = sqlx::query(
        "UPDATE vendor_whatsapp_templates
         SET template_name = $1, body_text_ar = $2, body_text_en = $3, updated_at = NOW()
         WHERE id = $4 AND vendor_id = $5",
    )
    .bind(&payload.template_name)
    .bind(&payload.body_text_ar)
    .bind(&payload.body_text_en)
    .bind(template_id)
    .bind(vendor_id)
    .execute(&mut *rls_tx.tx)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound(
            "Template not found or access denied".to_string(),
        ));
    }

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Template updated successfully"
    })))
}

async fn delete_template(
    _auth: RequireVendorOwner,
    mut rls_tx: RlsTx,
    axum::extract::Path(template_id): axum::extract::Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    let rows_affected =
        sqlx::query("DELETE FROM vendor_whatsapp_templates WHERE id = $1 AND vendor_id = $2")
            .bind(template_id)
            .bind(vendor_id)
            .execute(&mut *rls_tx.tx)
            .await?
            .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound(
            "Template not found or access denied".to_string(),
        ));
    }

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Template deleted successfully"
    })))
}

async fn get_vendor_chat_messages(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    _auth: RequireVendor,
    mut rls_tx: RlsTx,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;
    let repo = crate::repositories::vendor_repository::PgVendorRepository::new(state.db.clone());

    // 1. Get or Create the single support chat thread for this vendor
    let chat_id = repo.get_or_create_admin_chat(&mut rls_tx.tx, vendor_id).await?;

    // 2. Fetch all messages in the thread
    let messages = repo.get_chat_messages(&mut rls_tx.tx, chat_id, vendor_id).await?;

    // 3. Mark admin messages as read by vendor
    repo.mark_admin_messages_read(&mut rls_tx.tx, chat_id, vendor_id).await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "chat_id": chat_id,
        "messages": messages,
    })))
}

async fn post_vendor_chat_reply(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    _auth: RequireVendor,
    mut rls_tx: RlsTx,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<Value>, AppError> {
    let mut body = String::new();
    let mut file_url: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|err| AppError::BadRequest(err.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "body" {
            body = field
                .text()
                .await
                .map_err(|err| AppError::BadRequest(err.to_string()))?;
        } else if name == "file" {
            let file_name = field.file_name().unwrap_or("upload.png").to_string();
            let target_dir = "assets/uploads/chat/";
            let url_prefix = "/assets/uploads/chat/";
            let max_bytes = 10 * 1024 * 1024; // 10 MB limit for chat
            let max_dimension = 1280; // slightly smaller for chat

            let processed = crate::services::media::process_and_save_upload(
                field,
                &file_name,
                target_dir,
                url_prefix,
                max_bytes,
                max_dimension,
                &state.minio_client,
            )
            .await?;

            file_url = Some(processed.file_url);
        }
    }

    if body.trim().is_empty() && file_url.is_none() {
        return Err(AppError::BadRequest(
            "Message body or file attachment is required".to_string(),
        ));
    }

    let vendor_id = rls_tx.get_vendor_id().await?;
    let repo = crate::repositories::vendor_repository::PgVendorRepository::new(state.db.clone());

    // 1. Get or Create chat
    let chat_id = repo.get_or_create_admin_chat(&mut rls_tx.tx, vendor_id).await?;

    // 2. Insert message
    let message_id = repo.insert_chat_message(
        &mut rls_tx.tx,
        chat_id,
        vendor_id,
        &body,
        file_url.as_deref()
    ).await?;

    // 3. Update chat timestamp
    repo.update_chat_timestamp(&mut rls_tx.tx, chat_id).await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message_id": message_id,
        "message": "Message sent successfully",
        "file_url": file_url
    })))
}

