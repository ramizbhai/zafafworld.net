// GET /api/v1/metrics — Prometheus text format counter exposition.
// NOTE: This endpoint is currently publicly reachable (no nginx IP allow-list
// restriction is configured for it). If Prometheus scraper access should be
// limited, add an `allow 127.0.0.1; deny all;` block in infra/nginx/conf.d/10-api.conf
// for a `location = /api/v1/metrics` stanza.

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
// Intentional no-op: impression events are batched by the frontend BFF and
// acknowledged here to avoid 404s, but no server-side persistence is
// implemented. Raw impression volume can be inferred from nginx access logs.
// If a server-side analytics store is added later, this is the integration point.
async fn track_impressions() -> impl IntoResponse {
    axum::http::StatusCode::NO_CONTENT
}
