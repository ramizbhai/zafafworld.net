use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{get, post},
    Json, Router,
};
use rust_decimal::Decimal;
use rust_decimal::RoundingStrategy;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;
use validator::Validate;

use crate::state::AppState;
use crate::utils::sanitize::{limits, sanitize_opt, sanitize_str};

use crate::errors::AppError;
use crate::middleware::auth::{RequireAuth, RlsTx};
use crate::utils::crypto::hash_password;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        // ── Legacy vendor-centric routes (kept for backward compat) ──────────
        .route("/vendors", get(list_vendors))
        .route("/vendors/:slug", get(get_vendor_by_slug))
        .route("/vendors/:id/reviews", get(get_vendor_reviews))
        .route("/catalog", get(list_catalog))
        // ── New listing-centric routes ───────────────────────────────────────
        .route("/search/suggestions", get(get_search_suggestions))
        .route("/listings", get(list_listings))
        .route("/listings/:slug", get(get_listing_by_slug))

        .route("/platform/stats", get(get_platform_stats))
        .route("/support", post(create_support_message))
        .route("/sitemap-data", get(get_sitemap_data))
}



async fn get_vendor_reviews(
    State(state): State<AppState>,
    Path(vendor_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    // 1. Fetch approved reviews along with client names
    let rows = sqlx::query(
        "SELECT 
            r.id, r.client_id, r.vendor_id, r.rating, r.review_text, r.created_at,
            cp.first_name AS client_first_name, cp.last_name AS client_last_name
         FROM vendor_reviews r
         LEFT JOIN client_profiles cp ON r.client_id = cp.client_id
         WHERE r.vendor_id = $1 AND r.status = 'approved'
         ORDER BY r.created_at DESC",
    )
    .bind(vendor_id)
    .fetch_all(&state.db)
    .await?;

    // 2. Fetch associated review photo attachments for each review
    let mut reviews = Vec::new();
    for row in rows {
        let review_id: Uuid = row.get("id");

        let photo_rows =
            sqlx::query("SELECT file_path FROM vendor_review_attachments WHERE review_id = $1")
                .bind(review_id)
                .fetch_all(&state.db)
                .await?;

        let attachments: Vec<String> = photo_rows
            .iter()
            .map(|pr| pr.get::<String, _>("file_path"))
            .collect();

        reviews.push(json!({
            "id": review_id.to_string(),
            "client_id": row.get::<Uuid, _>("client_id").to_string(),
            "rating": row.get::<i32, _>("rating"),
            "review_text": row.get::<String, _>("review_text"),
            "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339(),
            "client_name": format!("{} {}", 
                row.get::<Option<String>, _>("client_first_name").unwrap_or_else(|| "User".to_string()),
                row.get::<Option<String>, _>("client_last_name").unwrap_or_default()
            ).trim().to_string(),
            "attachments": attachments
        }));
    }

    // 3. Compute active rating aggregations
    let summary = sqlx::query(
        "SELECT COALESCE(AVG(rating), 0.0)::float8 as avg_rating, COUNT(*)::bigint as total_count 
         FROM vendor_reviews 
         WHERE vendor_id = $1 AND status = 'approved'",
    )
    .bind(vendor_id)
    .fetch_one(&state.db)
    .await?;

    let average_rating: f64 = summary.get("avg_rating");
    let total_count: i64 = summary.get("total_count");

    Ok(Json(json!({
        "status": "success",
        "reviews": reviews,
        "average_rating": average_rating,
        "total_count": total_count
    })))
}

#[derive(Deserialize)]
pub struct VendorsQuery {
    pub category: Option<String>,
    pub city: Option<String>,
    pub city_id: Option<String>,
    pub country_id: Option<String>,

    // Phase 3 Saudi Filters
    pub partition: Option<bool>,
    pub min_capacity: Option<i32>,
    pub max_capacity: Option<i32>,
    pub amenities: Option<String>, // comma-separated e.g. "valet_parking,bridal_suite"
}

async fn list_vendors(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<VendorsQuery>,
) -> Result<Json<Value>, AppError> {
    let country_id = headers
        .get("X-Country-ID")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_lowercase())
        .or_else(|| query.country_id.clone())
        .unwrap_or_else(|| "sa".to_string());

    tracing::info!(
        "Querying public approved vendors with filters for country: {}...",
        country_id
    );

    let repo = crate::repositories::vendor_repository::PgVendorRepository::new(state.db.clone());
    let vendors_json = repo.list_public_vendors(&query, &country_id).await?;

    Ok(Json(json!({
        "status": "success",
        "vendors": vendors_json,
        "total": vendors_json.len(),
        "page": 1,
        "totalPages": 1
    })))
}

