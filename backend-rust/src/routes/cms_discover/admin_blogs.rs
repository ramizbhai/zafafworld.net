use crate::errors::AppError;
use crate::state::AppState;
use axum::{
    extract::{Multipart, Path, Query, State},
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/blogs/upload", post(upload_blog_image))
        .route("/blogs", get(list_admin_blogs).post(create_blog))
        .route(
            "/blogs/:id",
            get(get_admin_blog).put(update_blog).delete(delete_blog),
        )
        .route("/tags", get(list_tags).post(create_tag))
        .route("/tags/:id", delete(delete_tag))
        .route("/categories", get(list_categories).post(create_category))
        .route("/categories/:id", delete(delete_category))
        .route("/comments", get(list_comments))
        .route("/comments/:id/approve", patch(approve_comment))
        .route("/comments/:id/reject", patch(reject_comment))
        .route("/analytics/discover", get(get_discover_analytics))
}

#[derive(Deserialize)]
pub struct AdminBlogsQuery {
    page: Option<i64>,
    limit: Option<i64>,
}

pub async fn list_admin_blogs(
    State(state): State<AppState>,
    Query(query): Query<AdminBlogsQuery>,
) -> Result<Json<Value>, AppError> {
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;

    let rows = sqlx::query(
        "
        SELECT id, title, slug, is_published, published_at, created_at
        FROM blogs
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
    ",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch admin blogs: {}", e);
        AppError::Internal("Database error".into())
    })?;

    let blogs: Vec<Value> = rows
        .into_iter()
        .map(|r| {
            json!({
                "id": r.get::<Uuid, _>("id"),
                "title": r.get::<String, _>("title"),
                "slug": r.get::<String, _>("slug"),
                "is_published": r.get::<bool, _>("is_published"),
                "published_at": r.try_get::<chrono::DateTime<chrono::Utc>, _>("published_at").ok(),
                "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at")
            })
        })
        .collect();

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM blogs")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    Ok(Json(
        json!({ "status": "success", "data": blogs, "total": total, "page": page, "limit": limit }),
    ))
}

#[derive(Deserialize)]
pub struct CreateBlogPayload {
    slug: String,
    title: String,
    title_ar: Option<String>,
    title_en: Option<String>,
    excerpt: Option<String>,
    content_html: String,
    content_markdown: String,
    cover_image_url: Option<String>,
    meta_title: Option<String>,
    meta_title_ar: Option<String>,
    meta_title_en: Option<String>,
    meta_description: Option<String>,
    meta_description_ar: Option<String>,
    meta_description_en: Option<String>,
    focus_keywords: Option<String>,
    read_time_minutes: i32,
    is_published: bool,
    published_at: Option<chrono::DateTime<chrono::Utc>>,
    categories: Option<Vec<Uuid>>,
    tags: Option<Vec<Uuid>>,
    canonical_url: Option<String>,
}

use crate::middleware::auth::{RequireAdmin, RequireAuth};

