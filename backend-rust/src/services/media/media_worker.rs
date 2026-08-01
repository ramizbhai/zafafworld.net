use crate::state::AppState;
use sqlx::{PgPool, Row};
use std::time::{Duration, Instant};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use uuid::Uuid;

pub fn start_media_worker(state: AppState, cancel_token: CancellationToken) {
    tokio::spawn(async move {
        let mut restart_count = 0;
        let mut last_restart = Instant::now();

        loop {
            if cancel_token.is_cancelled() {
                break;
            }

            let state_clone = state.clone();
            let token_clone = cancel_token.clone();

            info!("Supervisor: Launching Media Worker thread instance...");
            let handle = tokio::spawn(async move {
                run_worker_loop(state_clone, token_clone).await;
            });

            match handle.await {
                Ok(_) => {
                    info!("Supervisor: Media Worker finished cleanly.");
                    break;
                }
                Err(join_err) => {
                    if join_err.is_cancelled() {
                        info!("Supervisor: Media Worker loop cancelled.");
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
                        "Supervisor: Media Worker panicked/crashed! Reason: {}.",
                        panic_reason
                    );

                    if last_restart.elapsed() < Duration::from_secs(5) {
                        restart_count += 1;
                    } else {
                        restart_count = 1;
                    }
                    last_restart = Instant::now();

                    if restart_count > 5 {
                        error!("Supervisor: Media Worker crashed repeatedly (5 times in < 5s). Suspending restarts for 30 seconds to prevent resource thrashing...");
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
    // 1. Startup Recovery: Convert zombie processing rows back to pending
    if let Err(e) = recover_abandoned_jobs(&state.db).await {
        error!("Media Worker startup recovery error: {}", e);
    }

    let mut poll_interval = tokio::time::interval(Duration::from_millis(500));
    let mut join_set = JoinSet::new();

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                info!("Media Worker loop received cancellation signal. Cleaning up active tasks...");
                break;
            }
            _ = poll_interval.tick() => {
                // Drain completed tasks
                while let Some(Ok(())) = join_set.try_join_next() {}

                // Enforce concurrent media processing limits
                if join_set.len() < 4 {
                    if let Err(e) = poll_and_process_job(&state, &cancel_token, &mut join_set).await {
                        error!("Media Worker processing error: {}", e);
                    }
                }
            }
        }
    }

    // Wait for all active tasks to complete before exiting
    while let Some(res) = join_set.join_next().await {
        if let Err(e) = res {
            error!("Active media task join error during shutdown: {}", e);
        }
    }
}

async fn recover_abandoned_jobs(db: &PgPool) -> Result<(), String> {
    let res = sqlx::query(
        "UPDATE media_processing_jobs 
         SET status = 'pending', started_at = NULL, worker_id = NULL 
         WHERE status = 'processing'",
    )
    .execute(db)
    .await
    .map_err(|e| e.to_string())?;

    let count = res.rows_affected();
    if count > 0 {
        info!("Startup Recovery: Reset {} abandoned 'processing' media jobs to 'pending'", count);
    }
    Ok(())
}

async fn poll_and_process_job(
    state: &AppState,
    cancel_token: &CancellationToken,
    join_set: &mut JoinSet<()>,
) -> Result<(), String> {
    let worker_id = Uuid::new_v4();

    // Claim a job using FOR UPDATE SKIP LOCKED
    let row = sqlx::query(
        "UPDATE public.media_processing_jobs
         SET status = 'processing', started_at = NOW(), worker_id = $1, attempt_count = attempt_count + 1
         WHERE id = (
             SELECT id 
             FROM public.media_processing_jobs
             WHERE status IN ('pending', 'failed') AND attempt_count < max_attempts
             ORDER BY priority DESC, created_at ASC
             LIMIT 1
             FOR UPDATE SKIP LOCKED
         )
         RETURNING id, uploaded_file_id, job_type, attempt_count",
    )
    .bind(worker_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| e.to_string())?;

    let row = match row {
        Some(r) => r,
        None => return Ok(()),
    };

    let job_id: Uuid = row.get("id");
    let uploaded_file_id: Uuid = row.get("uploaded_file_id");
    let job_type: String = row.get("job_type");
    let attempt_count: i32 = row.get("attempt_count");

    let state_clone = state.clone();
    let token_clone = cancel_token.clone();

    join_set.spawn(async move {
        let start_time = Instant::now();
        info!(
            target: "media_worker",
            "Worker Started: job_id={} uploaded_file_id={} type={} attempt={}",
            job_id, uploaded_file_id, job_type, attempt_count
        );

        let run_result = if job_type == "image" {
            process_image_job(&state_clone, uploaded_file_id, &token_clone).await
        } else {
            process_video_job(&state_clone, uploaded_file_id, &token_clone).await
        };

        let elapsed = start_time.elapsed().as_millis() as i64;

        match run_result {
            Ok(variant_count) => {
                info!(
                    target: "media_worker",
                    "Worker Completed: job_id={} uploaded_file_id={} type={} variants={} elapsed_ms={}",
                    job_id, uploaded_file_id, job_type, variant_count, elapsed
                );

                let _ = sqlx::query(
                    "UPDATE public.media_processing_jobs
                     SET status = 'completed', completed_at = NOW(), processing_time_ms = $2
                     WHERE id = $1",
                )
                .bind(job_id)
                .bind(elapsed)
                .execute(&state_clone.db)
                .await;
            }
            Err(err_msg) => {
                error!(
                    target: "media_worker",
                    "Worker Failed: job_id={} uploaded_file_id={} error={}",
                    job_id, uploaded_file_id, err_msg
                );

                // Update job status
                let is_final_failure = attempt_count >= 5;
                let new_status = if is_final_failure { "failed" } else { "pending" };

                let _ = sqlx::query(
                    "UPDATE public.media_processing_jobs
                     SET status = $2, error_message = $3, completed_at = CASE WHEN $2 = 'failed' THEN NOW() ELSE NULL END
                     WHERE id = $1",
                )
                .bind(job_id)
                .bind(new_status)
                .bind(&err_msg)
                .execute(&state_clone.db)
                .await;

                // Also update parent uploaded_files table to 'failed' if it's the final failure
                if is_final_failure {
                    let _ = sqlx::query(
                        "UPDATE public.uploaded_files
                         SET status = 'failed', error_message = $2
                         WHERE id = $1",
                    )
                    .bind(uploaded_file_id)
                    .bind(&err_msg)
                    .execute(&state_clone.db)
                    .await;
                }
            }
        }
    });

    Ok(())
}

async fn process_image_job(
    state: &AppState,
    uploaded_file_id: Uuid,
    _cancel_token: &CancellationToken,
) -> Result<usize, String> {
    // 1. Fetch metadata of raw upload from database
    let raw_row = sqlx::query(
        "SELECT id, bucket_name, object_key, file_name, mime_type, file_size
         FROM public.uploaded_files
         WHERE file_name LIKE 'ZWI' || $1 || '_raw.%'
            OR file_name LIKE 'ZWV' || $1 || '_raw.%'"
    )
    .bind(uploaded_file_id.to_string())
    .fetch_optional(&state.db)
    .await
    .map_err(|e| format!("Database query error for raw file: {}", e))?;

    let raw_record = raw_row.ok_or_else(|| {
        format!("Raw file record matching ID {} not found in public.uploaded_files", uploaded_file_id)
    })?;

    let raw_key: String = raw_record.get("object_key");
    let _raw_bucket: String = raw_record.get("bucket_name");
    let _raw_file_name: String = raw_record.get("file_name");
    let _raw_mime_type: String = raw_record.get("mime_type");

    // Also get the optimized file metadata to keep file_name logic correct
    let parent = sqlx::query(
        "SELECT id, file_name
         FROM public.uploaded_files
         WHERE id = $1",
    )
    .bind(uploaded_file_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("Uploaded file record {} not found in db", uploaded_file_id))?;

    let file_name: String = parent.get("file_name");

    // 2. Raw key is resolved directly from database row
    // 3. Download raw bytes from S3 MinIO
    let bucket = state.minio_client.bucket().map_err(|e| e.to_string())?;
    let response = bucket.get_object(&raw_key).await.map_err(|e| e.to_string())?;
    
    if response.status_code() != 200 {
        return Err(format!(
            "MinIO GET failed with HTTP {} for key {}. Response: {}",
            response.status_code(),
            raw_key,
            String::from_utf8_lossy(response.bytes())
        ));
    }
    
    let raw_bytes = response.bytes();
    tracing::info!("Downloaded raw image file key={} bytes={}", raw_key, raw_bytes.len());

    // 4. Save raw bytes to a temp staging file
    let temp_dir = "assets/uploads/temp/";
    let _ = tokio::fs::create_dir_all(temp_dir).await;
    let temp_staging_path = format!("{}{}_processing.tmp", temp_dir, uploaded_file_id);
    tokio::fs::write(&temp_staging_path, &raw_bytes)
        .await
        .map_err(|e| format!("Failed to write staging file: {}", e))?;

    let target_dir = format!("assets/uploads/gallery/{}/{}/", uploaded_file_id, chrono::Utc::now().format("%Y-%m-%d"));
    let url_prefix = format!("/assets/uploads/gallery/{}/{}/", uploaded_file_id, chrono::Utc::now().format("%Y-%m-%d"));

    // 5. Call image processing pipeline
    let process_res = crate::services::media::image_processing::process_image(
        temp_staging_path.clone(),
        uploaded_file_id,
        &file_name,
        &target_dir,
        &url_prefix,
        1920,
        &state.minio_client,
    )
    .await;

    // Clean up local temp file
    let _ = tokio::fs::remove_file(&temp_staging_path).await;

    match process_res {
        Ok(processed) => {
            // Update db status to ready, update correct file size
            let _ = state.minio_client.update_upload_status(
                uploaded_file_id,
                "ready",
                None,
                Some(processed.file_size as i64),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .await;
            Ok(11) // 11 variants (1 original + 10 optimized WebP/AVIF variants)
        }
        Err(err) => {
            Err(format!("Image processing pipeline error: {:?}", err))
        }
    }
}

async fn process_video_job(
    state: &AppState,
    uploaded_file_id: Uuid,
    _cancel_token: &CancellationToken,
) -> Result<usize, String> {
    // 1. Fetch metadata of raw upload from database
    let raw_row = sqlx::query(
        "SELECT id, bucket_name, object_key, file_name, mime_type, file_size
         FROM public.uploaded_files
         WHERE file_name LIKE 'ZWI' || $1 || '_raw.%'
            OR file_name LIKE 'ZWV' || $1 || '_raw.%'"
    )
    .bind(uploaded_file_id.to_string())
    .fetch_optional(&state.db)
    .await
    .map_err(|e| format!("Database query error for raw file: {}", e))?;

    let raw_record = raw_row.ok_or_else(|| {
        format!("Raw file record matching ID {} not found in public.uploaded_files", uploaded_file_id)
    })?;

    let raw_key: String = raw_record.get("object_key");
    let _raw_bucket: String = raw_record.get("bucket_name");
    let _raw_file_name: String = raw_record.get("file_name");
    let _raw_mime_type: String = raw_record.get("mime_type");

    // Also get the optimized file metadata to keep file_name logic correct
    let parent = sqlx::query(
        "SELECT id, file_name
         FROM public.uploaded_files
         WHERE id = $1",
    )
    .bind(uploaded_file_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("Uploaded file record {} not found in db", uploaded_file_id))?;

    let file_name: String = parent.get("file_name");

    // 2. Raw key is resolved directly from database row
    // 3. Download raw bytes from S3 MinIO
    let bucket = state.minio_client.bucket().map_err(|e| e.to_string())?;
    let response = bucket.get_object(&raw_key).await.map_err(|e| e.to_string())?;
    
    if response.status_code() != 200 {
        return Err(format!(
            "MinIO GET failed with HTTP {} for key {}. Response: {}",
            response.status_code(),
            raw_key,
            String::from_utf8_lossy(response.bytes())
        ));
    }
    
    let raw_bytes = response.bytes();
    tracing::info!("Downloaded raw video file key={} bytes={}", raw_key, raw_bytes.len());

    // 4. Save raw bytes to a temp staging file
    let temp_dir = "assets/uploads/temp/";
    let _ = tokio::fs::create_dir_all(temp_dir).await;
    let temp_staging_path = format!("{}{}_processing.tmp", temp_dir, uploaded_file_id);
    tokio::fs::write(&temp_staging_path, &raw_bytes)
        .await
        .map_err(|e| format!("Failed to write staging file: {}", e))?;

    let target_dir = format!("assets/uploads/gallery/{}/{}/", uploaded_file_id, chrono::Utc::now().format("%Y-%m-%d"));
    let url_prefix = format!("/assets/uploads/gallery/{}/{}/", uploaded_file_id, chrono::Utc::now().format("%Y-%m-%d"));

    // 5. Call video processing pipeline
    let process_res = crate::services::media::video_processing::process_video(
        temp_staging_path.clone(),
        uploaded_file_id,
        &file_name,
        &target_dir,
        &url_prefix,
        &state.minio_client,
    )
    .await;

    // Clean up local temp file
    let _ = tokio::fs::remove_file(&temp_staging_path).await;

    match process_res {
        Ok(processed) => {
            // Update db status to ready, update correct file size
            let _ = state.minio_client.update_upload_status(
                uploaded_file_id,
                "ready",
                None,
                Some(processed.file_size as i64),
                None,
                None,
                processed.duration_seconds,
                None,
                None,
                None,
                None,
            )
            .await;
            Ok(3) // 3 variants (raw + remuxed mp4 + thumbnails)
        }
        Err(err) => {
            // Clean up partial uploads from MinIO if failed
            let final_filename = format!("ZWV{}.mp4", uploaded_file_id);
            let final_url = format!("{}{}", url_prefix, final_filename);
            state.minio_client.delete_gallery_item(&final_url, "video").await;

            // Also clean up thumbnail variants
            let final_thumb_filename = format!("ZWI{}_thumb.webp", uploaded_file_id);
            let final_thumb_avif_filename = format!("ZWI{}_thumb.avif", uploaded_file_id);
            let _ = state.minio_client.delete(&format!("{}{}", target_dir, final_thumb_filename)).await;
            let _ = state.minio_client.delete(&format!("{}{}", target_dir, final_thumb_avif_filename)).await;

            Err(format!("Video processing pipeline error: {:?}", err))
        }
    }
}
