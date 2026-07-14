use crate::errors::AppError;
use crate::middleware::auth::{RequireAuth, RlsTx};
use crate::models::user::DomainType;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;
use sqlx::Row;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/favorites", get(list_client_favorites).post(add_client_favorite))
        .route("/favorites/toggle", post(toggle_client_favorite))
        .route("/favorites/:vendor_id", axum::routing::delete(remove_client_favorite))
        .route("/shortlists", get(list_client_shortlists))
        .route("/compare", post(compare_vendors).with_state(state))
}

#[derive(Deserialize)]
struct AddFavoriteRequest {
    #[serde(rename = "vendorId")]
    vendor_id: Uuid,
    #[serde(rename = "shortlistName")]
    shortlist_name: Option<String>,
}

#[derive(Deserialize)]
struct ToggleFavoriteRequest {
    #[serde(rename = "vendorId")]
    vendor_id: Uuid,
}

#[derive(Deserialize)]
pub struct CompareVendorsRequest {
    #[serde(rename = "vendorIds")]
    pub vendor_ids: Vec<Uuid>,
}

async fn list_client_favorites(
    auth: RequireAuth,
    mut rls_tx: RlsTx,
) -> Result<Json<Value>, AppError> {
    if auth.role != DomainType::Client {
        return Err(AppError::Forbidden(
            "Client credentials required".to_string(),
        ));
    }
    let client_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid client session".to_string()))?;

    let rows = sqlx::query(
        "SELECT f.id, f.vendor_id, f.shortlist_name, f.created_at::text, \
                v.name_ar, v.name_en, v.slug, v.category, c.name_ar AS city_name_ar, c.name_en AS city_name_en \
         FROM client_favorites f \
         JOIN vendors v ON f.vendor_id = v.id \
         LEFT JOIN cities c ON v.city_id = c.id \
         WHERE f.client_id = $1 \
         ORDER BY f.created_at DESC"
    )
    .bind(client_uuid)
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let favorites: Vec<Value> = rows
        .iter()
        .map(|row| {
            json!({
                "id": row.get::<Uuid, _>("id").to_string(),
                "vendorId": row.get::<Uuid, _>("vendor_id").to_string(),
                "shortlistName": row.get::<String, _>("shortlist_name"),
                "createdAt": row.get::<String, _>("created_at"),
                "vendor": {
                    "nameAr": row.get::<String, _>("name_ar"),
                    "nameEn": row.get::<String, _>("name_en"),
                    "slug": row.get::<String, _>("slug"),
                    "category": row.get::<Option<String>, _>("category").unwrap_or_default(),
                    "cityNameAr": row.get::<Option<String>, _>("city_name_ar").unwrap_or_default(),
                    "cityNameEn": row.get::<Option<String>, _>("city_name_en").unwrap_or_default(),
                }
            })
        })
        .collect();

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "data": favorites
    })))
}

async fn add_client_favorite(
    auth: RequireAuth,
    mut rls_tx: RlsTx,
    Json(payload): Json<AddFavoriteRequest>,
) -> Result<Json<Value>, AppError> {
    if auth.role != DomainType::Client {
        return Err(AppError::Forbidden(
            "Client credentials required".to_string(),
        ));
    }
    let client_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid client session".to_string()))?;

    let shortlist = payload
        .shortlist_name
        .unwrap_or_else(|| "Default".to_string());

    sqlx::query(
        "INSERT INTO client_favorites (client_id, vendor_id, shortlist_name) \
         VALUES ($1, $2, $3) \
         ON CONFLICT (client_id, vendor_id, shortlist_name) DO NOTHING",
    )
    .bind(client_uuid)
    .bind(payload.vendor_id)
    .bind(&shortlist)
    .execute(&mut *rls_tx.tx)
    .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Vendor added to favorites"
    })))
}

