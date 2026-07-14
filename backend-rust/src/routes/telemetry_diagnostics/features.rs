use crate::errors::AppError;
use crate::state::AppState;
use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/features", get(list_features))
        .with_state(state)
}

async fn list_features(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<Value>, AppError> {
    let rows = sqlx::query(
        r#"
        SELECT id, name_en, name_ar, category, input_type, created_at, updated_at
        FROM features
        ORDER BY category ASC, name_en ASC
        "#,
    )
    .fetch_all(&state.db)
    .await?;

    let mut features = Vec::new();
    for row in rows {
        use sqlx::Row;
        let id: uuid::Uuid = row.get("id");
        let name_en: String = row.get("name_en");
        let name_ar: String = row.get("name_ar");
        let category: String = row.get("category");
        let input_type: String = row.get("input_type");
        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");
        let updated_at: chrono::DateTime<chrono::Utc> = row.get("updated_at");

        features.push(json!({
            "id": id.to_string(),
            "nameEn": name_en,
            "nameAr": name_ar,
            "category": category,
            "inputType": input_type,
            "createdAt": created_at.to_rfc3339(),
            "updatedAt": updated_at.to_rfc3339(),
        }));
    }

    Ok(Json(json!({
        "status": "success",
        "data": features,
    })))
}
