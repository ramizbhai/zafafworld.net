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
pub mod deletion;
pub mod media_worker;


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

    let is_image = mime_type == "image/jpeg" || mime_type == "image/png" || mime_type == "image/webp" || mime_type == "image/avif" || mime_type == "image/heic" || mime_type == "image/heif";
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
            format!("Upload rejected: Unsupported file format ({}). Only JPEG, PNG, WEBP, AVIF, HEIC images and MP4, WEBM, MOV, AVI videos are allowed.", mime_type)
        ));
    }

    // ── 4. Dispatch to the appropriate pipeline ───────────────────────────────
    let mime_subtype = match mime_type.as_str() {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        "image/avif" => "avif",
        "image/heic" => "heic",
        "image/heif" => "heif",
        "video/mp4" | "application/mp4" => "mp4",
        "video/webm" => "webm",
        "video/quicktime" => "mov",
        _ => "raw",
    };

    let raw_filename = if is_image {
        format!("ZWI{}_raw.{}", temp_id, mime_subtype)
    } else {
        format!("ZWV{}_raw.{}", temp_id, mime_subtype)
    };

    // Compute checksum of original raw file before upload
    let raw_bytes = tokio::fs::read(&temp_path).await.map_err(|e| {
        AppError::Internal(format!("Failed to read raw temp file: {}", e))
    })?;
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(&raw_bytes);
    let checksum_str = format!("{:x}", hasher.finalize());

    // Upload raw original file directly to MinIO
    minio.upload(&temp_path, &hierarchical_dir, &raw_filename, &mime_type, None, None, None, Some(&checksum_str))
        .await
        .map_err(|e| AppError::Internal(e))?;

    // Clean up local temp file
    let _ = fs::remove_file(&temp_path).await;

    let final_filename = if is_image {
        format!("ZWI{}.webp", temp_id)
    } else {
        format!("ZWV{}.mp4", temp_id)
    };
    let final_url = format!("{}{}", hierarchical_prefix, final_filename);
    let final_disk_path = format!("{}{}", hierarchical_dir, final_filename);

    // Pre-register DB record as 'processing'
    minio.insert_processing_record(
        temp_id,
        &hierarchical_dir,
        &final_filename,
        if is_image { "image/webp" } else { "video/mp4" },
        None,
        None,
    )
    .await
    .map_err(|e| AppError::Internal(e))?;

    // Create the background processing job
    sqlx::query(
        "INSERT INTO public.media_processing_jobs (uploaded_file_id, job_type, status, priority)
         VALUES ($1, $2, 'pending', 0)"
    )
    .bind(temp_id)
    .bind(if is_image { "image" } else { "video" })
    .execute(minio.pool())
    .await
    .map_err(|e| AppError::Internal(format!("Failed to insert media processing job: {}", e)))?;

    Ok(ProcessedMedia {
        id: temp_id,
        file_name: original_file_name.to_string(),
        file_url: final_url,
        file_size: total_bytes,
        mime_type,
        disk_path: final_disk_path,
        media_type: if is_image { "image".to_string() } else { "video".to_string() },
        thumbnail_url: Some(format!("{}ZWI{}_thumb.webp", hierarchical_prefix, temp_id)),
        duration_seconds: None,
        status: "processing".to_string(),
    })
}
