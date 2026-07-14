use crate::errors::AppError;
use crate::middleware::auth::{RequireAdmin, RlsTx};
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    routing::{get, post, patch},
    Json, Router,
};
use serde_json::{json, Value};
use uuid::Uuid;
use sqlx::Row;
use serde::Deserialize;

#[derive(serde::Deserialize)]
pub struct UpdateVendorSubscriptionInput {
    pub subscription_status: String,
    pub subscription_tier_id: Option<String>,
    pub subscription_expires_at: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdateVendorFeaturedInput {
    pub is_featured: bool,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/vendors/:id", get(get_vendor_detail))
        .route("/vendors/:id/product-status", patch(update_vendor_product_status))
        .route("/vendors/:id/promote", post(promote_vendor_product))
        .route("/vendors/pending-listings", get(list_pending_listings))
        .route("/vendors/all-listings", get(list_all_listings))
        .route("/vendors", get(list_all_vendors))
        .route("/vendors/:id/reactivate", post(reactivate_vendor))
        .route("/vendors/:id/status", patch(update_vendor_status))
        .route("/vendors/:id/subscription", patch(update_vendor_subscription))
        .route("/vendors/:id/featured", patch(update_vendor_featured))
        .route("/vendors/subscription/requests", get(list_vendor_subscription_requests))
        .route("/vendors/subscription/requests/:id/approve", post(approve_vendor_subscription_request))
        .route("/vendors/subscription/requests/:id/reject", post(reject_vendor_subscription_request))
        .route("/vendors/context", get(get_admin_vendors_context))
}

async fn get_vendor_detail(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Path(id): Path<String>,
) -> Result<Json<Value>, AppError> {
    tracing::info!("Admin fetching full vendor detail for ID: {}", id);

    let vendor_uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid vendor UUID format".to_string()))?;

    // Fetch vendor row
    let vendor_row = sqlx::query(
        "SELECT 
            v.id, v.name_ar, v.name_en, v.slug, v.category, v.status, v.is_verified,
            v.email, v.phone, v.description_ar, v.description_en,
            v.address_ar, v.address_en,
            v.subscription_status, v.is_featured, v.created_at, v.updated_at,
            v.website, v.star_rating::float8 AS star_rating,
            c.name_en AS city_name_en, c.name_ar AS city_name_ar,
            COALESCE(
                CASE WHEN v.subscription_expires_at < CURRENT_TIMESTAMP THEN 'Free' ELSE st.name END, 
                'Free'
            ) AS current_tier,
            (SELECT count(*) FROM vendor_products wp WHERE wp.vendor_id = v.id AND wp.status != 'archived') AS used_products_count,
            COALESCE(
                CASE WHEN v.subscription_expires_at < CURRENT_TIMESTAMP THEN '1' ELSE st.policy_limits->>'max_products' END, 
                '1'
            )::bigint AS max_products
        FROM vendors v
        LEFT JOIN cities c ON v.city_id = c.id
        LEFT JOIN subscription_tiers st ON v.subscription_tier_id = st.id
        WHERE v.id = $1",
    )
    .bind(vendor_uuid)
    .fetch_optional(&state.db)
    .await?;

    let row = match vendor_row {
        Some(r) => r,
        None => return Err(AppError::NotFound("Vendor not found".to_string())),
    };

    let vendor_id: Uuid = row.get("id");

    // Fetch all halls/products for this vendor
    let product_rows = sqlx::query(
        "SELECT 
            p.id, p.title, p.description, p.status, p.base_price_sar::float8 AS base_price_sar,
            p.is_available, p.product_category, p.created_at, p.updated_at, p.attributes
        FROM vendor_products p
        WHERE p.vendor_id = $1
        ORDER BY p.created_at DESC",
    )
    .bind(vendor_id)
    .fetch_all(&state.db)
    .await?;

    let mut products = Vec::new();
    for p in product_rows {
        let pid: Uuid = p.get("id");
        let title: String = p.get::<Option<String>, _>("title").unwrap_or_default();
        let description: Option<String> = p.get("description");
        let status: String = p.get("status");
        let price: Option<f64> = p.get("base_price_sar");
        let is_available: bool = p.get("is_available");
        let product_category: String = p.get("product_category");
        let p_created: chrono::DateTime<chrono::Utc> = p.get("created_at");
        let p_updated: chrono::DateTime<chrono::Utc> = p.get("updated_at");
        let attributes: serde_json::Value = p.get("attributes");

        products.push(json!({
            "id":               pid.to_string(),
            "title":            title,
            "description":      description,
            "status":           status,
            "product_category": product_category,
            "base_price_sar":   price,
            "is_available":     is_available,
            "created_at":       p_created.to_rfc3339(),
            "updated_at":       p_updated.to_rfc3339(),
            "attributes":       attributes,
        }));
    }

    // Count reviews and rating
    let rating_row = sqlx::query(
        "SELECT COUNT(*) AS review_count, COALESCE(AVG(rating), 0)::float8 AS avg_rating
         FROM vendor_reviews WHERE vendor_id = $1 AND status = 'approved'",
    )
    .bind(vendor_id)
    .fetch_one(&state.db)
    .await?;
    let review_count: i64 = rating_row.get("review_count");
    let avg_rating: f64 = rating_row.get("avg_rating");

    let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
    let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

    Ok(Json(json!({
        "status": "success",
        "vendor": {
            "id":                  vendor_id.to_string(),
            "name_ar":             row.get::<String, _>("name_ar"),
            "name_en":             row.get::<String, _>("name_en"),
            "slug":                row.get::<String, _>("slug"),
            "category":            row.get::<Option<String>, _>("category").unwrap_or_default(),
            "status":              row.get::<String, _>("status"),
            "is_verified":         row.get::<bool, _>("is_verified"),
            "email":               row.get::<Option<String>, _>("email"),
            "phone":               row.get::<Option<String>, _>("phone"),
            "description_en":      row.get::<Option<String>, _>("description_en"),
            "description_ar":      row.get::<Option<String>, _>("description_ar"),
            "address_en":          row.get::<Option<String>, _>("address_en"),
            "address_ar":          row.get::<Option<String>, _>("address_ar"),
            "subscription_status": row.get::<String, _>("subscription_status"),
            "is_featured":         row.get::<bool, _>("is_featured"),
            "website":             row.get::<Option<String>, _>("website"),
            "star_rating":         row.get::<Option<f64>, _>("star_rating"),
            "city_name_en":        row.get::<Option<String>, _>("city_name_en").unwrap_or_else(|| "Unknown".to_string()),
            "city_name_ar":        row.get::<Option<String>, _>("city_name_ar").unwrap_or_else(|| "غير معروف".to_string()),
            "review_count":        review_count,
            "avg_rating":          avg_rating,
            "created_at":          created_at.to_rfc3339(),
            "updated_at":          updated_at.to_rfc3339(),
            "products":            products,
        }
    })))
}

// ─── LISTING MODERATION ───────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub struct UpdateProductStatusInput {
    pub status: String,
    pub reason: Option<String>,
}

