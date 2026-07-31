//! Image processing pipeline.
//!
//! Handles the image branch of [`super::process_and_save_upload`]:
//! - Dimension safety check
//! - Decode once via `image` crate
//! - EXIF orientation normalization & metadata stripping
//! - Produce 10 variants (5 AVIF, 5 WebP fallback):
//!   - Original: `ZWI{uuid}.avif` / `.webp` (max 1920px)
//!   - Large: `ZWI{uuid}_large.avif` / `.webp` (max 1200px)
//!   - Medium: `ZWI{uuid}_medium.avif` / `.webp` (max 800px)
//!   - Card: `ZWI{uuid}_card.avif` / `.webp` (max 400px)
//!   - Thumb: `ZWI{uuid}_thumb.avif` / `.webp` (max 150px)
//! - Archives original raw uploads as `ZWI{uuid}_raw.{ext}`.
//! - Generates lightweight Base64 WebP LQIP placeholders.
//! - Observability: logs 6 distinct pipeline states structurally.

use crate::errors::AppError;
use crate::services::media::minio_client::MinioClient;
use crate::services::media::ProcessedMedia;
use image::imageops::FilterType;
use std::collections::HashMap;
use std::sync::OnceLock;
use tokio::sync::Semaphore;
use uuid::Uuid;
use base64::Engine;

// ── Semaphore ─────────────────────────────────────────────────────────────────

pub static IMAGE_SEMAPHORE: OnceLock<Semaphore> = OnceLock::new();

/// Chosen limit of 4 concurrent image encodes: image optimization is CPU-bound
/// but fast (typically ~50ms–200ms per image). Allocating 4 slots prevents CPU core
/// thrashing while allowing good concurrency throughput for multi-image listings uploads.
pub fn get_image_semaphore() -> &'static Semaphore {
    IMAGE_SEMAPHORE.get_or_init(|| Semaphore::new(4))
}

// ── EXIF & Color Normalization Helpers ────────────────────────────────────────

/// Extract orientation tag from image EXIF metadata.
fn get_exif_orientation(temp_path: &str) -> Option<u32> {
    let file = std::fs::File::open(temp_path).ok()?;
    let mut bufreader = std::io::BufReader::new(file);
    let exifreader = exif::Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader).ok()?;
    let field = exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY)?;
    field.value.get_uint(0)
}

/// Rotate or flip DynamicImage according to EXIF orientation.
fn apply_exif_orientation(img: image::DynamicImage, orientation: u32) -> image::DynamicImage {
    match orientation {
        2 => img.fliph(),
        3 => img.rotate180(),
        4 => img.flipv(),
        5 => {
            let flipped = img.fliph();
            flipped.rotate270()
        }
        6 => img.rotate90(),
        7 => {
            let flipped = img.fliph();
            flipped.rotate90()
        }
        8 => img.rotate270(),
        _ => img,
    }
}

// ── Encoding Helpers ──────────────────────────────────────────────────────────

/// Encodes DynamicImage to WebP format.
fn encode_to_webp(img: &image::DynamicImage, quality: f32) -> Result<Vec<u8>, AppError> {
    let encoder = webp::Encoder::from_image(img)
        .map_err(|e| AppError::Internal(format!("Failed to create WebP encoder: {}", e)))?;
    let memory = encoder.encode(quality);
    Ok(memory.to_vec())
}

/// Encodes DynamicImage to AVIF format using the pure-Rust ravif encoder.
fn encode_to_avif(img: &image::DynamicImage, quality: u32) -> Result<Vec<u8>, AppError> {
    let rgba8 = img.to_rgba8();
    let (width, height) = rgba8.dimensions();
    
    // Safety cast to &[ravif::RGBA8]
    let pixels: &[ravif::RGBA8] = unsafe {
        std::slice::from_raw_parts(
            rgba8.as_ptr() as *const ravif::RGBA8,
            rgba8.len() / 4,
        )
    };
    
    let img_ref = ravif::Img::new(pixels, width as usize, height as usize);
    
    let res = ravif::Encoder::new()
        .with_quality(quality as f32)
        .with_speed(6)
        .encode_rgba(img_ref)
        .map_err(|e| AppError::Internal(format!("Failed to encode AVIF: {:?}", e)))?;
        
    Ok(res.avif_file)
}

/// Stepdown quality encoder for WebP.
fn encode_with_ceil_webp(
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
                    "WebP variant '{}' size ({} bytes) exceeded ceiling ({} bytes) even at min quality floor ({})",
                    variant_name, bytes.len(), max_bytes, quality
                );
            }
            return Ok(bytes);
        }
        quality -= 5.0;
    }
}

