use crate::errors::AppError;
use crate::middleware::auth::RlsTx;
use validator::Validate;
use crate::state::AppState;
use crate::utils::sanitize::{limits, sanitize_opt, sanitize_str};
use axum::{
    routing::{delete, get, patch},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

pub fn router(_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/products", get(list_products).post(create_product))
        .route(
            "/products/:id",
            get(get_product).put(update_product).delete(delete_product),
        )
        .route("/wizard-schema/:category_id", get(get_wizard_schema))
        .route("/products/:id/edit-context", get(get_edit_context))
        .route("/products/:id/status", patch(update_product_status))
        .route("/products/:id/archive", patch(archive_product))
        .route("/products/:id/restore", patch(restore_product))
        .route(
            "/products/:id/availability",
            patch(toggle_product_availability),
        )
        // ── Listing-scoped gallery endpoints ──────────────────────────────────
        .route(
            "/products/:id/images",
            get(list_product_images).post(add_product_image),
        )
        .route(
            "/products/:id/images/reorder",
            patch(reorder_product_images),
        )
        .route(
            "/products/:id/images/:img_id/cover",
            patch(set_product_cover_image),
        )
        .route("/products/:id/images/:img_id", delete(delete_product_image))
}

// ─── VALID ENUM VALUES (fully synced with DB constraint) ──────────────────────

const VALID_COORDINATOR_GENDERS: &[&str] = &["male", "female", "any"];

// ─── REQUEST STRUCTS ───────────────────────────────────────────────────────────

#[derive(Deserialize, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct GalleryItemInput {
    image_url: String,
    file_path: Option<String>,
    is_cover: Option<bool>,
    caption: Option<String>,
    sort_order: Option<i32>,
    media_type: Option<String>,
    thumbnail_url: Option<String>,
    file_size: Option<i64>,
    duration_seconds: Option<i32>,
}

#[derive(Deserialize, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductRequest {
    // Bilingual title (required: at least one of these)
    title: Option<String>, // legacy single-lang field (fallback)
    #[validate(length(min = 1, max = 255))]
    title_ar: Option<String>,
    #[validate(length(min = 1, max = 255))]
    title_en: Option<String>,
    // Bilingual description
    description: Option<String>, // legacy single-lang field (fallback)
    description_ar: Option<String>,
    description_en: Option<String>,
    // SEO fields
    meta_title_ar: Option<String>,
    meta_title_en: Option<String>,
    meta_description_ar: Option<String>,
    meta_description_en: Option<String>,
    product_category: Option<String>,
    #[validate(range(min = 0.01))]
    base_price_sar: Option<f64>,
    price_on_inquiry: Option<bool>,
    #[validate(range(min = 0, max = 100))]
    deposit_percentage: Option<i32>,
    // Gender section (KSA cultural requirement)
    gender_section: Option<String>,
    // Coordinator (bilingual)
    coordinator_name: Option<String>, // legacy (English fallback)
    coordinator_name_ar: Option<String>,
    coordinator_name_en: Option<String>,
    coordinator_phone: Option<String>,
    coordinator_whatsapp: Option<String>,
    coordinator_avatar: Option<String>,
    coordinator_gender: Option<String>,
    #[validate(email)]
    coordinator_email: Option<String>,
    coordinator_mobile: Option<String>,
    crm_product_id: Option<String>,
    city_id: Option<uuid::Uuid>,
    vendor_category: Option<String>,
    attributes: Option<serde_json::Value>,
    cultural_attributes: Option<serde_json::Value>,
    // Location & directions
    google_maps_url: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    // Dynamic features selection
    features_selection: Option<serde_json::Value>,
    // Gallery items for atomic creation
    #[validate(nested)]
    gallery_items: Option<Vec<GalleryItemInput>>,
}

#[derive(Deserialize, validator::Validate)]
#[serde(rename_all = "camelCase")]
struct UpdateProductRequest {
    // Bilingual title (required: at least title_en or title)
    title: Option<String>, // legacy single-lang field (fallback)
    #[validate(length(min = 1, max = 255))]
    title_ar: Option<String>,
    #[validate(length(min = 1, max = 255))]
    title_en: Option<String>,
    // Bilingual description
    description: Option<String>, // legacy
    description_ar: Option<String>,
    description_en: Option<String>,
    // SEO fields
    meta_title_ar: Option<String>,
    meta_title_en: Option<String>,
    meta_description_ar: Option<String>,
    meta_description_en: Option<String>,
    product_category: Option<String>,
    #[validate(range(min = 0.01))]
    base_price_sar: Option<f64>,
    price_on_inquiry: Option<bool>,
    #[validate(range(min = 0, max = 100))]
    deposit_percentage: Option<i32>,
    // Gender section
    gender_section: Option<String>,
    // Coordinator
    coordinator_name: Option<String>, // legacy
    coordinator_name_ar: Option<String>,
    coordinator_name_en: Option<String>,
    coordinator_phone: Option<String>,
    coordinator_whatsapp: Option<String>,
    coordinator_avatar: Option<String>,
    coordinator_gender: Option<String>,
    #[validate(email)]
    coordinator_email: Option<String>,
    coordinator_mobile: Option<String>,
    crm_product_id: Option<String>,
    version: i32,
    city_id: Option<uuid::Uuid>,
    vendor_category: Option<String>,
    attributes: Option<serde_json::Value>,
    cultural_attributes: Option<serde_json::Value>,
    // Location & directions
    google_maps_url: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    // Dynamic features selection
    features_selection: Option<serde_json::Value>,
    // Gallery items for atomic updates
    #[validate(nested)]
    gallery_items: Option<Vec<GalleryItemInput>>,
}

#[derive(Deserialize, validator::Validate)]
#[serde(deny_unknown_fields)]
struct UpdateProductStatusRequest {
    #[validate(length(min = 1, max = 50))]
    status: String,
}

#[derive(Deserialize, validator::Validate)]
#[serde(deny_unknown_fields)]
struct ToggleAvailabilityRequest {
    is_available: bool,
}

#[derive(Deserialize, validator::Validate)]
struct ReorderImageRequest {
    #[validate(nested)]
    images: Vec<ImageSortItem>,
}

// rename_all = "camelCase" ensures "sortOrder" from JSON maps to sort_order field.
#[derive(Deserialize, validator::Validate)]
#[serde(rename_all = "camelCase")]
struct ImageSortItem {
    id: Uuid,
    #[validate(range(min = 0))]
    sort_order: i32,
}

// ─── HELPERS ──────────────────────────────────────────────────────────────────

fn generate_product_slug(name_en: &str) -> String {
    name_en
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn validate_google_maps_url(url: &str) -> Result<(), AppError> {
    let url_trimmed = url.trim();
    if url_trimmed.is_empty() {
        return Ok(());
    }

    if !url_trimmed.starts_with("http://") && !url_trimmed.starts_with("https://") {
        return Err(AppError::BadRequest(
            "Invalid Google Maps URL: Must start with http:// or https://".to_string(),
        ));
    }

    let is_valid = url_trimmed.contains("google.com/maps")
        || url_trimmed.contains("maps.google.com")
        || url_trimmed.contains("maps.app.goo.gl")
        || url_trimmed.contains("goo.gl/maps");

    if !is_valid {
        return Err(AppError::BadRequest(
            "Invalid Google Maps URL: URL must be a valid Google Maps link".to_string(),
        ));
    }

    Ok(())
}

fn validate_email_simple(email: &str) -> bool {
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }
    let local = parts[0];
    let domain = parts[1];
    if local.is_empty() || domain.is_empty() {
        return false;
    }
    let dot_parts: Vec<&str> = domain.split('.').collect();
    if dot_parts.len() < 2 {
        return false;
    }
    if dot_parts[0].is_empty() || dot_parts[dot_parts.len() - 1].is_empty() {
        return false;
    }
    true
}

