use crate::errors::AppError;
use sqlx::{PgPool, Row};
use serde_json::{json, Value};
use uuid::Uuid;
use crate::routes::public::VendorsQuery;

pub struct PgVendorRepository {
    db: PgPool,
}

impl PgVendorRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn list_public_vendors(&self, query: &VendorsQuery, country_id: &str) -> Result<Vec<Value>, AppError> {
        let mut query_str = "
            SELECT 
                v.id,
                v.name_ar,
                v.name_en,
                v.slug,
                v.category,
                v.status,
                c.slug AS city_slug,
                c.name_ar AS city_name_ar,
                c.name_en AS city_name_en,
                c.country_id,
                (v.is_featured = TRUE AND (v.featured_expires_at IS NULL OR v.featured_expires_at > CURRENT_TIMESTAMP)) AS active_featured,
                COALESCE(rev.quality_avg, 5.0) as quality_avg,
                COALESCE(rev.staff_avg, 5.0) as staff_avg,
                COALESCE(rev.communication_avg, 5.0) as communication_avg,
                COALESCE(rev.review_count, 0) as review_count,
                (SELECT image_url FROM vendor_gallery WHERE vendor_id = v.id AND is_cover = TRUE LIMIT 1) AS cover_image,
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
                ) as packages
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
            WHERE v.status = 'active' AND (c.country_id IS NULL OR c.country_id = $1)
        ".to_string();

        let mut param_idx = 2;
        let mut category_val = None;
        let mut city_val = None;
        let mut city_id_val = None;
        let mut partition_val = None;
        let mut min_cap_val = None;
        let mut max_cap_val = None;
        let mut amenities_val = None;

        if let Some(ref cat) = query.category {
            query_str.push_str(&format!(" AND v.category = ${}", param_idx));
            param_idx += 1;
            category_val = Some(cat);
        }

        if let Some(ref city_slug) = query.city {
            query_str.push_str(&format!(" AND c.slug = ${}", param_idx));
            param_idx += 1;
            city_val = Some(city_slug);
        }

        if let Some(ref city_id) = query.city_id {
            if Uuid::parse_str(city_id).is_ok() {
                query_str.push_str(&format!(" AND v.city_id = ${}", param_idx));
            } else {
                query_str.push_str(&format!(" AND c.slug = ${}", param_idx));
            }
            param_idx += 1;
            city_id_val = Some(city_id);
        }

        if let Some(partition) = query.partition {
            query_str.push_str(&format!(" AND v.has_partition = ${}", param_idx));
            param_idx += 1;
            partition_val = Some(partition);
        }

        if let Some(min_cap) = query.min_capacity {
            query_str.push_str(&format!(" AND v.capacity_max >= ${}", param_idx));
            param_idx += 1;
            min_cap_val = Some(min_cap);
        }

        if let Some(max_cap) = query.max_capacity {
            query_str.push_str(&format!(" AND v.capacity_min <= ${}", param_idx));
            param_idx += 1;
            max_cap_val = Some(max_cap);
        }

        if let Some(ref amenities_str) = query.amenities {
            let list: Vec<String> = amenities_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            query_str.push_str(&format!(" AND v.amenities @> ${}", param_idx));
            amenities_val = Some(list);
        }

        query_str.push_str(" ORDER BY (v.is_featured = TRUE AND (v.featured_expires_at IS NULL OR v.featured_expires_at > CURRENT_TIMESTAMP)) DESC, v.created_at DESC");

        let mut db_query = sqlx::query(&query_str).bind(country_id);
        if let Some(cat) = category_val {
            db_query = db_query.bind(cat);
        }
        if let Some(city_slug) = city_val {
            db_query = db_query.bind(city_slug);
        }
        if let Some(city_id) = city_id_val {
            if let Ok(parsed_uuid) = Uuid::parse_str(city_id) {
                db_query = db_query.bind(parsed_uuid);
            } else {
                db_query = db_query.bind(city_id);
            }
        }
        if let Some(partition) = partition_val {
            db_query = db_query.bind(partition);
        }
        if let Some(min_cap) = min_cap_val {
            db_query = db_query.bind(min_cap);
        }
        if let Some(max_cap) = max_cap_val {
            db_query = db_query.bind(max_cap);
        }
        if let Some(ref amenities_list) = amenities_val {
            db_query = db_query.bind(amenities_list);
        }

        let rows = db_query.fetch_all(&self.db).await?;
        let mut vendors_json = Vec::with_capacity(rows.len());

        for row in rows {
            let id: Uuid = row.get("id");
            let name_ar: String = row.get("name_ar");
            let name_en: String = row.get("name_en");
            let slug: String = row.get("slug");
            let category: Option<String> = row.get("category");
            let category = category.unwrap_or_default();
            let active_featured: bool = row.get("active_featured");
            let cover_image: Option<String> = row.get("cover_image");

            let city_slug: Option<String> = row.get("city_slug");
            let city_name_ar: Option<String> = row.get("city_name_ar");
            let city_name_en: Option<String> = row.get("city_name_en");
            let row_country_id: Option<String> = row.get("country_id");

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

            let country_slug = row_country_id.unwrap_or_else(|| "sa".to_string());

            vendors_json.push(json!({
                "id": id.to_string(),
                "slug": slug,
                "categories": [category],
                "nameAr": name_ar,
                "nameEn": name_en,
                "citySlug": city_slug.unwrap_or_default(),
                "cityAr": city_name_ar.unwrap_or_default(),
                "cityEn": city_name_en.unwrap_or_default(),
                "districtAr": "",
                "districtEn": "",
                "countrySlug": country_slug,
                "coverImage": cover_image.unwrap_or_else(|| "/images/fallbacks/default-cover.svg".to_string()),
                "rating": {
                    "overall": overall_avg,
                    "quality": quality_avg,
                    "staff": staff_avg,
                    "communication": communication_avg,
                    "count": review_count
                },
                "startingPrice": starting_price,
                "originalPrice": original_price,
                "currency": if country_slug == "eg" { "EGP" } else if country_slug == "ae" { "AED" } else { "SAR" },
                "isFeatured": active_featured,
                "isZafafExclusive": is_zafaf_exclusive,
                "isVerified": false,
                "hasOffers": has_offers,
                "packages": packages.clone(),
                "offers": packages
            }));
        }

        Ok(vendors_json)
    }

    // Pre-existing wide signature — grouping into a struct requires changing all call sites.
    // Deferred to a future refactor session. See CONVENTIONS.md §TODO.
    #[allow(clippy::too_many_arguments)]
    pub async fn update_vendor_profile(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        vendor_id: Uuid,
        payload: &crate::routes::vendor_management::profile::UpdateProfileRequest,
        clean_name_ar: &str,
        clean_name_en: &str,
        clean_description_ar: Option<&str>,
        clean_description_en: Option<&str>,
        clean_address_ar: Option<&str>,
        clean_address_en: Option<&str>,
        clean_phone: Option<&str>,
        clean_email: Option<&str>,
        clean_crm_venue_id: Option<&str>,
        clean_website: Option<&str>,
        clean_maps_url: Option<&str>,
        clean_video_url_1: Option<&str>,
        clean_instagram_url: Option<&str>,
        clean_cr_number: Option<&str>,
        clean_coord_name_ar: Option<&str>,
        clean_coord_name_en: Option<&str>,
        clean_coord_phone: Option<&str>,
        clean_coord_whatsapp: Option<&str>,
        parsed_city_id: Option<Uuid>,
    ) -> Result<u64, AppError> {
        let rows_affected = sqlx::query!(
            "UPDATE vendors
             SET name_ar = $1,
                 name_en = $2,
                 description_ar = COALESCE($3, description_ar),
                 description_en = COALESCE($4, description_en),
                 address_ar = COALESCE($5, address_ar),
                 address_en = COALESCE($6, address_en),
                 phone = COALESCE($7, phone),
                 email = COALESCE($8, email),
                 latitude = COALESCE($9, latitude),
                 longitude = COALESCE($10, longitude),
                 crm_venue_id = COALESCE($11, crm_venue_id),
                 star_rating = COALESCE($12, star_rating),
                 website = COALESCE($13, website),
                 maps_url = COALESCE($14, maps_url),
                 video_url_1 = COALESCE($15, video_url_1),
                 has_partition = COALESCE($16, has_partition),
                 capacity_min = COALESCE($17, capacity_min),
                 capacity_max = COALESCE($18, capacity_max),
                 amenities = COALESCE($19, amenities),
                 city_id = COALESCE($20, city_id),
                 instagram_url = COALESCE($21, instagram_url),
                 cr_number = COALESCE($22, cr_number),
                 coordinator_name_ar = COALESCE($23, coordinator_name_ar),
                 coordinator_name_en = COALESCE($24, coordinator_name_en),
                 coordinator_phone = COALESCE($25, coordinator_phone),
                 coordinator_whatsapp = COALESCE($26, coordinator_whatsapp),
                 version = version + 1,
                 updated_at = CURRENT_TIMESTAMP
             WHERE id = $27 AND version = $28",
                clean_name_ar,
                clean_name_en,
                clean_description_ar,
                clean_description_en,
                clean_address_ar,
                clean_address_en,
                clean_phone,
                clean_email,
                payload.latitude,
                payload.longitude,
                clean_crm_venue_id,
                payload.star_rating,
                clean_website,
                clean_maps_url,
                clean_video_url_1,
                payload.has_partition,
                payload.capacity_min,
                payload.capacity_max,
                payload.amenities.as_deref().unwrap_or(&[]),
                parsed_city_id,
                clean_instagram_url,
                clean_cr_number,
                clean_coord_name_ar,
                clean_coord_name_en,
                clean_coord_phone,
                clean_coord_whatsapp,
                vendor_id,
                payload.version
            
        )
        .execute(&mut **tx)
        .await?
        .rows_affected();

        Ok(rows_affected)
    }

    // ─── PACKAGES / PRODUCTS ──────────────────────────────────────────────────────────

    pub async fn list_packages(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        vendor_id: Uuid,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows = sqlx::query!(
            "SELECT 
                id, 
                product_id as \"product_id?\",
                name_ar, 
                name_en, 
                description_ar, 
                description_en, 
                original_price::float8 as original_price, 
                discounted_price::float8 as discounted_price, 
                is_zafaf_exclusive, 
                expiry_date::text as expiry_date,
                version
             FROM vendor_packages 
             WHERE vendor_id = $1 
             ORDER BY created_at DESC",
                vendor_id
            
        )
        .fetch_all(&mut **tx)
        .await?;

        let packages = rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.id.to_string(),
                    "product_id": row.product_id.map(|u| u.to_string()),
                    "name_ar": row.name_ar,
                    "name_en": row.name_en,
                    "description_ar": row.description_ar.unwrap_or_default(),
                    "description_en": row.description_en.unwrap_or_default(),
                    "original_price": row.original_price,
                    "discounted_price": row.discounted_price,
                    "is_zafaf_exclusive": row.is_zafaf_exclusive,
                    "expiry_date": row.expiry_date,
                    "version": row.version,
                })
            })
            .collect();

        Ok(packages)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_package(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        new_id: Uuid,
        vendor_id: Uuid,
        product_id: Option<Uuid>,
        name_ar: &str,
        name_en: &str,
        description_ar: &str,
        description_en: &str,
        original_price: f64,
        discounted_price: f64,
        is_zafaf_exclusive: bool,
        expiry_date: chrono::NaiveDate,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO vendor_packages (
                id, vendor_id, product_id, name_ar, name_en, description_ar, description_en, 
                original_price, discounted_price, is_zafaf_exclusive, expiry_date
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8::numeric, $9::numeric, $10, $11)",
                new_id,
                vendor_id,
                product_id,
                name_ar,
                name_en,
                description_ar,
                description_en,
                rust_decimal::Decimal::try_from(original_price).unwrap_or_default(),
                rust_decimal::Decimal::try_from(discounted_price).unwrap_or_default(),
                is_zafaf_exclusive,
                expiry_date
            
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_package(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        package_id: Uuid,
        vendor_id: Uuid,
        product_id: Option<Uuid>,
        name_ar: &str,
        name_en: &str,
        description_ar: &str,
        description_en: &str,
        original_price: f64,
        discounted_price: f64,
        is_zafaf_exclusive: bool,
        expiry_date: chrono::NaiveDate,
        version: i32,
    ) -> Result<(), AppError> {
        let rows_affected = sqlx::query!(
            "UPDATE vendor_packages 
             SET name_ar = $1, name_en = $2, description_ar = $3, description_en = $4, 
                 original_price = $5::numeric, discounted_price = $6::numeric, is_zafaf_exclusive = $7, expiry_date = $8,
                 product_id = COALESCE($9, product_id),
                 version = version + 1
             WHERE id = $10 AND vendor_id = $11 AND version = $12",
                name_ar,
                name_en,
                description_ar,
                description_en,
                rust_decimal::Decimal::try_from(original_price).unwrap_or_default(),
                rust_decimal::Decimal::try_from(discounted_price).unwrap_or_default(),
                is_zafaf_exclusive,
                expiry_date,
                product_id,
                package_id,
                vendor_id,
                version
            
        )
        .execute(&mut **tx)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Err(AppError::Conflict("The package was updated by another operator. Please reload and try again.".to_string()));
        }

        Ok(())
    }

    pub async fn delete_package(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        package_id: Uuid,
        vendor_id: Uuid,
    ) -> Result<(), AppError> {
        let rows_affected = sqlx::query!(
            "DELETE FROM vendor_packages WHERE id = $1 AND vendor_id = $2",
                package_id,
                vendor_id
            
        )
            .execute(&mut **tx)
            .await?
            .rows_affected();

        if rows_affected == 0 {
            return Err(AppError::NotFound("Package not found or access denied".to_string()));
        }
        Ok(())
    }

    pub async fn duplicate_package(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        package_id: Uuid,
        vendor_id: Uuid,
        new_id: Uuid,
    ) -> Result<(), AppError> {
        let row = sqlx::query!(
            "SELECT name_ar, name_en, description_ar, description_en, original_price, discounted_price, is_zafaf_exclusive, expiry_date, product_id
             FROM vendor_packages
             WHERE id = $1 AND vendor_id = $2",
                package_id,
                vendor_id
            
        )
        .fetch_optional(&mut **tx)
        .await?;

        let row = row.ok_or_else(|| AppError::NotFound("Package not found or access denied".to_string()))?;

        let name_ar: String = row.name_ar;
        let name_en: String = row.name_en;
        let description_ar: Option<String> = row.description_ar;
        let description_en: Option<String> = row.description_en;
        let original_price: rust_decimal::Decimal = row.original_price;
        let discounted_price: rust_decimal::Decimal = row.discounted_price;
        let is_zafaf_exclusive: bool = row.is_zafaf_exclusive;
        let expiry_date: chrono::NaiveDate = row.expiry_date;
        let product_id: Option<Uuid> = Some(row.product_id);

        let dup_name_ar = format!("{} (نسخة)", name_ar);
        let dup_name_en = format!("{} (Copy)", name_en);

        sqlx::query!(
            "INSERT INTO vendor_packages (
                id, vendor_id, product_id, name_ar, name_en, description_ar, description_en, 
                original_price, discounted_price, is_zafaf_exclusive, expiry_date
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
                new_id,
                vendor_id,
                product_id,
                &dup_name_ar,
                &dup_name_en,
                &description_ar.unwrap_or_default(),
                &description_en.unwrap_or_default(),
                original_price,
                discounted_price,
                is_zafaf_exclusive,
                expiry_date
            
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    // ─── GALLERY ──────────────────────────────────────────────────────────────

    pub async fn list_gallery(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        vendor_id: Uuid,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows = sqlx::query!(
            "SELECT 
                id, 
                image_url, 
                is_cover, 
                caption, 
                product_id,
                created_at::text as created_at
             FROM vendor_gallery 
             WHERE vendor_id = $1 
             ORDER BY is_cover DESC, created_at DESC",
                vendor_id
            
        )
        .fetch_all(&mut **tx)
        .await?;

        let gallery = rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.id.to_string(),
                    "image_url": row.image_url,
                    "is_cover": row.is_cover,
                    "caption": row.caption.unwrap_or_default(),
                    "product_id": row.product_id.map(|u| u.to_string()),
                    "created_at": row.created_at,
                })
            })
            .collect();

        Ok(gallery)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_gallery_image(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        vendor_id: Uuid,
        new_id: Uuid,
        url: &str,
        is_cover: bool,
        caption: &Option<String>,
        product_id: Option<Uuid>,
        file_id: Option<Uuid>,
    ) -> Result<(), AppError> {
        if is_cover {
            if let Some(pid) = product_id {
                sqlx::query!(
            "UPDATE vendor_gallery 
                     SET is_cover = FALSE 
                     WHERE product_id = $1",
                pid
            
        )
                .execute(&mut **tx)
                .await?;
            } else {
                sqlx::query!(
            "UPDATE vendor_gallery 
                     SET is_cover = FALSE 
                     WHERE vendor_id = $1 AND product_id IS NULL",
                vendor_id
            
        )
                .execute(&mut **tx)
                .await?;
            }
        }

        sqlx::query(
            "INSERT INTO vendor_gallery (id, vendor_id, image_url, file_url, file_path, is_cover, caption, product_id, media_type, file_id) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
        )
        .bind(new_id)
        .bind(vendor_id)
        .bind(url) // image_url
        .bind(url) // file_url
        .bind(None::<String>) // file_path
        .bind(is_cover)
        .bind(caption.as_deref().unwrap_or(""))
        .bind(product_id)
        .bind("image") // media_type
        .bind(file_id)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn set_cover_image(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        vendor_id: Uuid,
        image_id: Uuid,
    ) -> Result<(), AppError> {
        let exists = sqlx::query_scalar!(
            "SELECT EXISTS(SELECT 1 FROM vendor_gallery WHERE id = $1 AND vendor_id = $2)",
                image_id,
                vendor_id
            
        )
        .fetch_one(&mut **tx)
        .await?;

        if !exists.unwrap_or(false) {
            return Err(AppError::NotFound("Image not found or access denied".to_string()));
        }

        let product_id: Option<Uuid> = sqlx::query_scalar!(
            "SELECT product_id FROM vendor_gallery WHERE id = $1 AND vendor_id = $2",
                image_id,
                vendor_id
            
        )
        .fetch_one(&mut **tx)
        .await?;

        if let Some(pid) = product_id {
            sqlx::query!(
            "UPDATE vendor_gallery 
                 SET is_cover = FALSE 
                 WHERE product_id = $1",
                pid
            
        )
            .execute(&mut **tx)
            .await?;
        } else {
            sqlx::query!(
            "UPDATE vendor_gallery 
                 SET is_cover = FALSE 
                 WHERE vendor_id = $1 AND product_id IS NULL",
                vendor_id
            
        )
            .execute(&mut **tx)
            .await?;
        }

        sqlx::query!(
            "UPDATE vendor_gallery 
             SET is_cover = TRUE 
             WHERE id = $1 AND vendor_id = $2",
                image_id,
                vendor_id
            
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn get_gallery_image_path(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        vendor_id: Uuid,
        image_id: Uuid,
    ) -> Result<Option<String>, AppError> {
        let file_path: Option<String> = sqlx::query_scalar!(
            "SELECT file_path FROM vendor_gallery WHERE id = $1 AND vendor_id = $2",
                image_id,
                vendor_id
            
        )
        .fetch_optional(&mut **tx)
        .await?
        .flatten();

        Ok(file_path)
    }

    pub async fn delete_gallery_image(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        vendor_id: Uuid,
        image_id: Uuid,
    ) -> Result<(), AppError> {
        let rows_affected = sqlx::query!(
            "DELETE FROM vendor_gallery WHERE id = $1 AND vendor_id = $2",
                image_id,
                vendor_id
            
        )
            .execute(&mut **tx)
            .await?
            .rows_affected();

        if rows_affected == 0 {
            return Err(AppError::NotFound("Image not found or access denied".to_string()));
        }

        Ok(())
    }

    // ─── SUBSCRIPTIONS ────────────────────────────────────────────────────────

    pub async fn list_subscription_requests(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        vendor_id: Uuid,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let rows = sqlx::query!(
            "SELECT r.id, r.vendor_id, r.requested_tier_id, r.status, r.admin_notes, r.created_at, r.updated_at, t.name as requested_tier_name
             FROM vendor_subscription_requests r
             JOIN subscription_tiers t ON r.requested_tier_id = t.id
             WHERE r.vendor_id = $1
             ORDER BY r.created_at DESC",
            vendor_id
        )
        .fetch_all(&mut **tx)
        .await?;

        let requests = rows
            .into_iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.id.to_string(),
                    "vendor_id": row.vendor_id.to_string(),
                    "requested_tier_id": row.requested_tier_id.to_string(),
                    "requested_tier_name": row.requested_tier_name,
                    "status": row.status,
                    "admin_notes": row.admin_notes,
                    "created_at": row.created_at.to_string(),
                    "updated_at": row.updated_at.to_string(),
                })
            })
            .collect();

        Ok(requests)
    }

    pub async fn check_pending_subscription(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        vendor_id: Uuid,
    ) -> Result<bool, AppError> {
        let pending_count: Option<i64> = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM vendor_subscription_requests WHERE vendor_id = $1 AND status = 'pending'",
            vendor_id
        )
        .fetch_one(&mut **tx)
        .await?;

        Ok(pending_count.unwrap_or(0) > 0)
    }

    pub async fn create_subscription_request(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        request_id: Uuid,
        vendor_id: Uuid,
        tier_id: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "INSERT INTO vendor_subscription_requests (id, vendor_id, requested_tier_id, status) VALUES ($1, $2, $3, 'pending')",
            request_id,
            vendor_id,
            tier_id
        )
        .execute(&mut **tx)
        .await?;
        
        Ok(())
    }

    // ─── ANALYTICS ────────────────────────────────────────────────────────────

    pub async fn get_vendor_analytics_chart(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        vendor_id: Uuid,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let chart_rows = sqlx::query!(
            "SELECT 
                TO_CHAR(created_at, 'YYYY-MM') AS month,
                COUNT(*)::bigint AS booking_count,
                COALESCE(SUM(total_price) FILTER (WHERE status IN ('confirmed', 'Confirmed', 'completed', 'Completed')), 0.00)::float8 AS revenue
             FROM core_bookings
             WHERE vendor_id = (SELECT user_id FROM vendors WHERE id = $1 LIMIT 1)
               AND created_at >= NOW() - INTERVAL '6 months'
             GROUP BY TO_CHAR(created_at, 'YYYY-MM')
             ORDER BY month ASC",
                vendor_id
            
        )
        .fetch_all(&mut **tx)
        .await?;

        let chart_data = chart_rows
            .iter()
            .map(|row| {
                serde_json::json!({
                    "month": row.month,
                    "bookings": row.booking_count,
                    "revenue": row.revenue,
                })
            })
            .collect();

        Ok(chart_data)
    }

    pub async fn get_top_performing_products(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        vendor_id: Uuid,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let top_product_rows = sqlx::query!(
            "SELECT
                vp.id, COALESCE(vp.title_en, vp.title_ar, vp.title, 'Unnamed Product') AS title, vp.product_category,
                COUNT(cb.id)::bigint AS booking_count,
                COALESCE(SUM(cb.total_price) FILTER (WHERE cb.status IN ('confirmed', 'Confirmed', 'completed', 'Completed')), 0.00)::float8 AS total_revenue
             FROM vendor_products vp
             LEFT JOIN core_bookings cb ON vp.id = cb.product_id
             WHERE vp.vendor_id = $1 AND vp.status != 'archived'
             GROUP BY vp.id, vp.title_en, vp.title_ar, vp.title, vp.product_category
             ORDER BY booking_count DESC, total_revenue DESC
             LIMIT 5",
                vendor_id
            
        )
        .fetch_all(&mut **tx)
        .await?;

        let top_products = top_product_rows
            .iter()
            .map(|row| {
                serde_json::json!({
                    "id": row.id.to_string(),
                    "title": row.title,
                    "category": row.product_category,
                    "bookings": row.booking_count,
                    "revenue": row.total_revenue,
                })
            })
            .collect();

        Ok(top_products)
    }

    // ─── ADMIN CHATS (SUPPORT) ────────────────────────────────────────────────

    pub async fn get_or_create_admin_chat(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        vendor_id: Uuid,
    ) -> Result<Uuid, AppError> {
        let user_id = sqlx::query_scalar!("SELECT user_id FROM vendors WHERE id = $1", vendor_id)
            .fetch_one(&mut **tx)
            .await?;

        let conversation_id = sqlx::query_scalar!(
            "SELECT c.id FROM conversations c
             JOIN conversation_participants cp ON c.id = cp.conversation_id
             WHERE cp.user_id = $1 AND c.title = 'Admin Support Chat'
             LIMIT 1",
            user_id
        )
        .fetch_optional(&mut **tx)
        .await?;

        if let Some(id) = conversation_id {
            return Ok(id);
        }

        let new_id = sqlx::query_scalar!(
            "INSERT INTO conversations (title) VALUES ('Admin Support Chat') RETURNING id"
        )
        .fetch_one(&mut **tx)
        .await?;

        sqlx::query!(
            "INSERT INTO conversation_participants (conversation_id, user_id) VALUES ($1, $2)",
            new_id,
            user_id
        )
        .execute(&mut **tx)
        .await?;

        Ok(new_id)
    }

    pub async fn get_chat_messages(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        chat_id: Uuid,
        vendor_id: Uuid,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let user_id = sqlx::query_scalar!("SELECT user_id FROM vendors WHERE id = $1", vendor_id)
            .fetch_one(&mut **tx)
            .await?;

        let rows = sqlx::query!(
            "SELECT 
                m.id, 
                m.sender_id, 
                m.body, 
                (SELECT file_url FROM message_attachments a WHERE a.message_id = m.id LIMIT 1) as file_url,
                EXISTS(SELECT 1 FROM message_read_receipts r WHERE r.message_id = m.id AND r.user_id != m.sender_id) as is_read,
                m.created_at
             FROM messages m
             WHERE m.conversation_id = $1
             ORDER BY m.created_at ASC",
             chat_id
        )
        .fetch_all(&mut **tx)
        .await?;

        let messages = rows
            .into_iter()
            .map(|row| {
                let sender = if Some(row.sender_id) == user_id { "vendor" } else { "admin" };
                serde_json::json!({
                    "id": row.id.to_string(),
                    "sender": sender,
                    "body": row.body.unwrap_or_default(),
                    "file_url": row.file_url,
                    "is_read": row.is_read.unwrap_or(false),
                    "created_at": row.created_at.to_string(),
                })
            })
            .collect();

        Ok(messages)
    }

    pub async fn mark_admin_messages_read(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        chat_id: Uuid,
        vendor_id: Uuid,
    ) -> Result<(), AppError> {
        let user_id = sqlx::query_scalar!("SELECT user_id FROM vendors WHERE id = $1", vendor_id)
            .fetch_one(&mut **tx)
            .await?;

        sqlx::query!(
            "INSERT INTO message_read_receipts (message_id, user_id)
             SELECT m.id, $1 FROM messages m
             WHERE m.conversation_id = $2 AND m.sender_id != $1
             ON CONFLICT DO NOTHING",
             user_id, chat_id
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn insert_chat_message(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        chat_id: Uuid,
        vendor_id: Uuid,
        body: &str,
        file_url: Option<&str>,
    ) -> Result<Uuid, AppError> {
        let user_id = sqlx::query_scalar!("SELECT user_id FROM vendors WHERE id = $1", vendor_id)
            .fetch_one(&mut **tx)
            .await?;

        let message_id = sqlx::query_scalar!(
            "INSERT INTO messages (conversation_id, sender_id, body)
             VALUES ($1, $2, $3)
             RETURNING id",
             chat_id, user_id, body
        )
        .fetch_one(&mut **tx)
        .await?;

        if let Some(url) = file_url {
            sqlx::query!(
                "INSERT INTO message_attachments (message_id, file_name, file_url, file_type, file_size)
                 VALUES ($1, 'attachment', $2, 'unknown', 0)",
                 message_id, url
            )
            .execute(&mut **tx)
            .await?;
        }

        Ok(message_id)
    }

    pub async fn update_chat_timestamp(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        chat_id: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query!(
            "UPDATE conversations SET updated_at = NOW() WHERE id = $1",
            chat_id
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}
