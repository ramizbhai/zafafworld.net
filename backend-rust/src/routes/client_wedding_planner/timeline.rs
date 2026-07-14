use crate::errors::AppError;
use crate::middleware::auth::{RequireAuth, RlsTx};
use crate::models::user::DomainType;
use crate::state::AppState;
use axum::{
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;
use sqlx::Row;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/timeline", get(get_client_timeline).post(create_client_timeline_event))
}

#[derive(Deserialize)]
struct CreateTimelineEventPayload {
    title: String,
    #[serde(rename = "eventDate")]
    event_date: String,
    #[serde(rename = "eventType")]
    event_type: Option<String>,
}

async fn get_client_timeline(
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
        "SELECT id, title, event_date::text, event_type, is_completed, created_at::text \
         FROM client_timeline_events WHERE client_id = $1 ORDER BY event_date ASC",
    )
    .bind(client_uuid)
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let events: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "id": r.get::<Uuid, _>("id").to_string(),
                "title": r.get::<String, _>("title"),
                "eventDate": r.get::<String, _>("event_date"),
                "eventType": r.get::<String, _>("event_type"),
                "isCompleted": r.get::<bool, _>("is_completed"),
                "createdAt": r.get::<String, _>("created_at"),
            })
        })
        .collect();

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "data": events
    })))
}

async fn create_client_timeline_event(
    auth: RequireAuth,
    mut rls_tx: RlsTx,
    Json(payload): Json<CreateTimelineEventPayload>,
) -> Result<Json<Value>, AppError> {
    if auth.role != DomainType::Client {
        return Err(AppError::Forbidden(
            "Client credentials required".to_string(),
        ));
    }
    let client_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid client session".to_string()))?;

    let parsed_date =
        chrono::NaiveDate::parse_from_str(&payload.event_date, "%Y-%m-%d").map_err(|_| {
            AppError::BadRequest("Invalid eventDate format. Expected YYYY-MM-DD".to_string())
        })?;

    let event_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO client_timeline_events (id, client_id, title, event_date, event_type) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(event_id)
    .bind(client_uuid)
    .bind(&payload.title)
    .bind(parsed_date)
    .bind(payload.event_type.as_deref().unwrap_or("Custom"))
    .execute(&mut *rls_tx.tx)
    .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Timeline event created successfully",
        "eventId": event_id.to_string()
    })))
}
