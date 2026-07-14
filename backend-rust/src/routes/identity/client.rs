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
        .route("/profile", get(get_client_profile))
}

async fn get_client_profile(auth: RequireAuth, mut rls_tx: RlsTx) -> Result<Json<Value>, AppError> {
    if auth.role != DomainType::Client {
        return Err(AppError::Forbidden(
            "Client credentials required".to_string(),
        ));
    }
    let client_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid client ID in session".to_string()))?;

    let profile_row = sqlx::query("SELECT wedding_date FROM client_profiles WHERE client_id = $1")
        .bind(client_uuid)
        .fetch_optional(&mut *rls_tx.tx)
        .await?;

    let mut days_remaining = None;
    let mut wedding_date_str = None;
    if let Some(row) = profile_row {
        if let Ok(w_date) = row.try_get::<chrono::NaiveDate, _>("wedding_date") {
            let today = chrono::Utc::now().date_naive();
            days_remaining = Some((w_date - today).num_days());
            wedding_date_str = Some(w_date.to_string());
        }
    }

    let budget_row =
        sqlx::query("SELECT total_budget::float8 FROM client_budgets WHERE client_id = $1")
            .bind(client_uuid)
            .fetch_optional(&mut *rls_tx.tx)
            .await?;

    let mut total_budget = 0.0;
    if let Some(row) = budget_row {
        total_budget = row.get("total_budget");
    }

    // Dynamic spent budget calculation based on active / verified bookings
    let spent_sum: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(total_price), 0.0)::float8 FROM core_bookings WHERE client_id = $1 AND status NOT IN ('cancelled', 'Draft_Inquiry')"
    )
    .bind(client_uuid)
    .fetch_one(&mut *rls_tx.tx)
    .await?;

    let spent_amount = spent_sum;

    rls_tx
        .tx
        .commit()
        .await
        .map_err(|err| AppError::Database(err.to_string()))?;

    Ok(Json(json!({
        "status": "success",
        "data": {
            "weddingDate": wedding_date_str,
            "daysRemaining": days_remaining,
            "budget": {
                "totalBudget": total_budget,
                "spentAmount": spent_amount,
                "remainingBudget": total_budget - spent_amount,
            }
        }
    })))
}
