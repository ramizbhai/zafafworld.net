use crate::errors::AppError;
use crate::middleware::auth::RequireAuth;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

pub fn router(_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/blogs", get(list_blogs))
        .route("/blogs/:slug", get(get_blog_by_slug))
        .route("/blogs/related/:slug", get(get_related_blogs))
        .route("/blogs/:slug/comments", post(create_blog_comment))
        .route("/blogs/:slug/view", post(record_blog_view))
        .route("/blogs/:slug/track", post(track_blog_event))
        .route("/faqs", get(list_faqs))
}

#[derive(Deserialize)]
pub struct BlogsQuery {
    page: Option<i64>,
    limit: Option<i64>,
    category: Option<String>,
    tag: Option<String>,
    search: Option<String>,
    lang: Option<String>,
}

pub async fn list_blogs(
    State(state): State<AppState>,
    Query(query): Query<BlogsQuery>,
) -> Result<Json<Value>, AppError> {
    let limit = query.limit.unwrap_or(12).clamp(1, 50);
    let page = query.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;

    let mut sql = "
        SELECT b.id, b.slug, b.title, b.title_ar, b.title_en, b.excerpt, b.cover_image_url, b.read_time_minutes, b.published_at, b.lang, u.display_name 
        FROM blogs b
        LEFT JOIN global_users u ON b.author_id = u.id
        WHERE b.is_published = true AND (b.published_at IS NULL OR b.published_at <= NOW())
    ".to_string();

    let mut args = sqlx::postgres::PgArguments::default();
    use sqlx::Arguments;
    let mut param_idx = 1;

    if let Some(lang) = &query.lang {
        sql.push_str(&format!(" AND b.lang = ${}", param_idx));
        let _ = args.add(lang);
        param_idx += 1;
    }

    if let Some(search) = &query.search {
        sql.push_str(&format!(
            " AND (b.title ILIKE ${} OR b.excerpt ILIKE ${})",
            param_idx, param_idx
        ));
        let term = format!("%{}%", search);
        let _ = args.add(term.clone());
        param_idx += 1;
    }

    if let Some(category) = &query.category {
        sql.push_str(&format!(" AND b.id IN (SELECT bc.blog_id FROM blog_category_map bc JOIN blog_categories c ON bc.category_id = c.id WHERE c.slug = ${})", param_idx));
        let _ = args.add(category);
        param_idx += 1;
    }

    if let Some(tag) = &query.tag {
        sql.push_str(&format!(" AND b.id IN (SELECT bt.blog_id FROM blog_tags_map bt JOIN blog_tags t ON bt.tag_id = t.id WHERE t.slug = ${})", param_idx));
        let _ = args.add(tag);
        param_idx += 1;
    }


    sql.push_str(&format!(
        " ORDER BY b.published_at DESC LIMIT ${} OFFSET ${}",
        param_idx,
        param_idx + 1
    ));
    let _ = args.add(limit);
    let _ = args.add(offset);

    let rows = sqlx::query_with(&sql, args)
        .fetch_all(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch blogs: {}", e);
            AppError::Internal(e.to_string())
        })?;

    let blogs: Vec<Value> = rows.into_iter().map(|row| {
        let display_name: Option<String> = row.try_get("display_name").unwrap_or_default();
        let author_name = display_name.unwrap_or_else(|| "Zafaf World Team".to_string());

        json!({
            "id": row.get::<Uuid, _>("id"),
            "slug": row.get::<String, _>("slug"),
            "title": row.get::<String, _>("title"),
            "title_ar": row.try_get::<String, _>("title_ar").unwrap_or_default(),
            "title_en": row.try_get::<String, _>("title_en").unwrap_or_default(),
            "excerpt": row.try_get::<String, _>("excerpt").unwrap_or_default(),
            "cover_image_url": row.try_get::<String, _>("cover_image_url").unwrap_or_default(),
            "read_time_minutes": row.get::<i32, _>("read_time_minutes"),
            "published_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("published_at").ok(),
            "lang": row.try_get::<String, _>("lang").unwrap_or_else(|_| "en".to_string()),
            "author": author_name
        })
    }).collect();

    Ok(Json(json!({
        "status": "success",
        "data": blogs,
        "page": page,
        "limit": limit
    })))
}

