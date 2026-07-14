use crate::state::AppState;
use serde_json::Value;
use sqlx::{PgPool, Row};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

pub struct OutboxMetrics {
    pub total_processed: AtomicU64,
    pub total_success: AtomicU64,
    pub total_failures: AtomicU64,
    pub total_retries: AtomicU64,
    pub email_latency_sum_ms: AtomicU64,
    pub email_count: AtomicU64,
    pub whatsapp_latency_sum_ms: AtomicU64,
    pub whatsapp_count: AtomicU64,
}

pub static METRICS: OutboxMetrics = OutboxMetrics {
    total_processed: AtomicU64::new(0),
    total_success: AtomicU64::new(0),
    total_failures: AtomicU64::new(0),
    total_retries: AtomicU64::new(0),
    email_latency_sum_ms: AtomicU64::new(0),
    email_count: AtomicU64::new(0),
    whatsapp_latency_sum_ms: AtomicU64::new(0),
    whatsapp_count: AtomicU64::new(0),
};

pub fn start_outbox_worker(state: AppState, cancel_token: CancellationToken) {
    tokio::spawn(async move {
        let mut restart_count = 0;
        let mut last_restart = Instant::now();

        loop {
            if cancel_token.is_cancelled() {
                break;
            }

            let state_clone = state.clone();
            let token_clone = cancel_token.clone();

            info!("Supervisor: Launching Outbox Worker thread instance...");
            let handle = tokio::spawn(async move {
                run_worker_loop(state_clone, token_clone).await;
            });

            match handle.await {
                Ok(_) => {
                    info!("Supervisor: Outbox Worker finished cleanly.");
                    break;
                }
                Err(join_err) => {
                    if join_err.is_cancelled() {
                        info!("Supervisor: Worker loop cancelled.");
                        break;
                    }

                    let panic_reason = if join_err.is_panic() {
                        let payload = join_err.into_panic();
                        if let Some(s) = payload.downcast_ref::<&str>() {
                            s.to_string()
                        } else if let Some(s) = payload.downcast_ref::<String>() {
                            s.clone()
                        } else {
                            "Unknown panic payload".to_string()
                        }
                    } else {
                        join_err.to_string()
                    };

                    error!(
                        "Supervisor: Outbox Worker panicked/crashed! Reason: {}.",
                        panic_reason
                    );

                    if last_restart.elapsed() < Duration::from_secs(5) {
                        restart_count += 1;
                    } else {
                        restart_count = 1;
                    }
                    last_restart = Instant::now();

                    if restart_count > 5 {
                        error!("Supervisor: Outbox Worker crashed repeatedly (5 times in < 5s). Suspending restarts for 30 seconds to prevent resource thrashing...");
                        tokio::time::sleep(Duration::from_secs(30)).await;
                        restart_count = 0;
                    } else {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        }
    });
}

async fn run_worker_loop(state: AppState, cancel_token: CancellationToken) {
    // 1. Startup Recovery: Convert zombie PROCESSING rows back to RETRYING
    if let Err(e) = recover_abandoned_events(&state.db).await {
        error!("Startup recovery error: {}", e);
    }

    let mut poll_interval =
        tokio::time::interval(Duration::from_millis(state.config.outbox_poll_interval_ms));
    let mut clean_interval = tokio::time::interval(Duration::from_secs(
        state.config.outbox_cleanup_interval_secs,
    ));
    let mut join_set = JoinSet::new();

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                info!("Outbox worker loop received cancellation signal. Cleaning up active tasks...");
                break;
            }
            _ = poll_interval.tick() => {
                // Drain completed tasks from JoinSet to free memory
                while let Some(Ok(())) = join_set.try_join_next() {}

                // Check active count to enforce parallel delivery limit
                if join_set.len() < state.config.outbox_max_parallel_deliveries {
                    if let Err(e) = poll_and_process_outbox(&state, &cancel_token, &mut join_set).await {
                        error!("Outbox worker processing error: {}", e);
                    }
                } else {
                    debug!("Outbox worker parallel limit reached. Skipping this tick. Active: {}", join_set.len());
                }
            }
            _ = clean_interval.tick() => {
                if let Err(e) = run_outbox_cleanup(&state.db, &state.config).await {
                    error!("Outbox cleanup error: {}", e);
                }
            }
        }
    }

    // Wait for all active tasks in JoinSet to finish cleanly before exiting
    info!(
        "Outbox worker waiting for {} active background tasks to complete...",
        join_set.len()
    );
    while let Some(res) = join_set.join_next().await {
        if let Err(e) = res {
            error!("Active outbox task join error during shutdown: {}", e);
        }
    }
    info!("Outbox worker graceful shutdown completed.");
}

