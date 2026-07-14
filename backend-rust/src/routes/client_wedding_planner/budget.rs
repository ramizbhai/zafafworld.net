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
        .route("/budget", get(get_client_budget_summary).put(update_client_budget_total))
        .route("/budget/items", get(list_budget_items).post(create_budget_item))
        .route("/budget/items/:id", axum::routing::put(update_budget_item).delete(delete_budget_item))
        .route("/budget/analytics", get(get_budget_analytics))
}

#[derive(Deserialize)]
struct UpdateBudgetTotalRequest {
    #[serde(rename = "totalBudget")]
    total_budget: f64,
}

#[derive(Deserialize)]
struct CreateBudgetItemRequest {
    category: String,
    title: String,
    #[serde(rename = "plannedAmount")]
    planned_amount: f64,
    #[serde(rename = "actualAmount")]
    actual_amount: Option<f64>,
    status: Option<String>,
    #[serde(rename = "dueDate")]
    due_date: Option<String>,
    notes: Option<String>,
}

#[derive(Deserialize)]
struct UpdateBudgetItemRequest {
    category: Option<String>,
    title: Option<String>,
    #[serde(rename = "plannedAmount")]
    planned_amount: Option<f64>,
    #[serde(rename = "actualAmount")]
    actual_amount: Option<f64>,
    status: Option<String>,
    #[serde(rename = "dueDate")]
    due_date: Option<String>,
    notes: Option<String>,
}

async fn get_client_budget_summary(
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

    let total_budget: f64 = sqlx::query_scalar(
        "SELECT COALESCE(total_budget, 0.0)::float8 FROM client_budgets WHERE client_id = $1"
    )
    .bind(client_uuid)
    .fetch_one(&mut *rls_tx.tx)
    .await?;

    let spent_sum: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(total_price), 0.0)::float8 FROM core_bookings WHERE client_id = $1 AND status NOT IN ('cancelled', 'Draft_Inquiry')"
    )
    .bind(client_uuid)
    .fetch_one(&mut *rls_tx.tx)
    .await?;

    let item_planned_sum: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(planned_amount), 0.0)::float8 FROM client_budget_items WHERE client_id = $1"
    )
    .bind(client_uuid)
    .fetch_one(&mut *rls_tx.tx)
    .await?;

    let item_actual_sum: f64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(actual_amount), 0.0)::float8 FROM client_budget_items WHERE client_id = $1"
    )
    .bind(client_uuid)
    .fetch_one(&mut *rls_tx.tx)
    .await?;

    let total_actual_spent = spent_sum + item_actual_sum;
    let remaining_budget = total_budget - total_actual_spent;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "data": {
            "totalBudget": total_budget,
            "spentBookings": spent_sum,
            "spentItems": item_actual_sum,
            "totalSpent": total_actual_spent,
            "plannedItemsTotal": item_planned_sum,
            "remainingBudget": remaining_budget,
            "isExceeded": total_actual_spent > total_budget
        }
    })))
}

async fn update_client_budget_total(
    auth: RequireAuth,
    mut rls_tx: RlsTx,
    Json(payload): Json<UpdateBudgetTotalRequest>,
) -> Result<Json<Value>, AppError> {
    if auth.role != DomainType::Client {
        return Err(AppError::Forbidden(
            "Client credentials required".to_string(),
        ));
    }
    let client_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid client session".to_string()))?;

    sqlx::query(
        "INSERT INTO client_budgets (client_id, total_budget) VALUES ($1, $2) \
         ON CONFLICT (client_id) DO UPDATE SET total_budget = EXCLUDED.total_budget, updated_at = NOW()"
    )
    .bind(client_uuid)
    .bind(payload.total_budget)
    .execute(&mut *rls_tx.tx)
    .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Total budget updated successfully"
    })))
}