// Pre-existing function signature with 15 parameters. Refactoring to a request
// struct is a planned follow-up (requires updating all ~6 call sites). Suppress
// the clippy lint to avoid blocking the build until that refactor lands.
#[allow(clippy::too_many_arguments)]
fn validate_product_payload(
    title_en: &Option<String>,
    title_ar: &Option<String>,
    title_legacy: &Option<String>,
    base_price_sar: &Option<f64>,
    price_on_inquiry: &Option<bool>,
    city_id: &Option<uuid::Uuid>,
    description_en: &Option<String>,
    description_ar: &Option<String>,
    description_legacy: &Option<String>,
    coordinator_name_en: &Option<String>,
    coordinator_name_ar: &Option<String>,
    coordinator_name_legacy: &Option<String>,
    coordinator_phone: &Option<String>,
    coordinator_whatsapp: &Option<String>,
    coordinator_email: &Option<String>,
) -> Result<(), AppError> {
    let raw_title_en = title_en.clone().or_else(|| title_legacy.clone());
    let raw_title_ar = title_ar.clone().or_else(|| title_legacy.clone());

    let is_basic_info = raw_title_en.is_some() || raw_title_ar.is_some() || base_price_sar.is_some() || price_on_inquiry.is_some() || city_id.is_some();
    
    if is_basic_info {
        if let Some(ref en) = raw_title_en {
            if en.trim().is_empty() {
                return Err(AppError::BadRequest("English title cannot be empty".to_string()));
            }
        } else {
            return Err(AppError::BadRequest("English title is required".to_string()));
        }

        if let Some(ref ar) = raw_title_ar {
            if ar.trim().is_empty() {
                return Err(AppError::BadRequest("Arabic title cannot be empty".to_string()));
            }
        } else {
            return Err(AppError::BadRequest("Arabic title is required".to_string()));
        }

        let inquiry = price_on_inquiry.unwrap_or(false);
        if !inquiry && base_price_sar.is_none() {
            return Err(AppError::BadRequest("base_price_sar is required when price_on_inquiry is false".to_string()));
        }

        if city_id.is_none() {
            return Err(AppError::BadRequest("city_id is required".to_string()));
        }
    }

    let raw_desc_en = description_en.clone().or_else(|| description_legacy.clone());
    let raw_desc_ar = description_ar.clone().or_else(|| description_legacy.clone());
    if raw_desc_en.is_some() || raw_desc_ar.is_some() {
        let len_en = raw_desc_en.as_ref().map(|s| s.trim().len()).unwrap_or(0);
        let len_ar = raw_desc_ar.as_ref().map(|s| s.trim().len()).unwrap_or(0);
        if len_en < 50 && len_ar < 50 {
            return Err(AppError::BadRequest("At least one description (English or Arabic) must be 50 characters or longer".to_string()));
        }
    }

    let raw_coord_en = coordinator_name_en.clone().or_else(|| coordinator_name_legacy.clone());
    let is_coord = raw_coord_en.is_some() || coordinator_name_ar.is_some() || coordinator_phone.is_some() || coordinator_whatsapp.is_some() || coordinator_email.is_some();
    if is_coord {
        let name_en = raw_coord_en.as_deref().unwrap_or("").trim();
        let name_ar = coordinator_name_ar.as_deref().unwrap_or("").trim();
        let phone = coordinator_phone.as_deref().unwrap_or("").trim();
        let whatsapp = coordinator_whatsapp.as_deref().unwrap_or("").trim();
        let email = coordinator_email.as_deref().unwrap_or("").trim();

        if name_en.is_empty() {
            return Err(AppError::BadRequest("Coordinator Name (English) is required".to_string()));
        }
        if name_ar.is_empty() {
            return Err(AppError::BadRequest("Coordinator Name (Arabic) is required".to_string()));
        }
        if phone.is_empty() {
            return Err(AppError::BadRequest("Coordinator Phone is required".to_string()));
        }
        if whatsapp.is_empty() {
            return Err(AppError::BadRequest("Coordinator WhatsApp is required".to_string()));
        }
        if email.is_empty() {
            return Err(AppError::BadRequest("Coordinator Email is required".to_string()));
        }

        if !validate_email_simple(email) {
            return Err(AppError::BadRequest("Coordinator Email must be a valid email address".to_string()));
        }
    }

    Ok(())
}

fn compute_total_capacity(attrs: &serde_json::Value) -> Option<i32> {
    let men = attrs
        .get("men_capacity")
        .or_else(|| attrs.get("menCapacity"))
        .and_then(|v| v.as_i64())
        .map(|v| v as i32);
    let women = attrs
        .get("women_capacity")
        .or_else(|| attrs.get("womenCapacity"))
        .and_then(|v| v.as_i64())
        .map(|v| v as i32);
    
    let max_cap = attrs.get("max_capacity").or_else(|| attrs.get("maxCapacity")).and_then(|v| v.as_i64()).map(|v| v as i32);
    let seating = attrs.get("seating_capacity").or_else(|| attrs.get("seatingCapacity")).and_then(|v| v.as_i64()).map(|v| v as i32);
    let private_hall = attrs.get("private_hall_capacity").or_else(|| attrs.get("privateHallCapacity")).and_then(|v| v.as_i64()).map(|v| v as i32);

    match (men, women) {
        (Some(m), Some(w)) => Some(m + w),
        (Some(m), None) => Some(m),
        (None, Some(w)) => Some(w),
        (None, None) => max_cap.or(seating).or(private_hall),
    }
}

fn compute_searchable_amenities(
    attrs: &serde_json::Value,
    cultural: &serde_json::Value,
    features: &serde_json::Value,
) -> Vec<String> {
    let mut amenities = std::collections::HashSet::new();

    if let Some(obj) = cultural.as_object() {
        for (key, val) in obj {
            if val.as_bool() == Some(true) {
                amenities.insert(key.clone());
            }
        }
    }

    if let Some(obj) = attrs.as_object() {
        for (key, val) in obj {
            if val.as_bool() == Some(true) {
                amenities.insert(key.clone());
            }
        }
    }

    if let Some(obj) = features.as_object() {
        for (key, val) in obj {
            if val.as_bool() == Some(true) {
                amenities.insert(key.clone());
            } else if let Some(s) = val.as_str() {
                if !s.trim().is_empty() {
                    amenities.insert(s.trim().to_string());
                }
            } else if let Some(n) = val.as_i64() {
                amenities.insert(n.to_string());
            } else if let Some(f) = val.as_f64() {
                amenities.insert(f.to_string());
            }
        }
    }

    amenities.into_iter().collect()
}

async fn get_wizard_schema(
    _auth: crate::middleware::auth::RequireVendor,
    mut rls_tx: RlsTx,
    axum::extract::Path(category_id): axum::extract::Path<String>,
) -> Result<Json<Value>, AppError> {
    let rows = sqlx::query(
        "SELECT id, name_en, name_ar, category, input_type FROM features"
    )
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let db_features: Vec<crate::services::category_schema::RawDbFeature> = rows
        .into_iter()
        .map(|row| crate::services::category_schema::RawDbFeature {
            id: row.get("id"),
            name_en: row.get("name_en"),
            name_ar: row.get("name_ar"),
            category: row.get("category"),
            input_type: row.get("input_type"),
        })
        .collect();

    let schema = crate::services::category_schema::get_schema_for_category(&category_id, &db_features)?;
    // SAFETY: Struct serialization to json Value is guaranteed to be infallible
    Ok(Json(serde_json::to_value(schema).unwrap()))
}

