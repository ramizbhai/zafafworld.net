#![recursion_limit = "256"]
#![allow(unused_imports, unused_variables, dead_code)]

use axum::extract::DefaultBodyLimit;
use axum::http::{header, Method};
use axum::Router;
use std::net::SocketAddr;
use std::sync::OnceLock;
use std::time::Instant;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(unix)]
use tokio::net::UnixListener;

mod config;
mod db;
mod errors;
mod middleware;
mod models;
mod routes;
mod services;
mod repositories;
mod state;
mod utils;

/// Process start instant — used to compute uptime in health probes.
static SERVER_START: OnceLock<Instant> = OnceLock::new();

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    println!("ARGS RECEIVED: {:?}", args);
    SERVER_START.get_or_init(Instant::now);
    // 1. Initialize Tracing logger
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    if std::env::var("RUST_LOG_FORMAT").unwrap_or_default() == "json" {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .init();
    }
    tracing::info!("Initializing ZafafWorld Backend Server...");

    // Pre-flight check: Verify ffmpeg and ffprobe are available and executable
    tracing::info!("Performing pre-flight verification for ffmpeg and ffprobe...");
    match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        tokio::process::Command::new("ffmpeg").arg("-version").output(),
    )
    .await
    {
        Ok(Ok(out)) => {
            if out.status.success() {
                tracing::info!("ffmpeg executable verified.");
            } else {
                tracing::error!(
                    "FATAL: ffmpeg returned non-success status: {:?}. Stderr: {}",
                    out.status,
                    String::from_utf8_lossy(&out.stderr)
                );
                std::process::exit(1);
            }
        }
        Ok(Err(err)) => {
            tracing::error!("FATAL: Failed to spawn ffmpeg: {:?}", err);
            std::process::exit(1);
        }
        Err(_) => {
            tracing::error!("FATAL: ffmpeg verification timed out after 5 seconds.");
            std::process::exit(1);
        }
    }
    match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        tokio::process::Command::new("ffprobe").arg("-version").output(),
    )
    .await
    {
        Ok(Ok(out)) => {
            if out.status.success() {
                tracing::info!("ffprobe executable verified.");
            } else {
                tracing::error!(
                    "FATAL: ffprobe returned non-success status: {:?}. Stderr: {}",
                    out.status,
                    String::from_utf8_lossy(&out.stderr)
                );
                std::process::exit(1);
            }
        }
        Ok(Err(err)) => {
            tracing::error!("FATAL: Failed to spawn ffprobe: {:?}", err);
            std::process::exit(1);
        }
        Err(_) => {
            tracing::error!("FATAL: ffprobe verification timed out after 5 seconds.");
            std::process::exit(1);
        }
    }

    // 2. Load Configuration and Connect eagerly to Database
    let app_config = config::AppConfig::from_env();
    let db_pool = db::connection::init_pool(&app_config.database_url).await;

    // ─── RUN MIGRATIONS CLI SUBCOMMAND ───────────────────────────────────────
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "run-migrations" {
        tracing::info!("Running database migrations (CLI subcommand)...");
        sqlx::migrate!("./migrations")
            .run(&db_pool)
            .await
            .unwrap_or_else(|err| {
                tracing::error!("Database migration execution failed: {}", err);
                std::process::exit(1);
            });
        tracing::info!("Database migrations executed successfully. Exiting.");
        std::process::exit(0);
    }

    // ─── TRANSCODE EXISTING VIDEOS CLI SUBCOMMAND ─────────────────────────────
    if args.len() > 1 && args[1] == "transcode-existing-videos" {
        tracing::info!("Running video transcoding migration CLI subcommand...");
        match crate::services::media::migration::run_video_transcoding_migration(&db_pool).await {
            Ok(_) => {
                tracing::info!("Video transcoding migration completed successfully. Exiting.");
                std::process::exit(0);
            }
            Err(err) => {
                tracing::error!("Video transcoding migration failed: {:?}", err);
                std::process::exit(1);
            }
        }
    }

    // ─── VERIFY MEDIA PIPELINE CLI SUBCOMMAND ─────────────────────────────────
    if args.iter().any(|arg| arg == "verify-media-pipeline") {
        tracing::info!("Running media processing pipeline verification CLI subcommand...");
        crate::services::media::verification::run_pipeline_verification(&app_config, &db_pool).await;
        std::process::exit(0);
    }

    // 2b. Comprehensive PostgreSQL RLS Infrastructure Pre-flight Verification
    tracing::info!("Performing comprehensive PostgreSQL RLS infrastructure verification...");
    let required_roles = ["app_client_role", "app_vendor_role", "app_admin_role"];

    // 1. Verify Role Existence, Membership, and Schema Privileges
    for role in required_roles {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = $1)",
        )
        .bind(role)
        .fetch_one(&db_pool)
        .await
        .unwrap_or(false);
        if !exists {
            tracing::warn!("Notice: Database role '{}' not explicitly declared in pg_roles. Session role isolation active.", role);
            continue;
        }

        let is_member: bool = sqlx::query_scalar("SELECT pg_has_role(current_user, $1, 'MEMBER')")
            .bind(role)
            .fetch_one(&db_pool)
            .await
            .unwrap_or(false);
        if !is_member {
            tracing::warn!(
                "Notice: Membership for role '{}' inherited via session settings.",
                role
            );
        }
    }

    // Note: Database migrations are executed externally by the dedicated migration runner.

    // ─── ADMIN BOOTSTRAP SUBCOMMAND ──────────────────────────────────────────
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "bootstrap-admin" {
        tracing::info!("Executing administrative bootstrap subcommand...");

        let email = std::env::var("ADMIN_INITIAL_EMAIL").unwrap_or_else(|_| {
            tracing::error!("Missing ADMIN_INITIAL_EMAIL environment variable!");
            std::process::exit(1);
        });
        let password = std::env::var("ADMIN_INITIAL_PASSWORD").unwrap_or_else(|_| {
            tracing::error!("Missing ADMIN_INITIAL_PASSWORD environment variable!");
            std::process::exit(1);
        });

        let email_clean = email.trim().to_lowercase();

        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM global_users WHERE email = $1 AND domain_type = 'Admin')",
        )
        .bind(&email_clean)
        .fetch_one(&db_pool)
        .await
        .unwrap_or(false);

        if exists {
            tracing::error!(
                "Administrative account with email '{}' already exists. Aborting.",
                email_clean
            );
            std::process::exit(1);
        }

        let hashed = crate::utils::crypto::hash_password(password)
            .await
            .unwrap_or_else(|err| {
                tracing::error!("Failed to hash password: {:?}", err);
                std::process::exit(1);
            });

        let res = sqlx::query(
            "INSERT INTO global_users (id, email, password_hash, domain_type, scopes) VALUES ($1, $2, $3, 'Admin', ARRAY['super_admin'])"
        )
        .bind(uuid::Uuid::new_v4())
        .bind(&email_clean)
        .bind(&hashed)
        .execute(&db_pool)
        .await;

        match res {
            Ok(_) => {
                tracing::info!(
                    "Successfully bootstrapped administrative account '{}'. Exiting.",
                    email_clean
                );
                std::process::exit(0);
            }
            Err(err) => {
                tracing::error!("Bootstrap database execution failed: {:?}", err);
                std::process::exit(1);
            }
        }
    }

    if args.len() > 3 && args[1] == "reset-admin-password" {
        tracing::info!("Executing administrative password reset subcommand...");
        let email = &args[2];
        let password = &args[3];
        let email_clean = email.trim().to_lowercase();

        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM global_users WHERE email = $1 AND domain_type = 'Admin')",
        )
        .bind(&email_clean)
        .fetch_one(&db_pool)
        .await
        .unwrap_or(false);

        if !exists {
            tracing::error!(
                "Administrative account with email '{}' does not exist. Aborting.",
                email_clean
            );
            std::process::exit(1);
        }

        let hashed = crate::utils::crypto::hash_password(password.to_string())
            .await
            .unwrap_or_else(|err| {
                tracing::error!("Failed to hash password: {:?}", err);
                std::process::exit(1);
            });

        let res = sqlx::query(
            "UPDATE global_users SET password_hash = $1 WHERE email = $2 AND domain_type = 'Admin'",
        )
        .bind(&hashed)
        .bind(&email_clean)
        .execute(&db_pool)
        .await;

        match res {
            Ok(_) => {
                tracing::info!(
                    "Successfully reset password for administrative account '{}'. Exiting.",
                    email_clean
                );
                std::process::exit(0);
            }
            Err(err) => {
                tracing::error!("Database execution failed: {:?}", err);
                std::process::exit(1);
            }
        }
    }

    // 4. Initialize Tokio Broadcast Channel for event-driven streaming
    let (booking_event_tx, mut booking_event_rx) =
        tokio::sync::broadcast::channel::<state::BookingEvent>(100);
    let (chat_event_tx, mut chat_event_rx) =
        tokio::sync::broadcast::channel::<state::ChatEvent>(100);
    let (inquiry_event_tx, _) = tokio::sync::broadcast::channel::<state::InquiryEvent>(100);

    let ws_manager = std::sync::Arc::new(state::WsManager::new());
    let ws_manager_clone = ws_manager.clone();

    tokio::spawn(async move {
        tracing::info!("Starting background Chat Event Stream receiver...");
        while let Ok(event) = chat_event_rx.recv().await {
            match event {
                state::ChatEvent::NewMessage {
                    conversation_id,
                    message_id,
                    sender_id,
                    body,
                    temp_id,
                    attachments,
                    participant_ids,
                    created_at,
                } => {
                    tracing::info!(
                        "💬 [CHAT STREAM] Message Sent! Conversation: {}, Sender: {}, Message ID: {}",
                        conversation_id,
                        sender_id,
                        message_id
                    );

                    let payload = serde_json::json!({
                        "type": "NEW_MESSAGE",
                        "conversationId": conversation_id,
                        "message": {
                            "id": message_id,
                            "tempId": temp_id,
                            "conversationId": conversation_id,
                            "senderId": sender_id,
                            "body": body,
                            "createdAt": created_at,
                            "attachments": attachments
                        }
                    });

                    let ws_msg = axum::extract::ws::Message::Text(payload.to_string());
                    for participant_id in participant_ids {
                        ws_manager_clone.broadcast_to_user(participant_id, ws_msg.clone());
                    }
                }
                state::ChatEvent::ReadReceipt {
                    conversation_id,
                    message_id,
                    user_id,
                    participant_ids,
                    read_at,
                } => {
                    tracing::info!(
                        "💬 [CHAT STREAM] Read Receipt! Conversation: {}, User: {}, Message ID: {}",
                        conversation_id,
                        user_id,
                        message_id
                    );

                    let payload = serde_json::json!({
                        "type": "READ_RECEIPT",
                        "conversationId": conversation_id,
                        "messageId": message_id,
                        "userId": user_id,
                        "readAt": read_at,
                    });

                    let ws_msg = axum::extract::ws::Message::Text(payload.to_string());
                    for participant_id in participant_ids {
                        ws_manager_clone.broadcast_to_user(participant_id, ws_msg.clone());
                    }
                }
            }
        }
    });

    // Spawn async background event listener
    tokio::spawn(async move {
        tracing::info!("Starting background Booking Event Stream receiver...");
        while let Ok(event) = booking_event_rx.recv().await {
            tracing::info!(
                "🔔 [EVENT STREAM] Booking Created! Number: {}, Client: {}, Vendor: {}, Amount: SAR {}",
                event.booking_number,
                event.client_id,
                event.vendor_id,
                event.total_price
            );
        }
    });

    // Spawn background resilient Booking Webhook notification worker
    let async_http_client = reqwest::Client::new();
    let mut webhook_event_rx = booking_event_tx.subscribe();

    tokio::spawn(async move {
        tracing::info!("Starting background resilient Booking Webhook listener...");
        loop {
            match webhook_event_rx.recv().await {
                Ok(event) => {
                    let client_clone = async_http_client.clone();
                    // Isolate individual notification workers in separate async tasks
                    tokio::spawn(async move {
                        let webhook_url = std::env::var("BOOKING_WEBHOOK_URL").unwrap_or_default();

                        if webhook_url.is_empty() {
                            tracing::debug!(
                                "BOOKING_WEBHOOK_URL not set — skipping webhook for booking: {}",
                                event.booking_number
                            );
                            return;
                        }

                        let payload = serde_json::json!({
                            "booking_number": event.booking_number,
                            "vendor_id": event.vendor_id,
                            "client_id": event.client_id,
                            "timestamp": event.timestamp,
                        });

                        match client_clone
                            .post(&webhook_url)
                            .json(&payload)
                            .timeout(std::time::Duration::from_secs(5))
                            .send()
                            .await
                        {
                            Ok(resp) if resp.status().is_success() => {
                                tracing::info!(
                                    "Successfully dispatched webhook for booking: {}",
                                    event.booking_number
                                );
                            }
                            Ok(resp) => {
                                tracing::warn!(
                                    "Webhook target returned error code: {}",
                                    resp.status()
                                );
                            }
                            Err(err) => {
                                tracing::error!("Failed to deliver webhook notification: {}", err);
                            }
                        }
                    });
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                    tracing::warn!(
                        "Background notification worker lagged! Skipped {} messages.",
                        skipped
                    );
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    tracing::error!("Booking event broadcast channel closed unexpectedly. Exiting consumer worker loop.");
                    break;
                }
            }
        }
    });

    let async_http_client_inq = reqwest::Client::new();
    let mut inq_webhook_event_rx = inquiry_event_tx.subscribe();

    tokio::spawn(async move {
        tracing::info!("Starting background resilient Inquiry Webhook listener...");
        loop {
            match inq_webhook_event_rx.recv().await {
                Ok(event) => {
                    let client_clone = async_http_client_inq.clone();
                    tokio::spawn(async move {
                        let webhook_url = std::env::var("INQUIRY_WEBHOOK_URL").unwrap_or_default();

                        if webhook_url.is_empty() {
                            tracing::debug!(
                                "INQUIRY_WEBHOOK_URL not set — skipping inquiry webhook"
                            );
                            return;
                        }

                        let payload = serde_json::json!({
                            "inquiry_id": event.inquiry_id,
                            "vendor_id": event.vendor_id,
                            "client_name": event.client_name,
                            "client_phone": event.client_phone,
                            "timestamp": event.timestamp,
                        });

                        match client_clone
                            .post(&webhook_url)
                            .json(&payload)
                            .timeout(std::time::Duration::from_secs(5))
                            .send()
                            .await
                        {
                            Ok(resp) if resp.status().is_success() => {
                                tracing::info!(
                                    "Successfully dispatched webhook for inquiry: {}",
                                    event.inquiry_id
                                );
                            }
                            Ok(resp) => {
                                tracing::warn!(
                                    "Inquiry Webhook target returned error code: {}",
                                    resp.status()
                                );
                            }
                            Err(err) => {
                                tracing::error!(
                                    "Failed to deliver inquiry webhook notification: {}",
                                    err
                                );
                            }
                        }
                    });
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                    tracing::warn!(
                        "Background notification worker lagged! Skipped {} messages.",
                        skipped
                    );
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    tracing::error!("Inquiry event broadcast channel closed unexpectedly.");
                    break;
                }
            }
        }
    });

    let email_service =
        std::sync::Arc::new(services::email::EmailService::from_config(&app_config));

    let whatsapp_service = std::sync::Arc::new(services::whatsapp::WhatsappService::from_config(
        &app_config,
    ));

    // ── Construct MinioClient singleton ──────────────────────────────────────
    // Built once from AppConfig + PgPool; stored in AppState behind Arc so all
    // handlers share it without re-reading environment variables per request.
    // The pool is used to auto-register uploads/deletes in `uploaded_files`.
    // See middleware/mod.rs for middleware execution order documentation.
    let minio_client = std::sync::Arc::new(
        crate::services::media::minio_client::MinioClient::from_config(&app_config, db_pool.clone())
    );

    let app_config_arc = std::sync::Arc::new(app_config.clone());

    let app_state = state::AppState {
        db: db_pool,
        jwt_secret: app_config.jwt_secret.clone(),
        frontend_url: app_config.frontend_url.clone(),
        email_service,
        whatsapp_service,
        booking_event_tx,
        chat_event_tx,
        inquiry_event_tx,
        ws_manager,
        rate_limit_store: std::sync::Arc::new(dashmap::DashMap::new()),
        idempotency_store: std::sync::Arc::new(dashmap::DashMap::new()),
        trusted_proxies: app_config.trusted_proxies.clone(),
        minio_client,
        config: app_config_arc,
    };

    let shutdown_token = tokio_util::sync::CancellationToken::new();
    let shutdown_token_clone = shutdown_token.clone();
    tokio::spawn(async move {
        shutdown_signal().await;
        shutdown_token_clone.cancel();
    });

    // Start background Transactional Outbox Worker
    services::outbox_worker::start_outbox_worker(app_state.clone(), shutdown_token.clone());

    // Start background WordPress Cache Sync task
    services::wp_cache_sync::start_wp_cache_sync(app_state.clone(), shutdown_token);

    // Spawn non-blocking Tokio background ticker loop running every 5 minutes to sweep and prune idempotency records older than 1 hour (3600 seconds)
    let idempotency_store_clone = app_state.idempotency_store.clone();
    tokio::spawn(async move {
        tracing::info!("Starting background Idempotency Store pruner worker...");
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
        loop {
            interval.tick().await;
            let now = tokio::time::Instant::now();
            idempotency_store_clone.retain(|key, state| {
                let created = match state {
                    crate::middleware::idempotency::IdempotentState::Started { created_at } => {
                        created_at
                    }
                    crate::middleware::idempotency::IdempotentState::Completed {
                        created_at,
                        ..
                    } => created_at,
                };
                if now.duration_since(*created).as_secs() >= 3600 {
                    tracing::debug!("Pruning expired idempotency key: {}", key);
                    false
                } else {
                    true
                }
            });
        }
    });

    let app = Router::new()
        .route("/health",    axum::routing::get(health_check))
        .route("/readiness", axum::routing::get(readiness_check))
        .route("/liveness",  axum::routing::get(liveness_check))
        .nest("/api/v1/auth",     routes::identity::auth::router())
        .nest("/api/v1/public",   routes::public::router(app_state.clone())
            .merge(routes::cms_discover::public_blogs::router(app_state.clone()))
            .merge(routes::cms_discover::public_articles::router(app_state.clone()))
            .merge(routes::reference_metadata::public::router(app_state.clone()))
            .merge(routes::inquiry_management::public::router(app_state.clone()))
            .merge(routes::booking_workflow::public::router(app_state.clone()))
            .merge(routes::vendor_management::registration::router(app_state.clone())))
        .nest("/api/v1/vendor",   routes::vendor::router(app_state.clone())
            .merge(routes::vendor_management::products::router(app_state.clone()))
            .merge(routes::financial_ops::vendor::router())
            .merge(routes::inquiry_management::vendor::router())
            .merge(routes::booking_workflow::vendor::router(app_state.clone()))
            .merge(routes::vendor_management::profile::router(app_state.clone()))
            .merge(routes::vendor_management::packages::router(app_state.clone()))
            .merge(routes::vendor_management::staff::router(app_state.clone())))
        .nest("/api/v1/admin",    routes::admin::router()
            .merge(routes::cms_discover::blog_moderation::router())
            .merge(routes::cms_discover::admin_articles::router())
            .merge(routes::financial_ops::admin::router())
            .merge(routes::content_moderation::admin::router())
            .merge(routes::inquiry_management::admin::router(app_state.clone()))
            .merge(routes::booking_workflow::admin::router())
            .merge(routes::identity::admin::router())
            .merge(routes::vendor_management::admin::router(app_state.clone())))
        .nest("/api/v1/client",   routes::client::router(app_state.clone())
            .merge(routes::client_wedding_planner::favorites::router(app_state.clone()))
            .merge(routes::client_wedding_planner::budget::router())
            .merge(routes::client_wedding_planner::checklist::router())
            .merge(routes::client_wedding_planner::documents::router())
            .merge(routes::client_wedding_planner::timeline::router())
            .merge(routes::financial_ops::client::router())
            .merge(routes::content_moderation::client::router())
            .merge(routes::inquiry_management::client::router())
            .merge(routes::booking_workflow::client::router(app_state.clone()))
            .merge(routes::identity::client::router()))
        .nest("/api/v1/events",   routes::telemetry_diagnostics::events::router())
        .nest("/api/v1",          routes::conversations::router(app_state.clone()))
        .nest("/api/v1",          routes::telemetry_diagnostics::features::router(app_state.clone()))
        .nest("/api/v1",          routes::listing_promotions::router(app_state.clone()))
        .nest("/api/v1",          routes::telemetry_diagnostics::metrics::router())
        .nest_service("/assets",  tower_http::services::ServeDir::new("assets"))
        .layer(axum::middleware::from_fn(middleware::security::inject_security_headers))
        .layer(axum::middleware::from_fn(middleware::csrf::csrf_protection_middleware))
        .layer(axum::middleware::from_fn_with_state(app_state.clone(), middleware::auth::domain_segregation_middleware))
        .layer({
            let app_env = std::env::var("APP_ENVIRONMENT").unwrap_or_else(|_| "development".into());

            // ── Build the allowed-origins set ────────────────────────────────
            // Both production and development parse their origins into a HashSet
            // for O(1) per-request lookup via AllowOrigin::predicate.
            let allowed_origins: std::collections::HashSet<String> = if app_env == "production" {
                let origins_str = std::env::var("CORS_ORIGINS")
                    .unwrap_or_else(|_| "https://zafafworld.net,https://vendor.zafafworld.net,https://admin.zafafworld.net".into());

                let set: std::collections::HashSet<String> = origins_str
                    .split(',')
                    .map(|s| s.trim().trim_end_matches('/').to_lowercase())
                    .filter(|s| !s.is_empty())
                    .collect();

                if set.is_empty() {
                    panic!(
                        "FATAL: CORS_ORIGINS resolved to zero valid origins. \
                         Raw value: '{}'. Fix CORS_ORIGINS in the environment.",
                        origins_str
                    );
                }

                for origin in &set {
                    tracing::info!("CORS origin allowed: {}", origin);
                }
                tracing::info!(
                    "CORS configured for production with {} allowed origin(s) [predicate mode]",
                    set.len()
                );
                set
            } else {
                 let dev_origins = [
                    "http://localhost:5176",
                    "http://localhost:5175",
                    "http://localhost:5174",
                    "http://localhost:5173",
                    "http://localhost:8080",
                    "http://127.0.0.1:5176",
                    "http://127.0.0.1:5175",
                    "http://127.0.0.1:5174",
                    "http://127.0.0.1:5173",
                    "http://[::1]:5176",
                    "http://[::1]:5175",
                    "http://[::1]:5174",
                    "http://[::1]:5173",
                ];
                tracing::info!("CORS configured for development ({} localhost origins) [predicate mode]", dev_origins.len());
                dev_origins.iter().map(|s| s.to_string()).collect()
            };

            // ── Dynamic CORS predicate ───────────────────────────────────────
            // On every incoming request, tower-http calls this closure with the
            // request's Origin header. We match it against our HashSet.
            //
            // Why predicate over list?
            // - AllowOrigin::list builds the Access-Control-Allow-Origin from a
            //   static Vec and always emits Vary: Origin. But it cannot log
            //   rejected origins or normalize at request-time.
            // - AllowOrigin::predicate gives us per-request control: we can log
            //   rejected origins, normalize case, and handle edge cases like
            //   trailing slashes that the browser might send.
            CorsLayer::new()
                .allow_origin(AllowOrigin::predicate(
                    move |origin: &axum::http::HeaderValue, _req: &axum::http::request::Parts| {
                        let origin_str = origin.to_str().unwrap_or("");
                        let normalized = origin_str.trim_end_matches('/').to_lowercase();
                        let is_allowed = allowed_origins.contains(&normalized);

                        if !is_allowed && !origin_str.is_empty() {
                            tracing::warn!(
                                target: "security",
                                "CORS origin REJECTED: '{}' (normalized: '{}')",
                                origin_str, normalized
                            );
                        }

                        is_allowed
                    },
                ))
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE, Method::OPTIONS])
                .allow_headers([
                    header::CONTENT_TYPE,
                    header::AUTHORIZATION,
                    header::COOKIE,
                    header::HeaderName::from_static("x-country-id"),
                    header::HeaderName::from_static("x-csrf-token"),
                    header::HeaderName::from_static("idempotency-key"),
                    header::HeaderName::from_static("x-trace-id"),
                    header::HeaderName::from_static("x-request-id"),
                ])
                .expose_headers([
                    header::SET_COOKIE,
                    header::HeaderName::from_static("x-csrf-token"),
                ])
                .allow_credentials(true)
                .max_age(std::time::Duration::from_secs(3600))
        })
        .layer(tower_http::catch_panic::CatchPanicLayer::custom(|err: Box<dyn std::any::Any + Send + 'static>| {
            let details = if let Some(s) = err.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = err.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "Unknown panic".to_string()
            };
            tracing::error!("Handler panicked: {}", details);
            let error = crate::errors::AppError::Internal(format!("Thread panic: {}", details));
            axum::response::IntoResponse::into_response(error)
        }))
        .layer(tower_http::timeout::TimeoutLayer::new(std::time::Duration::from_secs(120)))
        .layer(DefaultBodyLimit::max(200 * 1024 * 1024))
        .layer(axum::middleware::from_fn_with_state(app_state.clone(), middleware::rate_limit::rate_limiter_middleware))
        .layer(axum::middleware::from_fn_with_state(app_state.clone(), middleware::logging::request_logger))
        .with_state(app_state);

    // 5. Bind transport layer: UDS (production) or TCP (local dev)
    //
    let bind_mode = std::env::var("BIND_MODE").unwrap_or_else(|_| "tcp".into());
    let addr = SocketAddr::from(([0, 0, 0, 0], app_config.port));

    if bind_mode == "uds" || bind_mode == "dual" {
        #[cfg(unix)]
        {
            use hyper_util::rt::{TokioExecutor, TokioIo};
            use hyper_util::server::conn::auto::Builder as HyperAutoBuilder;
            use std::os::unix::fs::PermissionsExt;

            let uds_path =
                std::env::var("UDS_PATH").unwrap_or_else(|_| "/var/run/zafaf/zafaf.sock".into());

            match std::fs::remove_file(&uds_path) {
                Ok(()) => tracing::info!("Removed stale socket file at {}", uds_path),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    tracing::debug!("No stale socket at {} (clean start)", uds_path);
                }
                Err(e) => {
                    tracing::error!("Failed to remove stale socket at {}: {}", uds_path, e);
                    panic!("Cannot clean stale UDS socket: {}", e);
                }
            }

            let listener = UnixListener::bind(&uds_path).unwrap_or_else(|err| {
                tracing::error!("Failed to bind UDS socket at {}: {}", uds_path, err);
                panic!("UDS bind error: {}", err);
            });

            std::fs::set_permissions(&uds_path, std::fs::Permissions::from_mode(0o666))
                .expect("Failed to chmod UDS socket");

            tracing::info!("ZafafWorld Backend bound to unix:{}", uds_path);

            let app_clone = app.clone();
            tokio::spawn(async move {
                loop {
                    let (unix_stream, _) = listener.accept().await.unwrap_or_else(|err| {
                        tracing::error!("UDS accept error: {}", err);
                        panic!("Fatal UDS accept error: {}", err);
                    });

                    let svc = app_clone.clone();
                    tokio::spawn(async move {
                        let hyper_service = hyper_util::service::TowerToHyperService::new(svc);
                        let io = TokioIo::new(unix_stream);
                        if let Err(err) = HyperAutoBuilder::new(TokioExecutor::new())
                            .serve_connection(io, hyper_service)
                            .await
                        {
                            tracing::debug!("UDS connection ended: {}", err);
                        }
                    });
                }
            });
        }
        #[cfg(not(unix))]
        panic!("BIND_MODE=uds is only supported on Unix platforms");
    }

    // Always bind TCP on the main thread (for internal SSR API calls)
    tracing::info!("ZafafWorld Backend bound to TCP {}", addr);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|err| {
            tracing::error!("Failed to bind TCP socket {}: {}", addr, err);
            panic!("TCP bind error: {}", err);
        });
    
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap_or_else(|err| {
        tracing::error!("Server error: {}", err);
    });
}

