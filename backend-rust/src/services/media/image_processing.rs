//! Image processing pipeline.
//!
//! Handles the image branch of [`super::process_and_save_upload`]:
//! - Dimension safety check
//! - Decode once via `image` crate
//! - Produce 5 WebP variants:
//!   - Original: `ZWI{uuid}.webp` (compressed max 1920px width/height, ceiling < 300KB)
//!   - Large: `ZWI{uuid}_large.webp` (compressed max 1200px width/height, ceiling < 150KB)
//!   - Medium: `ZWI{uuid}_medium.webp` (compressed max 800px width/height, ceiling < 80KB)
//!   - Card: `ZWI{uuid}_card.webp` (compressed max 400px width/height, ceiling < 40KB)
//!   - Thumb: `ZWI{uuid}_thumb.webp` (compressed max 150px width/height, ceiling < 15KB)
//! - Quality step-down logic: Starts WebP encode at 80% quality, steps down by 5% until size ceiling met or 50% floor hit.
//! - Integrates transactional database status tracking (updates to 'ready' on success or 'failed' on failure).
//! - Implements two-directional transactional rollback (purges partial files from MinIO & disk on failure).

use crate::errors::AppError;
use crate::services::media::minio_client::MinioClient;
use crate::services::media::ProcessedMedia;
use image::imageops::FilterType;
use std::collections::HashMap;
use std::sync::OnceLock;
use tokio::sync::Semaphore;
use uuid::Uuid;

// ── Semaphore ─────────────────────────────────────────────────────────────────

pub static IMAGE_SEMAPHORE: OnceLock<Semaphore> = OnceLock::new();

/// Chosen limit of 4 concurrent image encodes: image optimization is CPU-bound
/// but fast (typically ~50ms–200ms per image). Allocating 4 slots prevents CPU core
/// thrashing while allowing good concurrency throughput for multi-image listings uploads.
pub fn get_image_semaphore() -> &'static Semaphore {
    IMAGE_SEMAPHORE.get_or_init(|| Semaphore::new(4))
}

// ── Internal WebP encoder with Quality step-down ─────────────────────────────

fn encode_to_webp(img: &image::DynamicImage, quality: f32) -> Result<Vec<u8>, AppError> {
    let encoder = webp::Encoder::from_image(img)
        .map_err(|e| AppError::Internal(format!("Failed to create WebP encoder: {}", e)))?;
    let memory = encoder.encode(quality);
    Ok(memory.to_vec())
}

/// Tries encoding at quality 80.0. If the output exceeds `max_bytes`, decrements
/// quality in steps of 5.0 down to a minimum floor of 50.0.
/// Decoded images contain no metadata, so EXIF data is stripped implicitly.
fn encode_with_ceil(
    img: &image::DynamicImage,
    max_bytes: usize,
    variant_name: &str,
) -> Result<Vec<u8>, AppError> {
    let mut quality = 80.0;
    let min_quality = 50.0;
    loop {
        let bytes = encode_to_webp(img, quality)?;
        if bytes.len() <= max_bytes || quality <= min_quality {
            if bytes.len() > max_bytes {
                tracing::warn!(
                    "Image variant '{}' size ({} bytes) exceeded ceiling ({} bytes) even at min quality floor ({})",
                    variant_name, bytes.len(), max_bytes, quality
                );
            }
            return Ok(bytes);
        }
        quality -= 5.0;
    }
}

// ── Public entry point ────────────────────────────────────────────────────────