async fn toggle_client_favorite(
    auth: RequireAuth,
    mut rls_tx: RlsTx,
    Json(payload): Json<ToggleFavoriteRequest>,
) -> Result<Json<Value>, AppError> {
    if auth.role != DomainType::Client {
        return Err(AppError::Forbidden(
            "Client credentials required".to_string(),
        ));
    }
    let client_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid client session".to_string()))?;

    let existing: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM client_favorites WHERE client_id = $1 AND vendor_id = $2",
    )
    .bind(client_uuid)
    .bind(payload.vendor_id)
    .fetch_optional(&mut *rls_tx.tx)
    .await?;

    let is_favorited = match existing {
        Some(fav_id) => {
            sqlx::query("DELETE FROM client_favorites WHERE id = $1")
                .bind(fav_id)
                .execute(&mut *rls_tx.tx)
                .await?;
            false
        }
        None => {
            sqlx::query("INSERT INTO client_favorites (client_id, vendor_id) VALUES ($1, $2)")
                .bind(client_uuid)
                .bind(payload.vendor_id)
                .execute(&mut *rls_tx.tx)
                .await?;
            true
        }
    };

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "isFavorited": is_favorited
    })))
}

async fn remove_client_favorite(
    auth: RequireAuth,
    mut rls_tx: RlsTx,
    Path(vendor_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    if auth.role != DomainType::Client {
        return Err(AppError::Forbidden(
            "Client credentials required".to_string(),
        ));
    }
    let client_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid client session".to_string()))?;

    sqlx::query("DELETE FROM client_favorites WHERE client_id = $1 AND vendor_id = $2")
        .bind(client_uuid)
        .bind(vendor_id)
        .execute(&mut *rls_tx.tx)
        .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Vendor removed from favorites"
    })))
}

async fn list_client_shortlists(
    auth: RequireAuth,
    mut rls_tx: RlsTx,
) -> Result<Json<Value>, AppError> {
    if auth.role != DomainType::Client {
        return Err(AppError::Forbidden(
            "Client credentials required".to_string(),
        ));
    }
    let client_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid client session".to_string()))?;

    let rows = sqlx::query(
        "SELECT shortlist_name, COUNT(*)::bigint AS item_count \
         FROM client_favorites \
         WHERE client_id = $1 \
         GROUP BY shortlist_name \
         ORDER BY shortlist_name ASC",
    )
    .bind(client_uuid)
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let shortlists: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "name": r.get::<String, _>("shortlist_name"),
                "count": r.get::<i64, _>("item_count")
            })
        })
        .collect();

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "data": shortlists
    })))
}

async fn compare_vendors(
    State(state): State<AppState>,
    Json(payload): Json<CompareVendorsRequest>,
) -> Result<Json<Value>, AppError> {
    if payload.vendor_ids.is_empty() || payload.vendor_ids.len() > 4 {
        return Err(AppError::BadRequest(
            "Please select between 1 and 4 vendors for comparison".to_string(),
        ));
    }

    let rows = sqlx::query(
        "SELECT v.id, v.name_ar, v.name_en, v.slug, v.category, v.capacity_min, v.capacity_max, \
                (v.is_featured = TRUE AND (v.featured_expires_at IS NULL OR v.featured_expires_at > CURRENT_TIMESTAMP)) AS is_featured, \
                v.is_verified, c.name_ar AS city_name_ar, c.name_en AS city_name_en \
         FROM vendors v \
         LEFT JOIN cities c ON v.city_id = c.id \
         WHERE v.id = ANY($1)"
    )
    .bind(&payload.vendor_ids)
    .fetch_all(&state.db)
    .await?;

    let comparison: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "id": r.get::<Uuid, _>("id").to_string(),
                "nameAr": r.get::<String, _>("name_ar"),
                "nameEn": r.get::<String, _>("name_en"),
                "slug": r.get::<String, _>("slug"),
                "category": r.get::<Option<String>, _>("category").unwrap_or_default(),
                "cityNameAr": r.get::<Option<String>, _>("city_name_ar").unwrap_or_default(),
                "cityNameEn": r.get::<Option<String>, _>("city_name_en").unwrap_or_default(),
                "capacity": {
                    "min": r.get::<Option<i32>, _>("capacity_min").unwrap_or(0),
                    "max": r.get::<Option<i32>, _>("capacity_max").unwrap_or(1000)
                },
                "isFeatured": r.get::<bool, _>("is_featured"),
                "isVerified": r.get::<Option<bool>, _>("is_verified").unwrap_or(false),
            })
        })
        .collect();

    Ok(Json(json!({
        "status": "success",
        "data": comparison
    })))
}
