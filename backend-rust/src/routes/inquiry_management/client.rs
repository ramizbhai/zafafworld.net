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
        .route("/inquiries", get(get_client_inquiries))
}

async fn get_client_inquiries(
    auth: RequireAuth,
    mut rls_tx: RlsTx,
) -> Result<Json<Value>, AppError> {
    if auth.role != DomainType::Client {
        return Err(AppError::Forbidden(
            "Client credentials required".to_string(),
        ));
    }
    let client_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid client ID in session".to_string()))?;

    let rows = sqlx::query(
        r#"SELECT 
            vi.id, vi.vendor_id, vi.product_id, vi.conversation_id, vi.name, vi.phone, vi.email,
            vi.event_date::text, vi.guest_count, vi.message, vi.status, vi.created_at::text, vi.updated_at::text,
            v.name_en AS vendor_name_en, v.name_ar AS vendor_name_ar,
            p.title AS product_name_en, p.title AS product_name_ar
         FROM vendor_inquiries vi
         LEFT JOIN vendors v ON vi.vendor_id = v.user_id
         LEFT JOIN vendor_products p ON vi.product_id = p.id
         WHERE vi.client_id = $1
         ORDER BY vi.created_at DESC"#
    )
    .bind(client_uuid)
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let mut inquiries = Vec::new();
    for row in rows {
        inquiries.push(json!({
            "id": row.get::<Uuid, _>("id"),
            "vendorId": row.get::<Uuid, _>("vendor_id"),
            "productId": row.get::<Option<Uuid>, _>("product_id"),
            "conversationId": row.get::<Option<Uuid>, _>("conversation_id"),
            "name": row.get::<Option<String>, _>("name"),
            "phone": row.get::<Option<String>, _>("phone"),
            "email": row.get::<Option<String>, _>("email"),
            "eventDate": row.get::<String, _>("event_date"),
            "guestCount": row.get::<i32, _>("guest_count"),
            "message": row.get::<String, _>("message"),
            "status": row.get::<String, _>("status"),
            "createdAt": row.get::<String, _>("created_at"),
            "updatedAt": row.get::<String, _>("updated_at"),
            "vendorNameEn": row.get::<Option<String>, _>("vendor_name_en"),
            "vendorNameAr": row.get::<Option<String>, _>("vendor_name_ar"),
            "productNameEn": row.get::<Option<String>, _>("product_name_en"),
            "productNameAr": row.get::<Option<String>, _>("product_name_ar"),
        }));
    }

    rls_tx
        .tx
        .commit()
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(json!({
        "status": "success",
        "data": inquiries
    })))
}
