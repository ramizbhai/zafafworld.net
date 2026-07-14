use crate::errors::AppError;
use crate::middleware::auth::{RequireAdmin, RlsTx};
use crate::state::AppState;
use axum::{
    extract::Path,
    routing::{get, patch},
    Json, Router,
};
use serde_json::{json, Value};
use uuid::Uuid;
use sqlx::Row;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/reviews/pending", get(get_pending_reviews))
        .route("/reviews/:id/approve", patch(approve_review))
        .route("/reviews/:id/status", patch(update_review_status))
}

#[derive(serde::Deserialize)]
struct ApproveReviewRequest {
    approve: bool,
}

async fn get_pending_reviews(
    mut rls_tx: RlsTx,
    _auth: RequireAdmin,
) -> Result<Json<Value>, AppError> {
    let rows = sqlx::query(
        "SELECT \
            r.id, r.client_id, r.vendor_id, r.rating, r.review_text, r.created_at, \
            cp.first_name, cp.last_name, \
            v.name_en AS vendor_name_en, v.name_ar AS vendor_name_ar \
         FROM vendor_reviews r \
         LEFT JOIN client_profiles cp ON r.client_id = cp.client_id \
         LEFT JOIN vendors v ON r.vendor_id = v.user_id \
         WHERE r.status = 'pending_approval' \
         ORDER BY r.created_at ASC",
    )
    .bind(&_auth.user_id) // bind to satisfy RLS policy constraints if present
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let mut pending = Vec::new();
    for row in rows {
        let review_id: Uuid = row.get("id");
        let photos =
            sqlx::query("SELECT file_path FROM vendor_review_attachments WHERE review_id = $1")
                .bind(review_id)
                .fetch_all(&mut *rls_tx.tx)
                .await?;

        let attachments: Vec<String> = photos
            .iter()
            .map(|p| p.get::<String, _>("file_path"))
            .collect();

        pending.push(json!({
            "id": review_id.to_string(),
            "client_name": format!("{} {}",
                row.get::<Option<String>, _>("first_name").unwrap_or_else(|| "User".to_string()),
                row.get::<Option<String>, _>("last_name").unwrap_or_default()
            ).trim().to_string(),
            "vendor_name_en": row.get::<String, _>("vendor_name_en"),
            "vendor_name_ar": row.get::<String, _>("vendor_name_ar"),
            "rating": row.get::<i32, _>("rating"),
            "review_text": row.get::<String, _>("review_text"),
            "attachments": attachments,
            "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at").to_rfc3339()
        }));
    }

    Ok(Json(json!({
        "status": "success",
        "reviews": pending
    })))
}

async fn approve_review(
    mut rls_tx: RlsTx,
    auth: RequireAdmin,
    Path(id): Path<Uuid>,
    Json(payload): Json<ApproveReviewRequest>,
) -> Result<Json<Value>, AppError> {
    let admin_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid admin user".to_string()))?;

    let target_status = if payload.approve {
        "approved"
    } else {
        "rejected"
    };

    // 1. Update review status
    let result = sqlx::query(
        "UPDATE vendor_reviews SET status = $1, updated_at = CURRENT_TIMESTAMP \
         WHERE id = $2 AND status = 'pending_approval' RETURNING vendor_id",
    )
    .bind(target_status)
    .bind(id)
    .fetch_optional(&mut *rls_tx.tx)
    .await?;

    let vendor_id = match result {
        Some(row) => row.get::<Uuid, _>("vendor_id"),
        None => return Err(AppError::NotFound("Pending review not found".to_string())),
    };

    // 2. Audit Moderation Event
    sqlx::query(
        "INSERT INTO system_events (user_id, target_vendor_id, event_type, message_ar, message_en) \
         VALUES ($1, $2, 'review_moderated', $3, $4)",
    )
    .bind(admin_uuid)
    .bind(vendor_id)
    .bind(format!("تم تحديث حالة المراجعة إلى '{}'", target_status))
    .bind(format!("Admin moderated review to '{}'", target_status))
    .execute(&mut *rls_tx.tx)
    .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": format!("Review has been successfully moderated to '{}'", target_status)
    })))
}

#[derive(serde::Deserialize)]
pub struct UpdateReviewStatusInput {
    pub status: String,
}

async fn update_review_status(
    mut rls_tx: RlsTx,
    auth: RequireAdmin,
    Path(id): Path<String>,
    Json(input): Json<UpdateReviewStatusInput>,
) -> Result<Json<Value>, AppError> {
    tracing::info!("Updating status of review ID: {} to {}", id, input.status);

    let review_uuid = Uuid::parse_str(&id)
        .map_err(|_| AppError::BadRequest("Invalid review UUID format".to_string()))?;

    let admin_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid admin user ID".to_string()))?;

    if input.status != "approved" && input.status != "rejected" {
        return Err(AppError::BadRequest(
            "Status must be either 'approved' or 'rejected'".to_string(),
        ));
    }

    let result = sqlx::query("UPDATE reviews SET status = $1 WHERE id = $2 AND status = 'pending'")
        .bind(&input.status)
        .bind(review_uuid)
        .execute(&mut *rls_tx.tx)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(
            "Pending review not found or status already finalized".to_string(),
        ));
    }

    // Audit trail for review moderation — review_uuid bound as target FK
    let event_type = if input.status == "approved" {
        "review_approved"
    } else {
        "review_rejected"
    };
    sqlx::query(
        "INSERT INTO system_events (user_id, target_vendor_id, event_type, message_ar, message_en) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(admin_uuid)
    .bind(review_uuid)
    .bind(event_type)
    .bind(format!(
        "قام المسؤول بتحديث حالة المراجعة إلى '{}'",
        input.status
    ))
    .bind(format!(
        "Administrator updated review status to '{}'",
        input.status
    ))
    .execute(&mut *rls_tx.tx)
    .await?;

    rls_tx.tx.commit().await?;

    tracing::info!(
        target: "audit",
        actor_id  = %auth.user_id,
        event     = "review_status_moderated",
        target_id = %review_uuid,
        next      = %input.status,
        "Admin review moderation committed"
    );

    Ok(Json(json!({
        "status": "success",
        "message": format!("Review {} status successfully updated to {}", id, input.status)
    })))
}
