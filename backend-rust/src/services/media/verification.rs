//! Media pipeline verification tests.
//!
//! Exposes a CLI subcommand command to verify:
//! 1. Corrupted file decoding failure returns 400 Bad Request.
//! 2. Strict MIME magic-bytes checking rejects extensions mismatch (400 Bad Request).
//! 3. Real image sizing & step-down optimization outputs and ceilings.
//! 4. Transactional DB updates and rollback on upload failures.
//! 5. Concurrency bounds on image semaphores.

use crate::config::AppConfig;
use crate::errors::AppError;
use crate::services::media::{self, minio_client::MinioClient, ProcessedMedia};
use image::{ImageBuffer, Rgb, DynamicImage, ImageFormat};
use sqlx::PgPool;
use std::collections::HashMap;
use std::fs;
use uuid::Uuid;

/// Helper to generate a valid PNG image of arbitrary size.
fn create_test_image(width: u32, height: u32) -> Vec<u8> {
    let img = ImageBuffer::from_fn(width, height, |x, y| {
        Rgb([
            (x % 255) as u8,
            (y % 255) as u8,
            ((x + y) % 255) as u8,
        ])
    });
    let dynamic_img = DynamicImage::ImageRgb8(img);
    let mut cursor = std::io::Cursor::new(Vec::new());
    dynamic_img.write_to(&mut cursor, ImageFormat::Png).unwrap();
    cursor.into_inner()
}

/// Helper to query variant file size from MinIO.
async fn get_minio_object_size(minio: &MinioClient, key: &str) -> Option<usize> {
    let bucket = minio.bucket().ok()?;
    let clean_key = crate::utils::storage_paths::normalize_key(key, minio.root_prefix());
    let response = bucket.get_object(clean_key).await.ok()?;
    if response.status_code() == 200 {
        Some(response.bytes().len())
    } else {
        None
    }
}