/// PATCH /admin/vendors/:id/products/:product_id/status — approve, reject, or suspend a listing
async fn update_vendor_product_status(
    State(state): State<AppState>,
    auth: RequireAdmin,
    Path((vendor_id_str, product_id_str)): Path<(String, String)>,
    Json(input): Json<UpdateProductStatusInput>,
) -> Result<Json<Value>, AppError> {
    let vendor_uuid = Uuid::parse_str(&vendor_id_str)
        .map_err(|_| AppError::BadRequest("Invalid vendor UUID".to_string()))?;
    let product_uuid = Uuid::parse_str(&product_id_str)
        .map_err(|_| AppError::BadRequest("Invalid product UUID".to_string()))?;
    let admin_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid admin user ID".to_string()))?;

    let allowed = [
        "active",
        "suspended",
        "pending_approval",
        "rejected",
        "archived",
    ];
    if !allowed.contains(&input.status.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Invalid product status '{}'. Must be one of: {}",
            input.status,
            allowed.join(", ")
        )));
    }

    let mut tx = state.db.begin().await?;

    // Verify product belongs to this vendor
    let product_row = sqlx::query(
        "SELECT id, title, status FROM vendor_products WHERE id = $1 AND vendor_id = $2",
    )
    .bind(product_uuid)
    .bind(vendor_uuid)
    .fetch_optional(&mut *tx)
    .await?;

    let product = match product_row {
        Some(r) => r,
        None => {
            return Err(AppError::NotFound(
                "Product/listing not found for this vendor".to_string(),
            ))
        }
    };

    let product_title: String = product.get::<Option<String>, _>("title").unwrap_or_default();
    let current_status: String = product.get("status");

    // Perform atomic status transition — also saves rejection_reason when status = 'rejected'
    let result = sqlx::query(
        "UPDATE vendor_products
         SET status = $1,
             rejection_reason = CASE WHEN $1 = 'rejected' THEN $2 ELSE rejection_reason END,
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $3",
    )
    .bind(&input.status)
    .bind(input.reason.as_deref().unwrap_or(""))
    .bind(product_uuid)
    .execute(&mut *tx)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Listing update failed — no rows affected".to_string(),
        ));
    }

    // Status transition history
    sqlx::query(
        "INSERT INTO status_history (entity_type, entity_id, old_status, new_status, changed_by, reason, vendor_id)
         VALUES ('product', $1, $2, $3, $4, $5, $6)",
    )
    .bind(product_uuid)
    .bind(&current_status)
    .bind(&input.status)
    .bind(admin_uuid)
    .bind(input.reason.as_deref().unwrap_or("Product status updated by admin"))
    .bind(vendor_uuid)
    .execute(&mut *tx)
    .await?;

    // Admin audit log
    sqlx::query(
        "INSERT INTO admin_audit_logs (entity_type, entity_id, actor_id, action, before_state, after_state)
         VALUES ('product', $1, $2, 'update_product_status', $3, $4)",
    )
    .bind(product_uuid)
    .bind(admin_uuid)
    .bind(json!({ "status": current_status }))
    .bind(json!({ "status": input.status }))
    .execute(&mut *tx)
    .await?;

    // Determine listing-level event type for the audit log
    let event_type = match input.status.as_str() {
        "active" => "listing_approved",
        "rejected" => "listing_rejected",
        "suspended" => "listing_suspended",
        _ => "system_alert",
    };

    let reason = input.reason.as_deref().unwrap_or("");
    sqlx::query(
        "INSERT INTO system_events (user_id, target_vendor_id, event_type, message_ar, message_en)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(admin_uuid)
    .bind(vendor_uuid)
    .bind(event_type)
    .bind(format!(
        "قام المسؤول بتغيير حالة الإعلان «{}» إلى '{}'. {}",
        product_title, input.status, reason
    ))
    .bind(format!(
        "Admin changed listing '{}' status from '{}' to '{}'. {}",
        product_title, current_status, input.status, reason
    ))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    tracing::info!(
        target: "audit",
        actor_id   = %auth.user_id,
        event      = "listing_status_changed",
        product_id = %product_uuid,
        vendor_id  = %vendor_uuid,
        prev       = %current_status,
        next       = %input.status,
        "Admin listing status mutation committed"
    );

    Ok(Json(json!({
        "status": "success",
        "message": format!("Listing '{}' status updated to '{}'", product_title, input.status),
        "previous_status": current_status,
        "new_status": input.status
    })))
}

#[derive(serde::Deserialize)]
pub struct PromoteProductInput {
    pub days: i32,
}

