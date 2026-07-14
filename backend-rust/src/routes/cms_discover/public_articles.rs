use crate::errors::AppError;
use crate::state::AppState;
use axum::{
    extract::{Path, State, Query},
    http::HeaderMap,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;
use sqlx::Row;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/articles", get(list_articles))
        .route("/articles/:slug", get(get_article_by_slug))
        .with_state(state)
}

#[derive(Deserialize)]
pub struct ArticlesQuery {
    pub category: Option<String>,
}

async fn list_articles(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ArticlesQuery>,
) -> Result<Json<Value>, AppError> {
    let country_id = headers
        .get("X-Country-ID")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| "sa".to_string());

    tracing::info!("Querying public articles for country: {}...", country_id);

    let mut query_str = " \
        SELECT \
            a.id, \
            a.title_ar, \
            a.title_en, \
            a.slug, \
            a.content_ar, \
            a.content_en, \
            a.cover_image, \
            a.category_id, \
            a.city_id, \
            a.created_at, \
            a.updated_at \
        FROM articles a \
        LEFT JOIN cities c ON a.city_id = c.id \
        WHERE (a.city_id IS NULL OR c.country_id = $1) \
    "
    .to_string();

    let mut category_val = None;

    if let Some(ref cat) = query.category {
        query_str.push_str(" AND a.category_id = $2");
        category_val = Some(cat);
    }

    query_str.push_str(" ORDER BY a.created_at DESC");

    let mut db_query = sqlx::query(&query_str).bind(&country_id);
    if let Some(cat) = category_val {
        db_query = db_query.bind(cat);
    }

    let rows = db_query.fetch_all(&state.db).await?;
    let mut articles_json = Vec::new();

    for row in rows {
        let id: Uuid = row.get("id");
        let title_ar: String = row.get("title_ar");
        let title_en: String = row.get("title_en");
        let slug: String = row.get("slug");
        let content_ar: String = row.get::<Option<String>, _>("content_ar").unwrap_or_default();
        let content_en: String = row.get::<Option<String>, _>("content_en").unwrap_or_default();
        let cover_image: String = row.get::<Option<String>, _>("cover_image").unwrap_or_default();
        let category_id: Option<String> = row.get("category_id");
        let city_id: Option<Uuid> = row.get("city_id");
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

        articles_json.push(json!({
            "id": id.to_string(),
            "titleAr": title_ar,
            "titleEn": title_en,
            "slug": slug,
            "contentAr": content_ar,
            "contentEn": content_en,
            "coverImage": cover_image,
            "categoryId": category_id,
            "cityId": city_id.map(|u| u.to_string()),
            "createdAt": created_at.to_rfc3339(),
            "updatedAt": updated_at.to_rfc3339()
        }));
    }

    Ok(Json(json!({
        "status": "success",
        "articles": articles_json,
        "total": articles_json.len()
    })))
}

async fn get_article_by_slug(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(slug): Path<String>,
) -> Result<Json<Value>, AppError> {
    tracing::info!("Querying details for article: {}", slug);

    let country_id = headers
        .get("X-Country-ID")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| "sa".to_string());

    let query_str = " \
        SELECT \
            a.id, \
            a.title_ar, \
            a.title_en, \
            a.slug, \
            a.content_ar, \
            a.content_en, \
            a.cover_image, \
            a.category_id, \
            a.city_id, \
            a.created_at, \
            a.updated_at, \
            c.country_id \
        FROM articles a \
        LEFT JOIN cities c ON a.city_id = c.id \
        WHERE a.slug = $1 \
    ";

    let row = sqlx::query(query_str)
        .bind(&slug)
        .fetch_optional(&state.db)
        .await?;

    let row = match row {
        Some(r) => r,
        None => return Err(AppError::NotFound("Article not found".to_string())),
    };

    let article_country_id: Option<String> = row.get("country_id");
    let city_id: Option<Uuid> = row.get("city_id");

    // Geofencing: if city is linked, active country_id must match the city's country_id.
    if let Some(ref cid) = article_country_id {
        if cid != &country_id {
            return Err(AppError::NotFound("Article not found".to_string()));
        }
    }

    let id: Uuid = row.get("id");
    let title_ar: String = row.get("title_ar");
    let title_en: String = row.get("title_en");
    let slug: String = row.get("slug");
    let content_ar: String = row.get::<Option<String>, _>("content_ar").unwrap_or_default();
    let content_en: String = row.get::<Option<String>, _>("content_en").unwrap_or_default();
    let cover_image: String = row.get::<Option<String>, _>("cover_image").unwrap_or_default();
    let category_id: Option<String> = row.get("category_id");
    let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
    let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

    Ok(Json(json!({
        "status": "success",
        "article": {
            "id": id.to_string(),
            "titleAr": title_ar,
            "titleEn": title_en,
            "slug": slug,
            "contentAr": content_ar,
            "contentEn": content_en,
            "coverImage": cover_image,
            "categoryId": category_id,
            "cityId": city_id.map(|u| u.to_string()),
            "createdAt": created_at.to_rfc3339(),
            "updatedAt": updated_at.to_rfc3339()
        }
    })))
}
