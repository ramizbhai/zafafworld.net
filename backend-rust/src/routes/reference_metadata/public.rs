use axum::{
    extract::{Path, State, Query},
    http::HeaderMap,
    Json, Router,
    routing::get,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;
use sqlx::Row;

use crate::errors::AppError;
use crate::state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/cities", get(list_cities))
        .route("/categories", get(list_categories))
        .route("/amenities", get(list_amenities))
        .route("/venue-types", get(list_venue_types))
        .route("/countries", get(list_countries))
        .route("/countries/:code", get(get_country_by_code))
        .route("/metadata/vendor-config", get(get_vendor_config))
        .with_state(state)
}

// ─── GET VENDOR CONFIG: GET /api/v1/public/metadata/vendor-config ────────────
async fn get_vendor_config() -> impl axum::response::IntoResponse {
    Json(json!({
        "status": "success",
        "data": {
            "genderSections": [
                { "id": "any", "labelEn": "Any/Mixed", "labelAr": "مختلط" },
                { "id": "men_only", "labelEn": "Men Only", "labelAr": "رجال فقط" },
                { "id": "women_only", "labelEn": "Women Only", "labelAr": "نساء فقط" },
                { "id": "dual_parallel", "labelEn": "Dual Parallel (Separated)", "labelAr": "قسمين منفصلين" }
            ],
            "productCategories": [
                { "id": "wedding_hall", "labelEn": "Wedding Hall", "labelAr": "قاعة زفاف" },
                { "id": "hotel_ballroom", "labelEn": "Hotel Ballroom", "labelAr": "قاعة فندق" },
                { "id": "outdoor_venue", "labelEn": "Outdoor Venue", "labelAr": "مكان خارجي" },
                { "id": "restaurant", "labelEn": "Restaurant", "labelAr": "مطعم" }
            ],
            "taskCategories": [
                { "id": "follow_up", "color": "bg-blue-100 text-blue-700" },
                { "id": "tour", "color": "bg-purple-100 text-purple-700" },
                { "id": "contract", "color": "bg-emerald-100 text-emerald-700" },
                { "id": "payment", "color": "bg-amber-100 text-amber-700" },
                { "id": "event_prep", "color": "bg-indigo-100 text-indigo-700" },
                { "id": "other", "color": "bg-gray-100 text-gray-700" }
            ]
        }
    }))
}

#[derive(Deserialize)]
pub struct CitiesQuery {
    pub country_id: Option<String>,
}

async fn list_cities(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<CitiesQuery>,
) -> Result<Json<Value>, AppError> {
    let country_id = headers
        .get("X-Country-ID")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_lowercase())
        .or_else(|| query.country_id.clone())
        .unwrap_or_else(|| "sa".to_string());

    tracing::info!("Querying active cities for country: {}...", country_id);

    let rows = sqlx::query(
        "SELECT ci.id, ci.slug, ci.name_ar, ci.name_en, co.iso_code AS country_id \
         FROM cities ci \
         JOIN countries co ON ci.country_id = co.id \
         WHERE LOWER(co.iso_code) = LOWER($1) OR LOWER(co.slug) = LOWER($1) \
         ORDER BY \
             CASE ci.slug \
                 WHEN 'riyadh' THEN 1 \
                 WHEN 'jeddah' THEN 2 \
                 WHEN 'khobar' THEN 3 \
                 WHEN 'dammam' THEN 4 \
                 ELSE 5 \
             END ASC, \
             ci.name_en ASC",
    )
    .bind(&country_id)
    .fetch_all(&state.db)
    .await?;

    let mut cities = Vec::new();
    for row in rows {
        let id: Uuid = row.get("id");
        let slug: String = row.get("slug");
        let ar: String = row.get("name_ar");
        let en: String = row.get("name_en");
        let cid: String = row.get("country_id");
        cities.push(json!({
            "id": id.to_string(),
            "slug": slug,
            "ar": ar.clone(),
            "en": en.clone(),
            "name_ar": ar,
            "name_en": en,
            "country_id": cid
        }));
    }

    if cities.is_empty() {
        tracing::warn!("No cities found for country_id: {}", country_id);
    }

    Ok(Json(json!({
        "status": "success",
        "cities": cities
    })))
}