/// Stepdown quality encoder for AVIF.
fn encode_with_ceil_avif(
    img: &image::DynamicImage,
    max_bytes: usize,
    variant_name: &str,
) -> Result<Vec<u8>, AppError> {
    let mut quality = 80;
    let min_quality = 50;
    loop {
        let bytes = encode_to_avif(img, quality)?;
        if bytes.len() <= max_bytes || quality <= min_quality {
            if bytes.len() > max_bytes {
                tracing::warn!(
                    "AVIF variant '{}' size ({} bytes) exceeded ceiling ({} bytes) even at min quality floor ({})",
                    variant_name, bytes.len(), max_bytes, quality
                );
            }
            return Ok(bytes);
        }
        quality -= 5;
    }
}

/// Generates a tiny, 16x16 WebP Base64 LQIP data URI.
fn generate_lqip(img: &image::DynamicImage) -> Result<String, AppError> {
    let tiny = img.resize(16, 16, FilterType::Nearest);
    let bytes = encode_to_webp(&tiny, 20.0)?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:image/webp;base64,{}", b64))
}

// ── Public Entry Point ────────────────────────────────────────────────────────

pub async fn process_image(
    temp_path: String,
    temp_id: Uuid,
    original_file_name: &str,
    target_dir: &str,
    url_prefix: &str,
    max_dimension: u32,
    minio: &MinioClient,
) -> Result<ProcessedMedia, AppError> {
    let operation_id = Uuid::new_v4();
    tracing::info!(
        target: "media_pipeline",
        "Upload Started: operation_id={} temp_id={} filename={}",
        operation_id, temp_id, original_file_name
    );

    let original_filename = format!("ZWI{}.webp", temp_id);
    let original_disk_path = format!("{}{}", target_dir, original_filename);
    let original_url = format!("{}{}", url_prefix, original_filename);
    let target_dir_str = target_dir.to_string();

    // 1. Pre-register DB record as 'processing'
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
    let _permit = get_image_semaphore().acquire().await.ok();

    let mut uploaded_keys = Vec::new();
    let mut written_disk_paths = Vec::new();

    let t_path = temp_path.clone();
    let orig_name = original_file_name.to_string();

    let run_processing = async {
        // Read original raw bytes for archiving/future reprocessing
        let raw_bytes = tokio::fs::read(&t_path).await.map_err(|e| {
            AppError::Internal(format!("Failed to read raw temp file: {}", e))
        })?;

        let variant_files = tokio::task::spawn_blocking(move || -> Result<(HashMap<String, Vec<u8>>, String), AppError> {
            // EXIF Orientation check
            let orientation = get_exif_orientation(&t_path).unwrap_or(1);

            // Decode image once
            let mut img = image::ImageReader::open(&t_path)
                .map_err(|e| AppError::BadRequest(format!("Failed to open image file: {}", e)))?
                .with_guessed_format()
                .map_err(|e| AppError::BadRequest(format!("Failed to recognize image format: {}", e)))?
                .decode()
                .map_err(|e| AppError::BadRequest(format!("Failed to decode image file: {}", e)))?;

            // Safety limit check
            let (width, height) = (img.width(), img.height());
            if width > 8000 || height > 8000 {
                return Err(AppError::BadRequest(format!(
                    "Image dimensions exceed the 8000x8000 safety limit (got {}x{}).",
                    width, height
                )));
            }

            tracing::info!(
                target: "media_pipeline",
                "Validation Complete: operation_id={} temp_id={} dimensions={}x{}",
                operation_id, temp_id, width, height
            );

            // EXIF Normalization & metadata stripping (implicit in DynamicImage manipulation)
            img = apply_exif_orientation(img, orientation);

            tracing::info!(
                target: "media_pipeline",
                "Normalization Complete: operation_id={} temp_id={} orientation_applied={}",
                operation_id, temp_id, orientation
            );

            let lqip = generate_lqip(&img)?;

            let mut results = HashMap::new();

            // Sizing limits: (name, max_edge, webp_ceiling, avif_ceiling)
            let sizes = [
                ("thumb", 150, 15 * 1024, 10 * 1024),
                ("card", 400, 40 * 1024, 30 * 1024),
                ("medium", 800, 80 * 1024, 60 * 1024),
                ("large", 1200, 150 * 1024, 100 * 1024),
                ("original", max_dimension, 300 * 1024, 200 * 1024),
            ];

            for (name, limit, max_webp, max_avif) in sizes {
                // Resize if needed, keeping aspect ratio
                let resized = if name == "original" && img.width() <= limit {
                    img.clone()
                } else if img.width() > limit {
                    img.resize(limit, 99999, FilterType::Lanczos3)
                } else {
                    img.clone()
                };

                // WebP variant
                let webp_bytes = encode_with_ceil_webp(&resized, max_webp, name)?;
                results.insert(format!("webp_{}", name), webp_bytes);

                // AVIF variant
                let avif_bytes = encode_with_ceil_avif(&resized, max_avif, name)?;
                results.insert(format!("avif_{}", name), avif_bytes);
            }

            // Cleanup local source temp file
            let _ = std::fs::remove_file(&t_path);

            Ok((results, lqip))
        }).await
        .map_err(|e| AppError::Internal(format!("Image variant generation task panicked: {}", e)))??;

        let (files_map, lqip) = variant_files;

        tracing::info!(
            target: "media_pipeline",
            "Variants Generated: operation_id={} temp_id={} variants_count={}",
            operation_id, temp_id, files_map.len() + 1 // +1 for raw
        );

        // Extract original file extension
        let orig_ext = std::path::Path::new(&orig_name)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("jpg");

        // Write processed variants to staging disk
        let mut original_webp_size = 0;

        // 1. Write the Raw original
        let raw_filename = format!("ZWI{}_raw.{}", temp_id, orig_ext);
        let raw_disk_path = format!("{}{}", target_dir_str, raw_filename);
        tokio::fs::write(&raw_disk_path, &raw_bytes).await.map_err(|e| {
            AppError::Internal(format!("Failed to write raw file to disk: {}", e))
        })?;
        written_disk_paths.push(raw_disk_path.clone());

        // 2. Write all WebP and AVIF variants
        for (key, bytes) in &files_map {
            let filename = if key == "webp_original" {
                format!("ZWI{}.webp", temp_id)
            } else if key == "avif_original" {
                format!("ZWI{}.avif", temp_id)
            } else if key.starts_with("webp_") {
                format!("ZWI{}_{}.webp", temp_id, &key[5..])
            } else {
                format!("ZWI{}_{}.avif", temp_id, &key[5..])
            };

            let disk_path = format!("{}{}", target_dir_str, filename);
            tokio::fs::write(&disk_path, bytes).await.map_err(|e| {
                AppError::Internal(format!("Failed to write variant {} to disk: {}", key, e))
            })?;
            written_disk_paths.push(disk_path);

            if key == "webp_original" {
                original_webp_size = bytes.len();
            }
        }

        // Upload original raw file to MinIO
        let inferred_mime = infer::get(&raw_bytes)
            .map(|k| k.mime_type())
            .unwrap_or("image/jpeg");
        minio.upload(&raw_disk_path, target_dir_str.as_str(), &raw_filename, inferred_mime, Some(temp_id))
            .await
            .map_err(|e| AppError::Internal(format!("MinIO raw upload failed: {}", e)))?;
        uploaded_keys.push(format!("{}{}", target_dir_str, raw_filename));

        // Upload WebP and AVIF variants to MinIO
        for key in files_map.keys() {
            let filename = if key == "webp_original" {
                format!("ZWI{}.webp", temp_id)
            } else if key == "avif_original" {
                format!("ZWI{}.avif", temp_id)
            } else if key.starts_with("webp_") {
                format!("ZWI{}_{}.webp", temp_id, &key[5..])
            } else {
                format!("ZWI{}_{}.avif", temp_id, &key[5..])
            };

            let disk_path = format!("{}{}", target_dir_str, filename);
            let mime = if key.starts_with("webp_") { "image/webp" } else { "image/avif" };
            
            let parent_id = if key == "webp_original" { None } else { Some(temp_id) };

            minio.upload(&disk_path, target_dir_str.as_str(), &filename, mime, parent_id)
                .await
                .map_err(|e| AppError::Internal(format!("MinIO upload failed for key {}: {}", key, e)))?;
            uploaded_keys.push(format!("{}{}", target_dir_str, filename));
        }

        tracing::info!(
            target: "media_pipeline",
            "Objects Uploaded: operation_id={} temp_id={} uploaded_keys_count={}",
            operation_id, temp_id, uploaded_keys.len()
        );

        Ok::<(_, usize), AppError>((lqip, original_webp_size))
    };

    match run_processing.await {
        Ok((lqip, final_size)) => {
            // Update db status to ready
            if let Err(e) = minio.update_upload_status(temp_id, "ready", None, Some(final_size as i64)).await {
                tracing::error!("Failed to update database status to ready for {}: {}", temp_id, e);
            }

            tracing::info!(
                target: "media_pipeline",
                "Metadata Saved: operation_id={} temp_id={} status=ready size={}",
                operation_id, temp_id, final_size
            );

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
                thumbnail_url: Some(lqip), // Return LQIP base64 here
                duration_seconds: None,
                status: "ready".to_string(),
            })
        }
        Err(err) => {
            tracing::error!(
                target: "media_pipeline",
                "Processing Failed: operation_id={} temp_id={} error={:?}",
                operation_id, temp_id, err
            );

            // Update db status to failed
            let error_msg = format!("{:?}", err);
            if let Err(e) = minio.update_upload_status(temp_id, "failed", Some(&error_msg), None).await {
                tracing::error!("Failed to update database status to failed for {}: {}", temp_id, e);
            }

            // Clean up staging disk files
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
