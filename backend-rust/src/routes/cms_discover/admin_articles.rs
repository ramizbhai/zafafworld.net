use crate::errors::AppError;
use crate::middleware::auth::{RequireAdmin, RlsTx};
use crate::state::AppState;
use axum::{
    extract::Path,
    routing::{get, put},
    Json, Router,
};
use serde_json::{json, Value};
use uuid::Uuid;
use sqlx::Row;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/cms/articles", get(list_cms_articles).post(create_cms_article))
        .route("/cms/articles/:id", put(update_cms_article).delete(delete_cms_article))
}

#[derive(serde::Deserialize)]
pub struct AdminArticleQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub search: Option<String>,
    pub published: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct CmsArticleDto {
    pub slug: String,
    pub category: String,
    pub title_ar: String,
    pub title_en: String,
    pub summary_ar: Option<String>,
    pub summary_en: Option<String>,
    pub body_ar: Option<String>,
    pub body_en: Option<String>,
    pub published: Option<bool>,
}

async fn list_cms_articles(
    _auth: RequireAdmin,
    mut rls_tx: RlsTx,
    axum::extract::Query(query): axum::extract::Query<AdminArticleQuery>,
) -> Result<Json<Value>, AppError> {
    tracing::info!("Admin listing CMS articles...");

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(10).clamp(1, 100);
    let offset = (page - 1) * limit;

    let mut query_builder = String::from("FROM seo_articles WHERE 1 = 1");
    let mut param_idx = 1;
    let mut bindings = Vec::new();

    if let Some(ref search_term) = query.search {
        let clean_search = search_term.trim();
        if !clean_search.is_empty() {
            query_builder.push_str(&format!(
                " AND (title_ar ILIKE ${} OR title_en ILIKE ${} OR slug ILIKE ${})",
                param_idx, param_idx, param_idx
            ));
            bindings.push(format!("%{}%", clean_search));
            param_idx += 1;
        }
    }

    if let Some(ref pub_filter) = query.published {
        if pub_filter != "all" && !pub_filter.trim().is_empty() {
            let is_pub = pub_filter == "true";
            query_builder.push_str(&format!(" AND published = ${}", param_idx));
            bindings.push(is_pub.to_string());
            param_idx += 1;
        }
    }

    // 1. Query count
    let count_query = format!("SELECT COUNT(*)::bigint {}", query_builder);
    let mut sql_count = sqlx::query_scalar(&count_query);
    for b in &bindings {
        if b == "true" || b == "false" {
            sql_count = sql_count.bind(b == "true");
        } else {
            sql_count = sql_count.bind(b);
        }
    }
    let total_count: i64 = sql_count.fetch_one(&mut *rls_tx.tx).await?;

    // 2. Query paginated articles
    let select_query = format!(
        "SELECT id, slug, category, title_ar, title_en, summary_ar, summary_en, body_ar, body_en, published, created_at, updated_at \
         {} \
         ORDER BY created_at DESC \
         LIMIT ${} OFFSET ${}",
        query_builder,
        param_idx,
        param_idx + 1
    );

    let mut sql_select = sqlx::query(&select_query);
    for b in &bindings {
        if b == "true" || b == "false" {
            sql_select = sql_select.bind(b == "true");
        } else {
            sql_select = sql_select.bind(b);
        }
    }
    sql_select = sql_select.bind(limit).bind(offset);
    let rows = sql_select.fetch_all(&mut *rls_tx.tx).await?;

    let mut articles_list = Vec::new();
    for row in rows {
        let id: Uuid = row.get("id");
        let slug: String = row.get("slug");
        let category: String = row.get::<Option<String>, _>("category").unwrap_or_default();
        let title_ar: String = row.get("title_ar");
        let title_en: String = row.get("title_en");
        let summary_ar: Option<String> = row.get("summary_ar");
        let summary_en: Option<String> = row.get("summary_en");
        let body_ar: Option<String> = row.get("body_ar");
        let body_en: Option<String> = row.get("body_en");
        let published: bool = row.get("published");
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

        articles_list.push(json!({
            "id": id.to_string(),
            "slug": slug,
            "category": category,
            "titleAr": title_ar,
            "titleEn": title_en,
            "summaryAr": summary_ar.unwrap_or_default(),
            "summaryEn": summary_en.unwrap_or_default(),
            "bodyAr": body_ar.unwrap_or_default(),
            "bodyEn": body_en.unwrap_or_default(),
            "published": published,
            "createdAt": created_at.to_rfc3339(),
            "updatedAt": updated_at.to_rfc3339(),
        }));
    }

    rls_tx.tx.commit().await?;

    let total_pages = ((total_count + limit - 1) / limit).max(1);

    Ok(Json(json!({
        "status": "success",
        "articles": articles_list,
        "total": total_count,
        "page": page,
        "totalPages": total_pages
    })))
}

