use crate::errors::AppError;
use serde_json::json;
use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct PolicyLimits {
    pub max_products: i64,
    pub max_cover_photos: i64,
    pub max_additional_photos: i64,
    pub max_videos: i64,
    pub max_description_blocks: i64,
    #[serde(default)]
    pub max_promotions: i64,
}

pub struct PolicyEngine;

impl PolicyEngine {
    pub async fn fetch_limits(
        vendor_id: Uuid,
        db: &mut sqlx::PgConnection,
    ) -> Result<(String, PolicyLimits), AppError> {
        let row = sqlx::query!(
            r#"SELECT 
                 st.id as "tier_id?", 
                 st.name as "tier_name?",
                 v.subscription_expires_at
               FROM vendors v 
               LEFT JOIN subscription_tiers st ON v.subscription_tier_id = st.id 
               WHERE v.id = $1"#,
            vendor_id
        )
        .fetch_optional(db)
        .await?;

        let mut is_expired = false;
        if let Some(r) = &row {
            if let Some(expires_at) = r.subscription_expires_at {
                if expires_at < chrono::Utc::now() {
                    is_expired = true;
                }
            }
        }

        let (tier_id, tier_name) = match row {
            Some(r) if !is_expired => (
                r.tier_id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "free".to_string()),
                r.tier_name.unwrap_or_else(|| "Free".to_string()),
            ),
            _ => ("free".to_string(), "Free".to_string()),
        };

        let name_lower = tier_name.to_lowercase();
        let limits = if name_lower.contains("diamond") {
            PolicyLimits {
                max_products: 50,
                max_cover_photos: 1,
                max_additional_photos: -1, // unlimited
                max_videos: -1,            // unlimited
                max_description_blocks: -1,
                max_promotions: -1,        // unlimited
            }
        } else if name_lower.contains("vip") {
            PolicyLimits {
                max_products: 15,
                max_cover_photos: 1,
                max_additional_photos: 30,
                max_videos: 10,
                max_description_blocks: 30,
                max_promotions: 10,
            }
        } else if name_lower.contains("gold") {
            PolicyLimits {
                max_products: 5,
                max_cover_photos: 1,
                max_additional_photos: 15,
                max_videos: 0,
                max_description_blocks: 15,
                max_promotions: 2,
            }
        } else {
            // Free
            PolicyLimits {
                max_products: 1,
                max_cover_photos: 1,
                max_additional_photos: 10,
                max_videos: 0,
                max_description_blocks: 5,
                max_promotions: 0,
            }
        };