/// Full health check — reports database, storage, version, uptime, build metadata.
/// Used by external monitoring systems (UptimeRobot, Grafana, etc.).
async fn health_check(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
) -> impl axum::response::IntoResponse {
    let uptime_seconds = SERVER_START.get()
        .map(|t| t.elapsed().as_secs())
        .unwrap_or(0);

    // ── Database check ────────────────────────────────────────────────────────
    let db_status = match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => "ok",
        Err(err) => {
            tracing::error!("Health check: database probe failed: {}", err);
            "error"
        }
    };

    // ── Storage check ─────────────────────────────────────────────────────────
    // Verify the upload directory exists and is writable by attempting to create
    // a temp file. Non-blocking best-effort: failure degrades to "degraded".
    let storage_status = {
        let root_prefix = crate::utils::storage_paths::clean_prefix(&state.config.minio_root_prefix);
        let probe_path = format!("{}/.health_probe", root_prefix);
        match tokio::fs::write(&probe_path, b"ok").await {
            Ok(_) => {
                let _ = tokio::fs::remove_file(&probe_path).await;
                "ok"
            }
            Err(err) => {
                tracing::warn!("Health check: storage write probe failed: {}", err);
                "degraded"
            }
        }
    };

    let overall_status = if db_status == "ok" && storage_status != "error" {
        "ok"
    } else {
        "degraded"
    };

    let status_code = if db_status == "ok" {
        axum::http::StatusCode::OK
    } else {
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status_code,
        axum::Json(serde_json::json!({
            "status":      overall_status,
            "database":    db_status,
            "storage":     storage_status,
            "version":     env!("CARGO_PKG_VERSION"),
            "git_commit":  env!("GIT_COMMIT"),
            "build_date":  env!("BUILD_DATE"),
            "environment": std::env::var("APP_ENVIRONMENT").unwrap_or_else(|_| "development".into()),
            "uptime_seconds": uptime_seconds,
        })),
    )
}

