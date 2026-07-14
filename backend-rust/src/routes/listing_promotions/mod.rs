#![allow(clippy::explicit_auto_deref)]

use axum::{
    extract::{Multipart, Path, State},
    routing::post,
    Json, Router,
};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::LazyLock;
use std::time::Instant;
use uuid::Uuid;

use crate::errors::AppError;
use crate::middleware::auth::{RequireAdmin, RequireVendor, RlsTx};
use crate::state::AppState;
use crate::utils::ip::SecureClientIp;

// In-memory deduplication cache for view/click tracking (5-minute sliding window)
static TRACK_DEDUPLICATION_CACHE: LazyLock<DashMap<String, Instant>> = LazyLock::new(DashMap::new);

pub fn router(_state: AppState) -> Router<AppState> {
    Router::new()
        // Public/Client APIs
        .route(
            "/public/promotions",
            axum::routing::get(list_public_promotions_handler),
        )
        .route(
            "/public/promotions/:id/track",
            post(track_promotion_handler),
        )
        // Vendor/Protected APIs
        .route(
            "/vendor/promotions",
            post(create_promotion_handler).get(list_vendor_promotions_handler),
        )
        .route(
            "/vendor/promotions/:id",
            axum::routing::get(get_promotion_handler)
                .put(update_promotion_handler)
                .delete(delete_promotion_handler),
        )
        // Banner cleanup (must come before /:id to avoid param capture)
        .route(
            "/vendor/promotions/cleanup-banner",
            axum::routing::delete(cleanup_promotion_banner_handler),
        )
        .route(
            "/vendor/promotions/:id/duplicate",
            post(duplicate_promotion_handler),
        )
        .route(
            "/vendor/promotions/:id/renew",
            post(renew_promotion_handler),
        )
        .route(
            "/vendor/promotions/:id/pause",
            post(pause_promotion_handler),
        )
        .route(
            "/vendor/promotions/:id/resume",
            post(resume_promotion_handler),
        )
        // Admin Moderation APIs
        .route(
            "/admin/promotions",
            axum::routing::get(list_admin_promotions_handler),
        )
        .route(
            "/admin/promotions/:id/approve",
            post(approve_promotion_handler),
        )
        .route(
            "/admin/promotions/:id/reject",
            post(reject_promotion_handler),
        )
        // Banner upload
        .route(
            "/vendor/promotions/upload-banner",
            post(upload_promotion_banner_handler),
        )
}

use rust_decimal::Decimal;

#[derive(Debug, Deserialize, Clone)]
pub struct CreatePromotionPayload {
    pub listing_id: Uuid,
    pub promo_type: String, // "discount" or "benefit"
    pub discount_type: Option<String>, // "percentage" or "fixed_amount"
    pub discount_percentage: Option<i32>,
    pub discount_fixed_amount: Option<Decimal>,
    pub benefit_description_en: Option<String>,
    pub benefit_description_ar: Option<String>,
    pub use_listing_cover_image: bool,
    pub custom_banner_image_url: Option<String>,
    pub title_en: String,
    pub title_ar: String,
    pub description_en: Option<String>,
    pub description_ar: Option<String>,
    pub badge_text_en: Option<String>,
    pub badge_text_ar: Option<String>,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct TrackPromotionPayload {
    pub event_type: String, // "view" or "click"
}

#[derive(Debug, Deserialize, Clone)]
pub struct PublicPromotionsQuery {
    pub category: Option<String>,
    pub city_id: Option<Uuid>,
    pub vendor_id: Option<Uuid>,
    #[allow(dead_code)]
    pub min_discount: Option<i32>,
    #[allow(dead_code)]
    pub max_discount: Option<i32>,
    pub search: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct VendorPromotionsQuery {
    pub status: Option<String>,
    pub search: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AdminPromotionsQuery {
    pub status: Option<String>,
    pub vendor_id: Option<Uuid>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct RejectPromotionPayload {
    pub rejection_reason: String,
}

// --- 1. Validation Layer ---

pub struct PromotionValidator;

impl PromotionValidator {
    fn count_words(text: &str) -> usize {
        let mut in_tag = false;
        let mut clean_text = String::with_capacity(text.len());
        for c in text.chars() {
            if c == '<' {
                in_tag = true;
                clean_text.push(' ');
            } else if c == '>' {
                in_tag = false;
            } else if !in_tag {
                clean_text.push(c);
            }
        }
        clean_text.split_whitespace().filter(|w| !w.is_empty()).count()
    }

    pub fn validate_create(payload: &CreatePromotionPayload) -> Result<(), AppError> {
        if payload.listing_id.is_nil() {
            return Err(AppError::BadRequest(
                "Listing target is required.".to_string(),
            ));
        }

        // ── Word count limit validation (max 2000 words after stripping HTML) ──
        if let Some(ref desc_en) = payload.description_en {
            if Self::count_words(desc_en) > 2000 {
                return Err(AppError::BadRequest(
                    "Description (English) must not exceed 2000 words.".to_string(),
                ));
            }
        }
        if let Some(ref desc_ar) = payload.description_ar {
            if Self::count_words(desc_ar) > 2000 {
                return Err(AppError::BadRequest(
                    "Description (Arabic) must not exceed 2000 words.".to_string(),
                ));
            }
        }

        // ── Length validation (VARCHAR(255) fields) ──────────────────────────
        if payload.title_en.len() > 255 {
            return Err(AppError::BadRequest(
                "Title (English) must not exceed 255 characters.".to_string(),
            ));
        }
        if payload.title_ar.len() > 255 {
            return Err(AppError::BadRequest(
                "Title (Arabic) must not exceed 255 characters.".to_string(),
            ));
        }
        if payload.title_en.trim().is_empty() || payload.title_ar.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Both English and Arabic titles are required.".to_string(),
            ));
        }

        // ── Promotion type validation ────────────────────────────────────────
        if payload.promo_type != "discount" && payload.promo_type != "benefit" {
            return Err(AppError::BadRequest(
                "Invalid promotion type. Must be 'discount' or 'benefit'.".to_string(),
            ));
        }

        if payload.promo_type == "discount" {
            let dtype = payload.discount_type.as_deref().unwrap_or("");
            if dtype != "percentage" && dtype != "fixed_amount" {
                return Err(AppError::BadRequest(
                    "Discount type must be 'percentage' or 'fixed_amount'.".to_string(),
                ));
            }

            if dtype == "percentage" {
                let pct = payload.discount_percentage.unwrap_or(0);
                if !(5..=90).contains(&pct) {
                    return Err(AppError::BadRequest(
                        "Discount percentage must be between 5 and 90.".to_string(),
                    ));
                }
            } else {
                let amount = payload.discount_fixed_amount.unwrap_or(Decimal::ZERO);
                if amount <= Decimal::ZERO {
                    return Err(AppError::BadRequest(
                        "Fixed discount amount must be greater than 0.".to_string(),
                    ));
                }
            }
        } else {
            // benefit type: required + length checks
            let desc_en = payload.benefit_description_en.as_deref().unwrap_or("").trim();
            let desc_ar = payload.benefit_description_ar.as_deref().unwrap_or("").trim();
            if desc_en.is_empty() || desc_ar.is_empty() {
                return Err(AppError::BadRequest(
                    "Benefit descriptions in both English and Arabic are required.".to_string(),
                ));
            }
            if desc_en.len() > 255 {
                return Err(AppError::BadRequest(
                    "Benefit description (English) must not exceed 255 characters.".to_string(),
                ));
            }
            if desc_ar.len() > 255 {
                return Err(AppError::BadRequest(
                    "Benefit description (Arabic) must not exceed 255 characters.".to_string(),
                ));
            }
        }

        if payload.end_at <= payload.start_at {
            return Err(AppError::BadRequest(
                "End date/time must be after start date/time.".to_string(),
            ));
        }

        Ok(())
    }

    pub fn validate_update(payload: &CreatePromotionPayload) -> Result<(), AppError> {
        Self::validate_create(payload)
    }
}

fn sanitize_html(input: &str) -> String {
    let mut builder = ammonia::Builder::new();
    
    let tags: std::collections::HashSet<&str> = ["p", "br", "strong", "b", "ul", "ol", "li"].into_iter().collect();
    builder.tags(tags);
    
    builder.generic_attributes(std::collections::HashSet::new());
    builder.tag_attributes(std::collections::HashMap::new());
    
    let clean_content_tags: std::collections::HashSet<&str> = ["script", "style"].into_iter().collect();
    builder.clean_content_tags(clean_content_tags);
    
    builder.clean(input).to_string()
}

// --- 2. Repository Layer ---

pub struct PromotionRepository;

impl PromotionRepository {
    pub async fn find_vendor_by_user_id(
        conn: &mut sqlx::PgConnection,
        user_uuid: Uuid,
    ) -> Result<Uuid, AppError> {
        let vendor = sqlx::query!("SELECT id FROM vendors WHERE user_id = $1", user_uuid)
            .fetch_optional(conn)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        match vendor {
            Some(v) => Ok(v.id),
            None => Err(AppError::Forbidden("Vendor registration not found".into())),
        }
    }