/// POST /admin/products/:id/promote — Features a paid listing for a duration
async fn promote_vendor_product(
    State(state): State<AppState>,
    auth: RequireAdmin,
    Path(id): Path<String>,
    Json(input): Json<PromoteProductInput>,
) -> Result<Json<Value>, AppError> {
    let product_uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid product UUID format".to_string()))?;
    let admin_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid admin user ID".to_string()))?;

    if input.days <= 0 {
        return Err(AppError::BadRequest(
            "Promotion days must be greater than zero".to_string(),
        ));
    }

    // Verify product exists
    let product_row = sqlx::query("SELECT id, title, vendor_id FROM vendor_products WHERE id = $1")
        .bind(product_uuid)
        .fetch_optional(&state.db)
        .await?;

    let product = match product_row {
        Some(r) => r,
        None => return Err(AppError::NotFound("Product not found".to_string())),
    };

    let product_title: String = product.get::<Option<String>, _>("title").unwrap_or_default();
    let vendor_uuid: Uuid = product.get("vendor_id");

    // Perform atomic promotion
    let result = sqlx::query(
        "UPDATE vendor_products
         SET is_featured = TRUE,
             featured_until = CURRENT_TIMESTAMP + ($1 * INTERVAL '1 day'),
             updated_at = CURRENT_TIMESTAMP
         WHERE id = $2",
    )
    .bind(input.days)
    .bind(product_uuid)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Promotion failed — product not found".to_string(),
        ));
    }

    // Audit event logging
    sqlx::query(
        "INSERT INTO system_events (user_id, target_vendor_id, event_type, message_ar, message_en)
         VALUES ($1, $2, 'listing_promoted', $3, $4)",
    )
    .bind(admin_uuid)
    .bind(vendor_uuid)
    .bind(format!(
        "قام المسؤول بترقية الإعلان «{}» ليصبح مميزًا لمدة {} أيام.",
        product_title, input.days
    ))
    .bind(format!(
        "Admin promoted listing '{}' to Featured status for {} days.",
        product_title, input.days
    ))
    .execute(&state.db)
    .await?;

    Ok(Json(json!({
        "status": "success",
        "message": format!("Listing '{}' promoted for {} days", product_title, input.days),
        "product_id": id,
        "is_featured": true,
        "days": input.days
    })))
}

// ─── VENDOR STATUS MANAGEMENT ────────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub struct UpdateVendorStatusInput {
    pub status: String,
    pub reason: Option<String>,
}

// ─── LISTING MODERATION QUEUE ─────────────────────────────────────────────────

#[derive(serde::Deserialize)]
pub struct AdminPaginationQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