async fn create_cms_article(
    _auth: RequireAdmin,
    mut rls_tx: RlsTx,
    Json(payload): Json<CmsArticleDto>,
) -> Result<Json<Value>, AppError> {
    tracing::info!("Admin creating CMS article with slug: {}", payload.slug);

    // Validate unique slug
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM seo_articles WHERE slug = $1)")
            .bind(&payload.slug)
            .fetch_one(&mut *rls_tx.tx)
            .await?;

    if exists {
        return Err(AppError::BadRequest(format!(
            "Article with slug '{}' already exists",
            payload.slug
        )));
    }

    let is_pub = payload.published.unwrap_or(true);

    let row = sqlx::query(
        "INSERT INTO seo_articles (slug, category, title_ar, title_en, summary_ar, summary_en, body_ar, body_en, published) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
         RETURNING id, slug, category, title_ar, title_en, summary_ar, summary_en, body_ar, body_en, published, created_at, updated_at"
    )
    .bind(&payload.slug)
    .bind(&payload.category)
    .bind(&payload.title_ar)
    .bind(&payload.title_en)
    .bind(payload.summary_ar.as_deref().unwrap_or(""))
    .bind(payload.summary_en.as_deref().unwrap_or(""))
    .bind(payload.body_ar.as_deref().unwrap_or(""))
    .bind(payload.body_en.as_deref().unwrap_or(""))
    .bind(is_pub)
    .fetch_one(&mut *rls_tx.tx)
    .await?;

    let id: Uuid = row.get("id");
    let slug: String = row.get("slug");
    let category: String = row.get::<Option<String>, _>("category").unwrap_or_default();
    let title_ar: String = row.get("title_ar");
    let title_en: String = row.get("title_en");
    let summary_ar: Option<String> = row.get("summary_ar");
    let summary_en: Option<String> = row.get("summary_en");
    let body_ar: Option<String> = row.get("body_ar");
    let body_en: Option<String> = row.get("body_en");
    let published: bool = row.get("published");
    let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
    let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "article": {
            "id": id.to_string(),
            "slug": slug,
            "category": category,
            "titleAr": title_ar,
            "titleEn": title_en,
            "summaryAr": summary_ar.unwrap_or_default(),
            "summaryEn": summary_en.unwrap_or_default(),
            "bodyAr": body_ar.unwrap_or_default(),
            "bodyEn": body_en.unwrap_or_default(),
            "published": published,
            "createdAt": created_at.to_rfc3339(),
            "updatedAt": updated_at.to_rfc3339(),
        }
    })))
}

async fn update_cms_article(
    _auth: RequireAdmin,
    mut rls_tx: RlsTx,
    Path(id): Path<Uuid>,
    Json(payload): Json<CmsArticleDto>,
) -> Result<Json<Value>, AppError> {
    tracing::info!("Admin updating CMS article: {}", id);

    // Check uniqueness of slug if changed
    let current_slug: Option<String> =
        sqlx::query_scalar("SELECT slug FROM seo_articles WHERE id = $1")
            .bind(id)
            .fetch_optional(&mut *rls_tx.tx)
            .await?;

    let current_slug = match current_slug {
        Some(slug) => slug,
        None => return Err(AppError::NotFound("Article not found".to_string())),
    };

    if current_slug != payload.slug {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM seo_articles WHERE slug = $1 AND id != $2)",
        )
        .bind(&payload.slug)
        .bind(id)
        .fetch_one(&mut *rls_tx.tx)
        .await?;

        if exists {
            return Err(AppError::BadRequest(format!(
                "Article with slug '{}' already exists on another record",
                payload.slug
            )));
        }
    }

    let is_pub = payload.published.unwrap_or(true);

    let row = sqlx::query(
        "UPDATE seo_articles \
         SET slug = $1, category = $2, title_ar = $3, title_en = $4, \
             summary_ar = $5, summary_en = $6, body_ar = $7, body_en = $8, \
             published = $9, updated_at = NOW() \
         WHERE id = $10 \
         RETURNING id, slug, category, title_ar, title_en, summary_ar, summary_en, body_ar, body_en, published, created_at, updated_at"
    )
    .bind(&payload.slug)
    .bind(&payload.category)
    .bind(&payload.title_ar)
    .bind(&payload.title_en)
    .bind(payload.summary_ar.as_deref().unwrap_or(""))
    .bind(payload.summary_en.as_deref().unwrap_or(""))
    .bind(payload.body_ar.as_deref().unwrap_or(""))
    .bind(payload.body_en.as_deref().unwrap_or(""))
    .bind(is_pub)
    .bind(id)
    .fetch_one(&mut *rls_tx.tx)
    .await?;

    let id: Uuid = row.get("id");
    let slug: String = row.get("slug");
    let category: String = row.get::<Option<String>, _>("category").unwrap_or_default();
    let title_ar: String = row.get("title_ar");
    let title_en: String = row.get("title_en");
    let summary_ar: Option<String> = row.get("summary_ar");
    let summary_en: Option<String> = row.get("summary_en");
    let body_ar: Option<String> = row.get("body_ar");
    let body_en: Option<String> = row.get("body_en");
    let published: bool = row.get("published");
    let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
    let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "article": {
            "id": id.to_string(),
            "slug": slug,
            "category": category,
            "titleAr": title_ar,
            "titleEn": title_en,
            "summaryAr": summary_ar.unwrap_or_default(),
            "summaryEn": summary_en.unwrap_or_default(),
            "bodyAr": body_ar.unwrap_or_default(),
            "bodyEn": body_en.unwrap_or_default(),
            "published": published,
            "createdAt": created_at.to_rfc3339(),
            "updatedAt": updated_at.to_rfc3339(),
        }
    })))
}

async fn delete_cms_article(
    _auth: RequireAdmin,
    mut rls_tx: RlsTx,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    tracing::info!("Admin deleting CMS article: {}", id);

    let rows_affected = sqlx::query("DELETE FROM seo_articles WHERE id = $1")
        .bind(id)
        .execute(&mut *rls_tx.tx)
        .await?
        .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound("Article not found".to_string()));
    }

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Article deleted successfully"
    })))
}