    pub async fn check_listing_ownership(
        conn: &mut sqlx::PgConnection,
        listing_id: Uuid,
        vendor_id: Uuid,
    ) -> Result<(), AppError> {
        let ownership_check = sqlx::query!(
            "SELECT id FROM vendor_products WHERE id = $1 AND vendor_id = $2",
            listing_id,
            vendor_id
        )
        .fetch_optional(conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        if ownership_check.is_none() {
            return Err(AppError::Forbidden(format!(
                "Listing {} does not belong to this vendor",
                listing_id
            )));
        }
        Ok(())
    }

    pub async fn create_promotion(
        conn: &mut sqlx::PgConnection,
        vendor_id: Uuid,
        payload: &CreatePromotionPayload,
    ) -> Result<Uuid, AppError> {
        let sanitized_desc_en = payload.description_en.as_ref().map(|s| sanitize_html(s));
        let sanitized_desc_ar = payload.description_ar.as_ref().map(|s| sanitize_html(s));

        let promotion_id = sqlx::query_scalar!(
            "INSERT INTO listing_promotions (
                vendor_id, listing_id, promo_type, discount_type, 
                discount_percentage, discount_fixed_amount,
                benefit_description_en, benefit_description_ar,
                use_listing_cover_image, custom_banner_image_url,
                title_en, title_ar, description_en, description_ar, 
                badge_text_en, badge_text_ar, start_at, end_at, status
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, 'pending')
             RETURNING id",
            vendor_id,
            payload.listing_id,
            payload.promo_type,
            payload.discount_type,
            payload.discount_percentage,
            payload.discount_fixed_amount,
            payload.benefit_description_en,
            payload.benefit_description_ar,
            payload.use_listing_cover_image,
            payload.custom_banner_image_url,
            payload.title_en,
            payload.title_ar,
            sanitized_desc_en,
            sanitized_desc_ar,
            payload.badge_text_en,
            payload.badge_text_ar,
            payload.start_at,
            payload.end_at
        )
        .fetch_one(conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(promotion_id)
    }

    pub async fn update_promotion(
        conn: &mut sqlx::PgConnection,
        promo_id: Uuid,
        payload: &CreatePromotionPayload,
        new_status: &str,
    ) -> Result<(), AppError> {
        let sanitized_desc_en = payload.description_en.as_ref().map(|s| sanitize_html(s));
        let sanitized_desc_ar = payload.description_ar.as_ref().map(|s| sanitize_html(s));

        sqlx::query!(
            "UPDATE listing_promotions
             SET listing_id = $1, promo_type = $2, discount_type = $3, 
                 discount_percentage = $4, discount_fixed_amount = $5,
                 benefit_description_en = $6, benefit_description_ar = $7,
                 use_listing_cover_image = $8, custom_banner_image_url = $9,
                 title_en = $10, title_ar = $11, description_en = $12, description_ar = $13,
                 badge_text_en = $14, badge_text_ar = $15, start_at = $16, end_at = $17, 
                 status = $18, updated_at = NOW()
             WHERE id = $19",
            payload.listing_id,
            payload.promo_type,
            payload.discount_type,
            payload.discount_percentage,
            payload.discount_fixed_amount,
            payload.benefit_description_en,
            payload.benefit_description_ar,
            payload.use_listing_cover_image,
            payload.custom_banner_image_url,
            payload.title_en,
            payload.title_ar,
            sanitized_desc_en,
            sanitized_desc_ar,
            payload.badge_text_en,
            payload.badge_text_ar,
            payload.start_at,
            payload.end_at,
            new_status,
            promo_id
        )
        .execute(conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }



    pub async fn update_status(
        conn: &mut sqlx::PgConnection,
        promo_id: Uuid,
        status: &str,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE listing_promotions SET status = $2, updated_at = NOW() WHERE id = $1",
            promo_id,
            status
        )
        .execute(conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn create_audit_log(
        conn: &mut sqlx::PgConnection,
        promo_id: Uuid,
        actor_user_id: Uuid,
        action: &str,
        prev_status: Option<&str>,
        new_status: Option<&str>,
        payload: Option<Value>,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO promotion_audit_logs (promotion_id, actor_user_id, action, previous_status, new_status, payload)
             VALUES ($1, $2, $3, $4, $5, $6)",
            promo_id,
            actor_user_id,
            action,
            prev_status,
            new_status,
            payload
        )
        .execute(conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }
}

// --- 3. Service Layer ---

pub struct PromotionService;

impl PromotionService {
    pub async fn create_promotion(
        conn: &mut sqlx::PgConnection,
        user_uuid: Uuid,
        payload: CreatePromotionPayload,
    ) -> Result<Uuid, AppError> {
        PromotionValidator::validate_create(&payload)?;

        let vendor_id = PromotionRepository::find_vendor_by_user_id(conn, user_uuid).await?;

        // Validate vendor account status and subscription status are active
        let vendor = sqlx::query!(
            "SELECT status, subscription_status FROM vendors WHERE id = $1",
            vendor_id
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        if vendor.status != "active" {
            return Err(AppError::Forbidden("Vendor account is suspended or banned".into()));
        }
        if vendor.subscription_status != "active" {
            return Err(AppError::BadRequest("Vendor subscription is not active".into()));
        }

        // Policy check: enforce max_promotions per subscription tier
        crate::utils::policy::PolicyEngine::check_promotion_limit(
            vendor_id,
            None, // No exclude — this is a new promotion
            conn,
        )
        .await?;

        // 1. Verify listing ownership
        PromotionRepository::check_listing_ownership(conn, payload.listing_id, vendor_id).await?;

        // 2. Insert promotion record
        let promo_id = PromotionRepository::create_promotion(conn, vendor_id, &payload).await?;

        // 3. Write audit log
        PromotionRepository::create_audit_log(
            conn,
            promo_id,
            user_uuid,
            "create",
            None,
            Some("pending"),
            Some(json!({
                "new_values": {
                    "listing_id": payload.listing_id,
                    "promo_type": payload.promo_type,
                    "discount_type": payload.discount_type,
                    "discount_percentage": payload.discount_percentage,
                    "discount_fixed_amount": payload.discount_fixed_amount,
                    "benefit_description_en": payload.benefit_description_en,
                    "benefit_description_ar": payload.benefit_description_ar,
                    "use_listing_cover_image": payload.use_listing_cover_image,
                    "custom_banner_image_url": payload.custom_banner_image_url,
                    "title_en": payload.title_en,
                    "title_ar": payload.title_ar,
                    "start_at": payload.start_at,
                    "end_at": payload.end_at,
                    "status": "pending"
                }
            })),
        )
        .await?;

        Ok(promo_id)
    }

    pub async fn update_promotion(
        conn: &mut sqlx::PgConnection,
        user_uuid: Uuid,
        promo_id: Uuid,
        payload: CreatePromotionPayload,
    ) -> Result<(), AppError> {
        PromotionValidator::validate_update(&payload)?;

        let vendor_id = PromotionRepository::find_vendor_by_user_id(conn, user_uuid).await?;

        // Validate vendor account status and subscription status are active
        let vendor = sqlx::query!(
            "SELECT status, subscription_status FROM vendors WHERE id = $1",
            vendor_id
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        if vendor.status != "active" {
            return Err(AppError::Forbidden("Vendor account is suspended or banned".into()));
        }
        if vendor.subscription_status != "active" {
            return Err(AppError::BadRequest("Vendor subscription is not active".into()));
        }

        // Validate promotion details
        let existing = sqlx::query!(
            "SELECT id, vendor_id, start_at, end_at, status, title_en, title_ar,
                    description_en, description_ar, badge_text_en, badge_text_ar,
                    use_listing_cover_image, custom_banner_image_url
             FROM listing_promotions WHERE id = $1",
            promo_id
        )
        .fetch_optional(&mut *conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let existing = match existing {
            Some(p) => {
                if p.vendor_id != vendor_id {
                    return Err(AppError::Forbidden("Access Denied".into()));
                }
                p
            }
            None => return Err(AppError::NotFound("Promotion not found".into())),
        };

        // Enforce restriction: active approved promotions cannot be edited
        if existing.status == "approved" {
            let now = chrono::Utc::now();
            if existing.start_at <= now && now <= existing.end_at {
                return Err(AppError::BadRequest("Cannot edit an active approved promotion".into()));
            }
        }

        // Determine the new status
        let new_status = match existing.status.as_str() {
            "rejected" | "draft" => "pending",
            other => other,
        };

        // 1. Verify listing ownership
        PromotionRepository::check_listing_ownership(conn, payload.listing_id, vendor_id).await?;

        // 2. Perform updates
        PromotionRepository::update_promotion(conn, promo_id, &payload, new_status).await?;

        // 3. Log audit log
        let audit_payload = json!({
            "old_values": {
                "title_en": existing.title_en,
                "title_ar": existing.title_ar,
                "description_en": existing.description_en,
                "description_ar": existing.description_ar,
                "badge_text_en": existing.badge_text_en,
                "badge_text_ar": existing.badge_text_ar,
                "use_listing_cover_image": existing.use_listing_cover_image,
                "custom_banner_image_url": existing.custom_banner_image_url,
                "start_at": existing.start_at,
                "end_at": existing.end_at,
                "status": existing.status,
            },
            "new_values": {
                "listing_id": payload.listing_id,
                "promo_type": payload.promo_type,
                "discount_type": payload.discount_type,
                "discount_percentage": payload.discount_percentage,
                "discount_fixed_amount": payload.discount_fixed_amount,
                "benefit_description_en": payload.benefit_description_en,
                "benefit_description_ar": payload.benefit_description_ar,
                "use_listing_cover_image": payload.use_listing_cover_image,
                "custom_banner_image_url": payload.custom_banner_image_url,
                "title_en": payload.title_en,
                "title_ar": payload.title_ar,
                "start_at": payload.start_at,
                "end_at": payload.end_at,
                "status": "pending",
            }
        });

        PromotionRepository::create_audit_log(
            conn,
            promo_id,
            user_uuid,
            "update",
            Some(&existing.status),
            Some("pending"),
            Some(audit_payload),
        )
        .await?;

        Ok(())
    }
}

// --- Handler Implementations ---

/// Fetch a single promotion by ID (vendor must own it)
pub async fn get_promotion_handler(
    auth: RequireVendor,
    mut rls_tx: RlsTx,
    Path(promo_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid vendor user ID".to_string()))?;

    let vendor_id = PromotionRepository::find_vendor_by_user_id(&mut *rls_tx.tx, user_uuid).await?;

    let row = sqlx::query!(
        r#"
        SELECT id, listing_id, promo_type, discount_type, discount_percentage,
               discount_fixed_amount, benefit_description_en, benefit_description_ar,
               use_listing_cover_image, custom_banner_image_url, title_en, title_ar,
               description_en, description_ar, badge_text_en, badge_text_ar,
               start_at, end_at, status as config_status, rejection_reason, created_at
        FROM listing_promotions
        WHERE id = $1 AND vendor_id = $2
        "#,
        promo_id,
        vendor_id
    )
    .fetch_optional(&mut *rls_tx.tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let row = match row {
        Some(r) => r,
        None => return Err(AppError::NotFound("Promotion not found".into())),
    };

    let now = Utc::now();
    let derived_status = match row.config_status.as_str() {
        "pending"   => "Pending",
        "rejected"  => "Rejected",
        "paused"    => "Paused",
        "cancelled" => "Cancelled",
        "approved" if now < row.start_at => "Scheduled",
        "approved" if now >= row.start_at && now <= row.end_at => "Active",
        _ => "Expired",
    };

    rls_tx
        .tx
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(json!({
        "status": "success",
        "promotion": {
            "id": row.id,
            "listing_id": row.listing_id,
            "promo_type": row.promo_type,
            "discount_type": row.discount_type,
            "discount_percentage": row.discount_percentage,
            "discount_fixed_amount": row.discount_fixed_amount,
            "benefit_description_en": row.benefit_description_en,
            "benefit_description_ar": row.benefit_description_ar,
            "use_listing_cover_image": row.use_listing_cover_image,
            "custom_banner_image_url": row.custom_banner_image_url,
            "title_en": row.title_en,
            "title_ar": row.title_ar,
            "description_en": row.description_en,
            "description_ar": row.description_ar,
            "badge_text_en": row.badge_text_en,
            "badge_text_ar": row.badge_text_ar,
            "start_at": row.start_at,
            "end_at": row.end_at,
            "config_status": row.config_status,
            "derived_status": derived_status,
            "rejection_reason": row.rejection_reason,
            "created_at": row.created_at
        }
    })))
}

/// Create a new Promotion + Targets + Analytics placeholder
pub async fn create_promotion_handler(
    auth: RequireVendor,
    mut rls_tx: RlsTx,
    Json(payload): Json<CreatePromotionPayload>,
) -> Result<Json<Value>, AppError> {
    let user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid vendor user ID".to_string()))?;

    let promo_id = PromotionService::create_promotion(&mut *rls_tx.tx, user_uuid, payload).await?;

    rls_tx
        .tx
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(
        json!({ "status": "success", "promotion_id": promo_id }),
    ))
}

/// List promotions belonging to the vendor, including analytics metrics
pub async fn list_vendor_promotions_handler(
    auth: RequireVendor,
    mut rls_tx: RlsTx,
    axum::extract::Query(params): axum::extract::Query<VendorPromotionsQuery>,
) -> Result<Json<Value>, AppError> {
    let user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid vendor user ID".to_string()))?;

    let vendor_id = PromotionRepository::find_vendor_by_user_id(&mut *rls_tx.tx, user_uuid).await?;

    let limit = params.limit.unwrap_or(10).clamp(1, 100);
    let page = params.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;

    let sort_by = params.sort_by.unwrap_or_else(|| "created_at".to_string());
    let sort_order = params.sort_order.unwrap_or_else(|| "desc".to_string());
    let search_term = params.search.map(|s| format!("%{}%", s));

    // --- Total count query (for pagination) ---
    let total_count = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::bigint as "count!"
        FROM listing_promotions lp
        WHERE lp.vendor_id = $1
          AND ($2::text IS NULL OR lp.status = $2::text)
          AND ($3::text IS NULL OR lp.title_en ILIKE $3::text OR lp.title_ar ILIKE $3::text)
          AND ($4::timestamptz IS NULL OR lp.start_at >= $4::timestamptz)
          AND ($5::timestamptz IS NULL OR lp.end_at <= $5::timestamptz)
        "#,
        vendor_id,
        params.status,
        search_term.clone(),
        params.start_date,
        params.end_date
    )
    .fetch_one(&mut *rls_tx.tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // --- Main query ---
    let rows = sqlx::query!(
        r#"
        SELECT 
            lp.id,
            lp.listing_id,
            lp.promo_type,
            lp.discount_type,
            lp.discount_percentage,
            lp.discount_fixed_amount,
            lp.benefit_description_en,
            lp.benefit_description_ar,
            lp.use_listing_cover_image,
            lp.custom_banner_image_url,
            lp.title_en,
            lp.title_ar,
            lp.description_en,
            lp.description_ar,
            lp.badge_text_en,
            lp.badge_text_ar,
            lp.start_at,
            lp.end_at,
            lp.status as config_status,
            lp.rejection_reason,
            lp.created_at,
            lp.views_count,
            lp.clicks_count
        FROM listing_promotions lp
        WHERE lp.vendor_id = $1
          AND ($2::text IS NULL OR lp.status = $2::text)
          AND ($3::text IS NULL OR lp.title_en ILIKE $3::text OR lp.title_ar ILIKE $3::text)
          AND ($4::timestamptz IS NULL OR lp.start_at >= $4::timestamptz)
          AND ($5::timestamptz IS NULL OR lp.end_at <= $5::timestamptz)
        ORDER BY 
            CASE WHEN $6 = 'created_at' AND $7 = 'desc' THEN lp.created_at END DESC,
            CASE WHEN $6 = 'created_at' AND $7 = 'asc' THEN lp.created_at END ASC,
            CASE WHEN $6 = 'start_at' AND $7 = 'desc' THEN lp.start_at END DESC,
            CASE WHEN $6 = 'start_at' AND $7 = 'asc' THEN lp.start_at END ASC,
            CASE WHEN $6 = 'end_at' AND $7 = 'desc' THEN lp.end_at END DESC,
            CASE WHEN $6 = 'end_at' AND $7 = 'asc' THEN lp.end_at END ASC,
            lp.created_at DESC
        LIMIT $8 OFFSET $9
        "#,
        vendor_id,
        params.status,
        search_term,
        params.start_date,
        params.end_date,
        sort_by,
        sort_order,
        limit,
        offset
    )
    .fetch_all(&mut *rls_tx.tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let mut list = Vec::new();
    for row in rows {
        let views = row.views_count;
        let clicks = row.clicks_count;
        let ctr = if views > 0 {
            clicks as f64 / views as f64
        } else {
            0.0
        };
        let now = Utc::now();
        let is_active = row.config_status == "approved" && row.start_at <= now && row.end_at >= now;

        let derived_status = match row.config_status.as_str() {
            "pending" => "Pending",
            "rejected" => "Rejected",
            "paused" => "Paused",
            "cancelled" => "Cancelled",
            "approved" if now < row.start_at => "Scheduled",
            "approved" if now >= row.start_at && now <= row.end_at => "Active",
            _ => "Expired",
        };

        list.push(json!({
            "id": row.id,
            "listing_id": row.listing_id,
            "promo_type": row.promo_type,
            "discount_type": row.discount_type,
            "discount_percentage": row.discount_percentage,
            "discount_fixed_amount": row.discount_fixed_amount,
            "benefit_description_en": row.benefit_description_en,
            "benefit_description_ar": row.benefit_description_ar,
            "use_listing_cover_image": row.use_listing_cover_image,
            "custom_banner_image_url": row.custom_banner_image_url,
            "title_en": row.title_en,
            "title_ar": row.title_ar,
            "description_en": row.description_en,
            "description_ar": row.description_ar,
            "badge_text_en": row.badge_text_en,
            "badge_text_ar": row.badge_text_ar,
            "start_at": row.start_at,
            "end_at": row.end_at,
            "config_status": row.config_status,
            "derived_status": derived_status,
            "rejection_reason": row.rejection_reason,
            "created_at": row.created_at,
            "derived_analytics": {
                "views": views,
                "clicks": clicks,
                "ctr": ctr,
                "active_status": is_active
            }
        }));
    }

    rls_tx
        .tx
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(json!({ "status": "success", "promotions": list, "total": total_count })))
}

/// Update an existing promotion
pub async fn update_promotion_handler(
    auth: RequireVendor,
    mut rls_tx: RlsTx,
    Path(promo_id): Path<Uuid>,
    Json(payload): Json<CreatePromotionPayload>,
) -> Result<Json<Value>, AppError> {
    let user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid vendor user ID".to_string()))?;

    PromotionService::update_promotion(&mut *rls_tx.tx, user_uuid, promo_id, payload).await?;

    rls_tx
        .tx
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(json!({
        "status": "success",
        "message": "Promotion updated and sent back for admin approval"
    })))
}

/// Set status to cancelled
pub async fn delete_promotion_handler(
    auth: RequireVendor,
    mut rls_tx: RlsTx,
    Path(promo_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid vendor user ID".to_string()))?;

    let vendor_id = PromotionRepository::find_vendor_by_user_id(&mut *rls_tx.tx, user_uuid).await?;

    let check = sqlx::query!(
        "SELECT vendor_id, status FROM listing_promotions WHERE id = $1",
        promo_id
    )
    .fetch_optional(&mut *rls_tx.tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let check = match check {
        Some(p) => {
            if p.vendor_id != vendor_id {
                return Err(AppError::Forbidden("Access Denied".into()));
            }
            p
        }
        None => return Err(AppError::NotFound("Promotion not found".into())),
    };

    PromotionRepository::update_status(&mut *rls_tx.tx, promo_id, "cancelled").await?;

    let audit_payload = json!({
        "old_values": { "status": check.status },
        "new_values": { "status": "cancelled" }
    });

    PromotionRepository::create_audit_log(
        &mut *rls_tx.tx,
        promo_id,
        user_uuid,
        "cancel",
        Some(&check.status),
        Some("cancelled"),
        Some(audit_payload),
    )
    .await?;

    rls_tx
        .tx
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(
        json!({ "status": "success", "message": "Promotion cancelled" }),
    ))
}

/// Duplicate an existing promotion (clones it into a Draft state)
pub async fn duplicate_promotion_handler(
    auth: RequireVendor,
    mut rls_tx: RlsTx,
    Path(promo_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid vendor user ID".to_string()))?;

    let vendor_id = PromotionRepository::find_vendor_by_user_id(&mut *rls_tx.tx, user_uuid).await?;

    let existing = sqlx::query!(
        "SELECT vendor_id, start_at, end_at, status, listing_id, promo_type, discount_type,
                discount_percentage, discount_fixed_amount, benefit_description_en,
                benefit_description_ar, use_listing_cover_image, custom_banner_image_url,
                title_en, title_ar, description_en, description_ar, badge_text_en,
                badge_text_ar
         FROM listing_promotions WHERE id = $1",
        promo_id
    )
    .fetch_optional(&mut *rls_tx.tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let promo = match existing {
        Some(p) => {
            if p.vendor_id != vendor_id {
                return Err(AppError::Forbidden("Access Denied".into()));
            }
            p
        }
        None => return Err(AppError::NotFound("Promotion not found".into())),
    };

    // Business Rule: Duplicate only Expired, Rejected or Cancelled promotions
    let now = Utc::now();
    let is_expired = promo.status == "approved" && promo.end_at < now;
    let is_rejected = promo.status == "rejected";
    let is_cancelled = promo.status == "cancelled";

    if !(is_expired || is_rejected || is_cancelled) {
        return Err(AppError::BadRequest(
            "Only expired, rejected, or cancelled promotions can be duplicated".into(),
        ));
    }

    // Insert copy payload
    let payload = CreatePromotionPayload {
        listing_id: promo.listing_id,
        promo_type: promo.promo_type,
        discount_type: promo.discount_type,
        discount_percentage: Some(promo.discount_percentage),
        discount_fixed_amount: promo.discount_fixed_amount,
        benefit_description_en: promo.benefit_description_en,
        benefit_description_ar: promo.benefit_description_ar,
        use_listing_cover_image: promo.use_listing_cover_image,
        custom_banner_image_url: promo.custom_banner_image_url,
        title_en: format!("Copy of {}", promo.title_en),
        title_ar: format!("نسخة من {}", promo.title_ar),
        description_en: promo.description_en,
        description_ar: promo.description_ar,
        badge_text_en: promo.badge_text_en,
        badge_text_ar: promo.badge_text_ar,
        start_at: promo.start_at,
        end_at: promo.end_at,
    };

    let new_id = PromotionRepository::create_promotion(&mut *rls_tx.tx, vendor_id, &payload).await?;

    // Explicitly update status to 'draft' as duplicates start in draft state
    sqlx::query!(
        "UPDATE listing_promotions SET status = 'draft' WHERE id = $1",
        new_id
    )
    .execute(&mut *rls_tx.tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // Log copy audit trail
    PromotionRepository::create_audit_log(
        &mut *rls_tx.tx,
        new_id,
        user_uuid,
        "create",
        None,
        Some("draft"),
        Some(json!({
            "copied_from_promotion_id": promo_id
        })),
    )
    .await?;

    rls_tx
        .tx
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(json!({ "status": "success", "promotion_id": new_id })))
}

#[derive(Debug, Deserialize)]
pub struct RenewPromotionPayload {
    pub days: i64,
}

/// Renew an expired promotion (extends end_at and submits to pending status)
pub async fn renew_promotion_handler(
    auth: RequireVendor,
    mut rls_tx: RlsTx,
    Path(promo_id): Path<Uuid>,
    Json(payload): Json<RenewPromotionPayload>,
) -> Result<Json<Value>, AppError> {
    let user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid vendor user ID".to_string()))?;

    let vendor_id = PromotionRepository::find_vendor_by_user_id(&mut *rls_tx.tx, user_uuid).await?;

    let existing = sqlx::query!(
        "SELECT vendor_id, status, end_at FROM listing_promotions WHERE id = $1",
        promo_id
    )
    .fetch_optional(&mut *rls_tx.tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let promo = match existing {
        Some(p) => {
            if p.vendor_id != vendor_id {
                return Err(AppError::Forbidden("Access Denied".into()));
            }
            p
        }
        None => return Err(AppError::NotFound("Promotion not found".into())),
    };

    // Business Rule: Renew only Expired promotions
    let now = Utc::now();
    let is_expired = promo.status == "approved" && promo.end_at < now;
    if !is_expired {
        return Err(AppError::BadRequest("Only expired promotions can be renewed".into()));
    }

    let start_at = now;
    let end_at = now + chrono::Duration::days(payload.days);

    sqlx::query!(
        "UPDATE listing_promotions
         SET start_at = $1, end_at = $2, status = 'pending', updated_at = NOW()
         WHERE id = $3",
        start_at,
        end_at,
        promo_id
    )
    .execute(&mut *rls_tx.tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    PromotionRepository::create_audit_log(
        &mut *rls_tx.tx,
        promo_id,
        user_uuid,
        "update",
        Some(&promo.status),
        Some("pending"),
        Some(json!({
            "renewed": true,
            "days_extended": payload.days,
            "new_start_at": start_at,
            "new_end_at": end_at
        })),
    )
    .await?;

    rls_tx
        .tx
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(json!({ "status": "success", "message": "Promotion renewed and sent back for review" })))
}

/// Pause a promotion
pub async fn pause_promotion_handler(
    auth: RequireVendor,
    mut rls_tx: RlsTx,
    Path(promo_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid vendor user ID".to_string()))?;

    let vendor_id = PromotionRepository::find_vendor_by_user_id(&mut *rls_tx.tx, user_uuid).await?;

    let check = sqlx::query!(
        "SELECT vendor_id, start_at, end_at, status FROM listing_promotions WHERE id = $1",
        promo_id
    )
    .fetch_optional(&mut *rls_tx.tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let check = match check {
        Some(p) => {
            if p.vendor_id != vendor_id {
                return Err(AppError::Forbidden("Access Denied".into()));
            }
            p
        }
        None => return Err(AppError::NotFound("Promotion not found".into())),
    };

    // Business Rule: Pause only Active promotions
    let now = Utc::now();
    let is_active = check.status == "approved" && check.start_at <= now && check.end_at >= now;

    if !is_active {
        return Err(AppError::BadRequest(
            "Only currently active promotions can be paused".into(),
        ));
    }

    PromotionRepository::update_status(&mut *rls_tx.tx, promo_id, "paused").await?;

    let audit_payload = json!({
        "old_values": { "status": check.status },
        "new_values": { "status": "paused" }
    });

    PromotionRepository::create_audit_log(
        &mut *rls_tx.tx,
        promo_id,
        user_uuid,
        "pause",
        Some(&check.status),
        Some("paused"),
        Some(audit_payload),
    )
    .await?;

    rls_tx
        .tx
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(
        json!({ "status": "success", "message": "Promotion paused" }),
    ))
}

/// Resume a promotion
pub async fn resume_promotion_handler(
    auth: RequireVendor,
    mut rls_tx: RlsTx,
    Path(promo_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid vendor user ID".to_string()))?;

    let vendor_id = PromotionRepository::find_vendor_by_user_id(&mut *rls_tx.tx, user_uuid).await?;

    // 1. Fetch promotion details
    let promo = sqlx::query!(
        "SELECT vendor_id, start_at, end_at, status, listing_id FROM listing_promotions WHERE id = $1",
        promo_id
    )
    .fetch_optional(&mut *rls_tx.tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let promo = match promo {
        Some(p) => {
            if p.vendor_id != vendor_id {
                return Err(AppError::Forbidden("Access Denied".into()));
            }
            p
        }
        None => return Err(AppError::NotFound("Promotion not found".into())),
    };

    // 2. Validate campaign not expired
    if promo.end_at <= Utc::now() {
        return Err(AppError::BadRequest(
            "Cannot resume an expired promotion".into(),
        ));
    }

    // 3. Validate vendor account and subscription active
    let vendor = sqlx::query!(
        "SELECT status, subscription_status FROM vendors WHERE id = $1",
        vendor_id
    )
    .fetch_one(&mut *rls_tx.tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    if vendor.status != "active" {
        return Err(AppError::Forbidden("Vendor account is suspended or banned".into()));
    }
    if vendor.subscription_status != "active" {
        return Err(AppError::BadRequest(
            "Vendor subscription is not active".into(),
        ));
    }

    // 4. Validate targeted listing is still active/published (status = 'active')
    let listing = sqlx::query!(
        "SELECT status FROM vendor_products WHERE id = $1",
        promo.listing_id
    )
    .fetch_optional(&mut *rls_tx.tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let listing = match listing {
        Some(l) => l,
        None => return Err(AppError::BadRequest("Targeted listing not found".into())),
    };

    if listing.status != "active" {
        return Err(AppError::BadRequest(format!(
            "Listing {} is not currently active/published",
            promo.listing_id
        )));
    }

    // 5. Validate no overlap conflicts
    let overlap_exists = sqlx::query_scalar!(
        "SELECT COUNT(*)::int 
         FROM listing_promotions lp
         WHERE lp.listing_id = $1
           AND lp.status = 'approved'
           AND lp.id != $2
           AND tstzrange(lp.start_at, lp.end_at, '[]') && tstzrange($3, $4, '[]')",
        promo.listing_id,
        promo_id,
        promo.start_at,
        promo.end_at
    )
    .fetch_one(&mut *rls_tx.tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    if overlap_exists.unwrap_or(0) > 0 {
        return Err(AppError::BadRequest(
            "Overlap conflict: The target listing already has an approved promotion during this range"
                .into(),
        ));
    }

    // 6. Resume promotion (returns it back to approved status)
    PromotionRepository::update_status(&mut *rls_tx.tx, promo_id, "approved").await?;

    let audit_payload = json!({
        "old_values": { "status": promo.status },
        "new_values": { "status": "approved" }
    });

    PromotionRepository::create_audit_log(
        &mut *rls_tx.tx,
        promo_id,
        user_uuid,
        "resume",
        Some(&promo.status),
        Some("approved"),
        Some(audit_payload),
    )
    .await?;

    rls_tx
        .tx
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(
        json!({ "status": "success", "message": "Promotion resumed" }),
    ))
}

// --- Public / Client Handle/// Fetch active promoted listings sorted by vendor tier subscription priority score.
pub async fn list_public_promotions_handler(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<PublicPromotionsQuery>,
) -> Result<Json<Value>, AppError> {
    let limit = params.limit.unwrap_or(10).clamp(1, 100);
    let page = params.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;

    let sort_by = params.sort_by.unwrap_or_else(|| "priority".to_string());
    let sort_order = params.sort_order.unwrap_or_else(|| "desc".to_string());

    let search_term = params.search.map(|s| format!("%{}%", s));

    let rows = sqlx::query!(
        r#"
        SELECT 
            lp.id as promotion_id,
            lp.promo_type,
            lp.discount_type,
            lp.discount_percentage,
            lp.discount_fixed_amount,
            lp.benefit_description_en,
            lp.benefit_description_ar,
            lp.use_listing_cover_image,
            lp.custom_banner_image_url,
            lp.title_en,
            lp.title_ar,
            lp.description_en,
            lp.description_ar,
            lp.badge_text_en,
            lp.badge_text_ar,
            lp.banner_image_url,
            lp.start_at,
            lp.end_at,
            lp.status,
            p.id as product_id,
            p.title_en as listing_name_en,
            p.title_ar as listing_name_ar,
            p.slug as listing_slug,
            (SELECT image_url FROM vendor_gallery WHERE product_id = p.id AND is_cover = TRUE LIMIT 1) AS cover_image,
            p.city_id,
            p.product_category as category,
            v.name_en as vendor_name,
            st.name as subscription_tier,
            st.priority_score
        FROM listing_promotions lp
        JOIN vendors v ON lp.vendor_id = v.id
        JOIN subscription_tiers st ON v.subscription_tier_id = st.id
        JOIN vendor_products p ON lp.listing_id = p.id
        WHERE fn_promo_is_active(lp.status, lp.start_at, lp.end_at)
          AND v.status = 'active'
          AND v.subscription_status = 'active'
          AND p.status = 'active'
          AND ($1::text IS NULL OR p.product_category = $1::text)
          AND ($2::uuid IS NULL OR p.city_id = $2::uuid)
          AND ($3::uuid IS NULL OR v.id = $3::uuid)
          AND ($4::text IS NULL OR lp.title_en ILIKE $4::text OR lp.title_ar ILIKE $4::text OR lp.description_en ILIKE $4::text OR lp.description_ar ILIKE $4::text)
          AND ($5::timestamptz IS NULL OR lp.start_at >= $5::timestamptz)
          AND ($6::timestamptz IS NULL OR lp.end_at <= $6::timestamptz)
        ORDER BY 
            CASE WHEN $7 = 'priority' AND $8 = 'desc' THEN st.priority_score END DESC,
            CASE WHEN $7 = 'priority' AND $8 = 'asc' THEN st.priority_score END ASC,
            CASE WHEN $7 = 'end_at' AND $8 = 'desc' THEN lp.end_at END DESC,
            CASE WHEN $7 = 'end_at' AND $8 = 'asc' THEN lp.end_at END ASC,
            CASE WHEN $7 = 'created_at' AND $8 = 'desc' THEN lp.created_at END DESC,
            CASE WHEN $7 = 'created_at' AND $8 = 'asc' THEN lp.created_at END ASC,
            st.priority_score DESC,
            lp.end_at ASC,
            lp.created_at DESC
        LIMIT $9 OFFSET $10
        "#,
        params.category,
        params.city_id,
        params.vendor_id,
        search_term,
        params.start_date,
        params.end_date,
        sort_by,
        sort_order,
        limit,
        offset
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;
 
    let list: Vec<Value> = rows
        .into_iter()
        .map(|row| {
            json!({
                "promotion_id": row.promotion_id,
                "promo_type": row.promo_type,
                "discount_type": row.discount_type,
                "discount_percentage": row.discount_percentage,
                "discount_fixed_amount": row.discount_fixed_amount,
                "benefit_description_en": row.benefit_description_en,
                "benefit_description_ar": row.benefit_description_ar,
                "use_listing_cover_image": row.use_listing_cover_image,
                "custom_banner_image_url": row.custom_banner_image_url,
                "title_en": row.title_en,
                "title_ar": row.title_ar,
                "description_en": row.description_en,
                "description_ar": row.description_ar,
                "badge_text_en": row.badge_text_en,
                "badge_text_ar": row.badge_text_ar,
                "banner_image_url": row.banner_image_url,
                "end_at": row.end_at,
                "product_id": row.product_id,
                "listing_name_en": row.listing_name_en,
                "listing_name_ar": row.listing_name_ar,
                "listing_slug": row.listing_slug,
                "cover_image": row.cover_image,
                "city_id": row.city_id,
                "category": row.category,
                "vendor_name": row.vendor_name,
                "subscription_tier": row.subscription_tier,
                "priority_score": row.priority_score
            })
        })
        .collect();

    Ok(Json(json!({ "status": "success", "promotions": list })))
}

/// Track Views and Clicks with duplicate tracking protection (sliding rate limiter)
pub async fn track_promotion_handler(
    State(state): State<AppState>,
    Path(promo_id): Path<Uuid>,
    ip: SecureClientIp,
    Json(payload): Json<TrackPromotionPayload>,
) -> Result<Json<Value>, AppError> {
    let ip_str = ip.0.to_string();
    let cache_key = format!("{}:{}:{}", ip_str, promo_id, payload.event_type);

    // Duplicate detection check
    if let Some(timestamp) = TRACK_DEDUPLICATION_CACHE.get(&cache_key) {
        if timestamp.elapsed() < std::time::Duration::from_secs(300) {
            // Deduplicated, fail-silent return
            return Ok(Json(json!({ "status": "success", "ignored": true })));
        }
    }

    // Update tracking cache timestamp
    TRACK_DEDUPLICATION_CACHE.insert(cache_key, Instant::now());

    if payload.event_type == "view" {
        sqlx::query!(
            "UPDATE listing_promotions SET views_count = views_count + 1 WHERE id = $1",
            promo_id
        )
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    } else if payload.event_type == "click" {
        sqlx::query!(
            "UPDATE listing_promotions SET clicks_count = clicks_count + 1 WHERE id = $1",
            promo_id
        )
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    } else {
        return Err(AppError::BadRequest("Invalid event type".to_string()));
    }

    Ok(Json(json!({ "status": "success" })))
}

// --- 4. Admin Moderation Handlers ---

pub async fn list_admin_promotions_handler(
    _auth: RequireAdmin,
    mut rls_tx: RlsTx,
    axum::extract::Query(params): axum::extract::Query<AdminPromotionsQuery>,
) -> Result<Json<Value>, AppError> {
    let limit = params.limit.unwrap_or(10).clamp(1, 100);
    let page = params.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;

    let sort_by = params.sort_by.unwrap_or_else(|| "created_at".to_string());
    let sort_order = params.sort_order.unwrap_or_else(|| "desc".to_string());
    let search_term = params.search.map(|s| format!("%{}%", s));
    let rows = sqlx::query!(
        r#"
        SELECT 
            lp.id,
            lp.vendor_id,
            v.name_en as vendor_name,
            lp.listing_id,
            p.title_en as listing_title_en,
            p.title_ar as listing_title_ar,
            lp.promo_type,
            lp.discount_type,
            lp.discount_percentage,
            lp.discount_fixed_amount,
            lp.benefit_description_en,
            lp.benefit_description_ar,
            lp.use_listing_cover_image,
            lp.custom_banner_image_url,
            lp.title_en,
            lp.title_ar,
            lp.description_en,
            lp.description_ar,
            lp.badge_text_en,
            lp.badge_text_ar,
            lp.start_at,
            lp.end_at,
            lp.status,
            lp.created_at,
            lp.updated_at
        FROM listing_promotions lp
        JOIN vendors v ON lp.vendor_id = v.id
        JOIN vendor_products p ON lp.listing_id = p.id
        WHERE ($1::text IS NULL OR lp.status = $1::text)
          AND ($2::uuid IS NULL OR lp.vendor_id = $2::uuid)
          AND ($3::text IS NULL OR lp.title_en ILIKE $3::text OR lp.title_ar ILIKE $3::text)
        ORDER BY 
            CASE WHEN $4 = 'created_at' AND $5 = 'desc' THEN lp.created_at END DESC,
            CASE WHEN $4 = 'created_at' AND $5 = 'asc' THEN lp.created_at END ASC,
            CASE WHEN $4 = 'status' AND $5 = 'desc' THEN lp.status END DESC,
            CASE WHEN $4 = 'status' AND $5 = 'asc' THEN lp.status END ASC,
            lp.created_at DESC
        LIMIT $6 OFFSET $7
        "#,
        params.status,
        params.vendor_id,
        search_term,
        sort_by,
        sort_order,
        limit,
        offset
    )
    .fetch_all(&mut *rls_tx.tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let mut list = Vec::new();
    for row in rows {
        list.push(json!({
            "id": row.id,
            "vendor_id": row.vendor_id,
            "vendor_name": row.vendor_name,
            "listing_id": row.listing_id,
            "listing_title_en": row.listing_title_en,
            "listing_title_ar": row.listing_title_ar,
            "promo_type": row.promo_type,
            "discount_type": row.discount_type,
            "discount_percentage": row.discount_percentage,
            "discount_fixed_amount": row.discount_fixed_amount,
            "benefit_description_en": row.benefit_description_en,
            "benefit_description_ar": row.benefit_description_ar,
            "use_listing_cover_image": row.use_listing_cover_image,
            "custom_banner_image_url": row.custom_banner_image_url,
            "title_en": row.title_en,
            "title_ar": row.title_ar,
            "description_en": row.description_en,
            "description_ar": row.description_ar,
            "badge_text_en": row.badge_text_en,
            "badge_text_ar": row.badge_text_ar,
            "start_at": row.start_at,
            "end_at": row.end_at,
            "status": row.status,
            "created_at": row.created_at,
            "updated_at": row.updated_at
        }));
    }    rls_tx
        .tx
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(json!({ "status": "success", "promotions": list })))
}

pub async fn approve_promotion_handler(
    auth: RequireAdmin,
    mut rls_tx: RlsTx,
    Path(promo_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let actor_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid admin user ID".to_string()))?;

    let promo = sqlx::query!(
        "SELECT status FROM listing_promotions WHERE id = $1",
        promo_id
    )
    .fetch_optional(&mut *rls_tx.tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let promo = match promo {
        Some(p) => p,
        None => return Err(AppError::NotFound("Promotion not found".into())),
    };

    if promo.status != "pending" {
        return Err(AppError::BadRequest(
            "Only pending promotions can be approved".into(),
        ));
    }

    PromotionRepository::update_status(&mut *rls_tx.tx, promo_id, "approved").await?;

    let audit_payload = json!({
        "old_values": { "status": promo.status },
        "new_values": { "status": "approved" }
    });

    PromotionRepository::create_audit_log(
        &mut *rls_tx.tx,
        promo_id,
        actor_uuid,
        "approve",
        Some(&promo.status),
        Some("approved"),
        Some(audit_payload),
    )
    .await?;

    rls_tx
        .tx
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(json!({
        "status": "success",
        "message": "Promotion approved successfully"
    })))
}

pub async fn reject_promotion_handler(
    auth: RequireAdmin,
    mut rls_tx: RlsTx,
    Path(promo_id): Path<Uuid>,
    Json(payload): Json<RejectPromotionPayload>,
) -> Result<Json<Value>, AppError> {
    let actor_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid admin user ID".to_string()))?;

    if payload.rejection_reason.trim().is_empty() {
        return Err(AppError::BadRequest("Rejection reason is required".into()));
    }

    let promo = sqlx::query!(
        "SELECT status FROM listing_promotions WHERE id = $1",
        promo_id
    )
    .fetch_optional(&mut *rls_tx.tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let promo = match promo {
        Some(p) => p,
        None => return Err(AppError::NotFound("Promotion not found".into())),
    };

    if promo.status != "pending" {
        return Err(AppError::BadRequest(
            "Only pending promotions can be rejected".into(),
        ));
    }

    // Update status to rejected AND store the rejection reason on the record
    sqlx::query!(
        "UPDATE listing_promotions SET status = 'rejected', rejection_reason = $2, updated_at = NOW() WHERE id = $1",
        promo_id,
        payload.rejection_reason.trim()
    )
    .execute(&mut *rls_tx.tx)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let audit_payload = json!({
        "old_values": { "status": promo.status },
        "new_values": { "status": "rejected" },
        "rejection_reason": payload.rejection_reason
    });

    PromotionRepository::create_audit_log(
        &mut *rls_tx.tx,
        promo_id,
        actor_uuid,
        "reject",
        Some(&promo.status),
        Some("rejected"),
        Some(audit_payload),
    )
    .await?;

    rls_tx
        .tx
        .commit()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(json!({
        "status": "success",
        "message": "Promotion rejected successfully"
    })))
}

/// Upload a banner image for a promotion.
/// Uses the existing media service to process, resize, and save the image.
/// Returns the public URL that can be passed to the create/update promotion payload.
pub async fn upload_promotion_banner_handler(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    auth: RequireVendor,
    mut multipart: Multipart,
) -> Result<Json<Value>, AppError> {
    let _user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid vendor user ID".to_string()))?;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart error: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            let file_name = field.file_name().unwrap_or("banner.jpg").to_string();

            let target_dir = "assets/uploads/promotions/";
            let url_prefix = "/assets/uploads/promotions/";

            let processed = crate::services::media::process_and_save_upload(
                field,
                &file_name,
                target_dir,
                url_prefix,
                10 * 1024 * 1024, // 10 MB max for banner images
                1920,             // max dimension
                &state.minio_client,
            )
            .await
            .map_err(|e| {
                tracing::error!("Promotion banner upload failed: {:?}", e);
                e
            })?;

            return Ok(Json(json!({
                "status": "success",
                "url": processed.file_url,
                "file_path": processed.disk_path,
                "file_size": processed.file_size,
                "mime_type": processed.mime_type
            })));
        }
    }

    Err(AppError::BadRequest(
        "A valid image file attachment is required in the 'file' field.".to_string(),
    ))
}

/// Delete an orphaned promotion banner file from disk.
/// Called by the BFF when a promotion create/update fails AFTER a banner was already uploaded.
/// Only files under the promotions upload directory are allowed (security guard).
#[derive(Debug, serde::Deserialize)]
pub struct CleanupBannerPayload {
    pub file_url: String,
}

pub async fn cleanup_promotion_banner_handler(
    State(state): State<AppState>,
    auth: RequireVendor,
    Json(payload): Json<CleanupBannerPayload>,
) -> Result<Json<Value>, AppError> {
    let _user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid vendor user ID".to_string()))?;

    let url = payload.file_url.trim();

    // Security: only allow deletion within the promotions upload directory
    let root_prefix = &state.config.minio_root_prefix;
    let allowed_prefix = format!("/{}/promotions/", crate::utils::storage_paths::clean_prefix(root_prefix));
    if !url.starts_with(&allowed_prefix) {
        return Err(AppError::BadRequest(
            "Invalid file URL: only promotion banner files can be deleted.".to_string(),
        ));
    }

    // Strip the leading slash to get the relative disk path (relative to the binary CWD)
    let disk_path = &url[1..]; // e.g. "assets/uploads/promotions/banner-xyz.webp"

    match std::fs::remove_file(disk_path) {
        Ok(_) => {
            tracing::info!("Cleaned up orphaned promotion banner: {}", disk_path);
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // Already gone — not an error
            tracing::debug!("Banner cleanup: file not found (already deleted?): {}", disk_path);
        }
        Err(e) => {
            tracing::warn!("Banner cleanup failed for {}: {}", disk_path, e);
            // Don't surface this as an error to the caller — best-effort
        }
    }

    Ok(Json(json!({ "status": "success" })))
}

// --- Unit & Integration Tests ---

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[tokio::test]
    async fn test_db_promotions_queries() {
        let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://zafaf_db_admin:ramizwebdeveloperproductionsafe@127.0.0.1:5434/zafaf_world"
                .to_string()
        });
        let pool = PgPool::connect(&db_url).await.unwrap();
        let mut tx = pool.begin().await.unwrap();

        let user_id = sqlx::query_scalar!(
            "INSERT INTO global_users (email, password_hash, domain_type) VALUES ('test_promo_user_v1@zafafworld.net', 'hash', 'Vendor'::user_domain_enum) RETURNING id"
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();

        let tier_id = sqlx::query_scalar!(
            "SELECT id FROM subscription_tiers WHERE name = 'Diamond'"
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();

        let vendor_id = sqlx::query_scalar!(
            "INSERT INTO vendors (user_id, name_en, name_ar, slug, subscription_status, subscription_tier_id) VALUES ($1, 'Test Promo Vendor', 'بائع تجريبي', 'test-promo-vendor-v1', 'active', $2) RETURNING id",
            user_id,
            tier_id
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();

        let listing_id = sqlx::query_scalar!(
            "INSERT INTO vendor_products (vendor_id, title, title_en, title_ar, slug, product_category, base_price_sar, status) VALUES ($1, 'Listing Test', 'Listing Test', 'قائمة اختبار', 'test-db-l1-slug', 'wedding-palace', 5000.0, 'active') RETURNING id",
            vendor_id
        ).fetch_one(&mut *tx).await.unwrap();

        let promo_id = sqlx::query_scalar!(
            "INSERT INTO listing_promotions (vendor_id, listing_id, title_en, title_ar, discount_percentage, start_at, end_at, status)
             VALUES ($1, $2, 'Promo 1', 'عرض 1', 15, NOW() - INTERVAL '1 hour', NOW() + INTERVAL '1 day', 'approved') RETURNING id",
            vendor_id,
            listing_id
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();

        let count = sqlx::query_scalar!(
            "SELECT COUNT(*)::int FROM listing_promotions WHERE id = $1",
            promo_id
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();

        assert_eq!(count, Some(1));

        tx.rollback().await.unwrap();
    }

    #[tokio::test]
    async fn test_validation_rules() {
        let now = Utc::now();

        // Too low discount (< 5%)
        let payload = CreatePromotionPayload {
            listing_id: Uuid::new_v4(),
            promo_type: "discount".to_string(),
            discount_type: Some("percentage".to_string()),
            discount_percentage: Some(3),
            discount_fixed_amount: None,
            benefit_description_en: None,
            benefit_description_ar: None,
            use_listing_cover_image: true,
            custom_banner_image_url: None,
            title_en: "Low Discount".to_string(),
            title_ar: "خصم منخفض".to_string(),
            description_en: None,
            description_ar: None,
            badge_text_en: None,
            badge_text_ar: None,
            start_at: now,
            end_at: now + chrono::Duration::hours(5),
        };
        assert!(PromotionValidator::validate_create(&payload).is_err());

        // Too high discount (> 90%)
        let payload_high = CreatePromotionPayload {
            discount_percentage: Some(95),
            ..payload.clone()
        };
        assert!(PromotionValidator::validate_create(&payload_high).is_err());

        // End time before start time
        let payload_time = CreatePromotionPayload {
            discount_percentage: Some(20),
            end_at: now - chrono::Duration::hours(2),
            ..payload.clone()
        };
        assert!(PromotionValidator::validate_create(&payload_time).is_err());

        // Empty listing target (Uuid::nil())
        let payload_empty = CreatePromotionPayload {
            discount_percentage: Some(20),
            listing_id: Uuid::nil(),
            ..payload.clone()
        };
        assert!(PromotionValidator::validate_create(&payload_empty).is_err());
    }

    #[tokio::test]
    async fn test_overlap_and_ownership_triggers() {
        let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://zafaf_db_admin:ramizwebdeveloperproductionsafe@127.0.0.1:5434/zafaf_world"
                .to_string()
        });
        let pool = PgPool::connect(&db_url).await.unwrap();
        let mut tx = pool.begin().await.unwrap();

        let tier_id = sqlx::query_scalar!(
            "SELECT id FROM subscription_tiers WHERE name = 'Diamond'"
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();

        let user1_id = sqlx::query_scalar!(
            "INSERT INTO global_users (email, password_hash, domain_type) VALUES ('v1_owner@zafafworld.net', 'hash', 'Vendor'::user_domain_enum) RETURNING id"
        ).fetch_one(&mut *tx).await.unwrap();

        let vendor1_id = sqlx::query_scalar!(
            "INSERT INTO vendors (user_id, name_en, name_ar, slug, subscription_status, subscription_tier_id) VALUES ($1, 'V1', 'بائع 1', 'v1-slug', 'active', $2) RETURNING id",
            user1_id,
            tier_id
        ).fetch_one(&mut *tx).await.unwrap();

        let user2_id = sqlx::query_scalar!(
            "INSERT INTO global_users (email, password_hash, domain_type) VALUES ('v2_owner@zafafworld.net', 'hash', 'Vendor'::user_domain_enum) RETURNING id"
        ).fetch_one(&mut *tx).await.unwrap();

        let vendor2_id = sqlx::query_scalar!(
            "INSERT INTO vendors (user_id, name_en, name_ar, slug, subscription_status, subscription_tier_id) VALUES ($1, 'V2', 'بائع 2', 'v2-slug', 'active', $2) RETURNING id",
            user2_id,
            tier_id
        ).fetch_one(&mut *tx).await.unwrap();

        let listing1_id = sqlx::query_scalar!(
            "INSERT INTO vendor_products (vendor_id, title, title_en, title_ar, slug, product_category, base_price_sar, status) VALUES ($1, 'Listing 1', 'Listing 1', 'قائمة 1', 'l1-slug', 'wedding-palace', 5000.0, 'active') RETURNING id",
            vendor1_id
        ).fetch_one(&mut *tx).await.unwrap();

        let listing2_id = sqlx::query_scalar!(
            "INSERT INTO vendor_products (vendor_id, title, title_en, title_ar, slug, product_category, base_price_sar, status) VALUES ($1, 'Listing 2', 'Listing 2', 'قائمة 2', 'l2-slug', 'wedding-palace', 4000.0, 'active') RETURNING id",
            vendor2_id
        ).fetch_one(&mut *tx).await.unwrap();

        let payload1 = CreatePromotionPayload {
            listing_id: listing1_id,
            promo_type: "discount".to_string(),
            discount_type: Some("percentage".to_string()),
            discount_percentage: Some(20),
            discount_fixed_amount: None,
            benefit_description_en: None,
            benefit_description_ar: None,
            use_listing_cover_image: true,
            custom_banner_image_url: None,
            title_en: "Promo 1".to_string(),
            title_ar: "عرض 1".to_string(),
            description_en: None,
            description_ar: None,
            badge_text_en: None,
            badge_text_ar: None,
            start_at: Utc::now(),
            end_at: Utc::now() + chrono::Duration::hours(5),
        };

        let promo1_id = PromotionService::create_promotion(&mut *tx, user1_id, payload1)
            .await
            .unwrap();
        assert!(promo1_id != Uuid::nil());

        // Try creating promotion with listing belonging to Vendor 2 (Ownership Violation Trigger)
        let bad_payload = CreatePromotionPayload {
            listing_id: listing2_id,
            promo_type: "discount".to_string(),
            discount_type: Some("percentage".to_string()),
            discount_percentage: Some(20),
            discount_fixed_amount: None,
            benefit_description_en: None,
            benefit_description_ar: None,
            use_listing_cover_image: true,
            custom_banner_image_url: None,
            title_en: "Promo 1".to_string(),
            title_ar: "عرض 1".to_string(),
            description_en: None,
            description_ar: None,
            badge_text_en: None,
            badge_text_ar: None,
            start_at: Utc::now(),
            end_at: Utc::now() + chrono::Duration::hours(5),
        };

        let bad_promo_res = PromotionService::create_promotion(&mut *tx, user1_id, bad_payload).await;
        assert!(bad_promo_res.is_err());

        // Approve promo 1 first
        sqlx::query!(
            "UPDATE listing_promotions SET status = 'approved' WHERE id = $1",
            promo1_id
        )
        .execute(&mut *tx)
        .await
        .unwrap();

        let payload2 = CreatePromotionPayload {
            listing_id: listing1_id,
            promo_type: "discount".to_string(),
            discount_type: Some("percentage".to_string()),
            discount_percentage: Some(25),
            discount_fixed_amount: None,
            benefit_description_en: None,
            benefit_description_ar: None,
            use_listing_cover_image: true,
            custom_banner_image_url: None,
            title_en: "Promo 2".to_string(),
            title_ar: "عرض 2".to_string(),
            description_en: None,
            description_ar: None,
            badge_text_en: None,
            badge_text_ar: None,
            start_at: Utc::now() + chrono::Duration::hours(1), // Overlaps
            end_at: Utc::now() + chrono::Duration::hours(4),   // Overlaps
        };

        // Create promo 2 (targets listing 1 overlapping range)
        let promo2_res = PromotionService::create_promotion(&mut *tx, user1_id, payload2).await;
        // Must fail immediately due to database overlap trigger protection
        assert!(promo2_res.is_err());

        tx.rollback().await.unwrap();
    }

    #[tokio::test]
    async fn test_active_promotion_edit_restriction() {
        let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://zafaf_db_admin:ramizwebdeveloperproductionsafe@127.0.0.1:5434/zafaf_world"
                .to_string()
        });
        let pool = PgPool::connect(&db_url).await.unwrap();
        let mut tx = pool.begin().await.unwrap();

        let tier_id = sqlx::query_scalar!(
            "SELECT id FROM subscription_tiers WHERE name = 'Diamond'"
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();

        let user_id = sqlx::query_scalar!(
            "INSERT INTO global_users (email, password_hash, domain_type) VALUES ('active_edit@zafafworld.net', 'hash', 'Vendor'::user_domain_enum) RETURNING id"
        ).fetch_one(&mut *tx).await.unwrap();

        let vendor_id = sqlx::query_scalar!(
            "INSERT INTO vendors (user_id, name_en, name_ar, slug, subscription_status, subscription_tier_id) VALUES ($1, 'Active Edit Vendor', 'بائع تعديل', 'active-edit-vendor', 'active', $2) RETURNING id",
            user_id,
            tier_id
        ).fetch_one(&mut *tx).await.unwrap();

        let listing_id = sqlx::query_scalar!(
            "INSERT INTO vendor_products (vendor_id, title, title_en, title_ar, slug, product_category, base_price_sar, status) VALUES ($1, 'Product', 'Product', 'منتج', 'p-slug', 'wedding-palace', 5000.0, 'active') RETURNING id",
            vendor_id
        ).fetch_one(&mut *tx).await.unwrap();

        // 1. Create promotion
        let payload = CreatePromotionPayload {
            listing_id,
            promo_type: "discount".to_string(),
            discount_type: Some("percentage".to_string()),
            discount_percentage: Some(20),
            discount_fixed_amount: None,
            benefit_description_en: None,
            benefit_description_ar: None,
            use_listing_cover_image: true,
            custom_banner_image_url: None,
            title_en: "Promo 1".to_string(),
            title_ar: "عرض 1".to_string(),
            description_en: None,
            description_ar: None,
            badge_text_en: None,
            badge_text_ar: None,
            start_at: Utc::now() - chrono::Duration::hours(1), // Currently active
            end_at: Utc::now() + chrono::Duration::hours(5),   // Currently active
        };

        let promo_id = PromotionService::create_promotion(&mut *tx, user_id, payload.clone())
            .await
            .unwrap();

        // Approve it so it becomes Active
        sqlx::query!(
            "UPDATE listing_promotions SET status = 'approved' WHERE id = $1",
            promo_id
        )
        .execute(&mut *tx)
        .await
        .unwrap();

        // 2. Try to update it while active (Must return AppError)
        let update_res =
            PromotionService::update_promotion(&mut *tx, user_id, promo_id, payload).await;
        assert!(update_res.is_err());

        tx.rollback().await.unwrap();
    }

    #[tokio::test]
    async fn test_inactive_vendor_restrictions() {
        let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://zafaf_db_admin:ramizwebdeveloperproductionsafe@127.0.0.1:5434/zafaf_world"
                .to_string()
        });
        let pool = PgPool::connect(&db_url).await.unwrap();
        let mut tx = pool.begin().await.unwrap();

        // 1. Create suspended vendor
        let tier_id = sqlx::query_scalar!(
            "SELECT id FROM subscription_tiers WHERE name = 'Diamond'"
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();

        let user_id = sqlx::query_scalar!(
            "INSERT INTO global_users (email, password_hash, domain_type) VALUES ('suspended_vendor@zafafworld.net', 'hash', 'Vendor'::user_domain_enum) RETURNING id"
        ).fetch_one(&mut *tx).await.unwrap();

        let vendor_id = sqlx::query_scalar!(
            "INSERT INTO vendors (user_id, name_en, name_ar, slug, status, subscription_status, subscription_tier_id) VALUES ($1, 'Suspended Vendor', 'بائع معلق', 'suspended-vendor', 'suspended', 'active', $2) RETURNING id",
            user_id,
            tier_id
        ).fetch_one(&mut *tx).await.unwrap();

        let listing_id = sqlx::query_scalar!(
            "INSERT INTO vendor_products (vendor_id, title, title_en, title_ar, slug, product_category, base_price_sar, status) VALUES ($1, 'Product', 'Product', 'منتج', 'suspended-p-slug', 'wedding-palace', 5000.0, 'active') RETURNING id",
            vendor_id
        ).fetch_one(&mut *tx).await.unwrap();

        let payload = CreatePromotionPayload {
            listing_id,
            promo_type: "discount".to_string(),
            discount_type: Some("percentage".to_string()),
            discount_percentage: Some(20),
            discount_fixed_amount: None,
            benefit_description_en: None,
            benefit_description_ar: None,
            use_listing_cover_image: true,
            custom_banner_image_url: None,
            title_en: "Promo 1".to_string(),
            title_ar: "عرض 1".to_string(),
            description_en: None,
            description_ar: None,
            badge_text_en: None,
            badge_text_ar: None,
            start_at: Utc::now(),
            end_at: Utc::now() + chrono::Duration::hours(5),
        };

        // Try to create promotion under suspended vendor account (must fail)
        let create_res = PromotionService::create_promotion(&mut *tx, user_id, payload.clone()).await;
        assert!(create_res.is_err());

        tx.rollback().await.unwrap();
    }

    #[test]
    fn test_sanitize_html() {
        // 1. Script/style tags and content completely removed
        let raw_script = "<script>alert(1)</script>";
        let sanitized_script = super::sanitize_html(raw_script);
        assert_eq!(sanitized_script, "");

        // 2. Attributes completely removed, safe tags kept
        let raw_attr = "<p onclick=\"x()\">Hello</p>";
        let sanitized_attr = super::sanitize_html(raw_attr);
        assert_eq!(sanitized_attr, "<p>Hello</p>");

        // 3. Lists and formatting tags preserved unchanged
        let raw_list = "<ul><li>Offer</li></ul>";
        let sanitized_list = super::sanitize_html(raw_list);
        assert_eq!(sanitized_list, "<ul><li>Offer</li></ul>");

        // 4. Malformed HTML handled safely without panicking
        let raw_malformed = "<p>Hello<b>world";
        let sanitized_malformed = super::sanitize_html(raw_malformed);
        assert_eq!(sanitized_malformed, "<p>Hello<b>world</b></p>");
    }
}