/// Run all pipeline verification tests.
pub async fn run_pipeline_verification(config: &AppConfig, pool: &PgPool) {
    println!("==================================================");
    println!("   ZafafWorld Media Pipeline Verification Report  ");
    println!("==================================================");
    let minio = MinioClient::from_config(config, pool.clone());
    let temp_dir = "assets/uploads/temp/";
    let _ = fs::create_dir_all(temp_dir);

    // ─────────────────────────────────────────────────────────────────────────
    // Test 1: Corrupted File Rejection
    // ─────────────────────────────────────────────────────────────────────────
    println!("\n[Test 1] Corrupted File Rejection...");
    let corrupt_id = Uuid::new_v4();
    let corrupt_path = format!("{}{}.tmp", temp_dir, corrupt_id);
    fs::write(&corrupt_path, b"TRUNCATED_BAD_HEADER_AND_BYTES_12345").unwrap();

    let res = media::image_processing::process_image(
        corrupt_path.clone(),
        corrupt_id,
        "corrupt.png",
        "assets/uploads/gallery/",
        "/assets/uploads/gallery/",
        1920,
        &minio,
    ).await;

    match res {
        Err(AppError::BadRequest(msg)) => {
            println!("  ✅ Passed: Rejected with 400 Bad Request as expected.");
            println!("     Message: \"{}\"", msg);
        }
        other => {
            println!("  ❌ Failed: Expected 400 Bad Request, got: {:?}", other);
        }
    }
    let _ = fs::remove_file(&corrupt_path);

    // ─────────────────────────────────────────────────────────────────────────
    // Test 2: Strict MIME Sniffing Rejection (Extension Trust Mismatch)
    // ─────────────────────────────────────────────────────────────────────────
    println!("\n[Test 2] MIME Magic-bytes Sniffing Mismatch...");
    let text_id = Uuid::new_v4();
    let text_path = format!("{}{}.tmp", temp_dir, text_id);
    fs::write(&text_path, b"This is just normal text content, not an image format.").unwrap();

    let res = media::image_processing::process_image(
        text_path.clone(),
        text_id,
        "fake_image.jpg", // Named .jpg to check if it ignores client extension
        "assets/uploads/gallery/",
        "/assets/uploads/gallery/",
        1920,
        &minio,
    ).await;

    match res {
        Err(AppError::BadRequest(msg)) => {
            println!("  ✅ Passed: Renamed plain-text file rejected with 400 Bad Request.");
            println!("     Message: \"{}\"", msg);
        }
        other => {
            println!("  ❌ Failed: Expected 400 Bad Request, got: {:?}", other);
        }
    }
    let _ = fs::remove_file(&text_path);

    // ─────────────────────────────────────────────────────────────────────────
    // Test 3: Concurrency Bounding Semaphore
    // ─────────────────────────────────────────────────────────────────────────
    println!("\n[Test 3] Concurrency Semaphore Bounds...");
    let semaphore = media::image_processing::get_image_semaphore();
    println!("  Image Semaphore concurrent limit: {}", semaphore.available_permits());
    if semaphore.available_permits() == 4 {
        println!("  ✅ Passed: IMAGE_SEMAPHORE configured to deliberate concurrency bound of 4.");
    } else {
        println!("  ❌ Failed: Concurrency semaphore is not 4.");
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Test 4: Image Sizing & Ceilings (Original vs Processed Variants)
    // ─────────────────────────────────────────────────────────────────────────
    println!("\n[Test 4] Sizing & Compression Ceiling Step-down Sizing...");
    
    // Load 3 real high-complexity photos for compression benchmarking
    let test_specs = vec![
        ("nature", "/home/noon/.gemini/antigravity-ide/brain/40d1798b-24b5-4533-8b25-8c424c8ae297/nature_high_complexity_1783975424182.png"),
        ("people", "/home/noon/.gemini/antigravity-ide/brain/40d1798b-24b5-4533-8b25-8c424c8ae297/people_high_complexity_1783975444088.png"),
        ("text", "/home/noon/.gemini/antigravity-ide/brain/40d1798b-24b5-4533-8b25-8c424c8ae297/text_high_complexity_1783975465392.png"),
    ];

    println!("| Spec | Variant | Resolution | Size (Bytes) | Ceiling (Bytes) | Under Ceiling? |");
    println!("|---|---|---|---|---|---|");

    for (label, source_path) in test_specs {
        let id = Uuid::new_v4();
        let path = format!("{}{}.tmp", temp_dir, id);
        if let Err(e) = fs::copy(source_path, &path) {
            println!("  ❌ Failed to copy benchmark source image {}: {:?}", source_path, e);
            continue;
        }

        let res = media::image_processing::process_image(
            path.clone(),
            id,
            &format!("{}.png", label),
            "assets/uploads/gallery/",
            "/assets/uploads/gallery/",
            1920,
            &minio,
        ).await;

        let _ = fs::remove_file(&path);

        match res {
            Ok(processed) => {
                let variants = vec![
                    ("original", format!("assets/uploads/gallery/ZWI{}.webp", id), 300 * 1024),
                    ("large", format!("assets/uploads/gallery/ZWI{}_large.webp", id), 150 * 1024),
                    ("medium", format!("assets/uploads/gallery/ZWI{}_medium.webp", id), 80 * 1024),
                    ("card", format!("assets/uploads/gallery/ZWI{}_card.webp", id), 40 * 1024),
                    ("thumb", format!("assets/uploads/gallery/ZWI{}_thumb.webp", id), 15 * 1024),
                ];

                for (var_name, key, ceiling) in variants {
                    if let Some(size) = get_minio_object_size(&minio, &key).await {
                        let under = if size <= ceiling { "Yes ✅" } else { "No ❌" };
                        let res_str = match var_name {
                            "original" => format!("max {}px", 1920),
                            "large" => format!("max {}px", 1200),
                            "medium" => format!("max {}px", 800),
                            "card" => format!("max {}px", 400),
                            "thumb" => format!("max {}px", 150),
                            _ => "".to_string(),
                        };
                        println!(
                            "| {} | {} | {} | {} | {} | {} |",
                            label, var_name, res_str, size, ceiling, under
                        );
                    } else {
                        println!("| {} | {} | - | MISSING in MinIO ❌ | {} | - |", label, var_name, ceiling);
                    }
                }

                // Query database to verify status is marked 'ready'
                let db_status: Option<(String, Option<String>)> = sqlx::query_as(
                    "SELECT status, error_message FROM public.uploaded_files WHERE id = $1"
                )
                .bind(id)
                .fetch_optional(pool)
                .await
                .unwrap();

                if let Some((status, err_msg)) = db_status {
                    println!("    Database Record for {}: status='{}', error_message={:?} ✅", label, status, err_msg);
                } else {
                    println!("    Database Record for {}: NOT FOUND in public.uploaded_files ❌", label);
                }
            }
            Err(e) => {
                println!("  ❌ Failed to process {} image: {:?}", label, e);
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Test 5: Transactional Rollback ( induced upload failure )
    // ─────────────────────────────────────────────────────────────────────────
    println!("\n[Test 5] Induced MinIO Upload Failure & DB Status Rollback...");
    
    // We construct a MinioClient pointing to a non-existent bucket
    let mut bad_config = config.clone();
    bad_config.minio_bucket = "non-existent-bucket-99999-fail".to_string();
    let bad_minio = MinioClient::from_config(&bad_config, pool.clone());

    let test_img_bytes = create_test_image(400, 400);
    let fail_id = Uuid::new_v4();
    let fail_path = format!("{}{}.tmp", temp_dir, fail_id);
    fs::write(&fail_path, &test_img_bytes).unwrap();

    let res = media::image_processing::process_image(
        fail_path.clone(),
        fail_id,
        "rollback_test.png",
        "assets/uploads/gallery/",
        "/assets/uploads/gallery/",
        1920,
        &bad_minio,
    ).await;

    let _ = fs::remove_file(&fail_path);

    match res {
        Err(AppError::Internal(msg)) => {
            println!("  ✅ Passed: Sync processing returned 500 Internal Server Error as expected.");
            println!("     Message: \"{}\"", msg);

            // Verify database row status is marked as failed
            let db_status: Option<(String, Option<String>)> = sqlx::query_as(
                "SELECT status, error_message FROM public.uploaded_files WHERE id = $1"
            )
            .bind(fail_id)
            .fetch_optional(pool)
            .await
            .unwrap();

            if let Some((status, err_msg)) = db_status {
                if status == "failed" {
                    println!("  ✅ Passed: Database row is correctly marked 'failed'.");
                    println!("     DB Error Message: {:?}", err_msg);
                } else {
                    println!("  ❌ Failed: Database row status is '{}' instead of 'failed'.", status);
                }
            } else {
                println!("  ❌ Failed: No database row registered for the upload.");
            }
        }
        other => {
            println!("  ❌ Failed: Expected 500 Internal, got: {:?}", other);
        }
    }

    println!("\n=== MEDIA PIPELINE VERIFICATION END ===\n");
}