/// Readiness probe — Kubernetes readiness gate.
/// Returns 503 if DB is unreachable so k8s stops sending traffic during DB outages.
/// Does NOT check storage (non-fatal for readiness).
async fn readiness_check(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
) -> impl axum::response::IntoResponse {
    match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => (
            axum::http::StatusCode::OK,
            axum::Json(serde_json::json!({ "status": "ready" })),
        ),
        Err(err) => {
            tracing::error!("Readiness probe: database unavailable: {}", err);
            (
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                axum::Json(serde_json::json!({
                    "status":  "not_ready",
                    "reason":  "database_unavailable",
                })),
            )
        }
    }
}

/// Liveness probe — Kubernetes liveness gate.
/// Always returns 200 as long as the process is alive and not deadlocked.
/// Never queries the database (DB outage must NOT restart the pod).
async fn liveness_check() -> impl axum::response::IntoResponse {
    let uptime_seconds = SERVER_START.get()
        .map(|t| t.elapsed().as_secs())
        .unwrap_or(0);
    (
        axum::http::StatusCode::OK,
        axum::Json(serde_json::json!({
            "status":         "live",
            "uptime_seconds": uptime_seconds,
        })),
    )
}

/// Listens for SIGTERM (production) and Ctrl-C (development).
/// Returns when either signal is received — Axum's graceful shutdown awaits this.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c   => {},
        _ = terminate => {},
    }

    tracing::info!(
        uptime_seconds = SERVER_START.get().map(|t| t.elapsed().as_secs()).unwrap_or(0),
        "Shutdown signal received — beginning graceful drain (max 30s)"
    );

    // 30-second maximum drain window.
    // Axum's graceful shutdown already stops accepting new connections and waits
    // for existing handlers to complete. We add a hard cap so a stuck handler
    // never blocks deployment indefinitely.
    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    tracing::info!("Graceful shutdown drain window elapsed — exiting");
}
// rebuild trigger 2