async fn recover_abandoned_events(db: &PgPool) -> Result<(), String> {
    let res = sqlx::query(
        "UPDATE notification_outbox 
         SET status = 'RETRYING', next_retry_at = NOW(), updated_at = NOW() 
         WHERE status = 'PROCESSING'",
    )
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;

    let count = res.rows_affected();
    if count > 0 {
        info!(
            "Startup Recovery: Reset {} abandoned 'PROCESSING' outbox events to 'RETRYING'",
            count
        );
    }
    Ok(())
}

async fn poll_and_process_outbox(
    state: &AppState,
    cancel_token: &CancellationToken,
    join_set: &mut JoinSet<()>,
) -> Result<(), String> {
    // 1. Begin transaction to lock rows
    let mut tx = state.db.begin().await.map_err(|e| e.to_string())?;

    // 2. Query pending/retrying events
    let limit = state.config.outbox_batch_size;
    let rows = sqlx::query(
        "SELECT id, event_type, aggregate_type, aggregate_id, payload, attempt_count, channel_delivery 
         FROM notification_outbox 
         WHERE status IN ('PENDING', 'RETRYING') AND next_retry_at <= NOW() 
         ORDER BY created_at 
         LIMIT $1 
         FOR UPDATE SKIP LOCKED"
    )
    .bind(limit)
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    if rows.is_empty() {
        let _ = tx.rollback().await;
        return Ok(());
    }

    let mut event_ids = Vec::new();
    let mut events = Vec::new();

    for row in rows {
        let id: Uuid = row.get("id");
        let event_type: String = row.get("event_type");
        let aggregate_type: String = row.get("aggregate_type");
        let aggregate_id: Uuid = row.get("aggregate_id");
        let payload: Value = row.get("payload");
        let attempt_count: i32 = row.get("attempt_count");
        let channel_delivery: Value = row.get("channel_delivery");

        event_ids.push(id);
        events.push((
            id,
            event_type,
            aggregate_type,
            aggregate_id,
            payload,
            attempt_count,
            channel_delivery,
        ));
    }

    // 3. Mark selected events as PROCESSING to release database lock quickly
    sqlx::query(
        "UPDATE notification_outbox 
         SET status = 'PROCESSING', last_attempt_at = NOW(), updated_at = NOW() 
         WHERE id = ANY($1)",
    )
    .bind(&event_ids)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    // 4. Spawn tasks into JoinSet
    for (
        id,
        event_type,
        _aggregate_type,
        _aggregate_id,
        payload,
        attempt_count,
        channel_delivery,
    ) in events
    {
        let email_service = state.email_service.clone();
        let whatsapp_service = state.whatsapp_service.clone();
        let db = state.db.clone();
        let config = state.config.clone();
        let cancel_token_clone = cancel_token.clone();

        join_set.spawn(async move {
            if cancel_token_clone.is_cancelled() {
                // Revert event back to PENDING if server is shutting down
                let _ = sqlx::query(
                    "UPDATE notification_outbox 
                     SET status = 'RETRYING', next_retry_at = NOW(), updated_at = NOW() 
                     WHERE id = $1 AND status = 'PROCESSING'"
                )
                .bind(id)
                .execute(&db)
                .await;
                return;
            }

            debug!("Processing outbox event {} (type: {})", id, event_type);
            METRICS.total_processed.fetch_add(1, Ordering::Relaxed);

            let mut channels = channel_delivery.clone();
            if !channels.is_object() {
                channels = serde_json::json!({});
            }

            let result = if event_type == "new_inquiry" {
                // Parse payload details safely
                let dest_email = payload.get("vendor_email").and_then(|v| v.as_str()).unwrap_or("");
                let dest_phone = payload.get("vendor_phone").and_then(|v| v.as_str());
                let customer_name = payload.get("customer_name").and_then(|v| v.as_str()).unwrap_or("");
                let customer_phone = payload.get("customer_phone").and_then(|v| v.as_str()).unwrap_or("");
                let customer_email = payload.get("customer_email").and_then(|v| v.as_str()).unwrap_or("");
                let event_date = payload.get("event_date").and_then(|v| v.as_str()).unwrap_or("");
                let guest_count = payload.get("guest_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
                let message = payload.get("message").and_then(|v| v.as_str()).unwrap_or("");
                let title = payload.get("listing_title").and_then(|v| v.as_str()).unwrap_or("");

                // ── 1. Dispatch Email (if configured & not delivered) ─────────────
                let mut email_needed = !dest_email.trim().is_empty();
                if email_needed {
                    if let Some(status) = channels.get("email").and_then(|c| c.get("status")).and_then(|s| s.as_str()) {
                        if status == "DELIVERED" {
                            email_needed = false;
                        }
                    }
                }

                let mut email_err = None;
                if email_needed {
                    let start = Instant::now();
                    match email_service.send_inquiry_notification(
                        dest_email,
                        customer_name,
                        customer_phone,
                        customer_email,
                        event_date,
                        guest_count,
                        message,
                        title,
                    ).await {
                        Ok(_) => {
                            let latency = start.elapsed().as_millis() as u64;
                            METRICS.email_latency_sum_ms.fetch_add(latency, Ordering::Relaxed);
                            METRICS.email_count.fetch_add(1, Ordering::Relaxed);
                            channels["email"] = serde_json::json!({
                                "status": "DELIVERED",
                                "last_attempt_at": chrono::Utc::now(),
                                "attempt_count": channels.get("email").and_then(|c| c.get("attempt_count")).and_then(|a| a.as_i64()).unwrap_or(0) + 1
                            });
                        }
                        Err(e) => {
                            let err_msg = format!("Email delivery failed: {}", e);
                            email_err = Some(err_msg.clone());
                            channels["email"] = serde_json::json!({
                                "status": "FAILED",
                                "error_message": err_msg,
                                "last_attempt_at": chrono::Utc::now(),
                                "attempt_count": channels.get("email").and_then(|c| c.get("attempt_count")).and_then(|a| a.as_i64()).unwrap_or(0) + 1
                            });
                        }
                    }
                }

                // ── 2. Dispatch WhatsApp (if configured & not delivered) ──────────
                let mut whatsapp_needed = matches!(dest_phone, Some(p) if !p.trim().is_empty());
                if whatsapp_needed {
                    if let Some(status) = channels.get("whatsapp").and_then(|c| c.get("status")).and_then(|s| s.as_str()) {
                        if status == "DELIVERED" {
                            whatsapp_needed = false;
                        }
                    }
                }

                let mut whatsapp_err = None;
                if whatsapp_needed {
                    if let Some(phone) = dest_phone {
                        let start = Instant::now();
                        match whatsapp_service.send_inquiry_alert(
                            phone,
                            customer_name,
                            customer_phone,
                            event_date,
                            message,
                        ).await {
                            Ok(_) => {
                                let latency = start.elapsed().as_millis() as u64;
                                METRICS.whatsapp_latency_sum_ms.fetch_add(latency, Ordering::Relaxed);
                                METRICS.whatsapp_count.fetch_add(1, Ordering::Relaxed);
                                channels["whatsapp"] = serde_json::json!({
                                    "status": "DELIVERED",
                                    "last_attempt_at": chrono::Utc::now(),
                                    "attempt_count": channels.get("whatsapp").and_then(|c| c.get("attempt_count")).and_then(|a| a.as_i64()).unwrap_or(0) + 1
                                });
                            }
                            Err(e) => {
                                let err_msg = format!("WhatsApp delivery failed: {}", e);
                                whatsapp_err = Some(err_msg.clone());
                                channels["whatsapp"] = serde_json::json!({
                                    "status": "FAILED",
                                    "error_message": err_msg,
                                    "last_attempt_at": chrono::Utc::now(),
                                    "attempt_count": channels.get("whatsapp").and_then(|c| c.get("attempt_count")).and_then(|a| a.as_i64()).unwrap_or(0) + 1
                                });
                            }
                        }
                    }
                }

                // ── 3. Resolve Joint Status ──────────────────────────────────────────
                let email_delivered = channels.get("email").and_then(|c| c.get("status")).and_then(|s| s.as_str()).unwrap_or("DELIVERED") == "DELIVERED";
                let whatsapp_delivered = channels.get("whatsapp").and_then(|c| c.get("status")).and_then(|s| s.as_str()).unwrap_or("DELIVERED") == "DELIVERED";

                if email_delivered && whatsapp_delivered {
                    Ok(())
                } else {
                    let errs = vec![email_err, whatsapp_err];
                    let joined_err = errs.into_iter().flatten().collect::<Vec<_>>().join(" | ");
                    Err(joined_err)
                }
            } else {
                Err(format!("Unknown outbox event type: {}", event_type))
            };

            // Update outbox event state in database
            let update_res = match result {
                Ok(_) => {
                    info!("Successfully delivered outbox event {}", id);
                    METRICS.total_success.fetch_add(1, Ordering::Relaxed);
                    sqlx::query(
                        "UPDATE notification_outbox 
                         SET status = 'DELIVERED', channel_delivery = $2, delivered_at = NOW(), updated_at = NOW() 
                         WHERE id = $1"
                    )
                    .bind(id)
                    .bind(channels)
                    .execute(&db)
                    .await
                }
                Err(err_msg) => {
                    let next_attempt = attempt_count + 1;
                    METRICS.total_failures.fetch_add(1, Ordering::Relaxed);
                    if next_attempt >= config.outbox_max_retries {
                        warn!("Outbox event {} failed permanently after max retries: {}", id, err_msg);
                        sqlx::query(
                            "UPDATE notification_outbox 
                             SET status = 'FAILED', error_message = $2, channel_delivery = $3, attempt_count = $4, updated_at = NOW() 
                             WHERE id = $1"
                        )
                        .bind(id)
                        .bind(&err_msg)
                        .bind(channels)
                        .bind(next_attempt)
                        .execute(&db)
                        .await
                    } else {
                        // Resolve backoff delay from schedule config
                        let idx = (next_attempt - 1) as usize;
                        let backoff_secs = config.outbox_retry_schedule
                            .get(idx)
                            .cloned()
                            .unwrap_or(1800);

                        METRICS.total_retries.fetch_add(1, Ordering::Relaxed);
                        warn!("Outbox event {} failed (attempt {}): {}. Retrying in {} seconds.", id, next_attempt, err_msg, backoff_secs);

                        sqlx::query(
                            "UPDATE notification_outbox 
                             SET status = 'RETRYING', error_message = $2, channel_delivery = $3, attempt_count = $4, next_retry_at = NOW() + ($5 * INTERVAL '1 second'), updated_at = NOW() 
                             WHERE id = $1"
                        )
                        .bind(id)
                        .bind(&err_msg)
                        .bind(channels)
                        .bind(next_attempt)
                        .bind(backoff_secs as i64)
                        .execute(&db)
                        .await
                    }
                }
            };

            if let Err(e) = update_res {
                error!("Failed to update status for outbox event {}: {}", id, e);
            }
        });
    }

    Ok(())
}

async fn run_outbox_cleanup(db: &PgPool, config: &crate::config::AppConfig) -> Result<(), String> {
    let res = sqlx::query(
        "DELETE FROM notification_outbox 
         WHERE status = 'DELIVERED' AND delivered_at < NOW() - ($1 * INTERVAL '1 day')",
    )
    .bind(config.outbox_retention_days)
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;

    if res.rows_affected() > 0 {
        info!(
            "Pruned {} delivered outbox notifications older than {} days",
            res.rows_affected(),
            config.outbox_retention_days
        );
    }

    Ok(())
}
