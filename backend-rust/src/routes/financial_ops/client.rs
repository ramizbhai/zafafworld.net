use crate::errors::AppError;
use crate::middleware::auth::{RequireAuth, RlsTx};
use crate::models::user::DomainType;
use crate::state::AppState;
use axum::{
    routing::get,
    Json, Router,
};
use serde_json::{json, Value};
use uuid::Uuid;
use sqlx::Row;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/invoices", get(list_client_invoices))
}

async fn list_client_invoices(
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
        "SELECT i.id, i.invoice_number, i.booking_id, i.amount::float8, i.tax_amount::float8, i.status, i.created_at::text, v.name_ar AS vendor_name_ar, v.name_en AS vendor_name_en \
         FROM invoices i \
         JOIN vendors v ON i.vendor_id = v.id \
         WHERE i.client_id = $1 \
         ORDER BY i.created_at DESC"
    )
    .bind(client_uuid)
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let invoices: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "id": r.get::<Uuid, _>("id").to_string(),
                "invoiceNumber": r.get::<String, _>("invoice_number"),
                "bookingId": r.get::<Option<Uuid>, _>("booking_id").map(|u| u.to_string()),
                "vendorNameAr": r.get::<String, _>("vendor_name_ar"),
                "vendorNameEn": r.get::<String, _>("vendor_name_en"),
                "amount": r.get::<f64, _>("amount"),
                "taxAmount": r.get::<f64, _>("tax_amount"),
                "status": r.get::<String, _>("status"),
                "createdAt": r.get::<String, _>("created_at"),
            })
        })
        .collect();

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "data": invoices
    })))
}
