use crate::errors::AppError;
use crate::middleware::auth::{RequireAuth, RlsTx};
use crate::models::user::DomainType;
use crate::state::AppState;
use axum::{
    routing::post,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;
use sqlx::Row;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/reviews", post(submit_vendor_review))
}

#[derive(Deserialize)]
struct SubmitReviewRequest {
    #[serde(rename = "vendorId")]
    vendor_id: Uuid,
    rating: i32,
    #[serde(rename = "reviewText")]
    review_text: String,
    attachments: Option<Vec<String>>,
}

async fn submit_vendor_review(
    auth: RequireAuth,
    mut rls_tx: RlsTx,
    Json(payload): Json<SubmitReviewRequest>,
) -> Result<Json<Value>, AppError> {
    if auth.role != DomainType::Client {
        return Err(AppError::Forbidden(
            "Client credentials required".to_string(),
        ));
    }
    let client_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid client session".to_string()))?;

    if payload.rating < 1 || payload.rating > 5 {
        return Err(AppError::BadRequest(
            "Rating must be between 1 and 5".to_string(),
        ));
    }

    // 1. Insert review under RLS transaction context
    let review_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO vendor_reviews (id, client_id, vendor_id, rating, review_text, status) \
         VALUES ($1, $2, $3, $4, $5, 'pending_approval')",
    )
    .bind(review_id)
    .bind(client_uuid)
    .bind(payload.vendor_id)
    .bind(payload.rating)
    .bind(&payload.review_text)
    .execute(&mut *rls_tx.tx)
    .await?;

    // 2. Insert attachments if present
    if let Some(files) = payload.attachments {
        for file in files {
            sqlx::query(
                "INSERT INTO vendor_review_attachments (review_id, file_path) VALUES ($1, $2)",
            )
            .bind(review_id)
            .bind(file)
            .execute(&mut *rls_tx.tx)
            .await?;
        }
    }

    // Resolve vendor profile ID
    let target_vendor_profile_id: Option<Uuid> =
        sqlx::query_scalar("SELECT id FROM vendors WHERE user_id = $1")
            .bind(payload.vendor_id)
            .fetch_optional(&mut *rls_tx.tx)
            .await?;

    let _ = sqlx::query(
        "INSERT INTO system_events (user_id, target_vendor_id, event_type, message_ar, message_en) \
         VALUES ($1, $2, 'review_received', $3, $4)",
    )
    .bind(client_uuid)
    .bind(target_vendor_profile_id)
    .bind(format!(
        "تم تقديم تقييم جديد للمورد بقيمة {} نجوم",
        payload.rating
    ))
    .bind(format!(
        "New review submitted for vendor with {} stars",
        payload.rating
    ))
    .execute(&mut *rls_tx.tx)
    .await;

    rls_tx
        .tx
        .commit()
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(json!({
        "status": "success",
        "message": "Review submitted successfully and is awaiting moderation.",
        "review_id": review_id.to_string()
    })))
}