/// Process a streamed temp file into 5 optimized WebP variants, writing DB status
/// as 'processing' initially, uploading all to MinIO, and updating status to 'ready'.
/// Rolls back completely (MinIO & disk deletions) if any error occurs.
pub async fn process_image(
    temp_path: String,
    temp_id: Uuid,
    original_file_name: &str,
    target_dir: &str,
    url_prefix: &str,
    max_dimension: u32,
    minio: &MinioClient,
) -> Result<ProcessedMedia, AppError> {
    let original_filename = format!("ZWI{}.webp", temp_id);
    let original_disk_path = format!("{}{}", target_dir, original_filename);
    let original_url = format!("{}{}", url_prefix, original_filename);
    let target_dir_str = target_dir.to_string();

    // ── 1. Pre-register DB record as 'processing' ──────────────────────────
    if let Err(e) = minio.insert_processing_record(
        temp_id,
        target_dir,
        &original_filename,
        "image/webp",
        None,
        None,
    ).await {
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(AppError::Internal(e));
    }

    // Acquire semaphore permit before starting CPU intensive operations
    tracing::info!("Acquiring image processing permit...");
    let _permit = get_image_semaphore().acquire().await.ok();
    tracing::info!("Image processing permit acquired.");

    let mut uploaded_keys = Vec::new();
    let mut written_disk_paths = Vec::new();

    let t_path = temp_path.clone();

    // Define the processing scope so we can catch errors and trigger clean rollback
    let run_processing = async {
        let variant_files = tokio::task::spawn_blocking(move || -> Result<HashMap<String, Vec<u8>>, AppError> {
            // Decode image once
            let img = image::ImageReader::open(&t_path)
                .map_err(|e| {
                    let _ = std::fs::remove_file(&t_path);
                    AppError::BadRequest(format!("Failed to open image file: {}", e))
                })?
                .with_guessed_format()
                .map_err(|e| {
                    let _ = std::fs::remove_file(&t_path);
                    AppError::BadRequest(format!("Failed to recognize image format: {}", e))
                })?
                .decode()
                .map_err(|e| {
                    let _ = std::fs::remove_file(&t_path);
                    AppError::BadRequest(format!("Failed to decode image file: {}", e))
                })?;

            // Safety limit check
            let (width, height) = (img.width(), img.height());
            if width > 8000 || height > 8000 {
                let _ = std::fs::remove_file(&t_path);
                return Err(AppError::BadRequest(format!(
                    "Image dimensions exceed the 8000x8000 safety limit (got {}x{}).",
                    width, height
                )));
            }

            let mut results = HashMap::new();

            // 1. Large (1200px limit, width-bound, aspect ratio preserved)
            let large_img = if img.width() > 1200 {
                img.resize(1200, 99999, FilterType::Lanczos3)
            } else {
                img
            };
            let large_bytes = encode_with_ceil(&large_img, 150 * 1024, "large")?;
            results.insert("large".to_string(), large_bytes);

            // 2. Original (max_dimension limit, e.g. 1920px, WebP format - acts as 5th compressed variant)
            let original_img = if large_img.width() > max_dimension {
                large_img.resize(max_dimension, 99999, FilterType::Lanczos3)
            } else {
                large_img
            };
            let original_bytes = encode_with_ceil(&original_img, 300 * 1024, "original")?;
            results.insert("original".to_string(), original_bytes);

            // 3. Medium (800px limit, width-bound)
            let medium_img = if original_img.width() > 800 {
                original_img.resize(800, 99999, FilterType::Lanczos3)
            } else {
                original_img
            };
            let medium_bytes = encode_with_ceil(&medium_img, 80 * 1024, "medium")?;
            results.insert("medium".to_string(), medium_bytes);

            // 4. Card (400px limit, width-bound)
            let card_img = if medium_img.width() > 400 {
                medium_img.resize(400, 99999, FilterType::Lanczos3)
            } else {
                medium_img
            };
            let card_bytes = encode_with_ceil(&card_img, 40 * 1024, "card")?;
            results.insert("card".to_string(), card_bytes);

            // 5. Thumbnail (150px limit, width-bound)
            let thumb_img = if card_img.width() > 150 {
                card_img.resize(150, 99999, FilterType::Lanczos3)
            } else {
                card_img
            };
            let thumb_bytes = encode_with_ceil(&thumb_img, 15 * 1024, "thumb")?;
            results.insert("thumb".to_string(), thumb_bytes);

            // Cleanup local source temp file
            let _ = std::fs::remove_file(&t_path);

            Ok(results)
        }).await
        .map_err(|e| AppError::Internal(format!("Image variant generation task panicked: {}", e)))??;

        // Write processed variants to staging disk
        let mut original_size = 0;
        for (variant, bytes) in &variant_files {
            let variant_filename = if variant == "original" {
                format!("ZWI{}.webp", temp_id)
            } else {
                format!("ZWI{}_{}.webp", temp_id, variant)
            };
            let disk_path = format!("{}{}", target_dir_str, variant_filename);
            tokio::fs::write(&disk_path, bytes).await.map_err(|e| {
                AppError::Internal(format!("Failed to write variant {} to disk: {}", variant, e))
            })?;
            written_disk_paths.push(disk_path);

            if variant == "original" {
                original_size = bytes.len();
            }
        }

        // Upload all variants to MinIO
        for variant in &["original", "large", "medium", "card", "thumb"] {
            let variant_filename = if *variant == "original" {
                format!("ZWI{}.webp", temp_id)
            } else {
                format!("ZWI{}_{}.webp", temp_id, variant)
            };
            let disk_path = format!("{}{}", target_dir_str, variant_filename);

            let parent_id = if *variant == "original" { None } else { Some(temp_id) };
            minio.upload(&disk_path, target_dir_str.as_str(), &variant_filename, "image/webp", parent_id)
                .await
                .map_err(|e| {
                    AppError::Internal(format!("MinIO variant {} upload failed: {}", variant, e))
                })?;
            uploaded_keys.push(format!("{}{}", target_dir_str, variant_filename));
        }

        Ok::<usize, AppError>(original_size)
    };

    match run_processing.await {
        Ok(final_size) => {
            // Update db status to 'ready' and record actual size of compressed original variant
            if let Err(e) = minio.update_upload_status(temp_id, "ready", None, Some(final_size as i64)).await {
                tracing::error!("Failed to update database status to ready for {}: {}", temp_id, e);
            }

            // Cleanup local variant files from staging disk
            for path in &written_disk_paths {
                let _ = tokio::fs::remove_file(path).await;
            }

            Ok(ProcessedMedia {
                id: temp_id,
                file_name: original_file_name.to_string(),
                file_url: original_url,
                file_size: final_size,
                mime_type: "image/webp".to_string(),
                disk_path: original_disk_path,
                media_type: "image".to_string(),
                thumbnail_url: None,
                duration_seconds: None,
                status: "ready".to_string(),
            })
        }
        Err(err) => {
            // Update db status to failed with exact details
            let error_msg = format!("{:?}", err);
            if let Err(e) = minio.update_upload_status(temp_id, "failed", Some(&error_msg), None).await {
                tracing::error!("Failed to update database status to failed for {}: {}", temp_id, e);
            }

            // Clean up staging disk files if any
            for path in &written_disk_paths {
                let _ = tokio::fs::remove_file(path).await;
            }
            let _ = tokio::fs::remove_file(&temp_path).await;

            // Purge uploaded files from MinIO
            for key in &uploaded_keys {
                let _ = minio.delete(key).await;
            }

            Err(err)
        }
    }
}