/// GET /admin/listings/pending
/// Returns all vendor_products awaiting admin moderation (status = 'pending_approval').
/// This is the new primary moderation surface — accounts are always active;
/// individual Listings go through the pending_approval → active/rejected lifecycle.
async fn list_pending_listings(
    _auth: RequireAdmin,
    mut rls_tx: RlsTx,
    axum::extract::Query(query): axum::extract::Query<AdminPaginationQuery>,
) -> Result<Json<Value>, AppError> {
    tracing::info!("Querying database for pending listing moderation queue...");

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(10).clamp(1, 100);
    let offset = (page - 1) * limit;

    let count_row = sqlx::query(
        "SELECT COUNT(*)::bigint AS count FROM vendor_products WHERE status = 'pending_approval'"
    )
    .fetch_one(&mut *rls_tx.tx)
    .await?;
    let total_count: i64 = count_row.get("count");

    let rows = sqlx::query(
        "SELECT
            vp.id AS listing_id,
            vp.title,
            vp.description,
            vp.slug,
            vp.product_category,
            vp.status,
            vp.base_price_sar::float8 AS base_price_sar,
            vp.created_at,
            vp.rejection_reason,
            vp.attributes,
            v.id AS vendor_id,
            v.name_en AS vendor_name_en,
            v.name_ar AS vendor_name_ar,
            v.email AS vendor_email,
            v.phone AS vendor_phone,
            c.name_en AS city_name_en,
            c.name_ar AS city_name_ar,
            COALESCE(st.name, 'Free') AS current_tier,
            (SELECT count(*) FROM vendor_products wp WHERE wp.vendor_id = v.id AND wp.status != 'archived') AS used_products_count,
            COALESCE((st.policy_limits->>'max_products'), '5')::bigint AS max_products
        FROM vendor_products vp
        JOIN vendors v ON vp.vendor_id = v.id
        LEFT JOIN cities c ON vp.city_id = c.id
        LEFT JOIN subscription_tiers st ON v.subscription_tier_id = st.id
        WHERE vp.status = 'pending_approval'
        ORDER BY vp.created_at ASC
        LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let mut listings_json = Vec::new();
    for row in rows {
        let listing_id: Uuid = row.get("listing_id");
        let vendor_id: Uuid = row.get("vendor_id");
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        let base_price: Option<f64> = row.get("base_price_sar");

        listings_json.push(json!({
            "id": listing_id.to_string(),
            "title": row.get::<String, _>("title"),
            "description": row.get::<Option<String>, _>("description"),
            "slug": row.get::<String, _>("slug"),
            "product_category": row.get::<String, _>("product_category"),
            "status": row.get::<String, _>("status"),
            "base_price_sar": base_price,
            "rejection_reason": row.get::<Option<String>, _>("rejection_reason"),
            "created_at": created_at.to_rfc3339(),
            "attributes": row.get::<serde_json::Value, _>("attributes"),
            "vendor_id": vendor_id.to_string(),
            "vendor_name_en": row.get::<String, _>("vendor_name_en"),
            "vendor_name_ar": row.get::<String, _>("vendor_name_ar"),
            "vendor_email": row.get::<Option<String>, _>("vendor_email"),
            "vendor_phone": row.get::<Option<String>, _>("vendor_phone"),
            "city_name_en": row.get::<Option<String>, _>("city_name_en").unwrap_or_else(|| "Unspecified".to_string()),
            "city_name_ar": row.get::<Option<String>, _>("city_name_ar").unwrap_or_else(|| "غير محدد".to_string()),
        }));
    }

    let total_pages = ((total_count + limit - 1) / limit).max(1);

    Ok(Json(json!({
        "status": "success",
        "listings": listings_json,
        "total": total_count,
        "page": page,
        "totalPages": total_pages,
    })))
}

/// GET /admin/listings
/// Returns all vendor_products regardless of status.
async fn list_all_listings(
    _auth: RequireAdmin,
    mut rls_tx: RlsTx,
    axum::extract::Query(query): axum::extract::Query<AdminPaginationQuery>,
) -> Result<Json<Value>, AppError> {
    tracing::info!("Querying database for all listings...");

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(10).clamp(1, 100);
    let offset = (page - 1) * limit;

    let count_row = sqlx::query(
        "SELECT COUNT(*)::bigint AS count FROM vendor_products"
    )
    .fetch_one(&mut *rls_tx.tx)
    .await?;
    let total_count: i64 = count_row.get("count");

    let rows = sqlx::query(
        "SELECT
            vp.id AS listing_id,
            vp.title,
            vp.title_ar,
            vp.title_en,
            vp.description,
            vp.slug,
            vp.product_category,
            vp.status,
            vp.base_price_sar::float8 AS base_price_sar,
            vp.created_at,
            vp.rejection_reason,
            vp.attributes,
            v.id AS vendor_id,
            v.name_en AS vendor_name_en,
            v.name_ar AS vendor_name_ar,
            v.email AS vendor_email,
            v.phone AS vendor_phone,
            c.name_en AS city_name_en,
            c.name_ar AS city_name_ar,
            COALESCE(st.name, 'Free') AS current_tier,
            (SELECT count(*) FROM vendor_products wp WHERE wp.vendor_id = v.id AND wp.status != 'archived') AS used_products_count,
            COALESCE((st.policy_limits->>'max_products'), '5')::bigint AS max_products
        FROM vendor_products vp
        JOIN vendors v ON vp.vendor_id = v.id
        LEFT JOIN cities c ON vp.city_id = c.id
        LEFT JOIN subscription_tiers st ON v.subscription_tier_id = st.id
        ORDER BY vp.created_at DESC
        LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let mut listings_json = Vec::new();
    for row in rows {
        let listing_id: Uuid = row.get("listing_id");
        let vendor_id: Uuid = row.get("vendor_id");
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        let base_price: Option<f64> = row.get("base_price_sar");

        listings_json.push(json!({
            "id": listing_id.to_string(),
            "title": row.get::<String, _>("title"),
            "titleAr": row.get::<Option<String>, _>("title_ar"),
            "titleEn": row.get::<Option<String>, _>("title_en"),
            "description": row.get::<Option<String>, _>("description"),
            "slug": row.get::<String, _>("slug"),
            "product_category": row.get::<String, _>("product_category"),
            "status": row.get::<String, _>("status"),
            "base_price_sar": base_price,
            "rejection_reason": row.get::<Option<String>, _>("rejection_reason"),
            "created_at": created_at.to_rfc3339(),
            "attributes": row.get::<serde_json::Value, _>("attributes"),
            "vendor_id": vendor_id.to_string(),
            "vendor_name_en": row.get::<String, _>("vendor_name_en"),
            "vendor_name_ar": row.get::<String, _>("vendor_name_ar"),
            "vendor_email": row.get::<Option<String>, _>("vendor_email"),
            "vendor_phone": row.get::<Option<String>, _>("vendor_phone"),
            "city_name_en": row.get::<Option<String>, _>("city_name_en").unwrap_or_else(|| "Unspecified".to_string()),
            "city_name_ar": row.get::<Option<String>, _>("city_name_ar").unwrap_or_else(|| "غير محدد".to_string()),
        }));
    }

    let total_pages = ((total_count + limit - 1) / limit).max(1);

    Ok(Json(json!({
        "status": "success",
        "listings": listings_json,
        "total": total_count,
        "page": page,
        "totalPages": total_pages,
    })))
}

async fn list_all_vendors(
    _auth: RequireAdmin,
    mut rls_tx: RlsTx,
    axum::extract::Query(query): axum::extract::Query<AdminPaginationQuery>,
) -> Result<Json<Value>, AppError> {
    tracing::info!("Querying all vendors for administrative management...");

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(10).clamp(1, 100);
    let offset = (page - 1) * limit;

    let count_row = sqlx::query(
        "SELECT COUNT(*)::bigint AS count FROM vendors"
    )
    .fetch_one(&mut *rls_tx.tx)
    .await?;
    let total_count: i64 = count_row.get("count");

    let rows = sqlx::query(
        "SELECT 
            v.id,
            v.name_ar,
            v.name_en,
            v.slug,
            v.category,
            v.status,
            v.is_verified,
            v.created_at,
            v.email,
            v.phone,
            v.subscription_status,
            v.is_featured,
            v.featured_expires_at,
            c.name_en AS city_name_en,
            c.name_ar AS city_name_ar,
            COALESCE(
                CASE WHEN v.subscription_expires_at < CURRENT_TIMESTAMP THEN 'Free' ELSE st.name END, 
                'Free'
            ) AS current_tier,
            (SELECT count(*) FROM vendor_products wp WHERE wp.vendor_id = v.id AND wp.status != 'archived') AS used_products_count,
            COALESCE(
                CASE WHEN v.subscription_expires_at < CURRENT_TIMESTAMP THEN '1' ELSE st.policy_limits->>'max_products' END, 
                '1'
            )::bigint AS max_products
        FROM vendors v
        LEFT JOIN cities c ON v.city_id = c.id
        LEFT JOIN subscription_tiers st ON v.subscription_tier_id = st.id
        ORDER BY v.created_at DESC
        LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let mut vendors_json = Vec::new();

    for row in rows {
        let id: Uuid = row.get("id");
        let name_ar: String = row.get("name_ar");
        let name_en: String = row.get("name_en");
        let slug: String = row.get("slug");
        let category: Option<String> = row.get("category");
        let status: String = row.get("status");
        let is_verified: bool = row.get("is_verified");
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        let email: Option<String> = row.get("email");
        let phone: Option<String> = row.get("phone");
        let subscription_status: String = row.get("subscription_status");
        let is_featured: bool = row.get("is_featured");
        let featured_expires_at: Option<chrono::DateTime<chrono::Utc>> =
            row.get("featured_expires_at");
        let city_name_en: Option<String> = row.get("city_name_en");
        let city_name_ar: Option<String> = row.get("city_name_ar");
        let current_tier: String = row.get("current_tier");
        let used_products_count: i64 = row.get("used_products_count");
        let max_products: i64 = row.get("max_products");

        vendors_json.push(json!({
            "id": id.to_string(),
            "name_ar": name_ar,
            "name_en": name_en,
            "slug": slug,
            "category": category.unwrap_or_default(),
            "status": status,
            "is_verified": is_verified,
            "created_at": created_at.to_rfc3339(),
            "email": email.unwrap_or_default(),
            "phone": phone.unwrap_or_default(),
            "subscription_status": subscription_status,
            "is_featured": is_featured,
            "featured_expires_at": featured_expires_at.map(|dt| dt.to_rfc3339()),
            "city_name_en": city_name_en.unwrap_or_else(|| "Unknown".to_string()),
            "city_name_ar": city_name_ar.unwrap_or_else(|| "غير معروف".to_string()),
            "current_tier": current_tier,
            "used_products_count": used_products_count,
            "max_products": max_products,
        }));
    }

    let total_pages = ((total_count + limit - 1) / limit).max(1);

    Ok(Json(json!({
        "status": "success",
        "vendors": vendors_json,
        "total": total_count,
        "page": page,
        "totalPages": total_pages,
    })))
}

/// PATCH /admin/vendors/:id/approve
/// Reactivates a vendor account that was previously suspended or banned.
/// NOTE: In the new architecture this is no longer an "onboarding approval" —
/// vendor accounts are active by default after registration.
async fn reactivate_vendor(
    State(state): State<AppState>,
    auth: RequireAdmin,
    Path(id): Path<String>,
) -> Result<Json<Value>, AppError> {
    tracing::info!("Reactivating vendor account with ID: {}", id);

    let vendor_uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid vendor UUID format".to_string()))?;

    let admin_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid admin user ID".to_string()))?;

    let mut tx = state.db.begin().await?;

    // Fetch vendor name and current status for audit log
    let vendor_row =
        sqlx::query("SELECT user_id, name_en, name_ar, status FROM vendors WHERE id = $1")
            .bind(vendor_uuid)
            .fetch_optional(&mut *tx)
            .await?;

    let vendor = match vendor_row {
        Some(row) => row,
        None => return Err(AppError::NotFound("Vendor not found".to_string())),
    };

    let vendor_user_id: Option<Uuid> = vendor.get("user_id");
    let vendor_name_en: String = vendor.get("name_en");
    let vendor_name_ar: String = vendor.get("name_ar");
    let current_status: String = vendor.get("status");

    if current_status == "active" {
        return Err(AppError::BadRequest(
            "Vendor account is already active".to_string(),
        ));
    }

    // Reactivate the account — the DB trigger will auto-restore suspended listings
    sqlx::query(
        "UPDATE vendors SET status = 'active', is_verified = TRUE, updated_at = CURRENT_TIMESTAMP WHERE id = $1"
    )
    .bind(vendor_uuid)
    .execute(&mut *tx)
    .await?;

    // Status transition history
    sqlx::query(
        "INSERT INTO status_history (entity_type, entity_id, old_status, new_status, changed_by, reason, vendor_id)
         VALUES ('vendor', $1, $2, 'active', $3, 'Vendor reactivated by admin', $1)",
    )
    .bind(vendor_uuid)
    .bind(&current_status)
    .bind(admin_uuid)
    .execute(&mut *tx)
    .await?;

    // Admin audit log
    sqlx::query(
        "INSERT INTO admin_audit_logs (entity_type, entity_id, actor_id, action, before_state, after_state)
         VALUES ('vendor', $1, $2, 'reactivate_vendor', $3, $4)",
    )
    .bind(vendor_uuid)
    .bind(admin_uuid)
    .bind(json!({ "status": current_status }))
    .bind(json!({ "status": "active" }))
    .execute(&mut *tx)
    .await?;

    // Vendor-facing notification
    if let Some(target_user_id) = vendor_user_id {
        sqlx::query(
            "INSERT INTO system_events (user_id, target_vendor_id, event_type, message_ar, message_en)
             VALUES ($1, $2, 'vendor_approved', $3, $4)"
        )
        .bind(target_user_id)
        .bind(vendor_uuid)
        .bind(format!("تم إعادة تفعيل حسابكم التجاري «{}» على منصة زفاف وورلد.", vendor_name_ar))
        .bind(format!("Your vendor account '{}' has been reactivated on ZafafWorld.", vendor_name_en))
        .execute(&mut *tx)
        .await?;
    }

    // Admin-side audit trail
    sqlx::query(
        "INSERT INTO system_events (user_id, target_vendor_id, event_type, message_ar, message_en)
         VALUES ($1, $2, 'vendor_approved', $3, $4)",
    )
    .bind(admin_uuid)
    .bind(vendor_uuid)
    .bind(format!(
        "قام المسؤول بإعادة تفعيل المورد «{}» من حالة '{}'",
        vendor_name_ar, current_status
    ))
    .bind(format!(
        "Administrator reactivated vendor '{}' from '{}' status",
        vendor_name_en, current_status
    ))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    tracing::info!(
        target: "audit",
        actor_id  = %auth.user_id,
        event     = "vendor_reactivated",
        target_id = %vendor_uuid,
        prev      = %current_status,
        next      = "active",
        "Admin vendor reactivation committed"
    );

    Ok(Json(json!({
        "status": "success",
        "message": format!("Vendor '{}' account successfully reactivated", id)
    })))
}

/// Universal vendor status kill-switch — supports approved, suspended, rejected transitions
async fn update_vendor_status(
    State(state): State<AppState>,
    auth: RequireAdmin,
    Path(id): Path<String>,
    Json(input): Json<UpdateVendorStatusInput>,
) -> Result<Json<Value>, AppError> {
    tracing::info!("Updating vendor {} status to: {}", id, input.status);

    let vendor_uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid vendor UUID format".to_string()))?;

    let admin_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid admin user ID".to_string()))?;

    // Validate target status against new 3-value account health model
    let allowed_statuses = ["active", "suspended", "banned"];
    if !allowed_statuses.contains(&input.status.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Invalid status '{}'. Must be one of: active, suspended, banned",
            input.status
        )));
    }

    let mut tx = state.db.begin().await?;

    // Fetch vendor details for audit logging
    let vendor_row =
        sqlx::query("SELECT user_id, name_en, name_ar, status FROM vendors WHERE id = $1")
            .bind(vendor_uuid)
            .fetch_optional(&mut *tx)
            .await?;

    let vendor = match vendor_row {
        Some(row) => row,
        None => return Err(AppError::NotFound("Vendor not found".to_string())),
    };

    let vendor_user_id: Option<Uuid> = vendor.get("user_id");
    let vendor_name_en: String = vendor.get("name_en");
    let vendor_name_ar: String = vendor.get("name_ar");
    let current_status: String = vendor.get("status");

    if current_status == input.status {
        return Err(AppError::BadRequest(format!(
            "Vendor is already in '{}' status",
            input.status
        )));
    }

    // Determine is_verified flag: only active vendors are verified
    let is_verified = input.status == "active";

    // Perform atomic status update — the DB trigger handles cascade suspend/restore on listings
    let result = sqlx::query(
        "UPDATE vendors SET status = $1, is_verified = $2, updated_at = CURRENT_TIMESTAMP WHERE id = $3"
    )
    .bind(&input.status)
    .bind(is_verified)
    .bind(vendor_uuid)
    .execute(&mut *tx)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Vendor not found".to_string()));
    }

    // Status transition history
    sqlx::query(
        "INSERT INTO status_history (entity_type, entity_id, old_status, new_status, changed_by, reason, vendor_id)
         VALUES ('vendor', $1, $2, $3, $4, $5, $6)",
    )
    .bind(vendor_uuid)
    .bind(&current_status)
    .bind(&input.status)
    .bind(admin_uuid)
    .bind(input.reason.as_deref().unwrap_or("Status updated by admin"))
    .bind(vendor_uuid)
    .execute(&mut *tx)
    .await?;

    // Admin audit log
    sqlx::query(
        "INSERT INTO admin_audit_logs (entity_type, entity_id, actor_id, action, before_state, after_state)
         VALUES ('vendor', $1, $2, 'update_vendor_status', $3, $4)",
    )
    .bind(vendor_uuid)
    .bind(admin_uuid)
    .bind(json!({ "status": current_status }))
    .bind(json!({ "status": input.status }))
    .execute(&mut *tx)
    .await?;

    // Determine event_type for system_events
    let event_type = match input.status.as_str() {
        "active" => "vendor_approved",
        "suspended" => "vendor_suspended",
        "banned" => "vendor_rejected",
        _ => "system_alert",
    };

    let reason_suffix_en = input.reason.as_deref().unwrap_or("");
    let reason_suffix_ar = if reason_suffix_en.is_empty() {
        ""
    } else {
        reason_suffix_en
    };

    // Vendor-facing notification event
    if let Some(target_user_id) = vendor_user_id {
        let (msg_ar, msg_en) = match input.status.as_str() {
            "active" => (
                format!(
                    "تم إعادة تفعيل حسابكم التجاري «{}» على منصة زفاف وورلد.",
                    vendor_name_ar
                ),
                format!(
                    "Your vendor account '{}' has been reactivated on ZafafWorld.",
                    vendor_name_en
                ),
            ),
            "suspended" => (
                format!(
                    "تم تعليق حسابكم التجاري «{}» مؤقتاً. {}",
                    vendor_name_ar, reason_suffix_ar
                ),
                format!(
                    "Your vendor account '{}' has been temporarily suspended. {}",
                    vendor_name_en, reason_suffix_en
                ),
            ),
            "banned" => (
                format!(
                    "تم إيقاف حسابكم التجاري «{}» بشكل دائم. {}",
                    vendor_name_ar, reason_suffix_ar
                ),
                format!(
                    "Your vendor account '{}' has been permanently banned. {}",
                    vendor_name_en, reason_suffix_en
                ),
            ),
            _ => (
                format!("تم تحديث حالة حسابكم التجاري «{}».", vendor_name_ar),
                format!(
                    "Your vendor account '{}' status has been updated.",
                    vendor_name_en
                ),
            ),
        };

        sqlx::query(
            "INSERT INTO system_events (user_id, target_vendor_id, event_type, message_ar, message_en) 
             VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(target_user_id)
        .bind(vendor_uuid)
        .bind(event_type)
        .bind(&msg_ar)
        .bind(&msg_en)
        .execute(&mut *tx)
        .await?;
    }

    // Admin-side audit trail
    sqlx::query(
        "INSERT INTO system_events (user_id, target_vendor_id, event_type, message_ar, message_en) 
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(admin_uuid)
    .bind(vendor_uuid)
    .bind(event_type)
    .bind(format!(
        "قام المسؤول بتغيير حالة المورد «{}» من '{}' إلى '{}'",
        vendor_name_ar, current_status, input.status
    ))
    .bind(format!(
        "Administrator changed vendor '{}' status from '{}' to '{}'",
        vendor_name_en, current_status, input.status
    ))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    tracing::info!(
        target: "audit",
        actor_id  = %auth.user_id,
        event     = "vendor_status_changed",
        target_id = %vendor_uuid,
        prev      = %current_status,
        next      = %input.status,
        "Admin vendor status mutation committed"
    );

    Ok(Json(json!({
        "status": "success",
        "message": format!("Vendor status updated to '{}'", input.status),
        "previous_status": current_status,
        "new_status": input.status
    })))
}


async fn update_vendor_subscription(
    mut rls_tx: RlsTx,
    _auth: RequireAdmin,
    Path(id): Path<String>,
    Json(input): Json<UpdateVendorSubscriptionInput>,
) -> Result<Json<Value>, AppError> {
    tracing::info!(
        "Updating vendor {} subscription. status: {}, tier: {:?}, expiry: {:?}",
        id,
        input.subscription_status,
        input.subscription_tier_id,
        input.subscription_expires_at
    );

    let vendor_uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid vendor UUID format".to_string()))?;

    let allowed_statuses = ["trial", "active", "stopped"];
    if !allowed_statuses.contains(&input.subscription_status.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Invalid subscription status '{}'. Must be one of: trial, active, stopped",
            input.subscription_status
        )));
    }

    let tier_uuid = if let Some(t_id) = &input.subscription_tier_id {
        if t_id.trim().is_empty() {
            None
        } else {
            Some(
                Uuid::parse_str(t_id)
                    .map_err(|_| AppError::BadRequest("Invalid tier UUID format".to_string()))?,
            )
        }
    } else {
        None
    };

    let expires_at = if let Some(exp) = &input.subscription_expires_at {
        if exp.trim().is_empty() {
            None
        } else if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(exp) {
            Some(dt.with_timezone(&chrono::Utc))
        } else if let Ok(d) = chrono::NaiveDate::parse_from_str(exp, "%Y-%m-%d") {
            // SAFETY: Statically valid hour/minute/second parameters (23, 59, 59)
            Some(d.and_hms_opt(23, 59, 59).unwrap().and_utc())
        } else {
            return Err(AppError::BadRequest(
                "Invalid expiry date format".to_string(),
            ));
        }
    } else {
        None
    };

    if input.subscription_tier_id.is_some() || input.subscription_expires_at.is_some() {
        let mut query_builder =
            String::from("UPDATE vendors SET subscription_status = $1, updated_at = NOW()");
        let mut idx = 3;

        if input.subscription_tier_id.is_some() {
            query_builder.push_str(&format!(", subscription_tier_id = ${}", idx));
            idx += 1;
        }

        if input.subscription_expires_at.is_some() {
            query_builder.push_str(&format!(", subscription_expires_at = ${}", idx));
        }

        query_builder.push_str(" WHERE id = $2");

        let mut query = sqlx::query(&query_builder)
            .bind(&input.subscription_status)
            .bind(vendor_uuid);

        if input.subscription_tier_id.is_some() {
            query = query.bind(tier_uuid);
        }

        if input.subscription_expires_at.is_some() {
            query = query.bind(expires_at);
        }

        query.execute(&mut *rls_tx.tx).await?;
    } else {
        sqlx::query(
            "UPDATE vendors
             SET subscription_status = $1, updated_at = NOW()
             WHERE id = $2",
        )
        .bind(&input.subscription_status)
        .bind(vendor_uuid)
        .execute(&mut *rls_tx.tx)
        .await?;
    }

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Vendor subscription updated successfully"
    })))
}