#[derive(serde::Deserialize)]
struct CategoriesQuery {
    country_id: Option<String>,
}

async fn list_categories(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<CategoriesQuery>,
) -> Result<Json<Value>, AppError> {
    let country_id = headers
        .get("X-Country-ID")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_lowercase())
        .or_else(|| query.country_id.clone())
        .unwrap_or_else(|| "sa".to_string());

    // Try reading from the categories DB table (V2). Fall back to hardcoded if table doesn't exist yet.
    let rows_result = sqlx::query(
        "SELECT c.slug, c.name_ar, c.name_en, c.parent_group::text AS parent_group, c.emoji, c.priority, c.sort_order, \
                COALESCE(count_table.cnt, 0) AS listings_count \
         FROM categories c \
         LEFT JOIN ( \
             SELECT vp.product_category, COUNT(vp.id) AS cnt \
             FROM vendor_products vp \
             JOIN vendors v ON vp.vendor_id = v.id \
             LEFT JOIN cities ci ON COALESCE(v.city_id, vp.city_id) = ci.id \
             WHERE vp.status = 'active' \
               AND vp.is_available = TRUE \
               AND v.status = 'active' \
               AND (ci.country_id IS NULL OR ci.country_id = $1) \
             GROUP BY vp.product_category \
         ) count_table ON c.slug = count_table.product_category \
         WHERE c.is_active = TRUE \
         ORDER BY c.parent_group, c.sort_order ASC"
    )
    .bind(&country_id)
    .fetch_all(&state.db)
    .await;

    match rows_result {
        Ok(rows) => {
            let mut grouped: std::collections::BTreeMap<String, Vec<Value>> =
                std::collections::BTreeMap::new();
            for row in &rows {
                let slug: String = row.get("slug");
                let name_ar: String = row.get("name_ar");
                let name_en: String = row.get("name_en");
                let parent: String = row.get("parent_group");
                let emoji: Option<String> = row.get("emoji");
                let priority: String = row.get("priority");
                let listings_count: i64 = row.get("listings_count");

                grouped.entry(parent).or_default().push(json!({
                    "slug": slug,
                    "ar": name_ar,
                    "en": name_en,
                    "emoji": emoji,
                    "priority": priority,
                    "listingsCount": listings_count
                }));
            }

            // Build a flat list for clients that want all categories
            let all_categories: Vec<Value> = rows
                .iter()
                .map(|row| {
                    let slug: String = row.get("slug");
                    let name_ar: String = row.get("name_ar");
                    let name_en: String = row.get("name_en");
                    let parent: String = row.get("parent_group");
                    let emoji: Option<String> = row.get("emoji");
                    let listings_count: i64 = row.get("listings_count");
                    json!({
                        "slug": slug,
                        "ar": name_ar,
                        "en": name_en,
                        "parentGroup": parent,
                        "emoji": emoji,
                        "listingsCount": listings_count
                    })
                })
                .collect();

            Ok(Json(json!({
                "status": "success",
                "categories": grouped,
                "allCategories": all_categories,
                "source": "database"
            })))
        }
        Err(_) => {
            // Fallback: hardcoded V1 categories (table not yet migrated)
            Ok(Json(json!({
                "status": "success",
                "source": "hardcoded_fallback",
                "categories": {
                    "venues": [
                        { "slug": "wedding-palace",   "ar": "قصور الأفراح",        "en": "Wedding Palace" },
                        { "slug": "hotel-venue",      "ar": "فنادق وقاعات",         "en": "Hotel Ballroom" },
                        { "slug": "villa-resort",     "ar": "استراحات وفلل",        "en": "Villa & Resort" },
                        { "slug": "restaurant-event", "ar": "مطاعم وقاعات خاصة",   "en": "Restaurant & Dining" },
                        { "slug": "outdoor-garden",   "ar": "حدائق وأماكن مفتوحة", "en": "Outdoor Garden" },
                        { "slug": "rooftop-venue",    "ar": "أماكن على السطح",      "en": "Rooftop Venue" },
                        { "slug": "private-beach",    "ar": "شاطئ خاص",             "en": "Private Beach Venue" },
                        { "slug": "chalet",           "ar": "شاليهات",              "en": "Chalet" }
                    ],
                    "fashion": [
                        { "slug": "wedding-gown",    "ar": "فساتين الزفاف",        "en": "Wedding Gown" },
                        { "slug": "haute-couture",   "ar": "مصممات أزياء وخياطة",          "en": "Haute Couture" },
                        { "slug": "abaya-jalabiya",  "ar": "عباية وجلابية وكفتان", "en": "Abaya & Jalabiya" },
                        { "slug": "groom-attire",    "ar": "ملابس العريس",         "en": "Groom Attire" }
                    ],
                    "beauty": [
                        { "slug": "hair-makeup",     "ar": "شعر ومكياج",   "en": "Hair & Makeup" },
                        { "slug": "henna-art",       "ar": "نقش حناء",     "en": "Henna Art" },
                        { "slug": "beauty-skincare", "ar": "العناية بالبشرة","en": "Beauty & Skincare" }
                    ],
                    "photography": [
                        { "slug": "photography-video", "ar": "مصورات وفيديو", "en": "Photography & Video" },
                        { "slug": "photo-studio",      "ar": "استوديوهات تصوير", "en": "Photo Studio" }
                    ],
                    "food": [
                        { "slug": "catering",        "ar": "ضيافة وطعام",           "en": "Wedding Catering" },
                        { "slug": "wedding-cake",    "ar": "كيك الزفاف",  "en": "Wedding Cake" },
                        { "slug": "wedding-sweets",  "ar": "شوكولاتة وحلويات الضيافة", "en": "Arabic Sweets" }
                    ],
                    "entertainment": [
                        { "slug": "entertainment-dj", "ar": "طقاقات ودي جي",       "en": "DJ & Entertainment" },
                        { "slug": "zaffa",            "ar": "زفة عروس",            "en": "Zaffa" },
                        { "slug": "nasheed-band",     "ar": "إنشاد وفرقة موسيقية", "en": "Nasheed & Band" }
                    ],
                    "jewelry_gifts": [
                        { "slug": "wedding-jewelry", "ar": "مجوهرات وشبكة", "en": "Bridal Jewelry" },
                        { "slug": "wedding-gifts",   "ar": "توزيعات وهدايا",  "en": "Wedding Gifts" }
                    ],
                    "planning_decor": [
                        { "slug": "wedding-planner",   "ar": "منظم حفلات",      "en": "Wedding Planner" },
                        { "slug": "khosha-decor",      "ar": "كوشة وديكور",      "en": "Khosha & Decor" },
                        { "slug": "flowers-floral",    "ar": "مسكة العروس وتنسيق الورد",        "en": "Flowers & Floral" },
                        { "slug": "wedding-invitation","ar": "دعوات زفاف",       "en": "Invitations" },
                        { "slug": "lighting-av",       "ar": "إضاءة وتقنية الصوت","en": "Lighting & AV" }
                    ],
                    "transportation": [
                        { "slug": "wedding-car", "ar": "سيارات الزفاف", "en": "Wedding Car" }
                    ]
                }
            })))
        }
    }
}