fn row_to_product_json(row: &sqlx::postgres::PgRow) -> Result<Value, AppError> {
    let id: uuid::Uuid = row.get("id");
    let vendor_id: uuid::Uuid = row.get("vendor_id");
    // Legacy single-lang title — used as fallback
    let title: Option<String> = row.try_get("title").ok();
    let title_ar: Option<String> = row.try_get("title_ar").ok().flatten();
    let title_en: Option<String> = row.try_get("title_en").ok().flatten();
    let slug: String = row.get("slug");
    let description: Option<String> = row.try_get("description").ok().flatten();
    let description_ar: Option<String> = row.try_get("description_ar").ok().flatten();
    let description_en: Option<String> = row.try_get("description_en").ok().flatten();
    let product_category: String = row.get("product_category");
    let base_price_sar: Option<rust_decimal::Decimal> = row.get("base_price_sar");
    let deposit_percentage: i32 = row.get("deposit_percentage");
    let price_on_inquiry: Option<bool> = row.try_get("price_on_inquiry").ok();
    let gender_section: Option<String> = row.try_get("gender_section").ok().flatten();
    let total_capacity: Option<i32> = row.try_get("total_capacity").ok().flatten();
    let quality_score: Option<i32> = row.try_get("quality_score").ok();
    let coordinator_name: Option<String> = row.try_get("coordinator_name").ok().flatten();
    let coordinator_name_ar: Option<String> = row.try_get("coordinator_name_ar").ok().flatten();
    let coordinator_name_en: Option<String> = row.try_get("coordinator_name_en").ok().flatten();
    let coordinator_phone: Option<String> = row.get("coordinator_phone");
    let coordinator_whatsapp: Option<String> = row.get("coordinator_whatsapp");
    let coordinator_avatar: Option<String> = row.get("coordinator_avatar");
    let coordinator_gender: Option<String> = row.get("coordinator_gender");
    let coordinator_email: Option<String> = row.try_get("coordinator_email").ok().flatten();
    let coordinator_mobile: Option<String> = row.try_get("coordinator_mobile").ok().flatten();
    let crm_product_id: Option<String> = row.get("crm_product_id");
    let status: String = row.get("status");
    let rejection_reason: Option<String> = row.get("rejection_reason");
    let is_available: bool = row.get("is_available");
    let is_featured: bool = row.get("is_featured");
    let featured_until: Option<chrono::DateTime<chrono::Utc>> = row.get("featured_until");
    let version: i32 = row.get("version");
    let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
    let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");
    let city_id: Option<uuid::Uuid> = row.get("city_id");
    let attributes: Value = row.get("attributes");
    let google_maps_url: Option<String> = row.try_get("google_maps_url").ok().flatten();
    let latitude: Option<f64> = row.try_get("latitude").ok().flatten();
    let longitude: Option<f64> = row.try_get("longitude").ok().flatten();
    let features_selection: Value = row
        .try_get("features_selection")
        .unwrap_or_else(|_| json!({}));
    let meta_title_ar: Option<String> = row.try_get("meta_title_ar").ok().flatten();
    let meta_title_en: Option<String> = row.try_get("meta_title_en").ok().flatten();
    let meta_description_ar: Option<String> = row.try_get("meta_description_ar").ok().flatten();
    let meta_description_en: Option<String> = row.try_get("meta_description_en").ok().flatten();
    let cultural_attributes: Value = row.try_get("cultural_attributes").unwrap_or_else(|_| json!({}));

    // Resolve effective title: prefer bilingual columns, fall back to legacy
    let effective_title_en = title_en.or_else(|| title.clone()).unwrap_or_default();
    let effective_title_ar = title_ar.or_else(|| title.clone()).unwrap_or_default();

    Ok(json!({
        "id": id.to_string(),
        "vendorId": vendor_id.to_string(),
        "culturalAttributes": cultural_attributes,
        // V2 bilingual fields
        "titleAr": effective_title_ar,
        "titleEn": effective_title_en,
        "descriptionAr": description_ar.or_else(|| description.clone()).unwrap_or_default(),
        "descriptionEn": description_en.or(description).unwrap_or_default(),
        "metaTitleAr": meta_title_ar.unwrap_or_default(),
        "metaTitleEn": meta_title_en.unwrap_or_default(),
        "metaDescriptionAr": meta_description_ar.unwrap_or_default(),
        "metaDescriptionEn": meta_description_en.unwrap_or_default(),
        // Legacy compat — single title field still returned
        "title": effective_title_en,
        "slug": slug,
        "productCategory": product_category,
        "genderSection": gender_section,
        "totalCapacity": total_capacity,
        "qualityScore": quality_score.unwrap_or(0),
        "googleMapsUrl": google_maps_url,
        "latitude": latitude,
        "longitude": longitude,
        // featuresSelection at canonical top-level position (no duplicate)
        "pricing": {
            "basePriceSar": match base_price_sar {
                Some(d) => Some(d.to_string().parse::<f64>().map_err(|_| AppError::Internal("Invalid price format in database".to_string()))?),
                None => None,
            },
            "depositPercentage": deposit_percentage,
            "priceOnInquiry": price_on_inquiry.unwrap_or(false)
        },
        "coordinator": {
            "nameAr": coordinator_name_ar.or_else(|| coordinator_name.clone()).unwrap_or_default(),
            "nameEn": coordinator_name_en.or(coordinator_name).unwrap_or_default(),
            "phone": coordinator_phone.unwrap_or_default(),
            "whatsapp": coordinator_whatsapp.unwrap_or_default(),
            "avatar": coordinator_avatar,
            "gender": coordinator_gender.unwrap_or_else(|| "any".to_string()),
            "email": coordinator_email.unwrap_or_default(),
            "mobile": coordinator_mobile.unwrap_or_default()
        },
        "attributes": attributes,
        "featuresSelection": features_selection,
        "metadata": {
            "crmProductId": crm_product_id,
            "status": status,
            "rejectionReason": rejection_reason,
            "isAvailable": is_available,
            "isFeatured": is_featured,
            "featuredUntil": featured_until.map(|dt| dt.to_rfc3339()),
            "version": version,
            "createdAt": created_at.to_rfc3339(),
            "updatedAt": updated_at.to_rfc3339(),
            "cityId": city_id.map(|u| u.to_string())
        }
    }))
}

fn row_to_gallery_json(row: &sqlx::postgres::PgRow) -> Value {
    let id: Uuid = row.get("id");
    let product_id: Option<Uuid> = row.get("product_id");
    let image_url: String = row.get("image_url");
    let file_path: Option<String> = row.get("file_path");
    let is_cover: bool = row.get("is_cover");
    let sort_order: i32 = row.get("sort_order");
    let caption: Option<String> = row.get("caption");
    let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
    let media_type: String = row
        .try_get("media_type")
        .unwrap_or_else(|_| "image".to_string());
    let file_url: String = row
        .try_get("file_url")
        .unwrap_or_else(|_| image_url.clone());
    let thumbnail_url: Option<String> = row.try_get("thumbnail_url").ok().flatten();
    let file_size: Option<i64> = row.try_get("file_size").ok().flatten();
    let duration_seconds: Option<i32> = row.try_get("duration_seconds").ok().flatten();

    json!({
        "id": id.to_string(),
        "productId": product_id.map(|u| u.to_string()),
        "imageUrl": image_url,
        "filePath": file_path,
        "isCover": is_cover,
        "sortOrder": sort_order,
        "caption": caption,
        "createdAt": created_at.to_rfc3339(),
        "mediaType": media_type,
        "fileUrl": file_url,
        "thumbnailUrl": thumbnail_url,
        "fileSize": file_size,
        "durationSeconds": duration_seconds
    })
}

const PRODUCT_SELECT: &str = "
    SELECT
        id, vendor_id, slug, product_category, base_price_sar, deposit_percentage,
        coordinator_phone, coordinator_whatsapp, coordinator_avatar, coordinator_gender,
        crm_product_id, city_id, status, rejection_reason, is_available, is_featured,
        version, created_at, updated_at,
        -- Legacy single-lang columns (may be NULL if migrated to bilingual)
        title, description, coordinator_name,
        -- V2 bilingual columns
        title_ar, title_en, description_ar, description_en,
        coordinator_name_ar, coordinator_name_en,
        -- V2 structured columns
        gender_section, total_capacity, quality_score, price_on_inquiry,
        attributes, featured_until,
        -- Location & directions
        google_maps_url, latitude, longitude,
        -- Dynamic features
        features_selection,
        -- SEO fields
        meta_title_ar, meta_title_en, meta_description_ar, meta_description_en,
        -- Contact Details
        coordinator_email, coordinator_mobile
";

// ─── LIST: GET /vendor/products ───────────────────────────────────────────────

async fn list_products(
    _auth: crate::middleware::auth::RequireVendor,
    mut rls_tx: RlsTx,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    let rows = sqlx::query(&format!(
        "{} FROM vendor_products WHERE vendor_id = $1 ORDER BY created_at ASC",
        PRODUCT_SELECT
    ))
    .bind(vendor_id)
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let products: Result<Vec<Value>, AppError> = rows.iter().map(row_to_product_json).collect();
    let products = products?;
    let total = products.len();

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "products": products,
        "total": total
    })))
}

// ─── GET ONE: GET /vendor/products/:id ───────────────────────────────────────

async fn get_product(
    _auth: crate::middleware::auth::RequireVendor,
    mut rls_tx: RlsTx,
    axum::extract::Path(product_id): axum::extract::Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    let row = sqlx::query(&format!(
        "{} FROM vendor_products WHERE id = $1 AND vendor_id = $2",
        PRODUCT_SELECT
    ))
    .bind(product_id)
    .bind(vendor_id)
    .fetch_optional(&mut *rls_tx.tx)
    .await?;

    let row =
        row.ok_or_else(|| AppError::NotFound("Product not found or access denied".to_string()))?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "product": row_to_product_json(&row)?
    })))
}

// ─── GET EDIT CONTEXT: GET /vendor/products/:id/edit-context ────────────────