async fn list_budget_items(auth: RequireAuth, mut rls_tx: RlsTx) -> Result<Json<Value>, AppError> {
    if auth.role != DomainType::Client {
        return Err(AppError::Forbidden(
            "Client credentials required".to_string(),
        ));
    }
    let client_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid client session".to_string()))?;

    let rows = sqlx::query(
        "SELECT id, category, title, planned_amount::float8, actual_amount::float8, status, due_date::text, notes, booking_id \
         FROM client_budget_items \
         WHERE client_id = $1 \
         ORDER BY created_at DESC"
    )
    .bind(client_uuid)
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let items: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "id": r.get::<Uuid, _>("id").to_string(),
                "category": r.get::<String, _>("category"),
                "title": r.get::<String, _>("title"),
                "plannedAmount": r.get::<f64, _>("planned_amount"),
                "actualAmount": r.get::<f64, _>("actual_amount"),
                "status": r.get::<String, _>("status"),
                "dueDate": r.get::<Option<String>, _>("due_date"),
                "notes": r.get::<Option<String>, _>("notes"),
                "bookingId": r.get::<Option<Uuid>, _>("booking_id").map(|u| u.to_string()),
            })
        })
        .collect();

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "data": items
    })))
}

async fn create_budget_item(
    auth: RequireAuth,
    mut rls_tx: RlsTx,
    Json(payload): Json<CreateBudgetItemRequest>,
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

    let new_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO client_budget_items (id, client_id, category, title, planned_amount, actual_amount, status, due_date, notes) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
    )
    .bind(new_id)
    .bind(client_uuid)
    .bind(&payload.category)
    .bind(&payload.title)
    .bind(payload.planned_amount)
    .bind(payload.actual_amount.unwrap_or(0.0))
    .bind(payload.status.as_deref().unwrap_or("Planned"))
    .bind(parsed_date)
    .bind(&payload.notes)
    .execute(&mut *rls_tx.tx)
    .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Budget item created successfully",
        "id": new_id.to_string()
    })))
}

async fn update_budget_item(
    auth: RequireAuth,
    mut rls_tx: RlsTx,
    axum::extract::Path(item_id): axum::extract::Path<Uuid>,
    Json(payload): Json<UpdateBudgetItemRequest>,
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
        "UPDATE client_budget_items \
         SET category = COALESCE($1, category), \
             title = COALESCE($2, title), \
             planned_amount = COALESCE($3, planned_amount), \
             actual_amount = COALESCE($4, actual_amount), \
             status = COALESCE($5, status), \
             due_date = COALESCE($6, due_date), \
             notes = COALESCE($7, notes), \
             updated_at = NOW() \
         WHERE id = $8 AND client_id = $9",
    )
    .bind(payload.category)
    .bind(payload.title)
    .bind(payload.planned_amount)
    .bind(payload.actual_amount)
    .bind(payload.status)
    .bind(parsed_date)
    .bind(payload.notes)
    .bind(item_id)
    .bind(client_uuid)
    .execute(&mut *rls_tx.tx)
    .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Budget item updated successfully"
    })))
}

async fn delete_budget_item(
    auth: RequireAuth,
    mut rls_tx: RlsTx,
    axum::extract::Path(item_id): axum::extract::Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    if auth.role != DomainType::Client {
        return Err(AppError::Forbidden(
            "Client credentials required".to_string(),
        ));
    }
    let client_uuid = Uuid::parse_str(&auth.user_id)
        .map_err(|_| AppError::BadRequest("Invalid client session".to_string()))?;

    sqlx::query("DELETE FROM client_budget_items WHERE id = $1 AND client_id = $2")
        .bind(item_id)
        .bind(client_uuid)
        .execute(&mut *rls_tx.tx)
        .await?;

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "message": "Budget item deleted successfully"
    })))
}

async fn get_budget_analytics(
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
        "SELECT category, SUM(planned_amount)::float8 AS planned_total, SUM(actual_amount)::float8 AS actual_total \
         FROM client_budget_items \
         WHERE client_id = $1 \
         GROUP BY category"
    )
    .bind(client_uuid)
    .fetch_all(&mut *rls_tx.tx)
    .await?;

    let category_breakdown: Vec<Value> = rows
        .iter()
        .map(|r| {
            json!({
                "category": r.get::<String, _>("category"),
                "plannedTotal": r.get::<f64, _>("planned_total"),
                "actualTotal": r.get::<f64, _>("actual_total")
            })
        })
        .collect();

    rls_tx.tx.commit().await?;

    Ok(Json(json!({
        "status": "success",
        "data": {
            "categories": category_breakdown
        }
    })))
}