async fn update_vendor_featured(
    mut rls_tx: RlsTx,
    _auth: RequireAdmin,
    Path(id): Path<String>,
    Json(input): Json<UpdateVendorFeaturedInput>,
) -> Result<Json<Value>, AppError> {
    tracing::info!(
        "Updating vendor {} featured status to: {} (expires: {:?})",
        id,
        input.is_featured,
        input.expires_at
    );

    let vendor_uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid vendor UUID format".to_string()))?;

    // Validate temporal boundary constraints
    if input.is_featured {
        if let Some(expiry) = input.expires_at {
            if expiry <= chrono::Utc::now() {
                return Err(AppError::BadRequest(
                    "Featured placement expiry timeline must be set in the future".to_string(),
                ));
            }
        }
    }

    sqlx::query(
        "UPDATE vendors
         SET is_featured = $1, featured_expires_at = $2, updated_at = NOW()
         WHERE id = $3",
    )
    .bind(input.is_featured)
    .bind(input.expires_at)
    .bind(vendor_uuid)
    .execute(&mut *rls_tx.tx)
    .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Vendor featured configuration updated successfully"
    })))
}


// ─── ADMIN USERS DIRECTORY ───────────────────────────────────────────────────




async fn list_vendor_subscription_requests(
    State(state): State<AppState>,
    _auth: RequireAdmin,
) -> Result<Json<Value>, AppError> {
    let rows = sqlx::query!(
        "SELECT r.id, r.vendor_id, r.requested_tier_id, r.status, r.admin_notes, r.created_at, r.updated_at,
                v.name_en as vendor_name_en, v.name_ar as vendor_name_ar,
                t.name as requested_tier_name
         FROM vendor_subscription_requests r
         JOIN vendors v ON r.vendor_id = v.id
         JOIN subscription_tiers t ON r.requested_tier_id = t.id
         ORDER BY CASE WHEN r.status = 'pending' THEN 0 ELSE 1 END, r.created_at DESC"
    )
    .fetch_all(&state.db)
    .await?;

    let requests: Vec<Value> = rows
        .into_iter()
        .map(|row| {
            json!({
                "id": row.id.to_string(),
                "vendor_id": row.vendor_id.to_string(),
                "vendor_name_en": row.vendor_name_en,
                "vendor_name_ar": row.vendor_name_ar,
                "requested_tier_id": row.requested_tier_id.to_string(),
                "requested_tier_name": row.requested_tier_name,
                "status": row.status,
                "admin_notes": row.admin_notes,
                "created_at": row.created_at,
                "updated_at": row.updated_at,
            })
        })
        .collect();

    Ok(Json(json!({
        "status": "success",
        "requests": requests
    })))
}

