use crate::errors::AppError;
use crate::middleware::auth::{RequireAuth, RlsTx};
use crate::models::user::DomainType;
use crate::state::AppState;
use axum::{
    extract::Path,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;
use sqlx::Row;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/tasks", get(list_client_tasks).post(create_client_task))
        .route("/tasks/:id", axum::routing::patch(update_client_task).delete(delete_client_task))
}

#[derive(Deserialize)]
struct CreateTaskPayload {
    title: String,
    category: Option<String>,
    #[serde(rename = "dueDate")]
    due_date: Option<String>,
    priority: Option<String>,
    notes: Option<String>,
}

#[derive(Deserialize)]
struct UpdateTaskPayload {
    title: Option<String>,
    category: Option<String>,
    #[serde(rename = "dueDate")]
    due_date: Option<String>,
    priority: Option<String>,
    #[serde(rename = "isCompleted")]
    is_completed: Option<bool>,
    notes: Option<String>,
}

async fn list_client_tasks(auth: RequireAuth, mut rls_tx: RlsTx) -> Result<Json<Value>, AppError> {
    if auth.role != DomainType::Client {
        return Err(AppError::Forbidden(
            "Client credentials required".to_string(),
        ));
    }
    let client_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid client session".to_string()))?;

    let rows = sqlx::query(
        "SELECT id, title, category, due_date::text, priority, is_completed, notes, created_at::text \
         FROM client_tasks WHERE client_id = $1 ORDER BY is_completed ASC, due_date ASC NULLS LAST"
    )
    .bind(client_uuid)
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let tasks: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "id": r.get::<Uuid, _>("id").to_string(),
                "title": r.get::<String, _>("title"),
                "category": r.get::<String, _>("category"),
                "dueDate": r.get::<Option<String>, _>("due_date"),
                "priority": r.get::<String, _>("priority"),
                "isCompleted": r.get::<bool, _>("is_completed"),
                "notes": r.get::<Option<String>, _>("notes"),
                "createdAt": r.get::<String, _>("created_at"),
            })
        })
        .collect();

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "data": tasks
    })))
}

async fn create_client_task(
    auth: RequireAuth,
    mut rls_tx: RlsTx,
    Json(payload): Json<CreateTaskPayload>,
) -> Result<Json<Value>, AppError> {
    if auth.role != DomainType::Client {
        return Err(AppError::Forbidden(
            "Client credentials required".to_string(),
        ));
    }
    let client_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid client session".to_string()))?;

    let parsed_date = match payload.due_date {
        Some(d) => chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok(),
        None => None,
    };

    let task_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO client_tasks (id, client_id, title, category, due_date, priority, notes) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(task_id)
    .bind(client_uuid)
    .bind(&payload.title)
    .bind(payload.category.as_deref().unwrap_or("General"))
    .bind(parsed_date)
    .bind(payload.priority.as_deref().unwrap_or("Medium"))
    .bind(&payload.notes)
    .execute(&mut *rls_tx.tx)
    .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Task created successfully",
        "taskId": task_id.to_string()
    })))
}

async fn update_client_task(
    auth: RequireAuth,
    mut rls_tx: RlsTx,
    Path(task_id): Path<Uuid>,
    Json(payload): Json<UpdateTaskPayload>,
) -> Result<Json<Value>, AppError> {
    if auth.role != DomainType::Client {
        return Err(AppError::Forbidden(
            "Client credentials required".to_string(),
        ));
    }
    let client_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid client session".to_string()))?;

    let parsed_date = match payload.due_date {
        Some(d) => chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok(),
        None => None,
    };

    sqlx::query(
        "UPDATE client_tasks \
         SET title = COALESCE($1, title), \
             category = COALESCE($2, category), \
             due_date = COALESCE($3, due_date), \
             priority = COALESCE($4, priority), \
             is_completed = COALESCE($5, is_completed), \
             notes = COALESCE($6, notes) \
         WHERE id = $7 AND client_id = $8",
    )
    .bind(payload.title)
    .bind(payload.category)
    .bind(parsed_date)
    .bind(payload.priority)
    .bind(payload.is_completed)
    .bind(payload.notes)
    .bind(task_id)
    .bind(client_uuid)
    .execute(&mut *rls_tx.tx)
    .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Task updated successfully"
    })))
}

async fn delete_client_task(
    auth: RequireAuth,
    mut rls_tx: RlsTx,
    Path(task_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    if auth.role != DomainType::Client {
        return Err(AppError::Forbidden(
            "Client credentials required".to_string(),
        ));
    }
    let client_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid client session".to_string()))?;

    sqlx::query("DELETE FROM client_tasks WHERE id = $1 AND client_id = $2")
        .bind(task_id)
        .bind(client_uuid)
        .execute(&mut *rls_tx.tx)
        .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Task deleted successfully"
    })))
}
