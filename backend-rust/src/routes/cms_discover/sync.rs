/// Internal sync route — WordPress → Rust backend bridge
///
/// This module exposes a single endpoint:
///   POST /api/v1/internal/wp-sync
///
/// It is NOT a public endpoint. It is:
///   - Only callable from inside `zafaf_network` (the Podman bridge network).
///   - NOT protected by JWT — it uses a shared secret in the X-Webhook-Secret header.
///   - NOT subject to CORS — browsers never call it; no CORS headers are applied.
///   - Excluded from the domain segregation middleware (which only guards /client/, /vendor/, /admin/).
///
/// Purpose: When WordPress publishes or updates a post, it fires a fire-and-forget
/// POST to this endpoint. The handler upserts a shadow row in the Postgres `blogs`
/// table (source='wordpress'), keeping all FK relationships (blog_views,
/// blog_comments, blog_funnel_events) intact without touching those tables.
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
    Json, Router,
};
use serde::Deserialize;

use crate::{errors::AppError, state::AppState};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/wp-sync", post(sync_wp_post))
        .route("/wp-media-sync", post(sync_wp_media))
        .with_state(state)
}

/// Payload sent by the WordPress `sync-webhook.php` mu-plugin on every publish or delete.
#[derive(Debug, Deserialize)]
pub struct WpSyncPayload {
    /// WordPress numeric post ID. Used as the conflict key for upserts.
    pub wp_post_id: i64,
    /// Action requested: "publish" or "delete"
    #[serde(default)]
    pub action: Option<String>,
    /// WordPress post slug (post_name). Must be unique in the `blogs` table.
    pub slug: Option<String>,
    /// English title — either the English translation (Polylang) or post_title fallback.
    pub title_en: Option<String>,
    /// Arabic title — the Arabic translation (Polylang) or empty string if unavailable.
    pub title_ar: Option<String>,
    /// Post publish date in UTC (ISO 8601), from WordPress post_date_gmt.
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Rendered HTML content of the post.
    pub content_html: Option<String>,
    /// Short excerpt text of the post.
    pub excerpt: Option<String>,
    /// Featured image URL.
    pub cover_image_url: Option<String>,
    /// Language code (e.g. 'en' or 'ar').
    pub lang: Option<String>,
    /// Polylang translation group ID (English post ID).
    pub translation_group_id: Option<i64>,
    /// Rank Math custom meta title.
    pub meta_title: Option<String>,
    /// Rank Math custom meta description.
    pub meta_description: Option<String>,
    /// Rank Math focus keywords.
    pub focus_keywords: Option<String>,
    /// Rank Math canonical URL.
    pub canonical_url: Option<String>,
    /// Categories from WordPress.
    pub categories: Option<Vec<WpTaxonomyPayload>>,
    /// Tags from WordPress.
    pub tags: Option<Vec<WpTaxonomyPayload>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WpTaxonomyPayload {
    pub name: String,
    pub slug: String,
}

/// Handler: POST /api/v1/internal/wp-sync
///
/// Authenticates via X-Webhook-Secret header (shared secret, not JWT).
/// On success:
///   - If action is "delete": Deletes the entire translation group (Arabic + English linked posts).
///   - If action is "publish": Upserts a shadow row in `blogs`.
///
/// Returns 200 OK on success, 401 on auth failure, 500 on DB error.
pub async fn sync_wp_post(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<WpSyncPayload>,
) -> Result<StatusCode, AppError> {
    // ── Shared-secret authentication ─────────────────────────────────────────
    // This is server-to-server auth, not user-session auth.
    // The secret is read from AppConfig which reads WP_SYNC_SECRET from the environment.
    let provided_secret = headers
        .get("X-Webhook-Secret")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let expected_secret = &app_state.config.wp_sync_secret;

    // Constant-time comparison to resist timing attacks (though this is internal-only).
    if expected_secret.is_empty() || provided_secret != expected_secret.as_str() {
        tracing::warn!(
            target: "security",
            "WP sync webhook rejected: invalid or missing X-Webhook-Secret. \
             wp_post_id={}, action={:?}",
            payload.wp_post_id,
            payload.action
        );
        return Err(AppError::Unauthorized(
            "Invalid webhook secret".to_string(),
        ));
    }

    let action = payload.action.as_deref().unwrap_or("publish");

    if action == "delete" {
        let mut tx = app_state
            .db
            .begin()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // 1. Retrieve translation_group_id for this WP post ID if it exists
        let translation_group: Option<i64> = sqlx::query_scalar(
            "SELECT translation_group_id FROM blogs WHERE wp_post_id = $1"
        )
        .bind(payload.wp_post_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        if let Some(tg_id) = translation_group {
            // Delete both Arabic & English translations linked to this group
            sqlx::query("DELETE FROM blogs WHERE translation_group_id = $1")
                .bind(tg_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;
            
            tracing::info!(
                "WP sync: deleted translation group {} (wp_post_id={})",
                tg_id,
                payload.wp_post_id
            );
        } else {
            // Fallback: delete the post by ID
            sqlx::query("DELETE FROM blogs WHERE wp_post_id = $1")
                .bind(payload.wp_post_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

            tracing::info!(
                "WP sync: deleted wp_post_id={}",
                payload.wp_post_id
            );
        }

        tx.commit()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        return Ok(StatusCode::OK);
    }

    // Unpack fields with safe fallbacks for upsert logic
    let slug = payload.slug.clone().unwrap_or_default();
    let title_en = payload.title_en.clone().unwrap_or_default();
    let title_ar = payload.title_ar.clone().unwrap_or_default();
    let content_html = payload.content_html.clone().unwrap_or_default();
    let excerpt = payload.excerpt.clone().unwrap_or_default();
    let cover_image_url = payload.cover_image_url.clone().unwrap_or_default();
    let lang = payload.lang.clone().unwrap_or_else(|| "ar".to_string());
    let translation_group_id = payload.translation_group_id.unwrap_or(payload.wp_post_id);
    let meta_title = payload.meta_title.clone().unwrap_or_default();
    let meta_description = payload.meta_description.clone().unwrap_or_default();
    let focus_keywords = payload.focus_keywords.clone().unwrap_or_default();
    let canonical_url = payload.canonical_url.clone().unwrap_or_default();
    let categories = payload.categories.clone().unwrap_or_default();
    let tags = payload.tags.clone().unwrap_or_default();
    let published_at = payload.published_at.unwrap_or_else(chrono::Utc::now);

    // ── Upsert in a Single Transaction ────────────────────────────────────────
    let mut tx = app_state
        .db
        .begin()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Determine language-specific SEO title/description values based on locale
    let (meta_title_en, meta_title_ar) = if lang == "ar" {
        ("", meta_title.as_str())
    } else {
        (meta_title.as_str(), "")
    };
    let (meta_description_en, meta_description_ar) = if lang == "ar" {
        ("", meta_description.as_str())
    } else {
        (meta_description.as_str(), "")
    };

    let blog_row_result: Result<(uuid::Uuid,), sqlx::Error> = sqlx::query_as(
        r#"
        INSERT INTO blogs (
            wp_post_id,
            slug,
            title,
            title_en,
            title_ar,
            content_html,
            content_markdown,
            excerpt,
            cover_image_url,
            author_id,
            is_published,
            published_at,
            source,
            lang,
            translation_group_id,
            meta_title,
            meta_title_en,
            meta_title_ar,
            meta_description,
            meta_description_en,
            meta_description_ar,
            focus_keywords,
            canonical_url
        )
        VALUES (
            $1, $2, CASE WHEN $9 = 'ar' THEN $4 ELSE $3 END, $3, $4, $5, '', $6, $7,
            (SELECT id FROM global_users WHERE email = 'afrah@zafafworld.net' LIMIT 1),
            true, $8, 'wordpress', $9, $10, $11, $12, $13, $14, $15, $16, $17, $18
        )
        ON CONFLICT (wp_post_id) DO UPDATE
        SET
            slug                 = EXCLUDED.slug,
            title                = EXCLUDED.title,
            title_en             = EXCLUDED.title_en,
            title_ar             = EXCLUDED.title_ar,
            content_html         = EXCLUDED.content_html,
            excerpt              = EXCLUDED.excerpt,
            cover_image_url      = EXCLUDED.cover_image_url,
            published_at         = EXCLUDED.published_at,
            lang                 = EXCLUDED.lang,
            translation_group_id = EXCLUDED.translation_group_id,
            meta_title           = EXCLUDED.meta_title,
            meta_title_en        = EXCLUDED.meta_title_en,
            meta_title_ar        = EXCLUDED.meta_title_ar,
            meta_description     = EXCLUDED.meta_description,
            meta_description_en  = EXCLUDED.meta_description_en,
            meta_description_ar  = EXCLUDED.meta_description_ar,
            focus_keywords       = EXCLUDED.focus_keywords,
            canonical_url        = EXCLUDED.canonical_url,
            updated_at           = now()
        RETURNING id
        "#,
    )
    .bind(payload.wp_post_id)
    .bind(&slug)
    .bind(&title_en)
    .bind(&title_ar)
    .bind(&content_html)
    .bind(&excerpt)
    .bind(&cover_image_url)
    .bind(published_at)
    .bind(&lang)
    .bind(translation_group_id)
    .bind(&meta_title)
    .bind(meta_title_en)
    .bind(meta_title_ar)
    .bind(&meta_description)
    .bind(meta_description_en)
    .bind(meta_description_ar)
    .bind(&focus_keywords)
    .bind(&canonical_url)
    .fetch_one(&mut *tx)
    .await;

    let blog_id = match blog_row_result {
        Ok(row) => row.0,
        Err(e) => {
            tracing::error!(
                "WP sync: DB upsert failed for wp_post_id={}, slug={}: {}",
                payload.wp_post_id,
                slug,
                e
            );
            return Err(AppError::Database(e.to_string()));
        }
    };

    // ── Synchronize Category Relationships ────────────────────────────────────
    if let Err(e) = sqlx::query("DELETE FROM blog_category_map WHERE blog_id = $1")
        .bind(blog_id)
        .execute(&mut *tx)
        .await
    {
        return Err(AppError::Database(e.to_string()));
    }

    for cat in &categories {
        // Safe check to see if category already exists by slug or name
        let existing: Option<uuid::Uuid> = sqlx::query_scalar(
            "SELECT id FROM blog_categories WHERE slug = $1 OR name = $2"
        )
        .bind(&cat.slug)
        .bind(&cat.name)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let cat_id = match existing {
            Some(id) => id,
            None => {
                let cat_row_result: Result<(uuid::Uuid,), sqlx::Error> = sqlx::query_as(
                    r#"
                    INSERT INTO blog_categories (name, slug)
                    VALUES ($1, $2)
                    RETURNING id
                    "#,
                )
                .bind(&cat.name)
                .bind(&cat.slug)
                .fetch_one(&mut *tx)
                .await;

                match cat_row_result {
                    Ok(row) => row.0,
                    Err(e) => return Err(AppError::Database(e.to_string())),
                }
            }
        };

        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO blog_category_map (blog_id, category_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(blog_id)
        .bind(cat_id)
        .execute(&mut *tx)
        .await {
            return Err(AppError::Database(e.to_string()));
        }
    }

    // ── Synchronize Tag Relationships ─────────────────────────────────────────
    if let Err(e) = sqlx::query("DELETE FROM blog_tags_map WHERE blog_id = $1")
        .bind(blog_id)
        .execute(&mut *tx)
        .await
    {
        return Err(AppError::Database(e.to_string()));
    }

    for tag in &tags {
        // Safe check to see if tag already exists by slug or name
        let existing: Option<uuid::Uuid> = sqlx::query_scalar(
            "SELECT id FROM blog_tags WHERE slug = $1 OR name = $2"
        )
        .bind(&tag.slug)
        .bind(&tag.name)
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let tag_id = match existing {
            Some(id) => id,
            None => {
                let tag_row_result: Result<(uuid::Uuid,), sqlx::Error> = sqlx::query_as(
                    r#"
                    INSERT INTO blog_tags (name, slug)
                    VALUES ($1, $2)
                    RETURNING id
                    "#,
                )
                .bind(&tag.name)
                .bind(&tag.slug)
                .fetch_one(&mut *tx)
                .await;

                match tag_row_result {
                    Ok(row) => row.0,
                    Err(e) => return Err(AppError::Database(e.to_string())),
                }
            }
        };

        if let Err(e) = sqlx::query(
            r#"
            INSERT INTO blog_tags_map (blog_id, tag_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(blog_id)
        .bind(tag_id)
        .execute(&mut *tx)
        .await {
            return Err(AppError::Database(e.to_string()));
        }
    }

    if let Err(e) = tx.commit().await {
        tracing::error!("WP sync transaction commit failed: {}", e);
        return Err(AppError::Database(e.to_string()));
    }

    tracing::info!(
        "WP sync: upserted shadow row, categories, and tags for wp_post_id={}, slug={}",
        payload.wp_post_id,
        slug
    );

    // ── Upload Cover Image to MinIO CDN Ingress Proxy ─────────────────────────
    if !cover_image_url.is_empty() {
        let prefix1 = "https://api.zafafworld.net/assets/uploads/";
        let prefix2 = "/assets/uploads/";
        let relative_path = cover_image_url.strip_prefix(prefix1)
            .or_else(|| cover_image_url.strip_prefix(prefix2));

        if let Some(rel_path) = relative_path {
            if let Some(pos) = rel_path.rfind('/') {
                let folder = &rel_path[..pos + 1]; // e.g. "2026/07/"
                let filename = &rel_path[pos + 1..]; // e.g. "image.jpg"
                let disk_path = format!("/app/assets/uploads/{}", rel_path);

                let ext = std::path::Path::new(filename)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                
                let mime_type = match ext.as_str() {
                    "png" => "image/png",
                    "webp" => "image/webp",
                    "gif" => "image/gif",
                    "svg" => "image/svg+xml",
                    _ => "image/jpeg",
                };

                if std::path::Path::new(&disk_path).exists() {
                    tracing::info!("WP Sync: Uploading cover image {} to MinIO...", disk_path);
                    if let Err(e) = app_state.minio_client.upload(
                        &disk_path,
                        &format!("assets/uploads/{}", folder),
                        filename,
                        mime_type,
                    ).await {
                        tracing::error!("WP Sync: Failed to upload cover image to MinIO: {}", e);
                    } else {
                        tracing::info!("WP Sync: Cover image {} uploaded successfully to MinIO", disk_path);
                    }
                } else {
                    tracing::warn!("WP Sync: Cover image file not found on disk at {}", disk_path);
                }
            }
        }
    }

    Ok(StatusCode::OK)
}

#[derive(Debug, Deserialize)]
pub struct WpMediaSyncPayload {
    pub relative_path: String,
    pub mime_type: String,
}

pub async fn sync_wp_media(
    State(app_state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<WpMediaSyncPayload>,
) -> Result<StatusCode, AppError> {
    let provided_secret = headers
        .get("X-Webhook-Secret")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let expected_secret = &app_state.config.wp_sync_secret;

    if provided_secret != expected_secret {
        return Err(AppError::Unauthorized("Invalid webhook secret".to_string()));
    }

    let rel_path = payload.relative_path.trim_start_matches('/');
    if let Some(pos) = rel_path.rfind('/') {
        let folder = &rel_path[..pos + 1]; // e.g. "2026/07/"
        let filename = &rel_path[pos + 1..];
        let disk_path = format!("/app/assets/uploads/{}", rel_path);

        if std::path::Path::new(&disk_path).exists() {
            tracing::info!("WP Media Sync: Uploading {} to MinIO...", disk_path);
            app_state.minio_client.upload(
                &disk_path,
                &format!("assets/uploads/{}", folder),
                filename,
                &payload.mime_type,
            )
            .await
            .map_err(|e| AppError::Internal(format!("Failed to upload media to MinIO: {}", e)))?;
            tracing::info!("WP Media Sync: Media {} uploaded successfully to MinIO", disk_path);
        } else {
            tracing::warn!("WP Media Sync: Media file not found on disk at {}", disk_path);
            return Err(AppError::BadRequest(format!("Media file not found on disk at {}", disk_path)));
        }
    } else {
        return Err(AppError::BadRequest("Invalid relative path".to_string()));
    }

    Ok(StatusCode::OK)
}