#[derive(serde::Deserialize)]
pub struct ApproveSubscriptionRequestInput {
    pub admin_notes: Option<String>,
}

async fn approve_vendor_subscription_request(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Path(id): Path<String>,
    Json(input): Json<ApproveSubscriptionRequestInput>,
) -> Result<Json<Value>, AppError> {
    let request_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid request ID format".to_string()))?;

    let mut tx = state.db.begin().await?;

    // 1. Fetch the request
    let req_row = sqlx::query!("SELECT vendor_id, requested_tier_id, status FROM vendor_subscription_requests WHERE id = $1", request_id)
        .fetch_optional(&mut *tx)
        .await?;

    let req = req_row.ok_or_else(|| AppError::NotFound("Request not found".to_string()))?;

    if req.status != "pending" {
        return Err(AppError::BadRequest(
            "Only pending requests can be approved".to_string(),
        ));
    }

    // 2. Update request status
    sqlx::query!(
        "UPDATE vendor_subscription_requests SET status = 'approved', admin_notes = $1 WHERE id = $2",
        input.admin_notes,
        request_id
    )
    .execute(&mut *tx)
    .await?;

    // 3. Update vendor tier and expiry (1 year)
    sqlx::query!(
        "UPDATE vendors SET subscription_tier_id = $1, subscription_expires_at = NOW() + INTERVAL '1 year', subscription_status = 'active' WHERE id = $2",
        req.requested_tier_id,
        req.vendor_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Subscription request approved successfully"
    })))
}

