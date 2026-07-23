use crate::state::AppState;
use crate::services::media::deletion::StorageDeletionService;
use sqlx::{PgPool, Row};
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn, debug};
use uuid::Uuid;

pub fn start_storage_cleanup_worker(state: AppState, cancel_token: CancellationToken) {
    let db = state.db.clone();
    let minio_client = state.minio_client.clone();
    
    tokio::spawn(async move {
        // Startup Recovery: Reset stuck 'processing' items to 'retrying'
        if let Err(e) = reset_zombie_cleanup_tasks(&db).await {
            error!("Storage Cleanup Worker startup recovery error: {}", e);
        }

        let mut poll_interval = tokio::time::interval(Duration::from_secs(5));
        info!("Supervisor: Launching Storage Cleanup Worker thread...");

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    info!("Storage Cleanup Worker received cancellation signal. Exiting...");
                    break;
                }
                _ = poll_interval.tick() => {
                    if let Err(e) = poll_and_process_cleanup(&db, &minio_client).await {
                        error!("Storage Cleanup Worker poll error: {}", e);
                    }
                }
            }
        }
        info!("Storage Cleanup Worker graceful shutdown completed.");
    });
}

async fn reset_zombie_cleanup_tasks(db: &PgPool) -> Result<(), String> {
    let res = sqlx::query(
        "UPDATE public.storage_deletion_queue 
         SET status = 'retrying', next_retry_at = NOW(), updated_at = NOW() 
         WHERE status = 'processing'"
    )
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;

    let count = res.rows_affected();
    if count > 0 {
        info!("Storage Cleanup Recovery: Reset {} abandoned 'processing' tasks to 'retrying'", count);
    }
    Ok(())
}

struct CleanupTask {
    id: Uuid,
    file_ids: Vec<Uuid>,
    object_keys: Vec<String>,
    local_paths: Vec<String>,
    attempt_count: i32,
}

async fn poll_and_process_cleanup(
    db: &PgPool,
    minio_client: &crate::services::media::minio_client::MinioClient,
) -> Result<(), String> {
    // 1. Begin short transaction to lock and select a task
    let mut tx = db.begin().await.map_err(|e| e.to_string())?;

    let row_opt = sqlx::query(
        "SELECT id, file_ids, object_keys, local_paths, attempt_count 
         FROM public.storage_deletion_queue 
         WHERE status IN ('pending', 'retrying') AND next_retry_at <= NOW() 
         ORDER BY created_at 
         LIMIT 1 
         FOR UPDATE SKIP LOCKED"
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    let task = match row_opt {
        Some(row) => {
            CleanupTask {
                id: row.get("id"),
                file_ids: row.get("file_ids"),
                object_keys: row.get("object_keys"),
                local_paths: row.get("local_paths"),
                attempt_count: row.get("attempt_count"),
            }
        }
        None => {
            let _ = tx.rollback().await;
            return Ok(());
        }
    };

    // 2. Mark task as processing
    sqlx::query(
        "UPDATE public.storage_deletion_queue 
         SET status = 'processing', updated_at = NOW() 
         WHERE id = $1"
    )
    .bind(task.id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    debug!("Storage Cleanup: Processing task {} (attempt {})", task.id, task.attempt_count);

    // 3. Perform actual physical storage cleanup
    match StorageDeletionService::execute_cleanup(minio_client, &task.object_keys, &task.local_paths).await {
        Ok(_) => {
            info!("Storage Cleanup: Successfully deleted physical objects for task {}", task.id);
            
            // Delete public.uploaded_files rows
            if let Err(e) = sqlx::query("DELETE FROM public.uploaded_files WHERE id = ANY($1)")
                .bind(&task.file_ids)
                .execute(db)
                .await 
            {
                error!("Storage Cleanup: Failed to delete uploaded_files registry rows for task {}: {}", task.id, e);
            }

            // Remove the queue task
            if let Err(e) = sqlx::query("DELETE FROM public.storage_deletion_queue WHERE id = $1")
                .bind(task.id)
                .execute(db)
                .await 
            {
                error!("Storage Cleanup: Failed to delete queue task {} after completion: {}", task.id, e);
            }
        }
        Err(err_msg) => {
            let next_attempt = task.attempt_count + 1;
            if next_attempt >= 5 {
                warn!("Storage Cleanup: Task {} failed permanently after max retries: {}", task.id, err_msg);
                let _ = sqlx::query(
                    "UPDATE public.storage_deletion_queue 
                     SET status = 'failed', error_message = $2, attempt_count = $3, updated_at = NOW() 
                     WHERE id = $1"
                )
                .bind(task.id)
                .bind(&err_msg)
                .bind(next_attempt)
                .execute(db)
                .await;
            } else {
                let backoff_secs = (next_attempt as u64) * 30; // 30s, 60s, 90s, 120s incremental backoff
                warn!("Storage Cleanup: Task {} failed (attempt {}): {}. Retrying in {} seconds.", task.id, next_attempt, err_msg, backoff_secs);
                let _ = sqlx::query(
                    "UPDATE public.storage_deletion_queue 
                     SET status = 'retrying', error_message = $2, attempt_count = $3, next_retry_at = NOW() + ($4 * INTERVAL '1 second'), updated_at = NOW() 
                     WHERE id = $1"
                )
                .bind(task.id)
                .bind(&err_msg)
                .bind(next_attempt)
                .bind(backoff_secs as i64)
                .execute(db)
                .await;
            }
        }
    }

    Ok(())
}
