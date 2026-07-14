//! Media processing subsystem.
//!
//! This module is the entry point for all media-related operations:
//!
//! | Sub-module            | Responsibility                                     |
//! |-----------------------|----------------------------------------------------|
//! | [`minio_client`]      | MinIO object-storage client (initialized once)     |
//! | [`image_processing`]  | WebP image pipeline (5 size variants, no clones)   |
//! | [`video_processing`]  | ffprobe + ffmpeg video pipeline                    |
//! | [`migration`]         | One-time video transcoding migration tool          |
//!
//! Callers import directly from `crate::services::media`:
//! - `process_and_save_upload` — main upload entry point
//! - `get_available_disk_space` — disk space utility
//! - `minio_client::MinioClient` — for `AppState` construction
//! - `migration::run_video_transcoding_migration` — called from main.rs on startup

pub mod image_processing;
pub mod migration;
pub mod minio_client;
pub mod video_processing;
pub mod verification;


use crate::errors::AppError;
use axum::extract::multipart::Field;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

pub const TEMP_DIR: &str = "assets/uploads/temp/";

/// Describes the output of a successfully processed media upload.
#[derive(Debug, Clone)]
pub struct ProcessedMedia {
    pub id: Uuid,
    pub file_name: String,
    pub file_url: String,
    pub file_size: usize,
    pub mime_type: String,
    pub disk_path: String,
    pub media_type: String,
    pub thumbnail_url: Option<String>,
    pub duration_seconds: Option<i32>,
    pub status: String,
}

/// Returns available disk space (bytes) for the given directory.
///
/// Uses `df -Pk <dir>` and parses the fourth field (available KiB).
/// Returns `None` if the command fails or output is unparseable.
pub async fn get_available_disk_space(dir: &str) -> Option<u64> {
    let output = tokio::process::Command::new("df")
        .args(["-Pk", dir])
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let s = String::from_utf8_lossy(&output.stdout);
    // Skip the header line; the fourth whitespace-delimited field is available KiB.
    s.lines().nth(1)?.split_whitespace().nth(3)?.parse::<u64>().ok().map(|kb| kb * 1024)
}