pub async fn get_blog_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Value>, AppError> {
    let row = sqlx::query("
        SELECT b.*, u.display_name 
        FROM blogs b
        LEFT JOIN global_users u ON b.author_id = u.id
        WHERE b.slug = $1 AND b.is_published = true AND (b.published_at IS NULL OR b.published_at <= NOW())
    ")
    .bind(&slug)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch blog by slug: {}", e);
        AppError::Internal(e.to_string())
    })?;

    let row = row.ok_or(AppError::NotFound("Blog not found".to_string()))?;

    let display_name: Option<String> = row.try_get("display_name").unwrap_or_default();
    let author_name = display_name.unwrap_or_else(|| "Zafaf World Team".to_string());

    let blog_id: Uuid = row.get("id");

    // Fetch tags
    let tags_rows = sqlx::query(
        "
        SELECT t.name, t.slug 
        FROM blog_tags t 
        JOIN blog_tags_map tm ON t.id = tm.tag_id 
        WHERE tm.blog_id = $1
    ",
    )
    .bind(blog_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let tags: Vec<Value> = tags_rows
        .into_iter()
        .map(|r| {
            json!({
                "name": r.get::<String, _>("name"),
                "slug": r.get::<String, _>("slug")
            })
        })
        .collect();

    // Fetch categories
    let categories_rows = sqlx::query(
        "
        SELECT c.name, c.slug 
        FROM blog_categories c 
        JOIN blog_category_map cm ON c.id = cm.category_id 
        WHERE cm.blog_id = $1
    ",
    )
    .bind(blog_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let categories: Vec<Value> = categories_rows
        .into_iter()
        .map(|r| {
            json!({
                "name": r.get::<String, _>("name"),
                "slug": r.get::<String, _>("slug")
            })
        })
        .collect();

    // Fetch translations
    let translation_group_id: Option<i64> = row.try_get("translation_group_id").ok();
    let lang: String = row.try_get("lang").unwrap_or_else(|_| "en".to_string());
    let canonical_url: Option<String> = row.try_get("canonical_url").ok();

    let mut translations = Vec::new();
    if let Some(group_id) = translation_group_id {
        let trans_rows = sqlx::query(
            "SELECT lang, slug FROM blogs WHERE translation_group_id = $1 AND is_published = true"
        )
        .bind(group_id)
        .fetch_all(&state.db)
        .await
        .unwrap_or_default();

        for tr in trans_rows {
            translations.push(json!({
                "lang": tr.get::<String, _>("lang"),
                "slug": tr.get::<String, _>("slug")
            }));
        }
    }

    // Fetch comments
    let comments_rows = sqlx::query(
        "
        SELECT id, parent_id, name, comment, created_at 
        FROM blog_comments 
        WHERE blog_id = $1 AND is_approved = true
        ORDER BY created_at ASC
    ",
    )
    .bind(blog_id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let comments: Vec<Value> = comments_rows
        .into_iter()
        .map(|r| {
            json!({
                "id": r.get::<Uuid, _>("id"),
                "parent_id": r.try_get::<Uuid, _>("parent_id").ok(),
                "name": r.get::<String, _>("name"),
                "comment": r.get::<String, _>("comment"),
                "created_at": r.get::<chrono::DateTime<chrono::Utc>, _>("created_at")
            })
        })
        .collect();

    let views_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM blog_views WHERE blog_id = $1")
        .bind(blog_id)
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let blog = json!({
        "id": blog_id,
        "slug": row.get::<String, _>("slug"),
        "title": row.get::<String, _>("title"),
        "title_ar": row.try_get::<String, _>("title_ar").unwrap_or_default(),
        "title_en": row.try_get::<String, _>("title_en").unwrap_or_default(),
        "excerpt": row.try_get::<String, _>("excerpt").unwrap_or_default(),
        "content_html": row.get::<String, _>("content_html"),
        "cover_image_url": row.try_get::<String, _>("cover_image_url").unwrap_or_default(),
        "meta_title": row.try_get::<String, _>("meta_title").unwrap_or_default(),
        "meta_title_ar": row.try_get::<String, _>("meta_title_ar").unwrap_or_default(),
        "meta_title_en": row.try_get::<String, _>("meta_title_en").unwrap_or_default(),
        "meta_description": row.try_get::<String, _>("meta_description").unwrap_or_default(),
        "meta_description_ar": row.try_get::<String, _>("meta_description_ar").unwrap_or_default(),
        "meta_description_en": row.try_get::<String, _>("meta_description_en").unwrap_or_default(),
        "focus_keywords": row.try_get::<String, _>("focus_keywords").unwrap_or_default(),
        "canonical_url": canonical_url.unwrap_or_default(),
        "read_time_minutes": row.get::<i32, _>("read_time_minutes"),
        "published_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("published_at").ok(),
        "author": author_name,
        "tags": tags,
        "categories": categories,
        "lang": lang,
        "translations": translations,
        "comments": comments,
        "views_count": views_count
    });

    Ok(Json(json!({
        "status": "success",
        "data": blog
    })))
}

pub async fn get_related_blogs(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Value>, AppError> {
    let blog_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM blogs WHERE slug = $1 AND is_published = true AND (published_at IS NULL OR published_at <= NOW())")
        .bind(&slug)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| AppError::Internal("Database error".into()))?;

    let blog_id = blog_id.ok_or(AppError::NotFound("Blog not found".to_string()))?;

    // Find related blogs based on shared tags
    let rows = sqlx::query("
        SELECT b.id, b.slug, b.title, b.title_ar, b.title_en, b.excerpt, b.cover_image_url, b.read_time_minutes, b.published_at
        FROM blogs b
        JOIN blog_tags_map btm1 ON b.id = btm1.blog_id
        JOIN blog_tags_map btm2 ON btm1.tag_id = btm2.tag_id
        WHERE btm2.blog_id = $1 AND b.id != $1 AND b.is_published = true AND (b.published_at IS NULL OR b.published_at <= NOW())
        GROUP BY b.id
        ORDER BY COUNT(btm1.tag_id) DESC, b.published_at DESC
        LIMIT 3
    ")
    .bind(blog_id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| AppError::Internal("Database error".into()))?;

    let related: Vec<Value> = rows.into_iter().map(|row| {
        json!({
            "id": row.get::<Uuid, _>("id"),
            "slug": row.get::<String, _>("slug"),
            "title": row.get::<String, _>("title"),
            "title_ar": row.try_get::<String, _>("title_ar").unwrap_or_default(),
            "title_en": row.try_get::<String, _>("title_en").unwrap_or_default(),
            "excerpt": row.try_get::<String, _>("excerpt").unwrap_or_default(),
            "cover_image_url": row.try_get::<String, _>("cover_image_url").unwrap_or_default(),
            "read_time_minutes": row.get::<i32, _>("read_time_minutes"),
            "published_at": row.try_get::<chrono::DateTime<chrono::Utc>, _>("published_at").ok(),
        })
    }).collect();

    Ok(Json(json!({
        "status": "success",
        "data": related
    })))
}

#[derive(Deserialize)]
pub struct CreateCommentPayload {
    comment: String,
    parent_id: Option<Uuid>,
}

pub async fn create_blog_comment(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    auth: RequireAuth,
    Json(payload): Json<CreateCommentPayload>,
) -> Result<Json<Value>, AppError> {
    let blog_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM blogs WHERE slug = $1 AND is_published = true AND (published_at IS NULL OR published_at <= NOW())")
        .bind(&slug)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| AppError::Internal("Database error".into()))?;

    let blog_id = blog_id.ok_or(AppError::NotFound("Blog not found".to_string()))?;

    let user_name = sqlx::query_scalar::<_, String>(
        "SELECT COALESCE(display_name, 'User') FROM global_users WHERE id = $1",
    )
    .bind(&auth.user_id)
    .fetch_one(&state.db)
    .await
    .unwrap_or_else(|_| "User".to_string());
    let name_to_use = if user_name.trim().is_empty() {
        "User"
    } else {
        user_name.trim()
    };

    // Auto-approve comments posted by admins
    let is_approved = auth.role == crate::models::user::DomainType::Admin;

    sqlx::query(
        "
        INSERT INTO blog_comments (blog_id, user_id, name, email, comment, parent_id, is_approved)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
    ",
    )
    .bind(blog_id)
    .bind(&auth.user_id)
    .bind(name_to_use)
    .bind(&auth.email)
    .bind(&payload.comment)
    .bind(payload.parent_id)
    .bind(is_approved)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to insert comment: {}", e);
        AppError::Internal(e.to_string())
    })?;

    let message = if is_approved {
        "Comment posted successfully."
    } else {
        "Comment submitted successfully and is awaiting moderation."
    };

    Ok(Json(json!({
        "status": "success",
        "message": message
    })))
}