async fn get_vendor_by_slug(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(slug): Path<String>,
) -> Result<Json<Value>, AppError> {
    tracing::info!("Querying details for vendor: {}", slug);

    let country_id = headers
        .get("X-Country-ID")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_lowercase());

    let mut query_str = "
        SELECT 
            v.id,
            v.name_ar,
            v.name_en,
            v.slug,
            v.category,
            v.status,
            v.description_ar,
            v.description_en,
            v.capacity_min,
            v.capacity_max,
            v.latitude,
            v.longitude,
            v.address_ar,
            v.address_en,
            NULL::text AS district_ar,
            NULL::text AS district_en,
            v.amenities,
            (v.is_featured = TRUE AND (v.featured_expires_at IS NULL OR v.featured_expires_at > CURRENT_TIMESTAMP)) AS is_featured,
            v.is_available,
            v.phone,
            v.email,
            v.coordinator_name_ar,
            v.coordinator_name_en,
            v.coordinator_phone,
            v.coordinator_whatsapp,
            v.coordinator_avatar,
            v.created_at,
            v.updated_at,
            v.crm_venue_id,
            v.star_rating,
            v.event_spaces_available,
            v.event_type,
            v.website,
            v.maps_url,
            v.video_url_1,
            v.city_id,
            c.slug AS city_slug,
            c.name_ar AS city_name_ar,
            c.name_en AS city_name_en,
            c.country_id,
            COALESCE(rev.quality_avg, 5.0)::FLOAT8 as quality_avg,
            COALESCE(rev.staff_avg, 5.0)::FLOAT8 as staff_avg,
            COALESCE(rev.communication_avg, 5.0)::FLOAT8 as communication_avg,
            COALESCE(rev.review_count, 0) as review_count,
            COALESCE(
                (SELECT json_agg(json_build_object(
                    'id', p.id,
                    'nameAr', p.name_ar,
                    'nameEn', p.name_en,
                    'descriptionAr', p.description_ar,
                    'descriptionEn', p.description_en,
                    'originalPrice', p.original_price::float,
                    'discountedPrice', p.discounted_price::float,
                    'isZafafExclusive', p.is_zafaf_exclusive,
                    'expiryDate', p.expiry_date::text
                ))
                FROM vendor_packages p
                WHERE p.vendor_id = v.id AND p.expiry_date >= CURRENT_DATE
                ), '[]'::json
            ) as packages,
            COALESCE(
                (SELECT json_agg(json_build_object(
                    'id', vg.id,
                    'url', vg.image_url,
                    'alt', COALESCE(vg.caption, ''),
                    'isPrimary', vg.is_cover
                ))
                FROM vendor_gallery vg
                WHERE vg.vendor_id = v.id
                ), '[]'::json
            ) as images,
            COALESCE(
                (SELECT json_agg(json_build_object(
                    'id', r.id,
                    'authorName', r.author_name,
                    'rating', ((r.rating_quality + r.rating_staff + r.rating_communication) / 3.0),
                    'ratingQuality', r.rating_quality,
                    'ratingStaff', r.rating_staff,
                    'ratingCommunication', r.rating_communication,
                    'weddingDate', r.wedding_date::text,
                    'date', r.created_at::text,
                    'comment', r.comment,
                    'eventType', v.category,
                    'helpful', 0
                ))
                FROM reviews r
                WHERE r.vendor_id = v.id AND r.status = 'approved'
                ), '[]'::json
            ) as reviews,
            COALESCE(
                (SELECT json_agg(json_build_object(
                    'id', vp.id,
                    'title', vp.title,
                    'description', COALESCE(vp.description, ''),
                    'slug', vp.slug,
                    'productCategory', vp.product_category,
                    'attributes', vp.attributes,
                    'basePriceSar', vp.base_price_sar::float,
                    'depositPercentage', vp.deposit_percentage,
                    'status', vp.status,
                    'isAvailable', vp.is_available,
                    'images', COALESCE(
                        (SELECT json_agg(json_build_object(
                            'id',              vh.id,
                            'url',             vh.image_url,
                            'alt',             COALESCE(vh.caption, ''),
                            'isPrimary',       vh.is_cover,
                            'mediaType',       vh.media_type,
                            'fileUrl',         vh.file_url,
                            'thumbnailUrl',    vh.thumbnail_url,
                            'fileSize',        vh.file_size,
                            'durationSeconds', vh.duration_seconds
                        ))
                        FROM vendor_gallery vh
                        WHERE vh.product_id = vp.id
                        ), '[]'::json
                    )
                ))
                FROM vendor_products vp
                WHERE vp.vendor_id = v.id AND vp.status = 'active'
                ), '[]'::json
            ) as halls
        FROM vendors v
        LEFT JOIN cities c ON v.city_id = c.id
        LEFT JOIN (
            SELECT 
                vendor_id,
                AVG(rating_quality)::float as quality_avg,
                AVG(rating_staff)::float as staff_avg,
                AVG(rating_communication)::float as communication_avg,
                COUNT(id)::bigint as review_count
            FROM reviews
            WHERE status = 'approved'
            GROUP BY vendor_id
        ) rev ON v.id = rev.vendor_id
        WHERE (v.slug = $1 OR v.id::text = $1) AND v.status = 'active'
    ".to_string();

    let vendor_row = if let Some(ref cid) = country_id {
        query_str.push_str(" AND (c.country_id IS NULL OR c.country_id = $2)");
        sqlx::query(&query_str)
            .bind(&slug)
            .bind(cid)
            .fetch_optional(&state.db)
            .await?
    } else {
        sqlx::query(&query_str)
            .bind(&slug)
            .fetch_optional(&state.db)
            .await?
    };

    let row = match vendor_row {
        Some(r) => r,
        None => return Err(AppError::NotFound("Vendor not found".to_string())),
    };

    let id: Uuid = row.get("id");
    let _name_ar: String = row.get("name_ar");
    let _name_en: String = row.get("name_en");
    let slug_db: String = row.get("slug");
    let category: Option<String> = row.get("category");
    let category = category.unwrap_or_default();
    let description_ar: Option<String> = row.get("description_ar");
    let description_en: Option<String> = row.get("description_en");

    let _city_slug: Option<String> = row.get("city_slug");
    let city_name_ar: Option<String> = row.get("city_name_ar");
    let city_name_en: Option<String> = row.get("city_name_en");
    let _country_id_db: Option<String> = row.get("country_id");

    let capacity_min: Option<i32> = row.get("capacity_min");
    let capacity_max: Option<i32> = row.get("capacity_max");
    let latitude: Option<f64> = row.get("latitude");
    let longitude: Option<f64> = row.get("longitude");
    let _address_ar: Option<String> = row.get("address_ar");
    let address_en: Option<String> = row.get("address_en");
    let district_ar: Option<String> = row.get("district_ar");
    let district_en: Option<String> = row.get("district_en");
    let amenities: Option<Vec<String>> = row.get("amenities");
    let is_featured: bool = row.get("is_featured");
    let is_available: bool = row.get("is_available");

    let phone: Option<String> = row.get("phone");
    let email: Option<String> = row.get("email");

    let coordinator_name_ar: Option<String> = row.get("coordinator_name_ar");
    let coordinator_name_en: Option<String> = row.get("coordinator_name_en");
    let coordinator_phone: Option<String> = row.get("coordinator_phone");
    let coordinator_whatsapp: Option<String> = row.get("coordinator_whatsapp");
    let coordinator_avatar: Option<String> = row.get("coordinator_avatar");

    let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
    let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

    let crm_venue_id: Option<String> = row.get("crm_venue_id");
    let star_rating: Option<Decimal> = row.get("star_rating");
    let event_spaces_available: Option<i32> = row.get("event_spaces_available");
    let event_type: Option<String> = row.get("event_type");
    let website: Option<String> = row.get("website");
    let maps_url: Option<String> = row.get("maps_url");
    let video_url_1: Option<String> = row.get("video_url_1");

    let quality_avg: f64 = row.get("quality_avg");
    let staff_avg: f64 = row.get("staff_avg");
    let communication_avg: f64 = row.get("communication_avg");
    let review_count: i64 = row.get("review_count");

    let overall_avg = (quality_avg + staff_avg + communication_avg) / 3.0;
    let overall_avg = (overall_avg * 10.0).round() / 10.0;
    let quality_avg = (quality_avg * 10.0).round() / 10.0;
    let staff_avg = (staff_avg * 10.0).round() / 10.0;
    let communication_avg = (communication_avg * 10.0).round() / 10.0;

    let packages_val: serde_json::Value = row.get("packages");
    let packages: Vec<serde_json::Value> = packages_val.as_array().cloned().unwrap_or_default();

    let images_val: serde_json::Value = row.get("images");
    let images: Vec<serde_json::Value> = images_val.as_array().cloned().unwrap_or_default();

    let reviews_val: serde_json::Value = row.get("reviews");
    let reviews: Vec<serde_json::Value> = reviews_val.as_array().cloned().unwrap_or_default();

    let halls_val: serde_json::Value = row.get("halls");
    let halls: Vec<serde_json::Value> = halls_val.as_array().cloned().unwrap_or_default();

    let mut starting_price: Option<f64> = None;
    let mut original_price: Option<f64> = None;
    let mut _is_zafaf_exclusive = false;
    let mut _has_offers = false;

    if !packages.is_empty() {
        _has_offers = true;
        let mut min_discounted = f64::MAX;
        let mut matching_original = 0.0;

        for pkg in &packages {
            let disc = pkg
                .get("discountedPrice")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let orig = pkg
                .get("originalPrice")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let excl = pkg
                .get("isZafafExclusive")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if disc > 0.0 && disc < min_discounted {
                min_discounted = disc;
                matching_original = orig;
            }
            if excl {
                _is_zafaf_exclusive = true;
            }
        }

        if min_discounted < f64::MAX {
            starting_price = Some(min_discounted);
            original_price = if matching_original > min_discounted {
                Some(matching_original)
            } else {
                None
            };
        }
    }

    let city_uuid_val: Option<Uuid> = row.get("city_id");
    let related_rows = sqlx::query(
        "SELECT v.id, v.name_ar, v.name_en, v.slug, c.name_ar AS city_name_ar, c.name_en AS city_name_en
         FROM vendors v
         LEFT JOIN cities c ON v.city_id = c.id
         WHERE v.id != $1 AND v.status = 'active' AND (v.category = $2 OR v.city_id = $3)
         LIMIT 4"
    )
    .bind(id)
    .bind(&category)
    .bind(city_uuid_val)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();

    let related_vendors: Vec<Value> = related_rows
        .into_iter()
        .map(|r| {
            json!({
                "id": r.get::<Uuid, _>("id").to_string(),
                "nameAr": r.get::<String, _>("name_ar"),
                "nameEn": r.get::<String, _>("name_en"),
                "slug": r.get::<String, _>("slug"),
                "cityNameAr": r.get::<Option<String>, _>("city_name_ar").unwrap_or_default(),
                "cityNameEn": r.get::<Option<String>, _>("city_name_en").unwrap_or_default(),
            })
        })
        .collect();

    Ok(Json(json!({
        "status": "success",
        "vendor": {
            "id": id.to_string(),
            "slug": slug_db,
            "category": category,
            "rating": overall_avg,
            "ratingQuality": quality_avg,
            "ratingStaff": staff_avg,
            "ratingCommunication": communication_avg,
            "reviewsCount": review_count,
            "reviews": reviews,
            "halls": halls,
            "descriptionAr": description_ar.unwrap_or_default(),
            "descriptionEn": description_en.unwrap_or_default(),
            "images": images,
            "packages": packages.clone(),
            "offers": packages,
            "capacity": {
                "min": capacity_min.unwrap_or(0),
                "max": capacity_max.unwrap_or(1000)
            },
            "location": {
                "city": city_name_en.clone().unwrap_or_default(),
                "cityAr": city_name_ar.clone().unwrap_or_default(),
                "cityEn": city_name_en.clone().unwrap_or_default(),
                "district": district_en.clone().unwrap_or_default(),
                "districtAr": district_ar.clone().unwrap_or_default(),
                "districtEn": district_en.clone().unwrap_or_default(),
                "address": address_en.clone().unwrap_or_default(),
                "lat": latitude.unwrap_or(0.0),
                "lng": longitude.unwrap_or(0.0)
            },
            "amenities": amenities.unwrap_or_default(),
            "isFeatured": is_featured,
            "isAvailable": is_available,
            "phone": phone.unwrap_or_default(),
            "email": email.unwrap_or_default(),
            "crmVenueId": crm_venue_id,
            "starRating": star_rating,
            "eventSpacesAvailable": event_spaces_available,
            "eventType": event_type,
            "website": website,
            "mapsUrl": maps_url,
            "videoUrl1": video_url_1,
            "vendor": {
                "id": id.to_string(),
                "nameAr": coordinator_name_ar.clone().unwrap_or_default(),
                "nameEn": coordinator_name_en.clone().unwrap_or_default(),
                "rating": overall_avg,
                "venueCount": 1,
                "verified": true
            },
            "coordinator": {
                "nameAr": coordinator_name_ar.unwrap_or_default(),
                "nameEn": coordinator_name_en.unwrap_or_default(),
                "phone": coordinator_phone.unwrap_or_default(),
                "whatsapp": coordinator_whatsapp.unwrap_or_default(),
                "avatar": coordinator_avatar
            },
            "pricing": {
                "basePrice": starting_price.unwrap_or(0.0),
                "originalPrice": original_price,
                "depositPercentage": 25,
                "includedServices": []
            },
            "relatedVendors": related_vendors,
            "createdAt": created_at.to_rfc3339(),
            "updatedAt": updated_at.to_rfc3339()
        }
    })))
}