async fn get_edit_context(
    _auth: crate::middleware::auth::RequireVendor,
    mut rls_tx: RlsTx,
    axum::extract::Path(product_id): axum::extract::Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    let product_row = sqlx::query(&format!(
        "{} FROM vendor_products WHERE id = $1 AND vendor_id = $2",
        PRODUCT_SELECT
    ))
    .bind(product_id)
    .bind(vendor_id)
    .fetch_optional(&mut *rls_tx.tx)
    .await?;

    let product_row = product_row.ok_or_else(|| AppError::NotFound("Product not found".into()))?;
    let product_json = row_to_product_json(&product_row)?;

    let image_rows = sqlx::query(
        "SELECT id, product_id, image_url, file_path, is_cover, sort_order, caption, created_at,
                media_type, file_url, thumbnail_url, file_size, duration_seconds
         FROM vendor_gallery WHERE product_id = $1
         ORDER BY sort_order ASC, created_at ASC",
    )
    .bind(product_id)
    .fetch_all(&mut *rls_tx.tx)
    .await?;
    let images_json: Vec<Value> = image_rows.iter().map(row_to_gallery_json).collect();

    let vendor_category: Option<String> = sqlx::query_scalar("SELECT category FROM vendors WHERE id = $1")
        .bind(vendor_id)
        .fetch_optional(&mut *rls_tx.tx)
        .await?
        .flatten();

    let city_rows = sqlx::query("SELECT id, slug, name_ar, name_en FROM cities ORDER BY name_en ASC")
        .fetch_all(&mut *rls_tx.tx)
        .await?;

    let cities_json: Vec<Value> = city_rows.iter().map(|r| {
        let id: Uuid = r.get("id");
        let slug: String = r.get("slug");
        let name_ar: String = r.get("name_ar");
        let name_en: String = r.get("name_en");
        json!({ "id": id, "slug": slug, "name_ar": name_ar, "name_en": name_en })
    }).collect();

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "product": product_json,
        "images": images_json,
        "vendorCategory": vendor_category.unwrap_or_default(),
        "cities": cities_json
    })))
}


// ─── CREATE: POST /vendor/products ───────────────────────────────────────────

async fn create_product(
    axum::extract::State(_state): axum::extract::State<AppState>,
    _auth: crate::middleware::auth::RequireVendor,
    mut rls_tx: RlsTx,
    crate::utils::validation::ValidatedJson(payload): crate::utils::validation::ValidatedJson<CreateProductRequest>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    validate_product_payload(
        &payload.title_en,
        &payload.title_ar,
        &payload.title,
        &payload.base_price_sar,
        &payload.price_on_inquiry,
        &payload.city_id,
        &payload.description_en,
        &payload.description_ar,
        &payload.description,
        &payload.coordinator_name_en,
        &payload.coordinator_name_ar,
        &payload.coordinator_name,
        &payload.coordinator_phone,
        &payload.coordinator_whatsapp,
        &payload.coordinator_email,
    )?;

    crate::utils::policy::PolicyEngine::check_product_limit(vendor_id, None, &mut rls_tx.tx).await?;
    crate::utils::policy::PolicyEngine::check_description_limit(
        vendor_id,
        &payload.description_ar,
        &payload.description_en,
        &mut rls_tx.tx,
    )
    .await?;

    if let Some(ref items) = payload.gallery_items {
        let mut cover_count = 0;
        let mut photo_count = 0;
        let mut video_count = 0;
        for item in items {
            if item.is_cover.unwrap_or(false) {
                cover_count += 1;
            } else if item.media_type.as_deref() == Some("video") {
                video_count += 1;
            } else {
                photo_count += 1;
            }
        }
        crate::utils::policy::PolicyEngine::check_gallery_batch_limit(
            vendor_id,
            cover_count,
            photo_count,
            video_count,
            &mut rls_tx.tx,
        )
        .await?;
    }

    // Resolve effective title: prefer bilingual, fall back to legacy `title`
    let raw_title_en = payload
        .title_en
        .clone()
        .or_else(|| payload.title.clone())
        .unwrap_or_default();
    let raw_title_ar = payload
        .title_ar
        .clone()
        .or_else(|| payload.title.clone())
        .unwrap_or_default();

    if let Some(ref map_url) = payload.google_maps_url {
        validate_google_maps_url(map_url)?;
    }

    let product_category = payload
        .product_category
        .as_deref()
        .unwrap_or("wedding-palace");

    if let Some(ref cg) = payload.coordinator_gender {
        if !VALID_COORDINATOR_GENDERS.contains(&cg.as_str()) {
            return Err(AppError::BadRequest(
                "Invalid coordinator_gender".to_string(),
            ));
        }
    }
    if let Some(price) = payload.base_price_sar {
        if price <= 0.0 {
            return Err(AppError::BadRequest(
                "base_price_sar must be > 0".to_string(),
            ));
        }
    }
    if payload.base_price_sar.is_none() && payload.price_on_inquiry != Some(true) {
        // Allow missing price — it just shows as TBD on the listing card
    }
    let deposit_pct = payload.deposit_percentage.unwrap_or(25);
    if !(10..=100).contains(&deposit_pct) {
        return Err(AppError::BadRequest(
            "deposit_percentage must be 10–100".to_string(),
        ));
    }

    // Validate gender_section value
    let valid_gender_sections = [
        "women_only",
        "men_only",
        "mixed",
        "dual_parallel",
        "family",
        "both_sections",
        "not_applicable",
    ];
    if let Some(ref gs) = payload.gender_section {
        if !valid_gender_sections.contains(&gs.as_str()) {
            return Err(AppError::BadRequest(format!(
                "Invalid gender_section: {}",
                gs
            )));
        }
    }

    let clean_title_en = sanitize_str(&raw_title_en, limits::NAME_LONG);
    let clean_title_ar = sanitize_str(&raw_title_ar, limits::NAME_LONG);
    let clean_desc_ar = sanitize_opt(
        &payload
            .description_ar
            .or_else(|| payload.description.clone()),
        limits::DESCRIPTION,
    );
    let clean_desc_en = sanitize_opt(
        &payload
            .description_en
            .or_else(|| payload.description.clone()),
        limits::DESCRIPTION,
    );
    // legacy column, stopped writing as of this change, column kept for backward read compatibility until confirmed unused elsewhere in the codebase.
    let clean_title_legacy: Option<String> = Some(clean_title_en.clone());
    // legacy column, stopped writing as of this change, column kept for backward read compatibility until confirmed unused elsewhere in the codebase.
    let clean_desc_legacy: Option<String> = None;

    let clean_coord_name_en = sanitize_opt(
        &payload
            .coordinator_name_en
            .or_else(|| payload.coordinator_name.clone()),
        limits::NAME_SHORT,
    );
    let clean_coord_name_ar = sanitize_opt(&payload.coordinator_name_ar, limits::NAME_SHORT);
    // legacy column, stopped writing as of this change, column kept for backward read compatibility until confirmed unused elsewhere in the codebase.
    let clean_coord_name_legacy: Option<String> = None;

    let clean_coord_phone = sanitize_opt(&payload.coordinator_phone, limits::PHONE);
    let clean_coord_whatsapp = sanitize_opt(&payload.coordinator_whatsapp, limits::PHONE);
    let clean_coord_email = sanitize_opt(&payload.coordinator_email, limits::EMAIL);
    let clean_coord_mobile = sanitize_opt(&payload.coordinator_mobile, limits::PHONE);
    let clean_crm_id = sanitize_opt(&payload.crm_product_id, limits::NAME_SHORT);

    let clean_meta_title_ar = sanitize_opt(&payload.meta_title_ar, limits::NAME_LONG);
    let clean_meta_title_en = sanitize_opt(&payload.meta_title_en, limits::NAME_LONG);
    let clean_meta_desc_ar = sanitize_opt(&payload.meta_description_ar, limits::DESCRIPTION);
    let clean_meta_desc_en = sanitize_opt(&payload.meta_description_en, limits::DESCRIPTION);

    // Slug is generated from English title (URL-safe)
    let base_slug = if clean_title_en.is_empty() {
        format!("draft-{}", uuid::Uuid::new_v4())
    } else {
        generate_product_slug(&clean_title_en)
    };
    let slug = {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM vendor_products WHERE slug LIKE $1")
                .bind(format!("{}%", base_slug))
                .fetch_one(&mut *rls_tx.tx)
                .await?;
        if count == 0 {
            base_slug
        } else {
            format!("{}-{}", base_slug, count + 1)
        }
    };

    if let Some(ref cat) = payload.vendor_category {
        sqlx::query("UPDATE vendors SET category = $1, updated_at = NOW() WHERE id = $2")
            .bind(cat)
            .bind(vendor_id)
            .execute(&mut *rls_tx.tx)
            .await?;
    }

    let new_id = uuid::Uuid::new_v4();
    let attrs = payload.attributes.unwrap_or_else(|| serde_json::json!({}));
    let price_on_inquiry = payload.price_on_inquiry.unwrap_or(false);

    let features_selection = payload.features_selection.unwrap_or_else(|| json!({}));
    if let Some(obj) = features_selection.as_object() {
        let keys: Vec<Uuid> = obj.keys().filter_map(|k| Uuid::parse_str(k).ok()).collect();

        if !keys.is_empty() {
            let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM features WHERE id = ANY($1)")
                .bind(&keys[..])
                .fetch_one(&mut *rls_tx.tx)
                .await?;

            if count as usize != keys.len() {
                return Err(AppError::BadRequest(
                    "One or more invalid feature IDs provided".to_string(),
                ));
            }
        }
    } else if !features_selection.is_null() {
        return Err(AppError::BadRequest(
            "features_selection must be an object".to_string(),
        ));
    }

    let cultural_attrs = payload.cultural_attributes.clone().unwrap_or_else(|| serde_json::json!({}));
    let computed_total_capacity = compute_total_capacity(&attrs);
    let computed_searchable_amenities = compute_searchable_amenities(&attrs, &cultural_attrs, &features_selection);

    sqlx::query(
        "INSERT INTO vendor_products (
            id, vendor_id,
            title, title_ar, title_en,
            description, description_ar, description_en,
            slug, product_category,
            base_price_sar, deposit_percentage, price_on_inquiry,
            gender_section,
            coordinator_name, coordinator_name_ar, coordinator_name_en,
            coordinator_phone, coordinator_whatsapp,
            coordinator_avatar, coordinator_gender,
            crm_product_id,
            status, is_available, is_featured, version,
            city_id, attributes,
            google_maps_url, latitude, longitude,
            features_selection,
            meta_title_ar, meta_title_en, meta_description_ar, meta_description_en,
            coordinator_email, coordinator_mobile,
            cultural_attributes,
            total_capacity, searchable_amenities
        ) VALUES (
            $1, $2,
            $3, $4, $5,
            $6, $7, $8,
            $9, $10,
            $11::numeric, $12, $13,
            $14,
            $15, $16, $17,
            $18, $19,
            $20, $21,
            $22,
            'draft', true, false, 1,
            $23, $24,
            $25, $26, $27,
            $28,
            $29, $30, $31, $32,
            $33, $34,
            $35,
            $36, $37
        )",
    )
    .bind(new_id)
    .bind(vendor_id)
    .bind(&clean_title_legacy)
    .bind(&clean_title_ar)
    .bind(&clean_title_en)
    .bind(&clean_desc_legacy)
    .bind(&clean_desc_ar)
    .bind(&clean_desc_en)
    .bind(&slug)
    .bind(product_category)
    .bind(payload.base_price_sar)
    .bind(deposit_pct)
    .bind(price_on_inquiry)
    .bind(&payload.gender_section)
    .bind(&clean_coord_name_legacy)
    .bind(&clean_coord_name_ar)
    .bind(&clean_coord_name_en)
    .bind(&clean_coord_phone)
    .bind(&clean_coord_whatsapp)
    .bind(&payload.coordinator_avatar)
    .bind(
        payload
            .coordinator_gender
            .clone()
            .unwrap_or_else(|| "any".to_string()),
    )
    .bind(&clean_crm_id)
    .bind(payload.city_id)
    .bind(&attrs)
    .bind(&payload.google_maps_url)
    .bind(payload.latitude)
    .bind(payload.longitude)
    .bind(&features_selection)
    .bind(&clean_meta_title_ar)
    .bind(&clean_meta_title_en)
    .bind(&clean_meta_desc_ar)
    .bind(&clean_meta_desc_en)
    .bind(&clean_coord_email)
    .bind(&clean_coord_mobile)
    .bind(&cultural_attrs)
    .bind(computed_total_capacity)
    .bind(&computed_searchable_amenities)
    .execute(&mut *rls_tx.tx)
    .await
    .map_err(crate::errors::map_db_error)?;

    if let Some(items) = payload.gallery_items {
        for (idx, item) in items.iter().enumerate() {
            let item_id = Uuid::new_v4();
            let is_cover = item.is_cover.unwrap_or(false);
            let sort_order = item.sort_order.unwrap_or(idx as i32);
            let media_type = item
                .media_type
                .clone()
                .unwrap_or_else(|| "image".to_string());
            let file_url = item.image_url.clone();

            sqlx::query(
                "INSERT INTO vendor_gallery (
                    id, vendor_id, product_id, image_url, file_path, is_cover, sort_order, caption,
                    media_type, file_url, thumbnail_url, file_size, duration_seconds
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
            )
            .bind(item_id)
            .bind(vendor_id)
            .bind(new_id)
            .bind(&file_url)
            .bind(&item.file_path)
            .bind(is_cover)
            .bind(sort_order)
            .bind(&item.caption)
            .bind(&media_type)
            .bind(&file_url)
            .bind(&item.thumbnail_url)
            .bind(item.file_size)
            .bind(item.duration_seconds)
            .execute(&mut *rls_tx.tx)
            .await
            .map_err(crate::errors::map_db_error)?;
        }
    }

    rls_tx.tx.commit().await?;
    crate::services::metrics::inc_listing_create();
    Ok(Json(json!({
        "status": "success",
        "message": "Listing created successfully",
        "productId": new_id.to_string(),
        "id": new_id.to_string(),
        "slug": slug,
        "product": {
            "id": new_id.to_string(),
            "version": 1
        }
    })))
}

