use crate::errors::AppError;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State, Multipart},
    routing::{get, patch, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

use crate::middleware::auth::RequireAdmin;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/blogs", get(list_admin_blogs).post(create_admin_blog))
        .route("/blogs/upload", post(upload_blog_cover_handler))
        .route("/blogs/:id", get(get_admin_blog).put(update_admin_blog))
        .route("/tags", get(list_tags))
        .route("/categories", get(list_categories))
        .route("/comments", get(list_comments))
        .route("/comments/:id/approve", patch(approve_comment))
        .route("/comments/:id/reject", patch(reject_comment))
        .route("/analytics/discover", get(get_discover_analytics))
}

#[derive(Deserialize)]
pub struct CreateBlogPayload {
    pub slug: String,
    pub title: String,
    pub excerpt: String,
    pub content_html: String,
    pub content_markdown: String,
    pub cover_image_url: String,
    pub meta_title: String,
    pub meta_description: String,
    pub focus_keywords: String,
    pub read_time_minutes: i32,
    pub is_published: bool,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
}

pub async fn create_admin_blog(
    State(state): State<AppState>,
    auth: RequireAdmin,
    Json(payload): Json<CreateBlogPayload>,
) -> Result<Json<Value>, AppError> {
    let blog_id = Uuid::new_v4();
    let published_at = if payload.is_published {
        payload.published_at.or_else(|| Some(chrono::Utc::now()))
    } else {
        None
    };
    
    let author_id: Uuid = auth.user_id.parse().map_err(|_| AppError::Internal("Invalid admin user id".into()))?;

    sqlx::query(
        r#"
        INSERT INTO blogs (
            id, slug, title, title_ar, title_en, excerpt, content_html, content_markdown,
            cover_image_url, meta_title, meta_description, focus_keywords,
            read_time_minutes, is_published, published_at,
            author_id
        ) VALUES (
            $1, $2, $3, '', '', $4, $5, $6,
            $7, $8, $9, $10,
            $11, $12, $13,
            $14
        )
        "#,
    )
    .bind(blog_id)
    .bind(&payload.slug)
    .bind(&payload.title)
    .bind(&payload.excerpt)
    .bind(&payload.content_html)
    .bind(&payload.content_markdown)
    .bind(&payload.cover_image_url)
    .bind(&payload.meta_title)
    .bind(&payload.meta_description)
    .bind(&payload.focus_keywords)
    .bind(payload.read_time_minutes)
    .bind(payload.is_published)
    .bind(published_at)
    .bind(author_id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create blog: {}", e);
        AppError::Internal("Failed to create blog".into())
    })?;

    Ok(Json(json!({ "status": "success", "data": { "id": blog_id } })))
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

pub async fn list_categories(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let rows = sqlx::query("SELECT * FROM blog_categories")
        .fetch_all(&state.db)
        .await?;
    let categories: Vec<Value> = rows.into_iter().map(|r| {
        json!({ "id": r.get::<Uuid, _>("id"), "name": r.get::<String, _>("name"), "slug": r.get::<String, _>("slug") })
    }).collect();
    Ok(Json(json!({ "status": "success", "data": categories })))
}

pub async fn upload_blog_cover_handler(
    State(state): State<AppState>,
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
            let file_name = field.file_name().unwrap_or("cover.jpg").to_string();
            let target_dir = "assets/uploads/blogs/";
            let url_prefix = "/assets/uploads/blogs/";

            let processed = crate::services::media::process_and_save_upload(
                field,
                &file_name,
                target_dir,
                url_prefix,
                5 * 1024 * 1024, // 5 MB max
                1920,            // max dimension
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
            })));
        }
    }

    Err(AppError::BadRequest("A valid image file attachment is required in the 'file' field.".to_string()))
}
pub async fn update_admin_blog(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateBlogPayload>,
) -> Result<Json<Value>, AppError> {
    let published_at = if payload.is_published {
        payload.published_at.or_else(|| Some(chrono::Utc::now()))
    } else {
        None
    };

    sqlx::query(
        r#"
        UPDATE blogs SET
            slug = $1, title = $2, excerpt = $3, content_html = $4, content_markdown = $5,
            cover_image_url = $6, meta_title = $7, meta_description = $8, focus_keywords = $9,
            read_time_minutes = $10, is_published = $11, published_at = $12
        WHERE id = $13
        "#,
    )
    .bind(&payload.slug)
    .bind(&payload.title)
    .bind(&payload.excerpt)
    .bind(&payload.content_html)
    .bind(&payload.content_markdown)
    .bind(&payload.cover_image_url)
    .bind(&payload.meta_title)
    .bind(&payload.meta_description)
    .bind(&payload.focus_keywords)
    .bind(payload.read_time_minutes)
    .bind(payload.is_published)
    .bind(published_at)
    .bind(id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update blog: {}", e);
        AppError::Internal("Failed to update blog".into())
    })?;

    Ok(Json(json!({ "status": "success", "data": { "id": id } })))
}