#[derive(Debug, Deserialize)]
pub struct CatalogQueryParams {
    pub category: Option<String>,
    pub city_id: Option<String>,
    pub q: Option<String>,
    pub price_min: Option<f64>,
    pub price_max: Option<f64>,
    pub guest_count: Option<i32>,
    pub rating: Option<f64>,
    pub is_featured: Option<bool>,
    pub is_verified: Option<bool>,
    pub sort_by: Option<String>,
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CatalogResponse {
    pub status: String,
    pub vendors: Vec<CatalogVendorDto>,
    pub total: usize,
    pub page: usize,
    pub total_pages: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CatalogVendorDto {
    pub id: String,
    pub slug: String,
    pub name_ar: String,
    pub name_en: String,
    pub description_ar: String,
    pub description_en: String,
    pub category: Vec<String>,
    pub cover_image: String,
    pub images: Vec<CatalogImageDto>,
    pub pricing: CatalogPricingDto,
    pub location: CatalogLocationDto,
    pub capacity: CatalogCapacityDto,
    pub amenities: Vec<String>,
    pub rating: f64,
    pub review_count: i64,
    pub reviews: Vec<CatalogReviewDto>,
    pub is_featured: bool,
    pub is_available: bool,
    pub vendor: CatalogVendorSummaryDto,
    pub created_at: String,
    pub updated_at: String,

    pub original_price: Option<f64>,
    pub starting_price: f64,
    pub has_offers: bool,
    pub is_zafaf_exclusive: bool,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CatalogImageDto {
    pub id: String,
    pub url: String,
    pub alt: String,
    pub is_primary: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CatalogPricingDto {
    pub base_price: f64,
    pub weekend_surcharge: Option<f64>,
    pub deposit_percentage: f64,
    pub included_services: Vec<String>,
    pub additional_services: Vec<CatalogAdditionalServiceDto>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CatalogAdditionalServiceDto {
    pub name: String,
    pub price: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CatalogLocationDto {
    pub city: String,
    pub city_ar: String,
    pub city_en: String,
    pub district: String,
    pub district_ar: String,
    pub district_en: String,
    pub address: String,
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CatalogCapacityDto {
    pub min: i32,
    pub max: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CatalogReviewDto {
    pub id: String,
    pub author_name: String,
    pub rating: f64,
    pub rating_quality: Option<i32>,
    pub rating_staff: Option<i32>,
    pub rating_communication: Option<i32>,
    pub wedding_date: Option<String>,
    pub date: String,
    pub comment: String,
    pub event_type: String,
    pub helpful: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CatalogVendorSummaryDto {
    pub id: String,
    pub name_ar: String,
    pub name_en: String,
    pub rating: f64,
    pub venue_count: i64,
    pub verified: bool,
}

async fn list_catalog(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<CatalogQueryParams>,
) -> Result<Json<CatalogResponse>, AppError> {
    let country_id = headers
        .get("X-Country-ID")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| "sa".to_string());

    tracing::info!(
        "Querying catalog with filters for country: {}...",
        country_id
    );

    let page = (query.page.unwrap_or(1) as i64).max(1);
    let limit = (query.limit.unwrap_or(12) as i64).max(1);
    let offset = (page - 1) * limit;

    // Stage 1: Construct and execute lightweight COUNT and ID Slicing queries in PostgreSQL
    let mut count_query_str = "
        SELECT COUNT(v.id)
        FROM vendors v
        LEFT JOIN cities c ON v.city_id = c.id
        LEFT JOIN (
            SELECT 
                vendor_id,
                AVG(rating_quality)::float as quality_avg,
                AVG(rating_staff)::float as staff_avg,
                AVG(rating_communication)::float as communication_avg
            FROM reviews
            WHERE status = 'approved'
            GROUP BY vendor_id
        ) rev ON v.id = rev.vendor_id
        WHERE v.status = 'active' AND (c.country_id IS NULL OR c.country_id = $1)
    "
    .to_string();

    let mut ids_query_str = "
        WITH filtered_vendors AS (
            SELECT 
                v.id,
                (v.is_featured = TRUE AND (v.featured_expires_at IS NULL OR v.featured_expires_at > CURRENT_TIMESTAMP)) AS is_featured,
                COALESCE((SELECT MIN(p.discounted_price::float) FROM vendor_packages p WHERE p.vendor_id = v.id AND p.expiry_date >= CURRENT_DATE), 0.0) as starting_price,
                COALESCE(ROUND(((rev.quality_avg + rev.staff_avg + rev.communication_avg) / 3.0)::numeric, 1)::float, 5.0) as overall_rating,
                v.created_at,
                t.name_en as tier_name
            FROM vendors v
            LEFT JOIN cities c ON v.city_id = c.id
            LEFT JOIN subscription_tiers t ON v.subscription_tier_id = t.id
            LEFT JOIN (
                SELECT 
                    vendor_id,
                    AVG(rating_quality)::float as quality_avg,
                    AVG(rating_staff)::float as staff_avg,
                    AVG(rating_communication)::float as communication_avg
                FROM reviews
                WHERE status = 'approved'
                GROUP BY vendor_id
            ) rev ON v.id = rev.vendor_id
            WHERE v.status = 'active' AND (c.country_id IS NULL OR c.country_id = $1)
    ".to_string();

    let mut filters = String::with_capacity(512);
    let mut param_idx = 2;
    let mut category_bind = None;
    let mut city_uuid_bind = None;
    let mut city_bind = None;
    let mut guest_count_bind = None;
    let mut price_min_bind = None;
    let mut price_max_bind = None;
    let mut rating_bind = None;
    let mut search_bind = None;

    if let Some(ref cat) = query.category {
        filters.push_str(&format!(" AND v.category = ${}", param_idx));
        param_idx += 1;
        category_bind = Some(cat.clone());
    }

    if let Some(ref city) = query.city_id {
        if let Ok(city_uuid) = Uuid::parse_str(city) {
            filters.push_str(&format!(" AND v.city_id = ${}", param_idx));
            param_idx += 1;
            city_uuid_bind = Some(city_uuid);
        } else {
            filters.push_str(&format!(
                " AND (c.slug = ${} OR v.city_id::text = ${})",
                param_idx, param_idx
            ));
            param_idx += 1;
            city_bind = Some(city.clone());
        }
    }

    if let Some(guest_count) = query.guest_count {
        filters.push_str(&format!(
            " AND v.capacity_min <= ${} AND v.capacity_max >= ${}",
            param_idx, param_idx
        ));
        param_idx += 1;
        guest_count_bind = Some(guest_count);
    }

    if let Some(price_min) = query.price_min {
        filters.push_str(&format!(" AND COALESCE((SELECT MIN(p.discounted_price::float) FROM vendor_packages p WHERE p.vendor_id = v.id AND p.expiry_date >= CURRENT_DATE), 0.0) >= ${}", param_idx));
        param_idx += 1;
        price_min_bind = Some(price_min);
    }

    if let Some(price_max) = query.price_max {
        filters.push_str(&format!(" AND COALESCE((SELECT MIN(p.discounted_price::float) FROM vendor_packages p WHERE p.vendor_id = v.id AND p.expiry_date >= CURRENT_DATE), 0.0) <= ${}", param_idx));
        param_idx += 1;
        price_max_bind = Some(price_max);
    }

    if let Some(rating) = query.rating {
        filters.push_str(&format!(" AND COALESCE(ROUND(((rev.quality_avg + rev.staff_avg + rev.communication_avg) / 3.0)::numeric, 1)::float, 5.0) >= ${}", param_idx));
        param_idx += 1;
        rating_bind = Some(rating);
    }

    let is_featured_bind = query.is_featured;
    if let Some(featured) = is_featured_bind {
        if featured {
            filters.push_str(" AND v.is_featured = TRUE AND (v.featured_expires_at IS NULL OR v.featured_expires_at > CURRENT_TIMESTAMP)");
        }
    }

    let is_verified_bind = query.is_verified;
    if let Some(verified) = is_verified_bind {
        if verified {
            filters.push_str(" AND v.is_verified = TRUE");
        }
    }

    if let Some(ref q) = query.q {
        filters.push_str(&format!(
            " AND (v.name_ar ILIKE ${0} OR v.name_en ILIKE ${0} OR v.description_ar ILIKE ${0} OR v.description_en ILIKE ${0})",
            param_idx
        ));
        param_idx += 1;
        search_bind = Some(format!("%{}%", q));
    }

    count_query_str.push_str(&filters);
    ids_query_str.push_str(&filters);

    let sort_by = query.sort_by.as_deref().unwrap_or("recommended");

    let local_order_by = match sort_by {
        "price_asc" | "priceAsc" => " ORDER BY starting_price ASC, created_at DESC",
        "price_desc" | "priceDesc" => " ORDER BY starting_price DESC, created_at DESC",
        "rating" => " ORDER BY overall_rating DESC, created_at DESC",
        "newest" => " ORDER BY created_at DESC",
        _ => " ORDER BY RANDOM()", // Use random for recommended inside tier
    };

    ids_query_str.push_str("), ");

    ids_query_str.push_str(&format!(
        "
        diamond AS (SELECT * FROM filtered_vendors WHERE tier_name = 'Enterprise' {local_order_by}),
        vip AS (SELECT * FROM filtered_vendors WHERE tier_name = 'Premium' {local_order_by}),
        gold AS (SELECT * FROM filtered_vendors WHERE tier_name = 'Basic' {local_order_by}),
        free AS (SELECT * FROM filtered_vendors WHERE tier_name = 'Free' {local_order_by}),
        unassigned AS (SELECT * FROM filtered_vendors WHERE tier_name IS NULL {local_order_by})
        SELECT id, is_featured, starting_price, overall_rating, created_at FROM (
            SELECT * FROM diamond
            UNION ALL
            SELECT * FROM vip
            UNION ALL
            SELECT * FROM gold
            UNION ALL
            SELECT * FROM free
            UNION ALL
            SELECT * FROM unassigned
        ) partitioned_results
    "
    ));

    let limit_param_idx = param_idx;
    let offset_param_idx = param_idx + 1;
    ids_query_str.push_str(&format!(
        " LIMIT ${} OFFSET ${}",
        limit_param_idx, offset_param_idx
    ));

    tracing::info!(
        target: "audit",
        action = "search_catalog",
        filters = ?filters,
        sort = %sort_by,
        "Executing UNION ALL partitioned search query"
    );

    // Execute Stage 1 COUNT
    let mut count_query = sqlx::query_scalar::<_, i64>(&count_query_str).bind(&country_id);

    if let Some(ref cat) = category_bind {
        count_query = count_query.bind(cat);
    }
    if let Some(ref city_uuid) = city_uuid_bind {
        count_query = count_query.bind(city_uuid);
    }
    if let Some(ref city) = city_bind {
        count_query = count_query.bind(city);
    }
    if let Some(guest_count) = guest_count_bind {
        count_query = count_query.bind(guest_count);
    }
    if let Some(price_min) = price_min_bind {
        count_query = count_query.bind(price_min);
    }
    if let Some(price_max) = price_max_bind {
        count_query = count_query.bind(price_max);
    }
    if let Some(rating) = rating_bind {
        count_query = count_query.bind(rating);
    }
    if let Some(ref search) = search_bind {
        count_query = count_query.bind(search);
    }

    let total_count: i64 = count_query.fetch_one(&state.db).await?;

    // Execute Stage 1 ID Slicing
    let mut ids_query = sqlx::query(&ids_query_str).bind(&country_id);

    if let Some(ref cat) = category_bind {
        ids_query = ids_query.bind(cat);
    }
    if let Some(ref city_uuid) = city_uuid_bind {
        ids_query = ids_query.bind(city_uuid);
    }
    if let Some(ref city) = city_bind {
        ids_query = ids_query.bind(city);
    }
    if let Some(guest_count) = guest_count_bind {
        ids_query = ids_query.bind(guest_count);
    }
    if let Some(price_min) = price_min_bind {
        ids_query = ids_query.bind(price_min);
    }
    if let Some(price_max) = price_max_bind {
        ids_query = ids_query.bind(price_max);
    }
    if let Some(rating) = rating_bind {
        ids_query = ids_query.bind(rating);
    }
    if let Some(ref search) = search_bind {
        ids_query = ids_query.bind(search);
    }

    ids_query = ids_query.bind(limit).bind(offset);

    let id_rows = ids_query.fetch_all(&state.db).await?;
    let vendor_ids: Vec<Uuid> = id_rows
        .into_iter()
        .map(|row| row.get::<Uuid, _>("id"))
        .collect();

    // Short-Circuit Gate
    if vendor_ids.is_empty() {
        return Ok(Json(CatalogResponse {
            status: "success".to_string(),
            vendors: vec![],
            total: 0,
            page: page as usize,
            total_pages: 0,
        }));
    }

    // Stage 2: Hydrate the exact vendor details, packages, and gallery lists targeting only vendor_ids
    let hydration_query = "
        SELECT 
            v.id,
            v.name_ar,
            v.name_en,
            v.slug,
            v.category,
            v.status,
            v.description_ar,
            v.description_en,
            v.capacity_min,
            v.capacity_max,
            v.latitude,
            v.longitude,
            v.address_ar,
            v.address_en,
            v.district_ar,
            v.district_en,
            v.amenities,
            (v.is_featured = TRUE AND (v.featured_expires_at IS NULL OR v.featured_expires_at > CURRENT_TIMESTAMP)) AS is_featured,
            v.is_available,
            v.phone,
            v.email,
            v.coordinator_name_ar,
            v.coordinator_name_en,
            v.coordinator_phone,
            v.coordinator_whatsapp,
            v.coordinator_avatar,
            v.created_at,
            v.updated_at,
            v.city_id,
            c.slug AS city_slug,
            c.name_ar AS city_name_ar,
            c.name_en AS city_name_en,
            c.country_id,
            COALESCE(rev.quality_avg, 5.0)::FLOAT8 as quality_avg,
            COALESCE(rev.staff_avg, 5.0)::FLOAT8 as staff_avg,
            COALESCE(rev.communication_avg, 5.0)::FLOAT8 as communication_avg,
            COALESCE(rev.review_count, 0) as review_count,
            COALESCE(
                (SELECT json_agg(json_build_object(
                    'id', p.id,
                    'nameAr', p.name_ar,
                    'nameEn', p.name_en,
                    'descriptionAr', p.description_ar,
                    'descriptionEn', p.description_en,
                    'originalPrice', p.original_price::float,
                    'discountedPrice', p.discounted_price::float,
                    'isZafafExclusive', p.is_zafaf_exclusive,
                    'expiryDate', p.expiry_date::text
                ))
                FROM vendor_packages p
                WHERE p.vendor_id = v.id AND p.expiry_date >= CURRENT_DATE
                ), '[]'::json
            ) as packages,
            COALESCE(
                (SELECT json_agg(json_build_object(
                    'id', vg.id,
                    'url', vg.image_url,
                    'alt', COALESCE(vg.caption, ''),
                    'isPrimary', vg.is_cover
                ))
                FROM vendor_gallery vg
                WHERE vg.vendor_id = v.id
                ), '[]'::json
            ) as images,
            COALESCE(
                (SELECT json_agg(json_build_object(
                    'id', r.id,
                    'authorName', r.author_name,
                    'rating', ((r.rating_quality + r.rating_staff + r.rating_communication) / 3.0),
                    'ratingQuality', r.rating_quality,
                    'ratingStaff', r.rating_staff,
                    'ratingCommunication', r.rating_communication,
                    'weddingDate', r.wedding_date::text,
                    'date', r.created_at::text,
                    'comment', r.comment,
                    'eventType', v.category,
                    'helpful', 0
                ))
                FROM reviews r
                WHERE r.vendor_id = v.id AND r.status = 'approved'
                ), '[]'::json
            ) as reviews
        FROM vendors v
        LEFT JOIN cities c ON v.city_id = c.id
        LEFT JOIN (
            SELECT 
                vendor_id,
                AVG(rating_quality)::float as quality_avg,
                AVG(rating_staff)::float as staff_avg,
                AVG(rating_communication)::float as communication_avg,
                COUNT(id)::bigint as review_count
            FROM reviews
            WHERE status = 'approved'
            GROUP BY vendor_id
        ) rev ON v.id = rev.vendor_id
        WHERE v.id = ANY($1::uuid[])
    ";

    let hydrated_rows = sqlx::query(hydration_query)
        .bind(&vendor_ids)
        .fetch_all(&state.db)
        .await?;

    let mut vendors_map = std::collections::HashMap::with_capacity(hydrated_rows.len());

    for row in hydrated_rows {
        let id: Uuid = row.get("id");
        let name_ar: String = row.get("name_ar");
        let name_en: String = row.get("name_en");
        let slug: String = row.get("slug");
        let category: Option<String> = row.get("category");
        let category = category.unwrap_or_default();

        let description_ar: Option<String> = row.get("description_ar");
        let description_en: Option<String> = row.get("description_en");

        let capacity_min: Option<i32> = row.get("capacity_min");
        let capacity_max: Option<i32> = row.get("capacity_max");
        let latitude: Option<f64> = row.get("latitude");
        let longitude: Option<f64> = row.get("longitude");
        let address_en: Option<String> = row.get("address_en");
        let district_ar: Option<String> = row.get("district_ar");
        let district_en: Option<String> = row.get("district_en");
        let amenities: Option<Vec<String>> = row.get("amenities");
        let is_featured: bool = row.get("is_featured");
        let is_available: bool = row.get("is_available");

        let coordinator_name_ar: Option<String> = row.get("coordinator_name_ar");
        let coordinator_name_en: Option<String> = row.get("coordinator_name_en");

        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

        let city_name_ar: Option<String> = row.get("city_name_ar");
        let city_name_en: Option<String> = row.get("city_name_en");

        let quality_avg: f64 = row.get("quality_avg");
        let staff_avg: f64 = row.get("staff_avg");
        let communication_avg: f64 = row.get("communication_avg");
        let review_count: i64 = row.get("review_count");

        let overall_avg = (quality_avg + staff_avg + communication_avg) / 3.0;
        let overall_avg = (overall_avg * 10.0).round() / 10.0;

        let packages_val: serde_json::Value = row.get("packages");
        let packages: Vec<serde_json::Value> = packages_val.as_array().cloned().unwrap_or_default();

        let mut starting_price: Option<f64> = None;
        let mut original_price: Option<f64> = None;
        let mut is_zafaf_exclusive = false;
        let mut has_offers = false;

        if !packages.is_empty() {
            has_offers = true;
            let mut min_discounted = f64::MAX;
            let mut matching_original = 0.0;

            for pkg in &packages {
                let disc = pkg
                    .get("discountedPrice")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let orig = pkg
                    .get("originalPrice")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let excl = pkg
                    .get("isZafafExclusive")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                if disc > 0.0 && disc < min_discounted {
                    min_discounted = disc;
                    matching_original = orig;
                }
                if excl {
                    is_zafaf_exclusive = true;
                }
            }

            if min_discounted < f64::MAX {
                starting_price = Some(min_discounted);
                original_price = if matching_original > min_discounted {
                    Some(matching_original)
                } else {
                    None
                };
            }
        }

        let images_val: serde_json::Value = row.get("images");
        let images: Vec<CatalogImageDto> = serde_json::from_value(images_val).unwrap_or_default();

        let reviews_val: serde_json::Value = row.get("reviews");
        let reviews: Vec<CatalogReviewDto> =
            serde_json::from_value(reviews_val).unwrap_or_default();

        let vendor_dto = CatalogVendorDto {
            id: id.to_string(),
            slug,
            name_ar,
            name_en,
            description_ar: description_ar.unwrap_or_default(),
            description_en: description_en.unwrap_or_default(),
            category: vec![category.clone()],
            cover_image: images
                .iter()
                .find(|img| img.is_primary)
                .map(|img| img.url.clone())
                .unwrap_or_else(|| "/images/fallbacks/default-cover.svg".to_string()),
            images,
            pricing: CatalogPricingDto {
                base_price: starting_price.unwrap_or(0.0),
                weekend_surcharge: None,
                deposit_percentage: 25.0,
                included_services: vec![],
                additional_services: vec![],
            },
            location: CatalogLocationDto {
                city: city_name_en.clone().unwrap_or_default(),
                city_ar: city_name_ar.clone().unwrap_or_default(),
                city_en: city_name_en.clone().unwrap_or_default(),
                district: district_en.clone().unwrap_or_default(),
                district_ar: district_ar.unwrap_or_default(),
                district_en: district_en.clone().unwrap_or_default(),
                address: address_en.unwrap_or_default(),
                lat: latitude.unwrap_or(0.0),
                lng: longitude.unwrap_or(0.0),
            },
            capacity: CatalogCapacityDto {
                min: capacity_min.unwrap_or(0),
                max: capacity_max.unwrap_or(1000),
            },
            amenities: amenities.unwrap_or_default(),
            rating: overall_avg,
            review_count,
            reviews,
            is_featured,
            is_available,
            vendor: CatalogVendorSummaryDto {
                id: id.to_string(),
                name_ar: coordinator_name_ar.unwrap_or_default(),
                name_en: coordinator_name_en.unwrap_or_default(),
                rating: overall_avg,
                venue_count: 1,
                verified: true,
            },
            created_at: created_at.to_rfc3339(),
            updated_at: updated_at.to_rfc3339(),
            original_price,
            starting_price: starting_price.unwrap_or(0.0),
            has_offers,
            is_zafaf_exclusive,
            currency: if country_id == "eg" {
                "EGP".to_string()
            } else if country_id == "ae" {
                "AED".to_string()
            } else {
                "SAR".to_string()
            },
        };

        vendors_map.insert(id, vendor_dto);
    }

    let mut sorted_vendors = Vec::new();
    for id in &vendor_ids {
        if let Some(vendor) = vendors_map.remove(id) {
            sorted_vendors.push(vendor);
        }
    }

    let total_pages = ((total_count + limit - 1) / limit) as usize;

    Ok(Json(CatalogResponse {
        status: "success".to_string(),
        vendors: sorted_vendors,
        total: total_count as usize,
        page: page as usize,
        total_pages,
    }))
}


#[derive(serde::Deserialize)]
pub struct ListingsQuery {
    pub category: Option<String>,
    pub city: Option<String>,
    pub city_id: Option<String>,
    pub gender: Option<String>,
    pub min_capacity: Option<i32>,
    pub max_capacity: Option<i32>,
    pub price_min: Option<f64>,
    pub price_max: Option<f64>,
    pub amenities: Option<String>,
    pub featured: Option<bool>,
    pub tier: Option<String>,
    pub sort: Option<String>,
    pub country_id: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

fn resolve_category_slugs(cat: &str) -> Vec<String> {
    let equivalents: &[&str] = match cat {
        "wedding-sweets" => &[
            "wedding-sweets",           // V2
            "wedding-gifts",            // V2 alt
        ],
        "wedding-flowers-and-bouquets" => &[
            "wedding-flowers-and-bouquets", // legacy
            "flowers-floral",               // V2
        ],
        // ── Any unrecognised slug passes through as-is ─────────────────────
        other => return vec![other.to_string()],
    };
    equivalents.iter().map(|s| s.to_string()).collect()
}

async fn list_listings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListingsQuery>,
) -> Result<Json<Value>, AppError> {
    let country_id = headers
        .get("X-Country-ID")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_lowercase())
        .or_else(|| query.country_id.clone())
        .unwrap_or_else(|| "sa".to_string());

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(12).clamp(1, 48);
    let offset = (page - 1) * limit;

    tracing::info!(
        "Listing-centric catalog query: country={}, page={}",
        country_id,
        page
    );

    // ── Build the WHERE clauses dynamically ──────────────────────────────────
    let mut conditions: Vec<String> = vec![
        "vp.status = 'active'".to_string(),
        "vp.is_available = TRUE".to_string(),
        "v.status = 'active'".to_string(),
        "(c.country_id IS NULL OR c.country_id = $1)".to_string(),
    ];
    let mut param_idx: usize = 2;
    let mut category_slugs_val: Option<Vec<String>> = None;
    let mut city_val: Option<String> = None;
    let mut city_id_val: Option<String> = None;
    let mut gender_val: Option<String> = None;
    let mut min_cap_val: Option<i32> = None;
    let mut max_cap_val: Option<i32> = None;
    let mut price_min_val: Option<f64> = None;
    let mut price_max_val: Option<f64> = None;
    let mut amenities_val: Option<Vec<String>> = None;
    let mut featured_val: Option<bool> = None;
    let mut tier_val: Option<String> = None;

    if let Some(ref cat) = query.category {
        // Bridge legacy client slugs to all equivalent DB values (legacy + V2).
        // This allows the client to keep using stable legacy URL slugs while the
        // DB contains a mix of old and new listings.
        let slugs = resolve_category_slugs(cat);
        conditions.push(format!("vp.product_category = ANY(${})", param_idx));
        param_idx += 1;
        category_slugs_val = Some(slugs);
    }

    if let Some(ref city_slug) = query.city {
        conditions.push(format!("c.slug = ${}", param_idx));
        param_idx += 1;
        city_val = Some(city_slug.clone());
    }

    if let Some(ref cid) = query.city_id {
        if Uuid::parse_str(cid).is_ok() {
            conditions.push(format!("vp.city_id = ${}", param_idx));
        } else {
            conditions.push(format!("c.slug = ${}", param_idx));
        }
        param_idx += 1;
        city_id_val = Some(cid.clone());
    }

    if let Some(ref g) = query.gender {
        // Use the dedicated indexed gender_section column (not JSONB)
        conditions.push(format!("vp.gender_section = ${}", param_idx));
        param_idx += 1;
        gender_val = Some(g.clone());
    }

    if let Some(min) = query.min_capacity {
        // Use the indexed total_capacity column (not JSONB path expression)
        conditions.push(format!("vp.total_capacity >= ${}", param_idx));
        param_idx += 1;
        min_cap_val = Some(min);
    }

    if let Some(max) = query.max_capacity {
        conditions.push(format!(
            "(vp.total_capacity IS NULL OR vp.total_capacity <= ${})",
            param_idx
        ));
        param_idx += 1;
        max_cap_val = Some(max);
    }

    if let Some(pmin) = query.price_min {
        conditions.push(format!("vp.base_price_sar >= ${}", param_idx));
        param_idx += 1;
        price_min_val = Some(pmin);
    }

    if let Some(pmax) = query.price_max {
        conditions.push(format!("vp.base_price_sar <= ${}", param_idx));
        param_idx += 1;
        price_max_val = Some(pmax);
    }

    if let Some(ref amenities_str) = query.amenities {
        let list: Vec<String> = amenities_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !list.is_empty() {
            // Use the GIN-indexed searchable_amenities TEXT[] column
            conditions.push(format!("vp.searchable_amenities @> ${}", param_idx));
            param_idx += 1;
            amenities_val = Some(list);
        }
    }

    if let Some(feat) = query.featured {
        if feat {
            conditions.push("(vp.is_featured = TRUE OR (v.is_featured = TRUE AND (v.featured_expires_at IS NULL OR v.featured_expires_at > CURRENT_TIMESTAMP)))".to_string());
        }
        featured_val = Some(feat);
    }

    if let Some(ref t) = query.tier {
        conditions.push(format!("st.name ILIKE ${} AND (v.subscription_expires_at IS NULL OR v.subscription_expires_at > CURRENT_TIMESTAMP)", param_idx));
        param_idx += 1;
        tier_val = Some(t.clone());
    }

    let _ = featured_val; // suppress unused warning if featured not used in bind

    let where_clause = conditions.join(" AND ");

    let order_clause = match query.sort.as_deref() {
        Some("price_asc")  => "vp.base_price_sar ASC NULLS LAST",
        Some("price_desc") => "vp.base_price_sar DESC NULLS LAST",
        Some("rating")     => "overall_rating DESC NULLS LAST",
        Some("newest")     => "vp.created_at DESC",
        _                  => "tier_priority_score DESC, RANDOM()",
    };

    // ── COUNT query (for pagination) ──────────────────────────────────────────
    let count_sql = format!(
        "SELECT COUNT(*) FROM vendor_products vp
         JOIN vendors v ON vp.vendor_id = v.id
         LEFT JOIN cities c ON COALESCE(v.city_id, vp.city_id) = c.id
         LEFT JOIN subscription_tiers st ON v.subscription_tier_id = st.id
         WHERE {}",
        where_clause
    );

    // ── Main data query ───────────────────────────────────────────────────────
    let data_sql = format!(
        "SELECT
            vp.id,
            vp.slug,
            -- V2 bilingual columns (fall back to legacy title if not set)
            COALESCE(vp.title_en, vp.title, '') AS title_en,
            COALESCE(vp.title_ar, vp.title, '') AS title_ar,
            COALESCE(vp.description_en, vp.description) AS description_en,
            COALESCE(vp.description_ar, vp.description) AS description_ar,
            -- Legacy (kept for backward compat)
            COALESCE(vp.title, vp.title_en, '') AS title,
            vp.description,
            vp.product_category,
            vp.attributes,
            vp.base_price_sar,
            vp.deposit_percentage,
            vp.price_on_inquiry,
            -- V2 structured columns
            vp.gender_section,
            vp.total_capacity,
            vp.quality_score,
            (vp.is_featured = TRUE AND (vp.featured_until IS NULL OR vp.featured_until > CURRENT_TIMESTAMP)) AS listing_featured,
            vp.is_available,
            vp.created_at,
            vp.google_maps_url,
            vp.latitude,
            vp.longitude,
            -- Vendor brand summary
            v.id AS vendor_id,
            v.slug AS vendor_slug,
            v.name_ar AS vendor_name_ar,
            v.name_en AS vendor_name_en,
            v.phone AS vendor_phone,
            v.email AS vendor_email,
            v.maps_url AS vendor_maps_url,
            v.website AS vendor_website,
            (v.is_featured = TRUE AND (v.featured_expires_at IS NULL OR v.featured_expires_at > CURRENT_TIMESTAMP)) AS vendor_featured,
            -- City
            v.city_id,
            c.slug AS city_slug,
            c.name_ar AS city_name_ar,
            c.name_en AS city_name_en,
            c.country_id,
            -- Rating (vendor-level reviews)
            COALESCE(rev.quality_avg, 5.0)::FLOAT8 AS quality_avg,
            COALESCE(rev.staff_avg, 5.0)::FLOAT8 AS staff_avg,
            COALESCE(rev.comm_avg, 5.0)::FLOAT8 AS comm_avg,
            COALESCE(rev.review_count, 0)  AS review_count,
            ((COALESCE(rev.quality_avg, 5.0)::FLOAT8 + COALESCE(rev.staff_avg, 5.0)::FLOAT8 + COALESCE(rev.comm_avg, 5.0)::FLOAT8) / 3.0) AS overall_rating,
            -- Cover image: listing-specific first, vendor fallback
            COALESCE(
                (SELECT image_url FROM vendor_gallery WHERE product_id = vp.id AND is_cover = TRUE LIMIT 1),
                (SELECT image_url FROM vendor_gallery WHERE product_id = vp.id LIMIT 1),
                (SELECT image_url FROM vendor_gallery WHERE vendor_id = v.id AND is_cover = TRUE LIMIT 1),
                (SELECT image_url FROM vendor_gallery WHERE vendor_id = v.id LIMIT 1)
            ) AS cover_image,
            -- Image count for the card badge
            (SELECT COUNT(*) FROM vendor_gallery WHERE product_id = vp.id)::int AS listing_image_count,
            -- Subscription Tier Validation & Extraction
            CASE
                WHEN v.subscription_expires_at IS NULL OR v.subscription_expires_at > CURRENT_TIMESTAMP THEN COALESCE(st.name, 'Free')
                ELSE 'Free'
            END AS tier_name,
            CASE
                WHEN v.subscription_expires_at IS NULL OR v.subscription_expires_at > CURRENT_TIMESTAMP THEN COALESCE(st.priority_score, 25)
                ELSE 25
            END AS tier_priority_score
         FROM vendor_products vp
         JOIN vendors v ON vp.vendor_id = v.id
         LEFT JOIN cities c ON COALESCE(v.city_id, vp.city_id) = c.id
         LEFT JOIN subscription_tiers st ON v.subscription_tier_id = st.id
         LEFT JOIN (
            SELECT
                vendor_id,
                AVG(rating_quality)::float AS quality_avg,
                AVG(rating_staff)::float   AS staff_avg,
                AVG(rating_communication)::float AS comm_avg,
                COUNT(id)::bigint          AS review_count
            FROM reviews
            WHERE status = 'approved'
            GROUP BY vendor_id
         ) rev ON v.id = rev.vendor_id
         WHERE {}
         ORDER BY {}
         LIMIT ${}  OFFSET ${}",
        where_clause, order_clause, param_idx, param_idx + 1
    );

    // ── Bind parameters (both count and data queries share same bind chain) ──
    macro_rules! bind_params {
        ($q:expr) => {{
            let mut q = $q.bind(&country_id);
            if let Some(ref v) = category_slugs_val {
                // Bind as TEXT[] — matches the ANY($N) condition
                q = q.bind(v.as_slice());
            }
            if let Some(ref v) = city_val {
                q = q.bind(v);
            }
            if let Some(ref v) = city_id_val {
                if let Ok(parsed_uuid) = Uuid::parse_str(v) {
                    q = q.bind(parsed_uuid);
                } else {
                    q = q.bind(v);
                }
            }
            if let Some(ref v) = gender_val {
                q = q.bind(v);
            }
            if let Some(v) = min_cap_val {
                q = q.bind(v);
            }
            if let Some(v) = max_cap_val {
                q = q.bind(v);
            }
            if let Some(v) = price_min_val {
                q = q.bind(v);
            }
            if let Some(v) = price_max_val {
                q = q.bind(v);
            }
            if let Some(ref v) = amenities_val {
                q = q.bind(v);
            }
            if let Some(ref v) = tier_val {
                q = q.bind(v);
            }
            q
        }};
    }

    let count_row = bind_params!(sqlx::query(&count_sql))
        .fetch_one(&state.db)
        .await?;
    let total: i64 = count_row.get(0);
    let total_pages = (total as f64 / limit as f64).ceil() as i64;

    let rows = bind_params!(sqlx::query(&data_sql))
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.db)
        .await?;

    let mut listings_json: Vec<serde_json::Value> = Vec::new();

    for row in rows {
        let listing_id: Uuid = row.get("id");
        let slug: String = row.get("slug");
        // V2 bilingual fields
        let title_en: String = row.try_get("title_en").unwrap_or_else(|_| row.get("title"));
        let title_ar: String = row.try_get("title_ar").unwrap_or_else(|_| title_en.clone());
        let description_en: Option<String> = row.try_get("description_en").ok().flatten();
        let description_ar: Option<String> = row.try_get("description_ar").ok().flatten();
        // Legacy fallback
        let title: String = title_en.clone();
        let description: Option<String> = row.try_get("description").ok().flatten();
        let category: String = row.get("product_category");
        let attributes: serde_json::Value = row.get("attributes");
        let base_price: Option<Decimal> = row.try_get("base_price_sar").ok().flatten();
        let deposit_pct: i32 = row.get("deposit_percentage");
        let price_on_inquiry: Option<bool> = row.try_get("price_on_inquiry").ok().flatten();
        let gender_section: Option<String> = row.try_get("gender_section").ok().flatten();
        let total_capacity: Option<i32> = row.try_get("total_capacity").ok().flatten();
        let quality_score: Option<i32> = row.try_get("quality_score").ok().flatten();
        let listing_feat: bool = row.get("listing_featured");
        let is_available: bool = row.get("is_available");
        let image_count: i32 = row.get("listing_image_count");
        let cover_image: Option<String> = row.get("cover_image");
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        let google_maps_url: Option<String> = row.try_get("google_maps_url").ok().flatten();
        let latitude: Option<f64> = row.try_get("latitude").ok().flatten();
        let longitude: Option<f64> = row.try_get("longitude").ok().flatten();

        // Vendor brand
        let vendor_id: Uuid = row.get("vendor_id");
        let vendor_slug: String = row.get("vendor_slug");
        let vendor_name_ar: String = row.get("vendor_name_ar");
        let vendor_name_en: String = row.get("vendor_name_en");
        let vendor_phone: Option<String> = row.get("vendor_phone");
        let vendor_email: Option<String> = row.get("vendor_email");
        let vendor_maps: Option<String> = row.get("vendor_maps_url");
        let vendor_website: Option<String> = row.get("vendor_website");
        let vendor_feat: bool = row.get("vendor_featured");

        // Location
        let city_slug: Option<String> = row.get("city_slug");
        let city_name_ar: Option<String> = row.get("city_name_ar");
        let city_name_en: Option<String> = row.get("city_name_en");
        let country_code: Option<String> = row.get("country_id");

        // Ratings
        let quality_avg: f64 = row.get("quality_avg");
        let staff_avg: f64 = row.get("staff_avg");
        let comm_avg: f64 = row.get("comm_avg");
        let review_count: i64 = row.get("review_count");
        let overall_rating: f64 =
            (((quality_avg + staff_avg + comm_avg) / 3.0) * 10.0).round() / 10.0;

        let is_featured = listing_feat || vendor_feat;
        let currency = match country_code.as_deref() {
            Some("eg") => "EGP",
            Some("ae") => "AED",
            _ => "SAR",
        };

        let tier_name: String = row
            .try_get("tier_name")
            .unwrap_or_else(|_| "Free".to_string());
        let subscription_badge = if tier_name.eq_ignore_ascii_case("free") {
            None
        } else {
            Some(json!({
                "tierId": tier_name.to_lowercase(),
                "tierName": tier_name
            }))
        };

        listings_json.push(json!({
            // Core listing identity
            "id":               listing_id.to_string(),
            "slug":             slug,
            // V2 bilingual fields
            "titleEn":          title_en,
            "titleAr":          title_ar,
            "descriptionEn":    description_en.or_else(|| description.clone()),
            "descriptionAr":    description_ar.or(description),
            // Legacy compat
            "title":            title,
            "category":         category,
            "attributes":       attributes,
            // V2 structured fields
            "genderSection":    gender_section,
            "totalCapacity":    total_capacity,
            "qualityScore":     quality_score.unwrap_or(0),
            "googleMapsUrl":    google_maps_url,
            "latitude":         latitude,
            "longitude":        longitude,

            // Pricing (per-listing)
            "basePriceSar":     base_price.map(|p| p.to_string()),
            "depositPercentage": deposit_pct,
            "priceOnInquiry":   price_on_inquiry.unwrap_or(false),
            "currency":         currency,

            // Images
            "coverImage":       cover_image.unwrap_or_else(|| "/images/fallbacks/default-cover.svg".to_string()),
            "imageCount":       image_count,

            // Vendor brand summary
            "vendor": {
                "id":        vendor_id.to_string(),
                "slug":      vendor_slug,
                "nameAr":    vendor_name_ar,
                "nameEn":    vendor_name_en,
                "phone":     vendor_phone,
                "email":     vendor_email,
                "mapsUrl":   vendor_maps,
                "website":   vendor_website
            },

            // Location
            "citySlug":      city_slug,
            "cityAr":        city_name_ar,
            "cityEn":        city_name_en,
            "countryCode":   country_code,

            // Ratings
            "rating": {
                "overall":       overall_rating,
                "quality":       quality_avg,
                "staff":         staff_avg,
                "communication": comm_avg,
                "count":         review_count
            },

            "isFeatured":    is_featured,
            "isAvailable":   is_available,
            "subscriptionBadge": subscription_badge,
            "createdAt":     created_at.to_rfc3339(),

            "detailUrl":     format!("/listings/{}", slug),
            "bookingUrl":    format!("/booking/listing/{}", listing_id)
        }));
    }

    Ok(Json(json!({
        "status":     "success",
        "listings":   listings_json,
        "total":      total,
        "page":       page,
        "limit":      limit,
        "totalPages": total_pages,
    })))
}

/// GET /api/v1/public/listings/:slug
/// Returns a single listing detail page response. The slug is the globally unique
/// vendor_products.slug (enforced by migration 007).
async fn get_listing_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Value>, AppError> {
    tracing::info!(
        "=== [DEBUG] Received slug in get_listing_by_slug: '{}' ===",
        slug
    );

    let row = sqlx::query(
        "SELECT
            vp.id,
            vp.slug,
            vp.title,
            vp.description,
            vp.title_ar,
            vp.title_en,
            vp.description_ar,
            vp.description_en,
            vp.meta_title_ar,
            vp.meta_title_en,
            vp.meta_description_ar,
            vp.meta_description_en,
            vp.product_category,
            vp.attributes,
            vp.base_price_sar,
            vp.deposit_percentage,
            vp.price_on_inquiry,
            vp.gender_section,
            vp.features_selection,
            vp.coordinator_name_ar AS prod_coord_name_ar,
            vp.coordinator_name_en AS prod_coord_name_en,
            vp.coordinator_phone   AS prod_coord_phone,
            vp.coordinator_whatsapp AS prod_coord_whatsapp,
            vp.coordinator_email   AS prod_coord_email,
            vp.coordinator_mobile  AS prod_coord_mobile,
            vp.coordinator_avatar AS prod_coord_avatar,
            (vp.is_featured = TRUE AND (vp.featured_until IS NULL OR vp.featured_until > CURRENT_TIMESTAMP)) AS listing_featured,
            vp.is_available,
            vp.created_at,
            vp.updated_at,
            vp.google_maps_url,
            vp.latitude,
            vp.longitude,
            -- Vendor brand
            v.id          AS vendor_id,
            v.slug        AS vendor_slug,
            v.name_ar     AS vendor_name_ar,
            v.name_en     AS vendor_name_en,
            v.description_ar AS vendor_desc_ar,
            v.description_en AS vendor_desc_en,
            v.phone       AS vendor_phone,
            v.email       AS vendor_email,
            v.website     AS vendor_website,
            v.maps_url    AS vendor_maps_url,
            v.video_url_1 AS vendor_video_url,
            v.latitude    AS vendor_lat,
            v.longitude   AS vendor_lng,
            v.address_ar  AS vendor_address_ar,
            v.address_en  AS vendor_address_en,
            NULL          AS vendor_district_ar,
            NULL          AS vendor_district_en,
            v.star_rating AS vendor_star_rating,
            (v.is_featured = TRUE AND (v.featured_expires_at IS NULL OR v.featured_expires_at > CURRENT_TIMESTAMP)) AS vendor_featured,
            -- City
            c.slug        AS city_slug,
            c.name_ar     AS city_name_ar,
            c.name_en     AS city_name_en,
            c.country_id,
            -- Ratings (vendor-level)
            COALESCE(rev.quality_avg, 5.0)::FLOAT8 AS quality_avg,
            COALESCE(rev.staff_avg, 5.0)::FLOAT8 AS staff_avg,
            COALESCE(rev.comm_avg, 5.0)::FLOAT8 AS comm_avg,
            COALESCE(rev.review_count, 0)  AS review_count,
            -- Listing images (product-specific gallery)
            COALESCE(
                (SELECT json_agg(json_build_object(
                    'id',              vg.id,
                    'url',             vg.image_url,
                    'alt',             COALESCE(vg.caption, ''),
                    'isPrimary',       vg.is_cover,
                    'mediaType',       vg.media_type,
                    'fileUrl',         vg.file_url,
                    'thumbnailUrl',    vg.thumbnail_url,
                    'fileSize',        vg.file_size,
                    'durationSeconds', vg.duration_seconds
                ) ORDER BY vg.is_cover DESC, vg.created_at ASC)
                FROM vendor_gallery vg WHERE vg.product_id = vp.id),
                '[]'::json
            ) AS images,
            -- Vendor reviews
            COALESCE(
                (SELECT json_agg(json_build_object(
                    'id',                 r.id,
                    'authorName',         r.author_name,
                    'rating',             ((r.rating_quality + r.rating_staff + r.rating_communication)::float / 3.0),
                    'ratingQuality',      r.rating_quality,
                    'ratingStaff',        r.rating_staff,
                    'ratingCommunication',r.rating_communication,
                    'weddingDate',        r.wedding_date::text,
                    'date',               r.created_at::text,
                    'comment',            r.comment
                ) ORDER BY r.created_at DESC)
                FROM reviews r WHERE r.vendor_id = v.id AND r.status = 'approved'),
                '[]'::json
            ) AS reviews,
            -- Active packages for this listing or vendor
            COALESCE(
                (SELECT json_agg(json_build_object(
                    'id',               p.id,
                    'nameAr',           p.name_ar,
                    'nameEn',           p.name_en,
                    'originalPrice',    p.original_price::float,
                    'discountedPrice',  p.discounted_price::float,
                    'isZafafExclusive', p.is_zafaf_exclusive,
                    'expiryDate',       p.expiry_date::text
                ))
                FROM vendor_packages p
                WHERE (p.product_id = vp.id OR (p.product_id IS NULL AND p.vendor_id = v.id))
                  AND p.expiry_date >= CURRENT_DATE),
                '[]'::json
            ) AS packages,
            -- All OTHER active listings from the same vendor (siblings)
            COALESCE(
                (SELECT json_agg(json_build_object(
                    'id',       sibling.id,
                    'slug',     sibling.slug,
                    'nameAr',   sibling.title,
                    'nameEn',   sibling.title,
                    'category', sibling.product_category,
                    'coverImage', (SELECT image_url FROM vendor_gallery WHERE product_id = sibling.id AND is_cover = TRUE LIMIT 1)
                ))
                FROM vendor_products sibling
                WHERE sibling.vendor_id = vp.vendor_id
                  AND sibling.id != vp.id
                  AND sibling.status = 'active'),
                '[]'::json
            ) AS sibling_listings
         FROM vendor_products vp
         JOIN vendors v ON vp.vendor_id = v.id
         LEFT JOIN cities c ON COALESCE(v.city_id, vp.city_id) = c.id
         LEFT JOIN (
             SELECT
                 vendor_id,
                 AVG(rating_quality)::float AS quality_avg,
                 AVG(rating_staff)::float   AS staff_avg,
                 AVG(rating_communication)::float AS comm_avg,
                 COUNT(id)::bigint          AS review_count
             FROM reviews
             WHERE status = 'approved'
             GROUP BY vendor_id
         ) rev ON v.id = rev.vendor_id
         WHERE (vp.slug = $1 OR vp.id::text = $1)
           AND vp.status = 'active'
           AND v.status = 'active'"
    )
    .bind(&slug)
    .fetch_optional(&state.db)
    .await?;

    let row = match row {
        Some(r) => r,
        None => return Err(AppError::NotFound("Listing not found".to_string())),
    };

    let listing_id: Uuid = row.get("id");
    let slug_db: String = row.get("slug");
    let title: String = row.get::<Option<String>, _>("title").unwrap_or_default();
    let description: Option<String> = row.get("description");
    let title_ar: Option<String> = row.get("title_ar");
    let title_en: Option<String> = row.get("title_en");
    let description_ar: Option<String> = row.get("description_ar");
    let description_en: Option<String> = row.get("description_en");
    let meta_title_ar: Option<String> = row.get("meta_title_ar");
    let meta_title_en: Option<String> = row.get("meta_title_en");
    let meta_description_ar: Option<String> = row.get("meta_description_ar");
    let meta_description_en: Option<String> = row.get("meta_description_en");
    let category: String = row.get("product_category");
    let attributes_val: serde_json::Value = row.get("attributes");
    let base_price: Option<Decimal> = row.try_get("base_price_sar").ok().flatten();
    let deposit_pct: i32 = row.get("deposit_percentage");
    let price_on_inquiry: Option<bool> = row.try_get("price_on_inquiry").ok();
    let gender_section: Option<String> = row.try_get("gender_section").ok().flatten();
    let features_selection: serde_json::Value = row
        .try_get("features_selection")
        .unwrap_or_else(|_| json!({}));

    let prod_coord_name_ar: Option<String> = row.try_get("prod_coord_name_ar").ok().flatten();
    let prod_coord_name_en: Option<String> = row.try_get("prod_coord_name_en").ok().flatten();
    let prod_coord_phone: Option<String> = row.try_get("prod_coord_phone").ok().flatten();
    let prod_coord_whatsapp: Option<String> = row.try_get("prod_coord_whatsapp").ok().flatten();
    let prod_coord_email: Option<String> = row.try_get("prod_coord_email").ok().flatten();
    let prod_coord_mobile: Option<String> = row.try_get("prod_coord_mobile").ok().flatten();
    let prod_coord_avatar: Option<String> = row.try_get("prod_coord_avatar").ok().flatten();

    let listing_feat: bool = row.get("listing_featured");
    let is_available: bool = row.get("is_available");
    let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
    let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");
    let google_maps_url: Option<String> = row.get("google_maps_url");
    let latitude: Option<f64> = row.get("latitude");
    let longitude: Option<f64> = row.get("longitude");

    // Vendor brand
    let vendor_id: Uuid = row.get("vendor_id");
    let vendor_slug: String = row.get("vendor_slug");
    let vendor_name_ar: String = row.get("vendor_name_ar");
    let vendor_name_en: String = row.get("vendor_name_en");
    let vendor_desc_ar: Option<String> = row.get("vendor_desc_ar");
    let vendor_desc_en: Option<String> = row.get("vendor_desc_en");
    let vendor_phone: Option<String> = row.get("vendor_phone");
    let vendor_email: Option<String> = row.get("vendor_email");
    let vendor_website: Option<String> = row.get("vendor_website");
    let vendor_maps: Option<String> = row.get("vendor_maps_url");
    let vendor_video: Option<String> = row.get("vendor_video_url");
    let vendor_lat: Option<f64> = row.get("vendor_lat");
    let vendor_lng: Option<f64> = row.get("vendor_lng");
    let vendor_addr_ar: Option<String> = row.get("vendor_address_ar");
    let vendor_addr_en: Option<String> = row.get("vendor_address_en");
    let vendor_dist_ar: Option<String> = row.get("vendor_district_ar");
    let vendor_dist_en: Option<String> = row.get("vendor_district_en");
    let vendor_stars: Option<Decimal> = row.try_get("vendor_star_rating").ok().flatten();
    let vendor_feat: bool = row.get("vendor_featured");

    // Location
    let city_slug: Option<String> = row.get("city_slug");
    let city_name_ar: Option<String> = row.get("city_name_ar");
    let city_name_en: Option<String> = row.get("city_name_en");
    let country_code: Option<String> = row.get("country_id");

    // Ratings
    let quality_avg: f64 = row.get("quality_avg");
    let staff_avg: f64 = row.get("staff_avg");
    let comm_avg: f64 = row.get("comm_avg");
    let review_count: i64 = row.get("review_count");
    let overall_rating = ((quality_avg + staff_avg + comm_avg) / 3.0 * 10.0).round() / 10.0;

    // JSON sub-objects
    let images_val: serde_json::Value = row.get("images");
    let reviews_val: serde_json::Value = row.get("reviews");
    let packages_val: serde_json::Value = row.get("packages");
    let siblings_val: serde_json::Value = row.get("sibling_listings");

    let images: Vec<serde_json::Value> = images_val.as_array().cloned().unwrap_or_default();
    let reviews: Vec<serde_json::Value> = reviews_val.as_array().cloned().unwrap_or_default();
    let packages: Vec<serde_json::Value> = packages_val.as_array().cloned().unwrap_or_default();
    let siblings: Vec<serde_json::Value> = siblings_val.as_array().cloned().unwrap_or_default();

    let is_featured = listing_feat || vendor_feat;
    let currency = match country_code.as_deref() {
        Some("eg") => "EGP",
        Some("ae") => "AED",
        _ => "SAR",
    };

    // Compute starting price from packages if listing has no base_price
    let mut starting_price_display: Option<f64> =
        base_price.map(|p| p.to_string().parse::<f64>().unwrap_or(0.0));
    if starting_price_display.is_none() && !packages.is_empty() {
        let mut min_pkg = f64::MAX;
        for pkg in &packages {
            let d = pkg
                .get("discountedPrice")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            if d > 0.0 && d < min_pkg {
                min_pkg = d;
            }
        }
        if min_pkg < f64::MAX {
            starting_price_display = Some(min_pkg);
        }
    }

    let cover_image = images
        .iter()
        .find(|i| {
            i.get("isPrimary")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        })
        .or_else(|| images.first())
        .and_then(|i| i.get("url").and_then(|v| v.as_str()))
        .unwrap_or("/images/fallbacks/default-cover.svg")
        .to_string();

    let image_count = images.len() as i32;

    Ok(Json(json!({
        "status": "success",
        "listing": {
            "id":           listing_id.to_string(),
            "slug":         slug_db,
            "title":        title.clone(),
            "titleAr":      title_ar.unwrap_or_else(|| title.clone()),
            "titleEn":      title_en.unwrap_or_else(|| title.clone()),
            "description":  description.clone(),
            "descriptionAr": description_ar.or_else(|| description.clone()).unwrap_or_default(),
            "descriptionEn": description_en.or(description).unwrap_or_default(),
            "metaTitleAr":        meta_title_ar,
            "metaTitleEn":        meta_title_en,
            "metaDescriptionAr":  meta_description_ar,
            "metaDescriptionEn":  meta_description_en,
            "category":     category,
            "attributes":   attributes_val,
            "genderSection": gender_section,
            "priceOnInquiry": price_on_inquiry.unwrap_or(false),
            "featuresSelection": features_selection,
            "coordinator": {
                "nameAr": prod_coord_name_ar.unwrap_or_default(),
                "nameEn": prod_coord_name_en.unwrap_or_default(),
                "phone": prod_coord_phone.unwrap_or_default(),
                "whatsapp": prod_coord_whatsapp.unwrap_or_default(),
                "email": prod_coord_email.unwrap_or_default(),
                "mobile": prod_coord_mobile.unwrap_or_default(),
                "avatar": prod_coord_avatar
            },
            "basePriceSar": base_price.map(|p| p.to_string()),
            "startingPrice": starting_price_display,
            "depositPercentage": deposit_pct,
            "currency":     currency,
            "isFeatured":   is_featured,
            "isAvailable":  is_available,
            "images":       images,
            "coverImage":   cover_image,
            "imageCount":   image_count,
            "reviews":      reviews,
            "packages":     packages,
            "rating": {
                "overall":       overall_rating,
                "quality":       (quality_avg * 10.0).round() / 10.0,
                "staff":         (staff_avg * 10.0).round() / 10.0,
                "communication": (comm_avg * 10.0).round() / 10.0,
                "count":         review_count,
            },
            "vendor": {
                "id":           vendor_id.to_string(),
                "slug":         vendor_slug,
                "nameAr":       vendor_name_ar,
                "nameEn":       vendor_name_en,
                "descriptionAr":vendor_desc_ar,
                "descriptionEn":vendor_desc_en,
                "phone":        vendor_phone,
                "email":        vendor_email,
                "website":      vendor_website,
                "mapsUrl":      vendor_maps,
                "videoUrl":     vendor_video,
                "starRating":   vendor_stars.map(|s| s.to_string()),
                "location": {
                    "latitude":    vendor_lat,
                    "longitude":   vendor_lng,
                    "addressAr":   vendor_addr_ar,
                    "addressEn":   vendor_addr_en,
                    "districtAr":  vendor_dist_ar,
                    "districtEn":  vendor_dist_en,
                },
            },
            "citySlug":     city_slug,
            "cityAr":       city_name_ar,
            "cityEn":       city_name_en,
            "countryCode":  country_code,
            "siblingListings": siblings,
            "createdAt":    created_at.to_rfc3339(),
            "updatedAt":    updated_at.to_rfc3339(),
            "googleMapsUrl": google_maps_url,
            "latitude":      latitude,
            "longitude":     longitude,
            // Routing helpers
            "detailUrl":    format!("/listings/{}", slug_db),
            "bookingUrl":   format!("/booking/listing/{}", listing_id),
        }
    })))
}

async fn get_platform_stats(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let venues: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vendors WHERE status = 'active'")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let bookings: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM core_bookings")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let cities: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM cities")
        .fetch_one(&state.db)
        .await
        .unwrap_or(0);

    let avg_rating: Option<f64> = sqlx::query_scalar("SELECT AVG(rating_quality + rating_staff + rating_communication)/3.0 FROM reviews WHERE status = 'approved'")
        .fetch_one(&state.db)
        .await
        .unwrap_or(None);

    let satisfaction = match avg_rating {
        Some(avg) => Some((avg / 5.0 * 100.0).round() as i64),
        None => None,
    };

    Ok(Json(json!({
        "status": "success",
        "data": {
            "venues": venues,
            "bookings": bookings,
            "cities": cities,
            "satisfaction": satisfaction
        }
    })))
}

#[derive(serde::Deserialize)]
pub struct SearchSuggestionsQuery {
    pub q: String,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct SearchSuggestionCategory {
    pub slug: String,
    pub name_ar: String,
    pub name_en: String,
    pub emoji: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct SearchSuggestionListing {
    pub slug: String,
    pub title_ar: String,
    pub title_en: String,
    pub product_category: String,
}

#[derive(Serialize)]
pub struct SearchSuggestionsResponse {
    pub categories: Vec<SearchSuggestionCategory>,
    pub listings: Vec<SearchSuggestionListing>,
}

async fn get_search_suggestions(
    Query(query): Query<SearchSuggestionsQuery>,
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    let clean_q = query.q.trim();
    if clean_q.len() < 2 {
        return Ok(Json(json!({
            "status": "success",
            "data": {
                "categories": [],
                "listings": []
            }
        })));
    }

    let words: Vec<&str> = clean_q.split_whitespace().collect();
    if words.is_empty() {
        return Ok(Json(json!({
            "status": "success",
            "data": {
                "categories": [],
                "listings": []
            }
        })));
    }

    let db = &state.db;

    let mut query_builder = sqlx::QueryBuilder::new(
        "SELECT slug, title_ar, title_en, product_category FROM vendor_products WHERE status = 'active' AND is_available = true"
    );

    for word in &words {
        query_builder.push(" AND (title_ar ILIKE ");
        query_builder.push_bind(format!("%{}%", word));
        query_builder.push(" OR title_en ILIKE ");
        query_builder.push_bind(format!("%{}%", word));
        query_builder.push(")");
    }

    query_builder.push(" ORDER BY created_at DESC LIMIT 5");

    let listings = query_builder
        .build_query_as::<SearchSuggestionListing>()
        .fetch_all(db)
        .await
        .unwrap_or_default();

    Ok(Json(json!({
        "status": "success",
        "data": SearchSuggestionsResponse {
            categories: vec![],
            listings,
        }
    })))
}

#[derive(Deserialize, Validate)]
pub struct CreateSupportMessageRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(max = 50))]
    pub phone: Option<String>,
    #[validate(length(min = 1, max = 255))]
    pub subject: String,
    #[validate(length(min = 1, max = 10000))]
    pub message: String,
}

pub async fn create_support_message(
    State(state): State<AppState>,
    Json(payload): Json<CreateSupportMessageRequest>,
) -> Result<Json<Value>, AppError> {
    payload.validate()?;

    let sanitized_name = sanitize_str(&payload.name, limits::LABEL);
    let sanitized_email = sanitize_str(&payload.email, limits::EMAIL).to_lowercase();
    let sanitized_phone = payload.phone.as_ref().map(|p| sanitize_str(p, limits::PHONE));
    let sanitized_subject = sanitize_str(&payload.subject, limits::LABEL);
    let sanitized_message = sanitize_str(&payload.message, 10000);

    let row = sqlx::query(
        "INSERT INTO public.support_messages (name, email, phone, subject, message)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, status, created_at"
    )
    .bind(sanitized_name)
    .bind(sanitized_email)
    .bind(sanitized_phone)
    .bind(sanitized_subject)
    .bind(sanitized_message)
    .fetch_one(&state.db)
    .await?;

    let message_id: Uuid = row.get("id");
    let status: String = row.get("status");
    let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");

    Ok(Json(json!({
        "status": "success",
        "message": "Support message submitted successfully",
        "data": {
            "id": message_id.to_string(),
            "status": status,
            "created_at": created_at.to_rfc3339()
        }
    })))
}

#[derive(Serialize, Deserialize)]
pub struct SitemapListingItem {
    pub slug: String,
    pub updated_at: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SitemapBlogItem {
    pub slug: String,
    pub updated_at: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SitemapComboItem {
    pub city: String,
    pub category: String,
    pub updated_at: Option<String>,
}

pub async fn get_sitemap_data(
    State(state): State<AppState>,
) -> Result<Json<Value>, AppError> {
    // 1. Fetch active listings
    let listing_rows = sqlx::query(
        "SELECT vp.slug, TO_CHAR(COALESCE(vp.updated_at, vp.created_at), 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at
         FROM vendor_products vp
         JOIN vendors v ON vp.vendor_id = v.id
         WHERE vp.status = 'active' AND vp.is_available = TRUE AND v.status = 'active'
         ORDER BY vp.created_at DESC"
    )
    .fetch_all(&state.db)
    .await?;

    let listings: Vec<SitemapListingItem> = listing_rows.into_iter().map(|r| {
        SitemapListingItem {
            slug: r.get("slug"),
            updated_at: r.get("updated_at"),
        }
    }).collect();

    // 2. Fetch published blogs
    let blog_rows = sqlx::query(
        "SELECT slug, TO_CHAR(COALESCE(updated_at, published_at, created_at), 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at
         FROM blogs
         WHERE is_published = true AND (published_at IS NULL OR published_at <= NOW())
         ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await?;

    let blogs: Vec<SitemapBlogItem> = blog_rows.into_iter().map(|r| {
        SitemapBlogItem {
            slug: r.get("slug"),
            updated_at: r.get("updated_at"),
        }
    }).collect();

    // 3. Fetch city x category combinations with at least 1 active listing
    let combo_rows = sqlx::query(
        "SELECT c.slug as city, vp.product_category as category, TO_CHAR(MAX(COALESCE(vp.updated_at, vp.created_at)), 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at
         FROM vendor_products vp
         JOIN vendors v ON vp.vendor_id = v.id
         JOIN cities c ON vp.city_id = c.id
         WHERE vp.status = 'active' AND vp.is_available = TRUE AND v.status = 'active'
         GROUP BY c.slug, vp.product_category"
    )
    .fetch_all(&state.db)
    .await?;

    let combos: Vec<SitemapComboItem> = combo_rows.into_iter().map(|r| {
        SitemapComboItem {
            city: r.get("city"),
            category: r.get("category"),
            updated_at: r.get("updated_at"),
        }
    }).collect();

    Ok(Json(json!({
        "status": "success",
        "data": {
            "listings": listings,
            "blogs": blogs,
            "combos": combos
        }
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use crate::state::AppState;
    use crate::routes::inquiry_management::public::{SubmitAfrahInquiryRequest, submit_afrah_inquiry};
    use crate::config::AppConfig;
    use crate::services::email::EmailService;
    use crate::services::whatsapp::WhatsappService;
    use crate::state::WsManager;
    use tokio::sync::broadcast;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_submit_afrah_inquiry_flow() {
        let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://zafaf_db_admin:ramizwebdeveloperproductionsafe@127.0.0.1:5434/zafaf_world".to_string()
        });
        let pool = PgPool::connect(&db_url).await.unwrap();

        // Clean up any leftover test data
        let _ = sqlx::query("DELETE FROM notification_outbox WHERE aggregate_id IN (SELECT id FROM afrah_inquiries WHERE phone = $1)")
            .bind("+966592112517")
            .execute(&pool)
            .await;
        let _ = sqlx::query("DELETE FROM afrah_inquiries WHERE phone = $1")
            .bind("+966592112517")
            .execute(&pool)
            .await;

        let mut config = AppConfig::from_env();
        config.smtp_host = None; // Disable real SMTP
        config.smtp_port = None;
        config.smtp_username = None;
        config.smtp_password = None;
        config.whatsapp_token = None; // Disable real WhatsApp Meta API requests
        config.afrah_notification_phone = Some("+966500000001".to_string());
        config.whatsapp_phone_number_id = Some("1092839201928392".to_string());
        config.app_environment = "development".to_string(); // bypass token bucket rate limit

        let (booking_event_tx, _) = broadcast::channel(100);
        let (chat_event_tx, _) = broadcast::channel(100);
        let (inquiry_event_tx, _) = broadcast::channel(100);
        let ws_manager = Arc::new(WsManager::new());

        let email_service = Arc::new(EmailService::from_config(&config));
        let whatsapp_service = Arc::new(WhatsappService::from_config(&config));

        let app_state = AppState {
            db: pool.clone(),
            jwt_secret: config.jwt_secret.clone(),
            frontend_url: config.frontend_url.clone(),
            email_service,
            whatsapp_service,
            booking_event_tx,
            chat_event_tx,
            inquiry_event_tx,
            ws_manager,
            rate_limit_store: Arc::new(dashmap::DashMap::new()),
            idempotency_store: Arc::new(dashmap::DashMap::new()),
            trusted_proxies: config.trusted_proxies.clone(),
            minio_client: Arc::new(crate::services::media::minio_client::MinioClient::from_config(&config, pool.clone())),
            location_cache: Arc::new(dashmap::DashMap::new()),
            active_location_requests: Arc::new(dashmap::DashMap::new()),
            config: Arc::new(config.clone()),
        };

        let payload = SubmitAfrahInquiryRequest {
            name: "Test Afrah User".to_string(),
            phone: "+966500000002".to_string(), // Unique user phone E.164
            is_whatsapp: true,
            event_date: "2035-12-31".to_string(),
            message: "Test message from integration test".to_string(),
            email: Some("test_afrah@example.com".to_string()),
        };

        // Submit inquiry via handler
        let response = submit_afrah_inquiry(
            axum::extract::State(app_state.clone()),
            crate::utils::ip::SecureClientIp(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))),
            axum::Json(payload),
        )
        .await
        .unwrap();

        let body = response.0;
        assert_eq!(body["status"], "success");

        // 1. Verify afrah_inquiries row was created
        let inquiry: (Uuid, String, String, String) = sqlx::query_as(
            "SELECT id, name, phone, status FROM afrah_inquiries WHERE phone = $1 ORDER BY created_at DESC LIMIT 1"
        )
        .bind("+966500000002")
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(inquiry.1, "Test Afrah User");
        assert_eq!(inquiry.2, "+966500000002");
        assert_eq!(inquiry.3, "pending");

        // 2. Verify notification_outbox PENDING row was created
        let outbox: (Uuid, String, serde_json::Value, String) = sqlx::query_as(
            "SELECT id, event_type, payload, status FROM notification_outbox WHERE aggregate_id = $1"
        )
        .bind(inquiry.0)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(outbox.1, "new_inquiry");
        assert_eq!(outbox.3, "PENDING");

        // Verify the payload contains correct recipient phone number ("vendor_phone")
        let vendor_phone = outbox.2.get("vendor_phone").and_then(|v| v.as_str()).unwrap();
        assert_eq!(vendor_phone, "+966500000001"); // Matches AFRAH_NOTIFICATION_PHONE

        // 3. Start outbox worker to process the event
        let cancel_token = tokio_util::sync::CancellationToken::new();
        crate::services::outbox_worker::start_outbox_worker(app_state.clone(), cancel_token.clone());

        // Wait for outbox worker to process the message
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        cancel_token.cancel();

        // Check if outbox event status is now DELIVERED
        let outbox_status: String = sqlx::query_scalar(
            "SELECT status FROM notification_outbox WHERE id = $1"
        )
        .bind(outbox.0)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(outbox_status, "DELIVERED");

        // Clean up test data
        let _ = sqlx::query("DELETE FROM notification_outbox WHERE aggregate_id = $1")
            .bind(inquiry.0)
            .execute(&pool)
            .await;
        let _ = sqlx::query("DELETE FROM afrah_inquiries WHERE id = $1")
            .bind(inquiry.0)
            .execute(&pool)
            .await;
    }

    #[tokio::test]
    async fn test_search_suggestions_flow() {
        let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://zafaf_db_admin:ramizwebdeveloperproductionsafe@127.0.0.1:5434/zafaf_world".to_string()
        });
        let pool = PgPool::connect(&db_url).await.unwrap();

        // ── Seed test data ───────────────────────────────────────────────────
        let tier_id: uuid::Uuid = sqlx::query_scalar(
            "SELECT id FROM subscription_tiers ORDER BY priority_score DESC LIMIT 1"
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let user_id: uuid::Uuid = sqlx::query_scalar(
            "INSERT INTO global_users (email, password_hash, domain_type) VALUES ('centro_test@zafafworld.net', 'hash', 'Vendor'::user_domain_enum) RETURNING id"
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        let vendor_id: uuid::Uuid = sqlx::query_scalar(
            "INSERT INTO vendors (user_id, name_en, name_ar, slug, subscription_status, subscription_tier_id) VALUES ($1, 'Centro Vendor', 'بائع سنترو', 'centro-vendor-test', 'active', $2) RETURNING id"
        )
        .bind(user_id)
        .bind(tier_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        let listing_id: uuid::Uuid = sqlx::query_scalar(
            "INSERT INTO vendor_products (vendor_id, title, title_en, title_ar, slug, product_category, base_price_sar, status) VALUES ($1, 'Centro Ballroom', 'Centro Ballroom', 'قاعة سنترو', 'centro-ballroom-test', 'wedding-palace', 1000.0, 'active') RETURNING id"
        )
        .bind(vendor_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        // ── Run test checks ──────────────────────────────────────────────────
        let mut config = AppConfig::from_env();
        config.smtp_host = None;
        config.smtp_port = None;
        config.smtp_username = None;
        config.smtp_password = None;
        config.whatsapp_token = None;

        let (booking_event_tx, _) = broadcast::channel(100);
        let (chat_event_tx, _) = broadcast::channel(100);
        let (inquiry_event_tx, _) = broadcast::channel(100);
        let ws_manager = Arc::new(WsManager::new());

        let email_service = Arc::new(EmailService::from_config(&config));
        let whatsapp_service = Arc::new(WhatsappService::from_config(&config));

        let app_state = AppState {
            db: pool.clone(),
            jwt_secret: config.jwt_secret.clone(),
            frontend_url: config.frontend_url.clone(),
            email_service,
            whatsapp_service,
            booking_event_tx,
            chat_event_tx,
            inquiry_event_tx,
            ws_manager,
            rate_limit_store: Arc::new(dashmap::DashMap::new()),
            idempotency_store: Arc::new(dashmap::DashMap::new()),
            trusted_proxies: config.trusted_proxies.clone(),
            minio_client: Arc::new(crate::services::media::minio_client::MinioClient::from_config(&config, pool.clone())),
            location_cache: Arc::new(dashmap::DashMap::new()),
            active_location_requests: Arc::new(dashmap::DashMap::new()),
            config: Arc::new(config.clone()),
        };

        let query = SearchSuggestionsQuery {
            q: "Centro".to_string(),
        };

        let response = get_search_suggestions(
            axum::extract::Query(query),
            axum::extract::State(app_state),
        )
        .await;

        // ── Clean up test data ───────────────────────────────────────────────
        let _ = sqlx::query("DELETE FROM vendor_products WHERE id = $1").bind(listing_id).execute(&pool).await;
        let _ = sqlx::query("DELETE FROM vendors WHERE id = $1").bind(vendor_id).execute(&pool).await;
        let _ = sqlx::query("DELETE FROM global_users WHERE id = $1").bind(user_id).execute(&pool).await;

        let response = response.unwrap();
        let body = response.0;
        assert_eq!(body["status"], "success");
        
        let data = &body["data"];
        let listings = data["listings"].as_array().unwrap();
        assert!(!listings.is_empty(), "Listings should not be empty!");

        let categories = data["categories"].as_array().unwrap();
        assert!(categories.is_empty(), "Categories should be empty!");

        let has_centro = listings.iter().any(|l| {
            l["title_en"].as_str().unwrap().contains("Centro") ||
            l["title_ar"].as_str().unwrap().contains("سنترو")
        });
        assert!(has_centro, "Should find Centro hotel in suggestions!");
    }
}