/// Stream a multipart field to disk, verify file type, then dispatch to the
/// image or video processing pipeline.
///
/// # Arguments
/// * `field`              — incoming multipart field.
/// * `original_file_name` — client-provided filename.
/// * `target_dir`         — destination directory (created if absent).
/// * `url_prefix`         — URL prefix prepended to output filenames.
/// * `max_bytes`          — upload byte limit; returns `BadRequest` if exceeded.
/// * `max_dimension`      — maximum pixel edge for the image "original" variant.
/// * `minio`              — shared MinIO client reference.
pub async fn process_and_save_upload(
    mut field: Field<'_>,
    original_file_name: &str,
    target_dir: &str,
    _url_prefix: &str,
    max_bytes: usize,
    max_dimension: u32,
    minio: &minio_client::MinioClient,
) -> Result<ProcessedMedia, AppError> {
    let temp_id = Uuid::new_v4();
    let date_str = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let root_prefix = minio.root_prefix();

    // Extract the entity type (the first subfolder under root prefix)
    let clean_dir = crate::utils::storage_paths::normalize_key(target_dir, root_prefix);
    let entity_type = clean_dir
        .trim_matches('/')
        .split('/')
        .next()
        .unwrap_or("general");

    let (hierarchical_dir, hierarchical_prefix) = crate::utils::storage_paths::build_hierarchical_paths(
        root_prefix,
        entity_type,
        &temp_id,
        &date_str,
    );

    let clean_root = crate::utils::storage_paths::clean_prefix(root_prefix);
    let temp_dir_dynamic = format!("{}/temp/", clean_root);

    // ── 1. Ensure directories exist ───────────────────────────────────────────
    fs::create_dir_all(&temp_dir_dynamic)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create temp dir: {}", e)))?;
    fs::create_dir_all(&hierarchical_dir)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create target dir: {}", e)))?;

    let temp_path = format!("{}{}.tmp", temp_dir_dynamic, temp_id);

    // ── 2. Stream to temp file with byte limit ────────────────────────────────
    let mut file = File::create(&temp_path)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to create temp file: {}", e)))?;
    let mut total_bytes = 0;

    let stream_result: Result<(), AppError> = async {
        while let Some(chunk) = field
            .chunk()
            .await
            .map_err(|e| AppError::BadRequest(format!("Multipart error: {}", e)))?
        {
            total_bytes += chunk.len();
            if total_bytes > max_bytes {
                return Err(AppError::BadRequest(format!(
                    "File exceeds the {} byte limit.",
                    max_bytes
                )));
            }
            file.write_all(&chunk).await.map_err(|e| {
                AppError::Internal(format!("Failed to write to temp file: {}", e))
            })?;
        }
        file.flush()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if total_bytes == 0 {
            return Err(AppError::BadRequest("Uploaded file is empty.".to_string()));
        }
        Ok(())
    }
    .await;

    if let Err(err) = stream_result {
        let _ = fs::remove_file(&temp_path).await;
        return Err(err);
    }

    // ── 3. Verify magic bytes & infer MIME type ───────────────────────────────
    let kind = infer::get_from_path(&temp_path).ok().flatten();

    let mime_type = if let Some(k) = kind {
        k.mime_type().to_string()
    } else {
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(AppError::BadRequest(
            "Upload rejected: File type could not be identified from magic bytes.".to_string(),
        ));
    };

    let is_image = mime_type == "image/jpeg" || mime_type == "image/png" || mime_type == "image/webp";
    let is_video = mime_type == "video/mp4"
        || mime_type == "video/webm"
        || mime_type == "video/quicktime"
        || mime_type == "application/mp4"
        || mime_type == "video/x-msvideo"
        || mime_type == "video/avi"
        || mime_type == "video/msvideo";

    if !is_image && !is_video {
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(AppError::BadRequest(
            format!("Upload rejected: Unsupported file format ({}). Only JPEG, PNG, WEBP images and MP4, WEBM, MOV, AVI videos are allowed.", mime_type)
        ));
    }

    // ── 4. Dispatch to the appropriate pipeline ───────────────────────────────
    if is_image {
        let allowed_image_mimes = ["image/jpeg", "image/png", "image/webp"];
        if !allowed_image_mimes.contains(&mime_type.as_str()) {
            let _ = tokio::fs::remove_file(&temp_path).await;
            return Err(AppError::BadRequest(
                "Unsupported image format. Allowed formats: jpeg, png, webp.".to_string(),
            ));
        }

        image_processing::process_image(
            temp_path,
            temp_id,
            original_file_name,
            &hierarchical_dir,
            &hierarchical_prefix,
            max_dimension,
            minio,
        )
        .await
    } else {
        // Video
        let allowed_video_mimes = [
            "video/mp4",
            "video/webm",
            "video/quicktime",
            "application/mp4",
        ];
        if !allowed_video_mimes.contains(&mime_type.as_str()) {
            let _ = tokio::fs::remove_file(&temp_path).await;
            return Err(AppError::BadRequest(
                "Unsupported video format. Allowed formats: mp4, webm, mov.".to_string(),
            ));
        }

        let final_filename = format!("ZWV{}.mp4", temp_id);
        let final_url = format!("{}{}", hierarchical_prefix, final_filename);
        let final_disk_path = format!("{}{}", hierarchical_dir, final_filename);

        // Pre-register status as 'processing' in uploaded_files table
        if let Err(e) = minio.insert_processing_record(
            temp_id,
            &hierarchical_dir,
            &final_filename,
            &mime_type,
            None,
        ).await {
            let _ = tokio::fs::remove_file(&temp_path).await;
            return Err(AppError::Internal(e));
        }

        // Spawn background worker task for async video transcoding/remuxing
        let minio_clone = minio.clone();
        let temp_path_clone = temp_path.clone();
        let original_name = original_file_name.to_string();
        let hierarchical_dir_clone = hierarchical_dir.clone();
        let hierarchical_prefix_clone = hierarchical_prefix.clone();
        let final_url_clone = final_url.clone();

        let final_disk_path_clone = final_disk_path.clone();

        tokio::spawn(async move {
            let res = video_processing::process_video(
                temp_path_clone.clone(),
                temp_id,
                &original_name,
                &hierarchical_dir_clone,
                &hierarchical_prefix_clone,
                &minio_clone,
            ).await;

            match res {
                Ok(processed) => {
                    // Update db status to ready, update correct file size
                    if let Err(e) = minio_clone.update_upload_status(
                        temp_id,
                        "ready",
                        None,
                        Some(processed.file_size as i64),
                    ).await {
                        tracing::error!("Failed to update processing status to ready for {}: {}", temp_id, e);
                    }
                }
                Err(err) => {
                    tracing::error!("Video transcoding failed for {}: {:?}", temp_id, err);
                    
                    // Update db status to failed, set error message
                    let error_msg = format!("{:?}", err);
                    if let Err(e) = minio_clone.update_upload_status(
                        temp_id,
                        "failed",
                        Some(&error_msg),
                        None,
                    ).await {
                        tracing::error!("Failed to update processing status to failed for {}: {}", temp_id, e);
                    }

                    // Clean up disk temp/staging file if they exist
                    let _ = tokio::fs::remove_file(&temp_path_clone).await;
                    let _ = tokio::fs::remove_file(&final_disk_path_clone).await;
                    let thumb_path = format!("{}ZWI{}_thumb.webp", hierarchical_dir_clone, temp_id);
                    let _ = tokio::fs::remove_file(&thumb_path).await;

                    // Clean up partial uploads from MinIO
                    minio_clone.delete_gallery_item(&final_url_clone, "video").await;
                }
            }
        });

        // Return immediately with 'processing' status
        Ok(ProcessedMedia {
            id: temp_id,
            file_name: original_file_name.to_string(),
            file_url: final_url,
            file_size: total_bytes, // Return uploaded size as placeholder
            mime_type,
            disk_path: final_disk_path,
            media_type: "video".to_string(),
            thumbnail_url: Some(format!("{}ZWI{}_thumb.webp", hierarchical_prefix, temp_id)),
            duration_seconds: None,
            status: "processing".to_string(),
        })
    }
}
