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
        .route("/documents", get(list_client_documents).post(upload_client_document))
        .route("/documents/:id", axum::routing::delete(delete_client_document))
}

#[derive(Deserialize)]
struct UploadDocumentPayload {
    #[serde(rename = "fileName")]
    file_name: String,
    #[serde(rename = "fileUrl")]
    file_url: String,
    category: Option<String>,
    #[serde(rename = "bookingId")]
    booking_id: Option<Uuid>,
}

async fn list_client_documents(
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
        "SELECT id, file_name, file_url, category, booking_id, created_at::text \
         FROM client_documents WHERE client_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
    )
    .bind(client_uuid)
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let docs: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "id": r.get::<Uuid, _>("id").to_string(),
                "fileName": r.get::<String, _>("file_name"),
                "fileUrl": r.get::<String, _>("file_url"),
                "category": r.get::<String, _>("category"),
                "bookingId": r.get::<Option<Uuid>, _>("booking_id").map(|u| u.to_string()),
                "createdAt": r.get::<String, _>("created_at"),
            })
        })
        .collect();

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "data": docs
    })))
}

async fn upload_client_document(
    auth: RequireAuth,
    mut rls_tx: RlsTx,
    Json(payload): Json<UploadDocumentPayload>,
) -> Result<Json<Value>, AppError> {
    if auth.role != DomainType::Client {
        return Err(AppError::Forbidden(
            "Client credentials required".to_string(),
        ));
    }
    let client_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid client session".to_string()))?;

    let doc_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO client_documents (id, client_id, file_name, file_url, category, booking_id) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(doc_id)
    .bind(client_uuid)
    .bind(&payload.file_name)
    .bind(&payload.file_url)
    .bind(payload.category.as_deref().unwrap_or("Contract"))
    .bind(payload.booking_id)
    .execute(&mut *rls_tx.tx)
    .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Document uploaded successfully",
        "documentId": doc_id.to_string()
    })))
}

async fn delete_client_document(
    auth: RequireAuth,
    mut rls_tx: RlsTx,
    Path(doc_id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    if auth.role != DomainType::Client {
        return Err(AppError::Forbidden(
            "Client credentials required".to_string(),
        ));
    }
    let client_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid client session".to_string()))?;

    sqlx::query("UPDATE client_documents SET deleted_at = CURRENT_TIMESTAMP WHERE id = $1 AND client_id = $2")
        .bind(doc_id)
        .bind(client_uuid)
        .execute(&mut *rls_tx.tx)
        .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Document deleted successfully"
    })))
}