// ─── UPDATE: PUT /vendor/products/:id ────────────────────────────────────────

async fn update_product(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    _auth: crate::middleware::auth::RequireVendor,
    mut rls_tx: RlsTx,
    axum::extract::Path(product_id): axum::extract::Path<uuid::Uuid>,
    crate::utils::validation::ValidatedJson(payload): crate::utils::validation::ValidatedJson<UpdateProductRequest>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    validate_product_payload(
        &payload.title_en,
        &payload.title_ar,
        &payload.title,
        &payload.base_price_sar,
        &payload.price_on_inquiry,
        &payload.city_id,
        &payload.description_en,
        &payload.description_ar,
        &payload.description,
        &payload.coordinator_name_en,
        &payload.coordinator_name_ar,
        &payload.coordinator_name,
        &payload.coordinator_phone,
        &payload.coordinator_whatsapp,
        &payload.coordinator_email,
    )?;

    crate::utils::policy::PolicyEngine::check_description_limit(
        vendor_id,
        &payload.description_ar,
        &payload.description_en,
        &mut rls_tx.tx,
    )
    .await?;

    if let Some(ref items) = payload.gallery_items {
        let mut cover_count = 0;
        let mut photo_count = 0;
        let mut video_count = 0;
        for item in items {
            if item.is_cover.unwrap_or(false) {
                cover_count += 1;
            } else if item.media_type.as_deref() == Some("video") {
                video_count += 1;
            } else {
                photo_count += 1;
            }
        }
        crate::utils::policy::PolicyEngine::check_gallery_batch_limit(
            vendor_id,
            cover_count,
            photo_count,
            video_count,
            &mut rls_tx.tx,
        )
        .await?;
    }

    // Resolve effective titles from bilingual or legacy field
    let raw_title_en = payload.title_en.clone().or_else(|| payload.title.clone());
    let raw_title_ar = payload.title_ar.clone().or_else(|| payload.title.clone());

    if let Some(ref en) = raw_title_en {
        if en.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Product title cannot be empty".to_string(),
            ));
        }
    }
    if let Some(ref ar) = raw_title_ar {
        if ar.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Product title cannot be empty".to_string(),
            ));
        }
    }

    if let Some(ref map_url) = payload.google_maps_url {
        validate_google_maps_url(map_url)?;
    }

    if let Some(ref cg) = payload.coordinator_gender {
        if !VALID_COORDINATOR_GENDERS.contains(&cg.as_str()) {
            return Err(AppError::BadRequest(
                "Invalid coordinator_gender".to_string(),
            ));
        }
    }

    if let Some(price) = payload.base_price_sar {
        if price <= 0.0 {
            return Err(AppError::BadRequest(
                "base_price_sar must be > 0".to_string(),
            ));
        }
    }

    if let Some(deposit_pct) = payload.deposit_percentage {
        if !(10..=100).contains(&deposit_pct) {
            return Err(AppError::BadRequest(
                "deposit_percentage must be 10–100".to_string(),
            ));
        }
    }

    let valid_gender_sections = [
        "women_only",
        "men_only",
        "mixed",
        "dual_parallel",
        "family",
        "both_sections",
        "not_applicable",
    ];
    if let Some(ref gs) = payload.gender_section {
        if !valid_gender_sections.contains(&gs.as_str()) {
            return Err(AppError::BadRequest(format!(
                "Invalid gender_section: {}",
                gs
            )));
        }
    }

    let clean_title_en = raw_title_en
        .as_deref()
        .map(|s| sanitize_str(s, limits::NAME_LONG));
    let clean_title_ar = raw_title_ar
        .as_deref()
        .map(|s| sanitize_str(s, limits::NAME_LONG));
    let clean_desc_ar = sanitize_opt(
        &payload
            .description_ar
            .or_else(|| payload.description.clone()),
        limits::DESCRIPTION,
    );
    let clean_desc_en = sanitize_opt(
        &payload
            .description_en
            .or_else(|| payload.description.clone()),
        limits::DESCRIPTION,
    );
    let clean_coord_name_en = sanitize_opt(
        &payload
            .coordinator_name_en
            .or_else(|| payload.coordinator_name.clone()),
        limits::NAME_SHORT,
    );
    let clean_coord_name_ar = sanitize_opt(&payload.coordinator_name_ar, limits::NAME_SHORT);
    
    // legacy column, stopped writing as of this change, column kept for backward read compatibility until confirmed unused elsewhere in the codebase.
    let clean_title_legacy: Option<String> = clean_title_en.clone();
    // legacy column, stopped writing as of this change, column kept for backward read compatibility until confirmed unused elsewhere in the codebase.
    let clean_desc_legacy: Option<String> = None;
    // legacy column, stopped writing as of this change, column kept for backward read compatibility until confirmed unused elsewhere in the codebase.
    let clean_coord_name_legacy: Option<String> = None;

    let clean_coord_phone = sanitize_opt(&payload.coordinator_phone, limits::PHONE);
    let clean_coord_whatsapp = sanitize_opt(&payload.coordinator_whatsapp, limits::PHONE);
    let clean_coord_email = sanitize_opt(&payload.coordinator_email, limits::EMAIL);
    let clean_coord_mobile = sanitize_opt(&payload.coordinator_mobile, limits::PHONE);
    let clean_crm_id = sanitize_opt(&payload.crm_product_id, limits::NAME_SHORT);

    let clean_meta_title_ar = sanitize_opt(&payload.meta_title_ar, limits::NAME_LONG);
    let clean_meta_title_en = sanitize_opt(&payload.meta_title_en, limits::NAME_LONG);
    let clean_meta_desc_ar = sanitize_opt(&payload.meta_description_ar, limits::DESCRIPTION);
    let clean_meta_desc_en = sanitize_opt(&payload.meta_description_en, limits::DESCRIPTION);

    if let Some(ref cat) = payload.vendor_category {
        sqlx::query("UPDATE vendors SET category = $1, updated_at = NOW() WHERE id = $2")
            .bind(cat)
            .bind(vendor_id)
            .execute(&mut *rls_tx.tx)
            .await?;
    }

    if let Some(ref features) = payload.features_selection {
        if let Some(obj) = features.as_object() {
            let keys: Vec<Uuid> = obj.keys().filter_map(|k| Uuid::parse_str(k).ok()).collect();

            if !keys.is_empty() {
                let count: i64 =
                    sqlx::query_scalar("SELECT COUNT(*) FROM features WHERE id = ANY($1)")
                        .bind(&keys[..])
                        .fetch_one(&mut *rls_tx.tx)
                        .await?;

                if count as usize != keys.len() {
                    return Err(AppError::BadRequest(
                        "One or more invalid feature IDs provided".to_string(),
                    ));
                }
            }
        } else if !features.is_null() {
            return Err(AppError::BadRequest(
                "features_selection must be an object".to_string(),
            ));
        }
    }

    let existing_row: Option<(i32, serde_json::Value, serde_json::Value, serde_json::Value)> = sqlx::query_as(
        "SELECT version, attributes, cultural_attributes, features_selection FROM vendor_products WHERE id = $1 AND vendor_id = $2"
    )
    .bind(product_id)
    .bind(vendor_id)
    .fetch_optional(&mut *rls_tx.tx)
    .await?;

    let (db_version, db_attributes, db_cultural, db_features) = match existing_row {
        Some(row) => row,
        None => return Err(AppError::NotFound("Product not found or access denied".to_string())),
    };

    if db_version != payload.version {
        tracing::warn!(
            "Version conflict for product {}. DB version={}, client version={}",
            product_id, db_version, payload.version
        );
        return Err(AppError::Conflict(
            "Version conflict: The listing was updated elsewhere. Please refresh and try again.".to_string()
        ));
    }

    let eff_attributes = payload.attributes.clone().unwrap_or(db_attributes);
    let eff_cultural = payload.cultural_attributes.clone().unwrap_or(db_cultural);
    let eff_features = payload.features_selection.clone().unwrap_or(db_features);

    let computed_total_capacity = compute_total_capacity(&eff_attributes);
    let computed_searchable_amenities = compute_searchable_amenities(&eff_attributes, &eff_cultural, &eff_features);

    let rows_affected = sqlx::query(
        "UPDATE vendor_products SET
            -- Legacy single-lang (English)
            title           = COALESCE($1, title),
            description     = COALESCE($2, description),
            -- V2 bilingual
            title_ar        = COALESCE($3, title_ar),
            title_en        = COALESCE($32, title_en),
            description_ar  = COALESCE($4, description_ar),
            description_en  = COALESCE($33, description_en),
            product_category = COALESCE($5, product_category),
            base_price_sar  = COALESCE($6::numeric, base_price_sar),
            deposit_percentage = COALESCE($7, deposit_percentage),
            price_on_inquiry = COALESCE($8, price_on_inquiry),
            gender_section  = COALESCE($9, gender_section),
            coordinator_name    = COALESCE($10, coordinator_name),
            coordinator_name_ar = COALESCE($11, coordinator_name_ar),
            coordinator_name_en = COALESCE($34, coordinator_name_en),
            coordinator_phone   = COALESCE($12, coordinator_phone),
            coordinator_whatsapp = COALESCE($13, coordinator_whatsapp),
            coordinator_avatar  = COALESCE($14, coordinator_avatar),
            coordinator_gender  = COALESCE($15, coordinator_gender),
            crm_product_id  = COALESCE($16, crm_product_id),
            city_id         = COALESCE($17, city_id),
            attributes      = COALESCE($18, attributes),
            google_maps_url = COALESCE($22, google_maps_url),
            latitude        = COALESCE($23, latitude),
            longitude       = COALESCE($24, longitude),
            features_selection = COALESCE($25, features_selection),
            meta_title_ar   = COALESCE($26, meta_title_ar),
            meta_title_en   = COALESCE($27, meta_title_en),
            meta_description_ar = COALESCE($28, meta_description_ar),
            meta_description_en = COALESCE($29, meta_description_en),
            coordinator_email   = COALESCE($30, coordinator_email),
            coordinator_mobile  = COALESCE($31, coordinator_mobile),
            cultural_attributes = COALESCE($35, cultural_attributes),
            total_capacity      = $36,
            searchable_amenities = $37,
            version         = version + 1,
            updated_at      = CURRENT_TIMESTAMP
          WHERE id = $19 AND vendor_id = $20 AND version = $21",
    )
    .bind(&clean_title_legacy) // $1  legacy title
    .bind(&clean_desc_legacy) // $2  legacy description
    .bind(&clean_title_ar) // $3  title_ar
    .bind(&clean_desc_ar) // $4  description_ar
    .bind(&payload.product_category) // $5
    .bind(payload.base_price_sar) // $6
    .bind(payload.deposit_percentage) // $7
    .bind(payload.price_on_inquiry) // $8
    .bind(&payload.gender_section) // $9
    .bind(&clean_coord_name_legacy) // $10 legacy coordinator_name
    .bind(&clean_coord_name_ar) // $11 coordinator_name_ar
    .bind(&clean_coord_phone) // $12
    .bind(&clean_coord_whatsapp) // $13
    .bind(&payload.coordinator_avatar) // $14
    .bind(&payload.coordinator_gender) // $15
    .bind(&clean_crm_id) // $16
    .bind(payload.city_id) // $17
    .bind(&payload.attributes) // $18
    .bind(product_id) // $19
    .bind(vendor_id) // $20
    .bind(payload.version) // $21
    .bind(&payload.google_maps_url) // $22
    .bind(payload.latitude) // $23
    .bind(payload.longitude) // $24
    .bind(&payload.features_selection) // $25
    .bind(&clean_meta_title_ar) // $26
    .bind(&clean_meta_title_en) // $27
    .bind(&clean_meta_desc_ar) // $28
    .bind(&clean_meta_desc_en) // $29
    .bind(&clean_coord_email) // $30
    .bind(&clean_coord_mobile) // $31
    .bind(&clean_title_en) // $32
    .bind(&clean_desc_en) // $33
    .bind(&clean_coord_name_en) // $34
    .bind(&payload.cultural_attributes) // $35
    .bind(computed_total_capacity) // $36
    .bind(&computed_searchable_amenities) // $37
    .execute(&mut *rls_tx.tx)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound("Product not found or access denied".to_string()));
    }

    if let Some(ref items) = payload.gallery_items {
        // 1. Fetch old file paths to see what to delete from disk and MinIO
        let old_items: Vec<(String, Option<String>, String)> =
            sqlx::query_as("SELECT file_url, file_path, media_type FROM vendor_gallery WHERE product_id = $1")
                .bind(product_id)
                .fetch_all(&mut *rls_tx.tx)
                .await?;

        // 2. Delete all gallery items from DB for this product
        sqlx::query("DELETE FROM vendor_gallery WHERE product_id = $1")
            .bind(product_id)
            .execute(&mut *rls_tx.tx)
            .await?;

        // 3. Insert the new items
        for (idx, item) in items.iter().enumerate() {
            let item_id = Uuid::new_v4();
            let is_cover = item.is_cover.unwrap_or(false);
            let sort_order = item.sort_order.unwrap_or(idx as i32);
            let media_type = item
                .media_type
                .clone()
                .unwrap_or_else(|| "image".to_string());
            let file_url = item.image_url.clone();

            sqlx::query(
                "INSERT INTO vendor_gallery (
                    id, vendor_id, product_id, image_url, file_path, is_cover, sort_order, caption,
                    media_type, file_url, thumbnail_url, file_size, duration_seconds
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
            )
            .bind(item_id)
            .bind(vendor_id)
            .bind(product_id)
            .bind(&file_url)
            .bind(&item.file_path)
            .bind(is_cover)
            .bind(sort_order)
            .bind(&item.caption)
            .bind(&media_type)
            .bind(&file_url)
            .bind(&item.thumbnail_url)
            .bind(item.file_size)
            .bind(item.duration_seconds)
            .execute(&mut *rls_tx.tx)
            .await?;
        }

        // 4. Delete physical files from disk & MinIO that are no longer referenced
        let new_urls: std::collections::HashSet<&str> =
            items.iter().map(|item| item.image_url.as_str()).collect();
        for (old_url, old_path, old_media_type) in old_items {
            if !new_urls.contains(old_url.as_str()) {
                if let Some(path) = old_path {
                    if let Err(e) = tokio::fs::remove_file(&path).await {
                        tracing::warn!("Could not delete orphaned gallery file '{}': {}", path, e);
                    }
                }
                state.minio_client.delete_gallery_item(&old_url, &old_media_type).await;
            }
        }
    }

    rls_tx.tx.commit().await?;
    crate::services::metrics::inc_listing_edit();
    Ok(Json(json!({
        "status": "success",
        "message": "Listing updated successfully",
        "product": {
            "titleEn": clean_title_en,
            "titleAr": clean_title_ar,
            "version": payload.version + 1
        }
    })))
}