pub async fn create_blog(
    State(state): State<AppState>,
    auth: RequireAuth,
    Json(payload): Json<CreateBlogPayload>,
) -> Result<Json<Value>, AppError> {
    let mut tx = state.db.begin().await?;

    let published_at = if payload.is_published {
        payload.published_at.or_else(|| Some(chrono::Utc::now()))
    } else {
        None
    };

    let res = sqlx::query("
        INSERT INTO blogs (slug, title, title_ar, title_en, excerpt, content_html, content_markdown, cover_image_url, author_id, meta_title, meta_title_ar, meta_title_en, meta_description, meta_description_ar, meta_description_en, focus_keywords, read_time_minutes, is_published, published_at, canonical_url)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
        RETURNING id
    ")
    .bind(&payload.slug)
    .bind(payload.title_en.as_deref().unwrap_or(&payload.title)) // Keep legacy title synced to English
    .bind(&payload.title_ar)
    .bind(payload.title_en.as_deref().unwrap_or(&payload.title))
    .bind(&payload.excerpt)
    // Basic sanitization (skipped)
    .bind(&payload.content_html)
    .bind(&payload.content_markdown)
    .bind(&payload.cover_image_url)
    .bind(Uuid::parse_str(&auth.user_id).unwrap_or_default())
    .bind(payload.meta_title_en.as_deref().or(payload.meta_title.as_deref())) // Legacy
    .bind(&payload.meta_title_ar)
    .bind(payload.meta_title_en.as_deref().or(payload.meta_title.as_deref()))
    .bind(payload.meta_description_en.as_deref().or(payload.meta_description.as_deref())) // Legacy
    .bind(&payload.meta_description_ar)
    .bind(payload.meta_description_en.as_deref().or(payload.meta_description.as_deref()))
    .bind(&payload.focus_keywords)
    .bind(payload.read_time_minutes)
    .bind(payload.is_published)
    .bind(published_at)
    .bind(&payload.canonical_url)
    .fetch_one(&mut *tx)
    .await;

    match res {
        Ok(row) => {
            let id: Uuid = row.get("id");

            if let Some(cats) = payload.categories {
                for cat_id in cats {
                    let _ = sqlx::query("INSERT INTO blog_category_map (blog_id, category_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                        .bind(id).bind(cat_id).execute(&mut *tx).await;
                }
            }
            if let Some(tgs) = payload.tags {
                for tag_id in tgs {
                    let _ = sqlx::query("INSERT INTO blog_tags_map (blog_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                        .bind(id).bind(tag_id).execute(&mut *tx).await;
                }
            }

            tx.commit().await?;
            Ok(Json(json!({ "status": "success", "data": { "id": id } })))
        }
        Err(e) => {
            let _ = tx.rollback().await;
            tracing::error!("Create blog error: {}", e);
            Err(AppError::BadRequest(format!(
                "Failed to create blog. error: {}",
                e
            )))
        }
    }
}

pub async fn get_admin_blog(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let row = sqlx::query("SELECT * FROM blogs WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?;
    let row = row.ok_or_else(|| AppError::NotFound("Blog not found".into()))?;

    let blog = json!({
        "id": row.get::<Uuid, _>("id"),
        "slug": row.get::<String, _>("slug"),
        "title": row.get::<String, _>("title"),
        "title_ar": row.try_get::<String, _>("title_ar").unwrap_or_default(),
        "title_en": row.try_get::<String, _>("title_en").unwrap_or_default(),
        "excerpt": row.try_get::<String, _>("excerpt").unwrap_or_default(),
        "content_html": row.get::<String, _>("content_html"),
        "content_markdown": row.get::<String, _>("content_markdown"),
        "cover_image_url": row.try_get::<String, _>("cover_image_url").unwrap_or_default(),
        "meta_title": row.try_get::<String, _>("meta_title").unwrap_or_default(),
        "meta_title_ar": row.try_get::<String, _>("meta_title_ar").unwrap_or_default(),
        "meta_title_en": row.try_get::<String, _>("meta_title_en").unwrap_or_default(),
        "meta_description": row.try_get::<String, _>("meta_description").unwrap_or_default(),
        "meta_description_ar": row.try_get::<String, _>("meta_description_ar").unwrap_or_default(),
        "meta_description_en": row.try_get::<String, _>("meta_description_en").unwrap_or_default(),
        "focus_keywords": row.try_get::<String, _>("focus_keywords").unwrap_or_default(),
        "read_time_minutes": row.get::<i32, _>("read_time_minutes"),
        "is_published": row.get::<bool, _>("is_published"),
        "published_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("published_at").ok(),
    });

    let tags_rows = sqlx::query("SELECT tag_id FROM blog_tags_map WHERE blog_id = $1")
        .bind(id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
    let tags: Vec<Uuid> = tags_rows.into_iter().map(|r| r.get("tag_id")).collect();

    let cat_rows = sqlx::query("SELECT category_id FROM blog_category_map WHERE blog_id = $1")
        .bind(id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();
    let categories: Vec<Uuid> = cat_rows.into_iter().map(|r| r.get("category_id")).collect();

    let mut blog_obj = blog
        .as_object()
        .ok_or_else(|| AppError::Internal("Blog serialization failure".to_string()))?
        .clone();
    blog_obj.insert("tags".to_string(), json!(tags));
    blog_obj.insert("categories".to_string(), json!(categories));

    Ok(Json(json!({ "status": "success", "data": blog_obj })))
}

pub async fn update_blog(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateBlogPayload>,
) -> Result<Json<Value>, AppError> {
    let published_at = if payload.is_published {
        payload.published_at.or_else(|| Some(chrono::Utc::now()))
    } else {
        None
    };

    let mut tx = state.db.begin().await?;

    let res = sqlx::query("
        UPDATE blogs 
        SET slug = $1, title = $2, title_ar = $3, title_en = $4, excerpt = $5, content_html = $6, content_markdown = $7, cover_image_url = $8, meta_title = $9, meta_title_ar = $10, meta_title_en = $11, meta_description = $12, meta_description_ar = $13, meta_description_en = $14, focus_keywords = $15, read_time_minutes = $16, is_published = $17, published_at = $18, canonical_url = $19, updated_at = now()
        WHERE id = $20
    ")
    .bind(&payload.slug)
    .bind(payload.title_en.as_deref().unwrap_or(&payload.title)) // Keep legacy title synced to English
    .bind(&payload.title_ar)
    .bind(payload.title_en.as_deref().unwrap_or(&payload.title))
    .bind(&payload.excerpt)
    .bind(&payload.content_html)
    .bind(&payload.content_markdown)
    .bind(&payload.cover_image_url)
    .bind(payload.meta_title_en.as_deref().or(payload.meta_title.as_deref())) // Legacy
    .bind(&payload.meta_title_ar)
    .bind(payload.meta_title_en.as_deref().or(payload.meta_title.as_deref()))
    .bind(payload.meta_description_en.as_deref().or(payload.meta_description.as_deref())) // Legacy
    .bind(&payload.meta_description_ar)
    .bind(payload.meta_description_en.as_deref().or(payload.meta_description.as_deref()))
    .bind(&payload.focus_keywords)
    .bind(payload.read_time_minutes)
    .bind(payload.is_published)
    .bind(published_at)
    .bind(&payload.canonical_url)
    .bind(id)
    .execute(&mut *tx)
    .await;

    if let Err(e) = res {
        let _ = tx.rollback().await;
        tracing::error!("Update blog error: {}", e);
        return Err(AppError::Internal("Database error".into()));
    }

    if let Some(cats) = payload.categories {
        sqlx::query("DELETE FROM blog_category_map WHERE blog_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        for cat_id in cats {
            let _ = sqlx::query("INSERT INTO blog_category_map (blog_id, category_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                .bind(id).bind(cat_id).execute(&mut *tx).await;
        }
    }

    if let Some(tgs) = payload.tags {
        sqlx::query("DELETE FROM blog_tags_map WHERE blog_id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        for tag_id in tgs {
            let _ = sqlx::query("INSERT INTO blog_tags_map (blog_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                .bind(id).bind(tag_id).execute(&mut *tx).await;
        }
    }

    tx.commit().await?;

    Ok(Json(json!({ "status": "success" })))
}

pub async fn delete_blog(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    sqlx::query("DELETE FROM blogs WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    Ok(Json(json!({ "status": "success" })))
}

pub async fn list_comments(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let rows = sqlx::query(
        "
        SELECT c.*, b.title as blog_title, b.slug as blog_slug 
        FROM blog_comments c
        JOIN blogs b ON c.blog_id = b.id
        ORDER BY c.created_at DESC
        LIMIT 50
    ",
    )
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let comments: Vec<Value> = rows
        .into_iter()
        .map(|r| {
            json!({
                "id": r.get::<Uuid, _>("id"),
                "parent_id": r.try_get::<Uuid, _>("parent_id").ok(),
                "blog_title": r.get::<String, _>("blog_title"),
                "blog_slug": r.get::<String, _>("blog_slug"),
                "name": r.get::<String, _>("name"),
                "email": r.try_get::<String, _>("email").unwrap_or_default(),
                "comment": r.get::<String, _>("comment"),
                "is_approved": r.get::<bool, _>("is_approved"),
                "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
            })
        })
        .collect();

    Ok(Json(json!({ "status": "success", "data": comments })))
}

pub async fn approve_comment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    sqlx::query("UPDATE blog_comments SET is_approved = true WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    Ok(Json(json!({ "status": "success" })))
}

pub async fn reject_comment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    sqlx::query("DELETE FROM blog_comments WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    Ok(Json(json!({ "status": "success" })))
}

pub async fn get_discover_analytics(
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let total_views: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM blog_views")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);
    let pending_comments: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM blog_comments WHERE is_approved = false")
            .fetch_one(&state.db)
            .await
            .unwrap_or(0);
    let total_posts: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM blogs WHERE is_published = true")
            .fetch_one(&state.db)
            .await
            .unwrap_or(0);

    let afrah_cta_clicks: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM blog_funnel_events WHERE event_type = 'cta_click_afrah'",
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);
    let vendor_cta_clicks: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM blog_funnel_events WHERE event_type = 'cta_click_vendor'",
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);
    let afrah_starts: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM blog_funnel_events WHERE event_type = 'afrah_start'",
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);
    let inquiry_starts: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM blog_funnel_events WHERE event_type = 'inquiry_start'",
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);
    let booking_conversions: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM blog_funnel_events WHERE event_type = 'booking_conversion'",
    )
    .fetch_one(&state.db)
    .await
    .unwrap_or(0);

    Ok(Json(json!({
        "status": "success",
        "data": {
            "total_views": total_views,
            "pending_comments": pending_comments,
            "total_posts": total_posts,
            "afrah_cta_clicks": afrah_cta_clicks,
            "vendor_cta_clicks": vendor_cta_clicks,
            "afrah_starts": afrah_starts,
            "inquiry_starts": inquiry_starts,
            "booking_conversions": booking_conversions
        }
    })))
}

pub async fn list_tags(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let rows = sqlx::query("SELECT * FROM blog_tags")
        .fetch_all(&state.db)
        .await?;
    let tags: Vec<Value> = rows.into_iter().map(|r| {
        json!({ "id": r.get::<Uuid, _>("id"), "name": r.get::<String, _>("name"), "slug": r.get::<String, _>("slug") })
    }).collect();
    Ok(Json(json!({ "status": "success", "data": tags })))
}

#[derive(Deserialize)]
pub struct CreateTagPayload {
    name: String,
    slug: String,
}

pub async fn create_tag(
    State(state): State<AppState>,
    Json(payload): Json<CreateTagPayload>,
) -> Result<Json<Value>, AppError> {
    sqlx::query("INSERT INTO blog_tags (name, slug) VALUES ($1, $2)")
        .bind(&payload.name)
        .bind(&payload.slug)
        .execute(&state.db)
        .await?;
    Ok(Json(json!({ "status": "success" })))
}

pub async fn delete_tag(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    sqlx::query("DELETE FROM blog_tags WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    Ok(Json(json!({ "status": "success" })))
}

pub async fn list_categories(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let rows = sqlx::query("SELECT * FROM blog_categories")
        .fetch_all(&state.db)
        .await?;
    let categories: Vec<Value> = rows.into_iter().map(|r| {
        json!({ "id": r.get::<Uuid, _>("id"), "name": r.get::<String, _>("name"), "slug": r.get::<String, _>("slug") })
    }).collect();
    Ok(Json(json!({ "status": "success", "data": categories })))
}

#[derive(Deserialize)]
pub struct CreateCategoryPayload {
    name: String,
    slug: String,
}

pub async fn create_category(
    State(state): State<AppState>,
    Json(payload): Json<CreateCategoryPayload>,
) -> Result<Json<Value>, AppError> {
    sqlx::query("INSERT INTO blog_categories (name, slug) VALUES ($1, $2)")
        .bind(&payload.name)
        .bind(&payload.slug)
        .execute(&state.db)
        .await?;
    Ok(Json(json!({ "status": "success" })))
}

pub async fn delete_category(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    sqlx::query("DELETE FROM blog_categories WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await?;
    Ok(Json(json!({ "status": "success" })))
}

pub async fn upload_blog_image(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    _auth: RequireAdmin,
    mut multipart: Multipart,
) -> Result<Json<Value>, AppError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart error: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            let file_name = field.file_name().unwrap_or("blog-cover.jpg").to_string();

            let target_dir = "assets/uploads/blogs/";
            let url_prefix = "/assets/uploads/blogs/";

            let processed = crate::services::media::process_and_save_upload(
                field,
                &file_name,
                target_dir,
                url_prefix,
                10 * 1024 * 1024, // 10 MB max for blog images
                1920,             // max dimension
                &state.minio_client,
            )
            .await
            .map_err(|e| {
                tracing::error!("Blog cover upload failed: {:?}", e);
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