        Ok((tier_id, limits))
    }

    fn error_response(tier_id: &str, limit_type: &str) -> AppError {
        if tier_id == "diamond" {
            AppError::Forbidden(format!("Diamond tier limits reached for {}", limit_type))
        } else {
            let meta = json!({
                "upgrade_required": true,
                "current_tier": tier_id,
                "limit_type": limit_type,
            });
            AppError::PaymentRequired(
                format!(
                    "Upgrade your subscription to increase {} limits.",
                    limit_type
                ),
                meta,
            )
        }
    }

    pub async fn check_product_limit(
        vendor_id: Uuid,
        exclude_product_id: Option<Uuid>,
        db: &mut sqlx::PgConnection,
    ) -> Result<(), AppError> {
        let (tier_id, limits) = Self::fetch_limits(vendor_id, db).await?;
        if limits.max_products == -1 {
            return Ok(());
        }

        let current_count: Option<i64> = if let Some(exclude_id) = exclude_product_id {
            sqlx::query_scalar!(
                "SELECT count(*) FROM vendor_products WHERE vendor_id = $1 AND status IN ('active', 'pending_approval') AND id != $2",
                vendor_id,
                exclude_id
            )
            .fetch_one(db)
            .await?
        } else {
            sqlx::query_scalar!(
                "SELECT count(*) FROM vendor_products WHERE vendor_id = $1 AND status IN ('active', 'pending_approval')",
                vendor_id
            )
            .fetch_one(db)
            .await?
        };

        if current_count.unwrap_or(0) >= limits.max_products {
            return Err(Self::error_response(&tier_id, "products"));
        }
        Ok(())
    }

    /// Enforce max active/pending promotions per subscription tier.
    /// Counts promotions with status in ('pending', 'approved', 'paused') — i.e. non-terminal states.
    /// Excludes a specific promotion ID when editing (so edits don't count against themselves).
    pub async fn check_promotion_limit(
        vendor_id: Uuid,
        exclude_promo_id: Option<Uuid>,
        db: &mut sqlx::PgConnection,
    ) -> Result<(), AppError> {
        let (tier_id, limits) = Self::fetch_limits(vendor_id, db).await?;

        // -1 means unlimited
        if limits.max_promotions == -1 {
            return Ok(());
        }

        // 0 means promotions are not available for this tier
        if limits.max_promotions == 0 {
            return Err(Self::error_response(&tier_id, "promotions"));
        }

        let current_count: Option<i64> = if let Some(exclude_id) = exclude_promo_id {
            sqlx::query_scalar!(
                "SELECT count(*) FROM listing_promotions WHERE vendor_id = $1 AND status IN ('pending', 'approved', 'paused') AND id != $2",
                vendor_id,
                exclude_id
            )
            .fetch_one(db)
            .await?
        } else {
            sqlx::query_scalar!(
                "SELECT count(*) FROM listing_promotions WHERE vendor_id = $1 AND status IN ('pending', 'approved', 'paused')",
                vendor_id
            )
            .fetch_one(db)
            .await?
        };

        if current_count.unwrap_or(0) >= limits.max_promotions {
            return Err(Self::error_response(&tier_id, "promotions"));
        }
        Ok(())
    }

    pub async fn check_media_limit(
        vendor_id: Uuid,
        product_id: Option<Uuid>,
        is_video: bool,
        is_cover: bool,
        db: &mut sqlx::PgConnection,
    ) -> Result<(), AppError> {
        if is_cover {
            return Ok(());
        }
        let (tier_id, limits) = Self::fetch_limits(vendor_id, db).await?;

        let media_type = if is_video { "video" } else { "image" };

        // Count existing media for this product.
        // If product_id is None, it's a new product being created, count is 0.
        let current_count = if let Some(pid) = product_id {
            let count: Option<i64> = sqlx::query_scalar!(
                "SELECT count(*) FROM vendor_gallery WHERE product_id = $1 AND media_type = $2 AND is_cover = $3",
                pid,
                media_type,
                is_cover
            )
            .fetch_one(db)
            .await?;
            count.unwrap_or(0)
        } else {
            0
        };

        let limit = if is_cover {
            limits.max_cover_photos
        } else if is_video {
            limits.max_videos
        } else {
            limits.max_additional_photos
        };

        if limit != -1 && current_count >= limit {
            return Err(Self::error_response(
                &tier_id,
                if is_cover {
                    "cover_photos"
                } else if is_video {
                    "videos"
                } else {
                    "photos"
                },
            ));
        }

        Ok(())
    }

    pub async fn check_gallery_batch_limit(
        vendor_id: Uuid,
        cover_count: i64,
        photo_count: i64,
        video_count: i64,
        db: &mut sqlx::PgConnection,
    ) -> Result<(), AppError> {
        let (tier_id, limits) = Self::fetch_limits(vendor_id, db).await?;

        if limits.max_cover_photos != -1 && cover_count > limits.max_cover_photos {
            return Err(Self::error_response(&tier_id, "cover_photos"));
        }
        if limits.max_additional_photos != -1 && photo_count > limits.max_additional_photos {
            return Err(Self::error_response(&tier_id, "photos"));
        }
        if limits.max_videos != -1 && video_count > limits.max_videos {
            return Err(Self::error_response(&tier_id, "videos"));
        }

        Ok(())
    }

    pub async fn check_description_limit(
        vendor_id: Uuid,
        description_ar: &Option<String>,
        description_en: &Option<String>,
        db: &mut sqlx::PgConnection,
    ) -> Result<(), AppError> {
        let (tier_id, limits) = Self::fetch_limits(vendor_id, db).await?;
        if limits.max_description_blocks == -1 {
            return Ok(());
        }

        let count_ar = Self::count_blocks(description_ar);
        let count_en = Self::count_blocks(description_en);

        if count_ar > limits.max_description_blocks || count_en > limits.max_description_blocks {
            return Err(Self::error_response(&tier_id, "description_blocks"));
        }

        Ok(())
    }

    fn count_blocks(desc: &Option<String>) -> i64 {
        if let Some(text) = desc {
            // Attempt to parse as JSON array
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) {
                if let Some(arr) = parsed.as_array() {
                    return arr.len() as i64;
                }
            }
            // If not a JSON array but not empty, it counts as 1 block
            if !text.trim().is_empty() {
                return 1;
            }
        }
        0
    }
}