// ─── DELETE: DELETE /vendor/products/:id ─────────────────────────────────────
// Soft-delete: archive the product. Gallery CASCADE is handled by DB ON DELETE CASCADE,
// but we collect file paths first so we can delete physical files from disk.

async fn delete_product(
    _auth: crate::middleware::auth::RequireVendor,
    mut rls_tx: RlsTx,
    axum::extract::Path(product_id): axum::extract::Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    // Always Soft-delete: archive to preserve product/booking history
    let result = sqlx::query(
        "UPDATE vendor_products
         SET status = 'archived', is_available = FALSE, updated_at = CURRENT_TIMESTAMP
         WHERE id = $1 AND vendor_id = $2 AND status != 'archived'",
    )
    .bind(product_id)
    .bind(vendor_id)
    .execute(&mut *rls_tx.tx)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Product not found, access denied, or already archived".to_string(),
        ));
    }

    rls_tx.tx.commit().await?;

    Ok(Json(
        json!({ "status": "success", "message": "Listing archived successfully" }),
    ))
}

// ─── STATUS PATCH: PATCH /vendor/products/:id/status ─────────────────────────

async fn update_product_status(
    _auth: crate::middleware::auth::RequireVendor,
    mut rls_tx: RlsTx,
    axum::extract::Path(product_id): axum::extract::Path<Uuid>,
    crate::utils::validation::ValidatedJson(payload): crate::utils::validation::ValidatedJson<UpdateProductStatusRequest>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    let vendor_allowed_statuses = ["draft", "pending_approval", "archived"];
    if !vendor_allowed_statuses.contains(&payload.status.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Vendors can only set status to: {:?}. Contact admin to activate.",
            vendor_allowed_statuses
        )));
    }

    if payload.status == "pending_approval" || payload.status == "active" {
        if let Err(e) = crate::utils::policy::PolicyEngine::check_product_limit(vendor_id, Some(product_id), &mut rls_tx.tx).await {
            // Count quota blocks only for 402-class policy rejections
            if matches!(e, AppError::PaymentRequired(_, _)) {
                crate::services::metrics::inc_quota_block();
            }
            return Err(e);
        }
    }

    let rows_affected = sqlx::query(
        "UPDATE vendor_products SET status = $1, updated_at = CURRENT_TIMESTAMP
         WHERE id = $2 AND vendor_id = $3",
    )
    .bind(&payload.status)
    .bind(product_id)
    .bind(vendor_id)
    .execute(&mut *rls_tx.tx)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound(
            "Product not found or access denied".to_string(),
        ));
    }

    rls_tx.tx.commit().await?;

    if payload.status == "pending_approval" {
        crate::services::metrics::inc_listing_submit();
    }

    Ok(Json(json!({
        "status": "success",
        "message": format!("Listing status updated to '{}'", payload.status)
    })))
}

