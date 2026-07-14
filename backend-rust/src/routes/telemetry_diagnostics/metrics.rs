// GET /api/v1/metrics — Prometheus text format counter exposition.
// NOTE: In production, restrict this endpoint to internal/loopback traffic only
// (e.g., nginx allow 127.0.0.1; deny all;) — no authentication is applied here
// as Prometheus scrapers do not send auth headers by default.

use axum::{http::header, response::IntoResponse};
use crate::services::metrics::render_prometheus_text;

pub fn router() -> axum::Router<crate::state::AppState> {
    axum::Router::new()
        .route("/metrics", axum::routing::get(get_metrics))
        .route("/analytics/impressions", axum::routing::post(track_impressions))
}

async fn get_metrics() -> impl IntoResponse {
    let body = render_prometheus_text();
    (
        [(header::CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")],
        body,
    )
}

// ─── POST IMPRESSIONS: /api/v1/analytics/impressions ─────────────────────────
async fn track_impressions() -> impl IntoResponse {
    // Fire and forget endpoint for frontend batch impressions.
    // In a real implementation this would write to Redis/Clickhouse or a queue.
    // For now we just return 204 to complete the circuit and avoid 404s.
    axum::http::StatusCode::NO_CONTENT
}
