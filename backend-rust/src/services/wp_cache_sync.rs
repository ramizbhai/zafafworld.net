use crate::state::AppState;
use std::collections::HashMap;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{info, error, warn};
use reqwest::Client;
use uuid::Uuid;

#[derive(Debug, serde::Deserialize, Clone)]
pub struct WpPost {
    pub id: i64,
    pub slug: String,
    pub title: WpRendered,
    #[serde(rename = "content")]
    pub content: WpRendered,
    pub excerpt: WpRendered,
    pub date_gmt: String,
    pub pll_lang: Option<String>,
    pub pll_translations: Option<HashMap<String, i64>>,
    pub rank_math_meta: Option<WpRankMathMeta>,
    #[serde(rename = "_embedded")]
    pub embedded: Option<WpEmbedded>,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct WpRendered {
    pub rendered: String,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct WpRankMathMeta {
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(alias = "focus_keyword")]
    pub focus_keyword: Option<String>,
    #[serde(alias = "canonical_url")]
    pub canonical: Option<String>,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct WpEmbedded {
    #[serde(rename = "wp:featuredmedia")]
    pub featuredmedia: Option<Vec<WpFeaturedMedia>>,
    #[serde(rename = "wp:term")]
    pub term: Option<Vec<Vec<WpTerm>>>,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct WpFeaturedMedia {
    pub source_url: Option<String>,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct WpTerm {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub taxonomy: String,
}

#[derive(Debug, Clone)]
pub struct WpTaxonomyPayload {
    pub name: String,
    pub slug: String,
}

pub fn start_wp_cache_sync(state: AppState, cancel_token: CancellationToken) {
    tokio::spawn(async move {
        // Runs every 10 minutes (600 seconds)
        let mut interval = tokio::time::interval(Duration::from_secs(600));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    info!("WP Cache Sync: Starting cycle...");
                    match run_sync(&state).await {
                        Ok((created, updated, deleted)) => {
                            info!("WP Cache Sync: Completed successfully. Created: {}, Updated: {}, Deleted: {}", created, updated, deleted);
                        }
                        Err(e) => {
                            warn!("WP Cache Sync: Cycle failed: {}", e);
                        }
                    }
                }
                _ = cancel_token.cancelled() => {
                    info!("WP Cache Sync: Worker received shutdown signal. Exiting loop.");
                    break;
                }
            }
        }
    });
}

async fn run_sync(state: &AppState) -> Result<(usize, usize, u64), String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| format!("Failed to create reqwest client: {:?}", e))?;

    let base_url = &state.config.wp_headless_base_url;
    let mut all_posts: Vec<WpPost> = Vec::new();
    let mut page = 1;
    let mut total_pages = 1;

    loop {
        let url = format!("{}/wp-json/wp/v2/posts?_embed&per_page=100&status=publish&page={}", base_url, page);
        info!("WP Cache Sync: Fetching page {} from URL {}", page, url);

        let response = client.get(&url)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {:?}", e))?;

        let status = response.status();
        if !status.is_success() {
            return Err(format!("WordPress API returned status code {}", status));
        }

        if let Some(total_pages_header) = response.headers().get("X-WP-TotalPages") {
            if let Ok(total_str) = total_pages_header.to_str() {
                if let Ok(parsed_total) = total_str.parse::<usize>() {
                    total_pages = parsed_total;
                }
            }
        }

        let posts: Vec<WpPost> = response.json()
            .await
            .map_err(|e| format!("JSON deserialization failed: {:?}", e))?;

        info!("WP Cache Sync: Fetched {} posts on page {}", posts.len(), page);
        all_posts.extend(posts);

        if page >= total_pages {
            break;
        }
        page += 1;
    }

    info!("WP Cache Sync: Total fetched posts to sync: {}", all_posts.len());

    let posts_by_id: HashMap<i64, WpPost> = all_posts.iter().map(|p| (p.id, p.clone())).collect();

    let mut created_count = 0;
    let mut updated_count = 0;

    for post in &all_posts {
        let lang = post.pll_lang.clone().unwrap_or_else(|| "en".to_string());
        let translations = post.pll_translations.as_ref();

        let translation_group_id = translations
            .and_then(|t| t.get("en").copied())
            .or(if lang == "en" { Some(post.id) } else { None });

        let title_en = if lang == "en" {
            post.title.rendered.clone()
        } else if let Some(en_id) = translations.and_then(|t| t.get("en").copied()) {
            posts_by_id.get(&en_id).map(|p| p.title.rendered.clone()).unwrap_or_default()
        } else {
            "".to_string()
        };

        let title_ar = if lang == "ar" {
            post.title.rendered.clone()
        } else if let Some(ar_id) = translations.and_then(|t| t.get("ar").copied()) {
            posts_by_id.get(&ar_id).map(|p| p.title.rendered.clone()).unwrap_or_default()
        } else {
            "".to_string()
        };

        let meta_title = post.rank_math_meta.as_ref().and_then(|m| m.title.clone()).unwrap_or_default();

        let meta_title_en = if lang == "en" {
            meta_title.clone()
        } else if let Some(en_id) = translations.and_then(|t| t.get("en").copied()) {
            posts_by_id.get(&en_id)
                .and_then(|p| p.rank_math_meta.as_ref())
                .and_then(|m| m.title.clone())
                .unwrap_or_default()
        } else {
            "".to_string()
        };

        let meta_title_ar = if lang == "ar" {
            meta_title.clone()
        } else if let Some(ar_id) = translations.and_then(|t| t.get("ar").copied()) {
            posts_by_id.get(&ar_id)
                .and_then(|p| p.rank_math_meta.as_ref())
                .and_then(|m| m.title.clone())
                .unwrap_or_default()
        } else {
            "".to_string()
        };

        let meta_description = post.rank_math_meta.as_ref().and_then(|m| m.description.clone()).unwrap_or_default();

        let meta_description_en = if lang == "en" {
            meta_description.clone()
        } else if let Some(en_id) = translations.and_then(|t| t.get("en").copied()) {
            posts_by_id.get(&en_id)
                .and_then(|p| p.rank_math_meta.as_ref())
                .and_then(|m| m.description.clone())
                .unwrap_or_default()
        } else {
            "".to_string()
        };

        let meta_description_ar = if lang == "ar" {
            meta_description.clone()
        } else if let Some(ar_id) = translations.and_then(|t| t.get("ar").copied()) {
            posts_by_id.get(&ar_id)
                .and_then(|p| p.rank_math_meta.as_ref())
                .and_then(|m| m.description.clone())
                .unwrap_or_default()
        } else {
            "".to_string()
        };

        let focus_keywords = post.rank_math_meta.as_ref().and_then(|m| m.focus_keyword.clone()).unwrap_or_default();
        let canonical_url = post.rank_math_meta.as_ref().and_then(|m| m.canonical.clone()).unwrap_or_default();

        let cover_image_url = post.embedded.as_ref()
            .and_then(|emb| emb.featuredmedia.as_ref())
            .and_then(|media| media.first())
            .and_then(|m| m.source_url.clone())
            .unwrap_or_default();

        let mut categories = Vec::new();
        let mut tags = Vec::new();

        if let Some(ref embedded) = post.embedded {
            if let Some(ref terms_lists) = embedded.term {
                for terms_list in terms_lists {
                    for term in terms_list {
                        if term.taxonomy == "category" {
                            categories.push(WpTaxonomyPayload {
                                name: term.name.clone(),
                                slug: term.slug.clone(),
                            });
                        } else if term.taxonomy == "post_tag" {
                            tags.push(WpTaxonomyPayload {
                                name: term.name.clone(),
                                slug: term.slug.clone(),
                            });
                        }
                    }
                }
            }
        }

        // Parse published_at
        let published_at = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&post.date_gmt) {
            dt.with_timezone(&chrono::Utc)
        } else if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(&post.date_gmt, "%Y-%m-%dT%H:%M:%S") {
            chrono::DateTime::from_naive_utc_and_offset(naive, chrono::Utc)
        } else if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(&post.date_gmt, "%Y-%m-%d %H:%M:%S") {
            chrono::DateTime::from_naive_utc_and_offset(naive, chrono::Utc)
        } else {
            chrono::Utc::now()
        };

        // Determine if this is a creation or an update
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM blogs WHERE wp_post_id = $1)")
            .bind(post.id)
            .fetch_one(&state.db)
            .await
            .unwrap_or(false);

        let mut tx = state.db.begin().await.map_err(|e| format!("DB transaction begin failed: {:?}", e))?;

        let blog_row: (Uuid,) = sqlx::query_as(
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
        .bind(post.id)
        .bind(&post.slug)
        .bind(&title_en)
        .bind(&title_ar)
        .bind(&post.content.rendered)
        .bind(&post.excerpt.rendered)
        .bind(&cover_image_url)
        .bind(published_at)
        .bind(&lang)
        .bind(translation_group_id)
        .bind(&meta_title)
        .bind(&meta_title_en)
        .bind(&meta_title_ar)
        .bind(&meta_description)
        .bind(&meta_description_en)
        .bind(&meta_description_ar)
        .bind(&focus_keywords)
        .bind(&canonical_url)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| format!("DB upsert failed for post ID {}: {:?}", post.id, e))?;

        let blog_id = blog_row.0;

        // Categories Sync
        sqlx::query("DELETE FROM blog_category_map WHERE blog_id = $1")
            .bind(blog_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("Failed to clear category map: {:?}", e))?;

        for cat in &categories {
            let existing: Option<Uuid> = sqlx::query_scalar(
                "SELECT id FROM blog_categories WHERE slug = $1 OR name = $2"
            )
            .bind(&cat.slug)
            .bind(&cat.name)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| format!("Failed to check category: {:?}", e))?;

            let cat_id = match existing {
                Some(id) => id,
                None => {
                    let cat_row: (Uuid,) = sqlx::query_as(
                        "INSERT INTO blog_categories (name, slug) VALUES ($1, $2) RETURNING id"
                    )
                    .bind(&cat.name)
                    .bind(&cat.slug)
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(|e| format!("Failed to create category: {:?}", e))?;
                    cat_row.0
                }
            };

            sqlx::query("INSERT INTO blog_category_map (blog_id, category_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                .bind(blog_id)
                .bind(cat_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("Failed to link category: {:?}", e))?;
        }

        // Tags Sync
        sqlx::query("DELETE FROM blog_tags_map WHERE blog_id = $1")
            .bind(blog_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| format!("Failed to clear tag map: {:?}", e))?;

        for tag in &tags {
            let existing: Option<Uuid> = sqlx::query_scalar(
                "SELECT id FROM blog_tags WHERE slug = $1 OR name = $2"
            )
            .bind(&tag.slug)
            .bind(&tag.name)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| format!("Failed to check tag: {:?}", e))?;

            let tag_id = match existing {
                Some(id) => id,
                None => {
                    let tag_row: (Uuid,) = sqlx::query_as(
                        "INSERT INTO blog_tags (name, slug) VALUES ($1, $2) RETURNING id"
                    )
                    .bind(&tag.name)
                    .bind(&tag.slug)
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(|e| format!("Failed to create tag: {:?}", e))?;
                    tag_row.0
                }
            };

            sqlx::query("INSERT INTO blog_tags_map (blog_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
                .bind(blog_id)
                .bind(tag_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| format!("Failed to link tag: {:?}", e))?;
        }

        tx.commit().await.map_err(|e| format!("Failed to commit tx: {:?}", e))?;

        if exists {
            updated_count += 1;
        } else {
            created_count += 1;
        }
    }

    // Detect deletions:
    let fetched_ids: Vec<i64> = all_posts.iter().map(|p| p.id).collect();
    let delete_res = sqlx::query("
        DELETE FROM blogs 
        WHERE source = 'wordpress' AND wp_post_id != ALL($1)
    ")
    .bind(&fetched_ids)
    .execute(&state.db)
    .await
    .map_err(|e| format!("Failed to delete stale WP posts: {:?}", e))?;

    let deleted_count = delete_res.rows_affected();

    Ok((created_count, updated_count, deleted_count))
}