async fn archive_product(
    _auth: crate::middleware::auth::RequireVendor,
    mut rls_tx: RlsTx,
    axum::extract::Path(product_id): axum::extract::Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    let rows_affected = sqlx::query(
        "UPDATE vendor_products 
         SET status = 'archived', is_available = FALSE, updated_at = CURRENT_TIMESTAMP
         WHERE id = $1 AND vendor_id = $2 AND status != 'archived'",
    )
    .bind(product_id)
    .bind(vendor_id)
    .execute(&mut *rls_tx.tx)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound(
            "Product not found, access denied, or already archived".to_string(),
        ));
    }

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Product archived successfully"
    })))
}

async fn restore_product(
    _auth: crate::middleware::auth::RequireVendor,
    mut rls_tx: RlsTx,
    axum::extract::Path(product_id): axum::extract::Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    let rows_affected = sqlx::query(
        "UPDATE vendor_products 
         SET status = 'draft', is_available = TRUE, updated_at = CURRENT_TIMESTAMP
         WHERE id = $1 AND vendor_id = $2 AND status = 'archived'",
    )
    .bind(product_id)
    .bind(vendor_id)
    .execute(&mut *rls_tx.tx)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound(
            "Product not found, access denied, or not currently archived".to_string(),
        ));
    }

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Product restored to draft successfully"
    })))
}

// ─── AVAILABILITY TOGGLE: PATCH /vendor/products/:id/availability ─────────────

async fn toggle_product_availability(
    _auth: crate::middleware::auth::RequireVendor,
    mut rls_tx: RlsTx,
    axum::extract::Path(product_id): axum::extract::Path<Uuid>,
    Json(payload): Json<ToggleAvailabilityRequest>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    let rows_affected = sqlx::query(
        "UPDATE vendor_products SET is_available = $1, updated_at = CURRENT_TIMESTAMP
         WHERE id = $2 AND vendor_id = $3",
    )
    .bind(payload.is_available)
    .bind(product_id)
    .bind(vendor_id)
    .execute(&mut *rls_tx.tx)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound(
            "Product not found or access denied".to_string(),
        ));
    }

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": format!("Listing marked as {}", if payload.is_available { "available" } else { "unavailable" }),
        "isAvailable": payload.is_available
    })))
}

// ═══════════════════════════════════════════════════════════════════════════════
// LISTING-SCOPED GALLERY ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

// ─── LIST IMAGES: GET /vendor/products/:id/images ────────────────────────────