#[derive(serde::Deserialize)]
pub struct RejectSubscriptionRequestInput {
    pub admin_notes: Option<String>,
}

async fn reject_vendor_subscription_request(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Path(id): Path<String>,
    Json(input): Json<RejectSubscriptionRequestInput>,
) -> Result<Json<Value>, AppError> {
    let request_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid request ID format".to_string()))?;

    let mut tx = state.db.begin().await?;

    let req_row = sqlx::query!(
        "SELECT status FROM vendor_subscription_requests WHERE id = $1",
        request_id
    )
    .fetch_optional(&mut *tx)
    .await?;

    let req = req_row.ok_or_else(|| AppError::NotFound("Request not found".to_string()))?;

    if req.status != "pending" {
        return Err(AppError::BadRequest(
            "Only pending requests can be rejected".to_string(),
        ));
    }

    sqlx::query!(
        "UPDATE vendor_subscription_requests SET status = 'rejected', admin_notes = $1 WHERE id = $2",
        input.admin_notes,
        request_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Subscription request rejected successfully"
    })))
}




async fn get_admin_vendors_context(
    _auth: RequireAdmin,
    mut rls_tx: RlsTx,
    State(_state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    // 1. Vendors
    let vendors_rows = sqlx::query(
        "SELECT v.id, v.slug, v.name_en, v.name_ar, v.email, v.phone, v.category, v.status, 
                v.subscription_status, v.subscription_tier_id, v.is_featured, v.featured_expires_at, v.created_at,
                c.name_en as city_name_en, c.name_ar as city_name_ar
         FROM vendors v
         LEFT JOIN cities c ON v.city_id = c.id
         ORDER BY v.created_at DESC"
    )
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let mut vendors = Vec::new();
    for row in vendors_rows {
        vendors.push(json!({
            "id": row.get::<Uuid, _>("id").to_string(),
            "slug": row.get::<String, _>("slug"),
            "name_en": row.get::<String, _>("name_en"),
            "name_ar": row.get::<String, _>("name_ar"),
            "email": row.try_get::<Option<String>, _>("email").unwrap_or_default(),
            "phone": row.try_get::<Option<String>, _>("phone").unwrap_or_default(),
            "category": row.try_get::<Option<String>, _>("category").unwrap_or_default(),
            "status": row.get::<String, _>("status"),
            "subscription_status": row.get::<String, _>("subscription_status"),
            "subscription_tier_id": row.try_get::<Option<Uuid>, _>("subscription_tier_id").unwrap_or_default().map(|u| u.to_string()),
            "is_featured": row.get::<bool, _>("is_featured"),
            "featured_expires_at": row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("featured_expires_at").unwrap_or_default().map(|t| t.to_rfc3339()),
            "created_at": row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("created_at").unwrap_or_default().map(|t| t.to_rfc3339()),
            "city_name_en": row.try_get::<Option<String>, _>("city_name_en").unwrap_or_default(),
            "city_name_ar": row.try_get::<Option<String>, _>("city_name_ar").unwrap_or_default()
        }));
    }

    // 2. Pending Listings
    let pending_rows = sqlx::query(
        "SELECT p.id, p.vendor_id, p.product_category, p.title_en, p.title_ar, p.status, p.created_at,
                v.name_en as vendor_name_en, c.name_en as city_name_en
         FROM vendor_products p
         JOIN vendors v ON p.vendor_id = v.id
         LEFT JOIN cities c ON v.city_id = c.id
         WHERE p.status = 'pending'
         ORDER BY p.created_at DESC"
    )
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let mut pending_listings = Vec::new();
    for row in pending_rows {
        pending_listings.push(json!({
            "id": row.get::<Uuid, _>("id").to_string(),
            "vendor_id": row.get::<Uuid, _>("vendor_id").to_string(),
            "product_category": row.get::<String, _>("product_category"),
            "title_en": row.get::<String, _>("title_en"),
            "title_ar": row.get::<String, _>("title_ar"),
            "status": row.get::<String, _>("status"),
            "created_at": row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("created_at").unwrap_or_default().map(|t| t.to_rfc3339()),
            "vendor_name_en": row.get::<String, _>("vendor_name_en"),
            "city_name_en": row.try_get::<Option<String>, _>("city_name_en").unwrap_or_default()
        }));
    }

    // 3. Tiers
    let tiers_rows = sqlx::query(
        "SELECT id, name, priority_score, price::FLOAT8 as price, billing_cycle
         FROM subscription_tiers
         ORDER BY priority_score ASC"
    )
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let mut tiers = Vec::new();
    for row in tiers_rows {
        tiers.push(json!({
            "id": row.get::<Uuid, _>("id").to_string(),
            "name": row.get::<String, _>("name"),
            "priorityScore": row.get::<i32, _>("priority_score"),
            "price": row.get::<f64, _>("price"),
            "billingCycle": row.get::<String, _>("billing_cycle")
        }));
    }

    Ok(Json(json!({
        "status": "success",
        "vendors": vendors,
        "pendingListings": pending_listings,
        "tiers": tiers
    })))
}
