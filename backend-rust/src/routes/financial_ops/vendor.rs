use crate::errors::AppError;
use crate::middleware::auth::{RequireVendor, RlsTx};
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
        .route("/wallet", get(get_vendor_wallet))
        .route("/payouts", get(list_vendor_payout_requests).post(create_vendor_payout_request))
}

#[derive(Deserialize)]
struct CreatePayoutRequestPayload {
    amount: f64,
    #[serde(rename = "bankName")]
    bank_name: Option<String>,
    iban: Option<String>,
}

async fn get_vendor_wallet(
    auth: RequireVendor,
    mut rls_tx: RlsTx,
) -> Result<Json<Value>, AppError> {
    let vendor_user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid vendor ID format".to_string()))?;

    let row = sqlx::query(
        "SELECT available_balance::float8, pending_escrow::float8, lifetime_earnings::float8 \
         FROM vendor_wallets WHERE vendor_id = $1",
    )
    .bind(vendor_user_uuid)
    .fetch_optional(&mut *rls_tx.tx)
    .await?;

    let wallet = match row {
        Some(r) => json!({
            "availableBalance": r.get::<f64, _>("available_balance"),
            "pendingEscrow": r.get::<f64, _>("pending_escrow"),
            "lifetimeEarnings": r.get::<f64, _>("lifetime_earnings"),
        }),
        None => json!({
            "availableBalance": 0.0,
            "pendingEscrow": 0.0,
            "lifetimeEarnings": 0.0,
        }),
    };

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "data": wallet
    })))
}

async fn list_vendor_payout_requests(
    auth: RequireVendor,
    mut rls_tx: RlsTx,
) -> Result<Json<Value>, AppError> {
    let vendor_user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid vendor ID format".to_string()))?;

    let rows = sqlx::query(
        "SELECT id, amount::float8, status, bank_name, iban, created_at::text \
         FROM payout_requests WHERE vendor_id = $1 ORDER BY created_at DESC",
    )
    .bind(vendor_user_uuid)
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let payouts: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "id": r.get::<Uuid, _>("id").to_string(),
                "amount": r.get::<f64, _>("amount"),
                "status": r.get::<String, _>("status"),
                "bankName": r.get::<Option<String>, _>("bank_name"),
                "iban": r.get::<Option<String>, _>("iban"),
                "createdAt": r.get::<String, _>("created_at"),
            })
        })
        .collect();

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "data": payouts
    })))
}

async fn create_vendor_payout_request(
    auth: RequireVendor,
    mut rls_tx: RlsTx,
    Json(payload): Json<CreatePayoutRequestPayload>,
) -> Result<Json<Value>, AppError> {
    let vendor_user_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid vendor ID format".to_string()))?;

    let payout_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO payout_requests (id, vendor_id, amount, status, bank_name, iban) \
         VALUES ($1, $2, $3, 'Pending', $4, $5)",
    )
    .bind(payout_id)
    .bind(vendor_user_uuid)
    .bind(payload.amount)
    .bind(&payload.bank_name)
    .bind(&payload.iban)
    .execute(&mut *rls_tx.tx)
    .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Payout request submitted successfully",
        "payoutId": payout_id.to_string()
    })))
}