async fn list_product_images(
    _auth: crate::middleware::auth::RequireVendor,
    mut rls_tx: RlsTx,
    axum::extract::Path(product_id): axum::extract::Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    // Verify the product belongs to this vendor
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM vendor_products WHERE id = $1 AND vendor_id = $2)",
    )
    .bind(product_id)
    .bind(vendor_id)
    .fetch_one(&mut *rls_tx.tx)
    .await?;

    if !exists {
        return Err(AppError::NotFound(
            "Product not found or access denied".to_string(),
        ));
    }

    let rows = sqlx::query(
        "SELECT id, product_id, image_url, file_path, is_cover, sort_order, caption, created_at,
                media_type, file_url, thumbnail_url, file_size, duration_seconds
         FROM vendor_gallery
         WHERE product_id = $1
         ORDER BY sort_order ASC, created_at ASC",
    )
    .bind(product_id)
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let images: Vec<Value> = rows.iter().map(row_to_gallery_json).collect();

    rls_tx.tx.commit().await?;

    Ok(Json(
        json!({ "status": "success", "images": images, "total": images.len() }),
    ))
}

// ─── ADD IMAGE: POST /vendor/products/:id/images ─────────────────────────────

// All fields use snake_case internally; serde maps camelCase JSON keys from the frontend.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddProductImageRequest {
    image_url: String,
    file_path: Option<String>,
    is_cover: Option<bool>,
    caption: Option<String>,
    sort_order: Option<i32>,
    media_type: Option<String>,
    thumbnail_url: Option<String>,
    file_size: Option<i64>,
    duration_seconds: Option<i32>,
}

async fn add_product_image(
    _auth: crate::middleware::auth::RequireVendor,
    mut rls_tx: RlsTx,
    axum::extract::Path(product_id): axum::extract::Path<Uuid>,
    Json(payload): Json<AddProductImageRequest>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    // Verify ownership
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM vendor_products WHERE id = $1 AND vendor_id = $2)",
    )
    .bind(product_id)
    .bind(vendor_id)
    .fetch_one(&mut *rls_tx.tx)
    .await?;

    if !exists {
        return Err(AppError::NotFound(
            "Product not found or access denied".to_string(),
        ));
    }

    if payload.image_url.trim().is_empty() {
        return Err(AppError::BadRequest("image_url is required".to_string()));
    }

    crate::utils::policy::PolicyEngine::check_media_limit(
        vendor_id,
        Some(product_id),
        payload.media_type.as_deref() == Some("video"),
        payload.is_cover.unwrap_or(false),
        &mut rls_tx.tx,
    )
    .await?;

    let is_cover = payload.is_cover.unwrap_or(false);
    let sort_order = payload.sort_order.unwrap_or(0);
    let media_type = payload
        .media_type
        .clone()
        .unwrap_or_else(|| "image".to_string());
    let file_url = payload.image_url.clone();
    let new_id = Uuid::new_v4();

    // If setting as cover, clear any existing cover for this product first
    if is_cover {
        sqlx::query(
            "UPDATE vendor_gallery SET is_cover = FALSE WHERE product_id = $1 AND is_cover = TRUE",
        )
        .bind(product_id)
        .execute(&mut *rls_tx.tx)
        .await?;
    }

    sqlx::query(
        "INSERT INTO vendor_gallery (
            id, vendor_id, product_id, image_url, file_path, is_cover, sort_order, caption,
            media_type, file_url, thumbnail_url, file_size, duration_seconds
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
    )
    .bind(new_id)
    .bind(vendor_id)
    .bind(product_id)
    .bind(&payload.image_url)
    .bind(&payload.file_path)
    .bind(is_cover)
    .bind(sort_order)
    .bind(&payload.caption)
    .bind(&media_type)
    .bind(&file_url)
    .bind(&payload.thumbnail_url)
    .bind(payload.file_size)
    .bind(payload.duration_seconds)
    .execute(&mut *rls_tx.tx)
    .await
    .map_err(crate::errors::map_db_error)?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Image added successfully",
        "id": new_id.to_string()
    })))
}

// ─── SET COVER: PATCH /vendor/products/:id/images/:img_id/cover ──────────────

async fn set_product_cover_image(
    _auth: crate::middleware::auth::RequireVendor,
    mut rls_tx: RlsTx,
    axum::extract::Path((product_id, img_id)): axum::extract::Path<(Uuid, Uuid)>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    // Verify ownership of image
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM vendor_gallery g
            JOIN vendor_products p ON g.product_id = p.id
            WHERE g.id = $1 AND g.product_id = $2 AND p.vendor_id = $3
         )",
    )
    .bind(img_id)
    .bind(product_id)
    .bind(vendor_id)
    .fetch_one(&mut *rls_tx.tx)
    .await?;

    if !exists {
        return Err(AppError::NotFound(
            "Image not found or access denied".to_string(),
        ));
    }

    // Clear existing cover for this product
    sqlx::query(
        "UPDATE vendor_gallery SET is_cover = FALSE WHERE product_id = $1 AND is_cover = TRUE",
    )
    .bind(product_id)
    .execute(&mut *rls_tx.tx)
    .await?;

    // Set new cover
    sqlx::query("UPDATE vendor_gallery SET is_cover = TRUE WHERE id = $1")
        .bind(img_id)
        .execute(&mut *rls_tx.tx)
        .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(
        json!({ "status": "success", "message": "Cover image updated" }),
    ))
}

// ─── DELETE IMAGE: DELETE /vendor/products/:id/images/:img_id ────────────────

async fn delete_product_image(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
    _auth: crate::middleware::auth::RequireVendor,
    mut rls_tx: RlsTx,
    axum::extract::Path((product_id, img_id)): axum::extract::Path<(Uuid, Uuid)>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    // Fetch file details BEFORE deleting (so we can remove the physical files from disk & MinIO)
    let file_info: Option<(Option<String>, String, String)> = sqlx::query_as(
        "SELECT g.file_path, g.image_url, g.media_type FROM vendor_gallery g
         JOIN vendor_products p ON g.product_id = p.id
         WHERE g.id = $1 AND g.product_id = $2 AND p.vendor_id = $3",
    )
    .bind(img_id)
    .bind(product_id)
    .bind(vendor_id)
    .fetch_optional(&mut *rls_tx.tx)
    .await?;

    let rows_affected = sqlx::query(
        "DELETE FROM vendor_gallery g
         USING vendor_products p
         WHERE g.product_id = p.id
           AND g.id = $1 AND g.product_id = $2 AND p.vendor_id = $3",
    )
    .bind(img_id)
    .bind(product_id)
    .bind(vendor_id)
    .execute(&mut *rls_tx.tx)
    .await?
    .rows_affected();

    if rows_affected == 0 {
        return Err(AppError::NotFound(
            "Image not found or access denied".to_string(),
        ));
    }

    rls_tx.tx.commit().await?;

    // Delete physical file from disk & MinIO (non-blocking, best-effort)
    if let Some((file_path, image_url, media_type)) = file_info {
        if let Some(path) = file_path {
            if let Err(e) = tokio::fs::remove_file(&path).await {
                tracing::warn!("Could not delete gallery file '{}': {}", path, e);
            }
        }
        state.minio_client.delete_gallery_item(&image_url, &media_type).await;
    }

    Ok(Json(
        json!({ "status": "success", "message": "Image deleted successfully" }),
    ))
}

// ─── REORDER: PATCH /vendor/products/:id/images/reorder ──────────────────────

async fn reorder_product_images(
    _auth: crate::middleware::auth::RequireVendor,
    mut rls_tx: RlsTx,
    axum::extract::Path(product_id): axum::extract::Path<Uuid>,
    Json(payload): Json<ReorderImageRequest>,
) -> Result<Json<Value>, AppError> {
    let vendor_id = rls_tx.get_vendor_id().await?;

    // Verify ownership
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM vendor_products WHERE id = $1 AND vendor_id = $2)",
    )
    .bind(product_id)
    .bind(vendor_id)
    .fetch_one(&mut *rls_tx.tx)
    .await?;

    if !exists {
        return Err(AppError::NotFound(
            "Product not found or access denied".to_string(),
        ));
    }

    for item in &payload.images {
        sqlx::query(
            "UPDATE vendor_gallery SET sort_order = $1
             WHERE id = $2 AND product_id = $3",
        )
        .bind(item.sort_order)
        .bind(item.id)
        .bind(product_id)
        .execute(&mut *rls_tx.tx)
        .await?;
    }

    rls_tx.tx.commit().await?;

    Ok(Json(
        json!({ "status": "success", "message": "Gallery order updated" }),
    ))
}