#[derive(Deserialize)]
pub struct TrackEventPayload {
    pub event_type: String,
    pub session_id: Option<String>,
}

pub async fn track_blog_event(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Json(payload): Json<TrackEventPayload>,
) -> Result<Json<Value>, AppError> {
    sqlx::query(
        "INSERT INTO blog_funnel_events (blog_slug, event_type, session_id) VALUES ($1, $2, $3)",
    )
    .bind(&slug)
    .bind(&payload.event_type)
    .bind(&payload.session_id)
    .execute(&state.db)
    .await?;

    Ok(Json(json!({ "status": "success" })))
}

pub async fn record_blog_view(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    crate::utils::ip::SecureClientIp(client_ip): crate::utils::ip::SecureClientIp,
) -> Result<Json<Value>, AppError> {
    let blog_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM blogs WHERE slug = $1 AND is_published = true AND (published_at IS NULL OR published_at <= NOW())")
        .bind(&slug)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| AppError::Internal("Database error".into()))?;

    let blog_id = match blog_id {
        Some(id) => id,
        None => return Ok(Json(json!({"status": "ignored"}))),
    };

    let ip_hash = format!("{:x}", md5::compute(client_ip.to_string()));

    let _ = sqlx::query(
        "
        INSERT INTO blog_views (blog_id, ip_hash)
        VALUES ($1, $2)
        ON CONFLICT (blog_id, ip_hash, viewed_date) DO NOTHING
    ",
    )
    .bind(blog_id)
    .bind(ip_hash)
    .execute(&state.db)
    .await;

    Ok(Json(json!({
        "status": "success"
    })))
}

pub async fn list_faqs() -> Result<Json<Value>, AppError> {
    Ok(Json(json!({
        "status": "success",
        "data": []
    })))
}