async fn list_countries(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let rows = sqlx::query(
        "SELECT id, slug, name_ar, name_en, currency FROM countries ORDER BY name_en ASC",
    )
    .fetch_all(&state.db)
    .await?;

    let mut countries = Vec::new();
    for row in rows {
        let id: String = row.get("id");
        let slug: String = row.get("slug");
        let name_ar: String = row.get("name_ar");
        let name_en: String = row.get("name_en");
        let currency: String = row.get("currency");

        let flag_emoji = match id.as_str() {
            "sa" => "🇸🇦",
            "eg" => "🇪🇬",
            "ae" => "🇦🇪",
            "kw" => "🇰🇼",
            "bh" => "🇧🇭",
            "qa" => "🇶🇦",
            "om" => "🇴🇲",
            "lb" => "🇱🇧",
            "tn" => "🇹🇳",
            "ma" => "🇲🇦",
            _ => "🏳️",
        };

        countries.push(json!({
            "id": id,
            "slug": slug,
            "name_ar": name_ar,
            "name_en": name_en,
            "currency": currency,
            "flag_emoji": flag_emoji
        }));
    }

    Ok(Json(json!({
        "status": "success",
        "countries": countries
    })))
}

async fn get_country_by_code(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<Value>, AppError> {
    let country_code = code.to_lowercase();

    let country_row =
        sqlx::query("SELECT id, slug, name_ar, name_en, currency FROM countries WHERE id = $1")
            .bind(&country_code)
            .fetch_optional(&state.db)
            .await?;

    let row = match country_row {
        Some(r) => r,
        None => return Err(AppError::NotFound("Country not found".to_string())),
    };

    let id: String = row.get("id");
    let slug: String = row.get("slug");
    let name_ar: String = row.get("name_ar");
    let name_en: String = row.get("name_en");
    let currency: String = row.get("currency");

    let flag_emoji = match id.as_str() {
        "sa" => "🇸🇦",
        "eg" => "🇪🇬",
        "ae" => "🇦🇪",
        "kw" => "🇰🇼",
        "bh" => "🇧🇭",
        "qa" => "🇶🇦",
        "om" => "🇴🇲",
        "lb" => "🇱🇧",
        "tn" => "🇹🇳",
        "ma" => "🇲🇦",
        _ => "🏳️",
    };

    Ok(Json(json!({
        "status": "success",
        "country": {
            "id": id,
            "slug": slug,
            "name_ar": name_ar,
            "name_en": name_en,
            "currency": currency,
            "flag_emoji": flag_emoji
        }
    })))
}

// ─── GET AMENITIES: GET /api/v1/public/amenities ─────────────────────────────
async fn list_amenities() -> Result<Json<Value>, AppError> {
    let amenities = vec![
        json!({"key": "wifi", "label_ar": "واي فاي", "label_en": "WiFi"}),
        json!({"key": "valet_parking", "label_ar": "خدمة ركن السيارات", "label_en": "Valet Parking"}),
        json!({"key": "bridal_suite", "label_ar": "جناح العروس", "label_en": "Bridal Suite"}),
        json!({"key": "audio_visual", "label_ar": "معدات صوت وضوء", "label_en": "Audio/Visual"}),
        json!({"key": "catering", "label_ar": "بوفيه", "label_en": "Catering"})
    ];
    Ok(Json(json!({
        "status": "success",
        "data": amenities
    })))
}

// ─── GET VENUE TYPES: GET /api/v1/public/venue-types ─────────────────────────
async fn list_venue_types() -> Result<Json<Value>, AppError> {
    let types = vec![
        json!({"key": "hotel", "label_ar": "فندق", "label_en": "Hotel"}),
        json!({"key": "wedding_hall", "label_ar": "قاعة افراح", "label_en": "Wedding Hall"}),
        json!({"key": "resort", "label_ar": "منتجع", "label_en": "Resort"}),
        json!({"key": "villa", "label_ar": "فيلا", "label_en": "Villa"})
    ];
    Ok(Json(json!({
        "status": "success",
        "data": types
    })))
}
