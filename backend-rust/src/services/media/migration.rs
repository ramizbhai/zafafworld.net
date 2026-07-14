//! One-time video transcoding migration.
//!
//! Formerly `services/media_migration.rs`. Moved into `services/media/`
//! to keep all media-related code co-located.
//!
//! This module is only invoked from `main.rs` during startup if the
//! `RUN_VIDEO_MIGRATION=true` environment variable is set.

use crate::errors::AppError;
use sqlx::Row;
use std::path::Path;
use tokio::fs;
use uuid::Uuid;

pub async fn run_video_transcoding_migration(db_pool: &sqlx::PgPool) -> Result<(), AppError> {
    tracing::info!("Starting video transcoding migration...");

    // Fetch all video rows from vendor_gallery
    let rows = sqlx::query(
        "SELECT id, file_url, thumbnail_url, vendor_id FROM vendor_gallery WHERE media_type = 'video'"
    )
    .fetch_all(db_pool)
    .await
    .map_err(|e| AppError::Internal(format!("Failed to fetch video gallery items: {}", e)))?;

    tracing::info!("Found {} video items in vendor_gallery.", rows.len());

    let mut success_count = 0;
    let mut skipped_count = 0;
    let mut failed_count = 0;

    for row in rows {
        let id: Uuid = row.try_get("id").map_err(|e| AppError::Internal(e.to_string()))?;
        let file_url: String = row.try_get("file_url").map_err(|e| AppError::Internal(e.to_string()))?;
        let thumbnail_url: Option<String> = row.try_get("thumbnail_url").unwrap_or(None);

        // Determine disk path from file_url
        let disk_relative_path = file_url.trim_start_matches('/');
        let file_path = Path::new(disk_relative_path);

        if !file_path.exists() {
            tracing::warn!("Video file does not exist on disk at {:?}. Skipping.", file_path);
            skipped_count += 1;
            continue;
        }

        // Disk Space Safety Check
        let file_size = match fs::metadata(disk_relative_path).await {
            Ok(m) => m.len(),
            Err(_) => 0,
        };

        if let Some(available_space) = crate::services::media::get_available_disk_space("assets/uploads/gallery/").await {
            let required_space = file_size * 3;
            let safe_margin = 500 * 1024 * 1024; // 500 MB
            if available_space < required_space || available_space < safe_margin {
                tracing::error!(
                    "Migration failed for item {}: insufficient disk space (Available: {} MB, Required: {} MB)",
                    id,
                    available_space / (1024 * 1024),
                    std::cmp::max(required_space, safe_margin) / (1024 * 1024)
                );
                failed_count += 1;
                continue;
            }
        }

        // Get extension
        let ext = file_path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();

        // 1. Probe video codec (Timeout: 10s)
        let ffprobe_codec = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            tokio::process::Command::new("ffprobe")
                .args([
                    "-v", "error",
                    "-select_streams", "v:0",
                    "-show_entries", "stream=codec_name",
                    "-of", "default=noprint_wrappers=1:nokey=1",
                    disk_relative_path,
                ])
                .output(),
        )
        .await;

        let video_codec = match ffprobe_codec {
            Ok(Ok(out)) if out.status.success() => {
                String::from_utf8_lossy(&out.stdout).trim().to_string().to_lowercase()
            }
            _ => String::new(),
        };

        // 2. Probe pixel format (Timeout: 10s)
        let ffprobe_pix = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            tokio::process::Command::new("ffprobe")
                .args([
                    "-v", "error",
                    "-select_streams", "v:0",
                    "-show_entries", "stream=pix_fmt",
                    "-of", "default=noprint_wrappers=1:nokey=1",
                    disk_relative_path,
                ])
                .output(),
        )
        .await;

        let pix_fmt = match ffprobe_pix {
            Ok(Ok(out)) if out.status.success() => {
                String::from_utf8_lossy(&out.stdout).trim().to_string().to_lowercase()
            }
            _ => String::new(),
        };

        // 3. Check if audio stream exists (Timeout: 10s)
        let ffprobe_audio_exists = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            tokio::process::Command::new("ffprobe")
                .args([
                    "-v", "error",
                    "-select_streams", "a",
                    "-show_entries", "stream=index",
                    "-of", "default=noprint_wrappers=1:nokey=1",
                    disk_relative_path,
                ])
                .output(),
        )
        .await;

        let has_audio = match ffprobe_audio_exists {
            Ok(Ok(out)) if out.status.success() => {
                !String::from_utf8_lossy(&out.stdout).trim().is_empty()
            }
            _ => false,
        };

        // 4. Probe audio codec if exists (Timeout: 10s)
        let audio_codec = if has_audio {
            let ffprobe_audio_codec = tokio::time::timeout(
                std::time::Duration::from_secs(10),
                tokio::process::Command::new("ffprobe")
                    .args([
                        "-v", "error",
                        "-select_streams", "a:0",
                        "-show_entries", "stream=codec_name",
                        "-of", "default=noprint_wrappers=1:nokey=1",
                        disk_relative_path,
                    ])
                    .output(),
            )
            .await;
            match ffprobe_audio_codec {
                Ok(Ok(out)) if out.status.success() => {
                    String::from_utf8_lossy(&out.stdout).trim().to_string().to_lowercase()
                }
                _ => String::new(),
            }
        } else {
            "aac".to_string() // Dummy value since there is no audio to be converted
        };

        // 5. Probe format container (Timeout: 10s)
        let ffprobe_container = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            tokio::process::Command::new("ffprobe")
                .args([
                    "-v", "error",
                    "-show_entries", "format=format_name",
                    "-of", "default=noprint_wrappers=1:nokey=1",
                    disk_relative_path,
                ])
                .output(),
        )
        .await;

        let container_format = match ffprobe_container {
            Ok(Ok(out)) if out.status.success() => {
                String::from_utf8_lossy(&out.stdout).trim().to_string().to_lowercase()
            }
            _ => String::new(),
        };

        let is_mp4_container = container_format.contains("mp4") || container_format.contains("mov");

        let is_web_compatible = ext == "mp4"
            && is_mp4_container
            && video_codec == "h264"
            && pix_fmt == "yuv420p"
            && audio_codec == "aac";

        // Concurrency Control: Acquire Semaphore Permit for Migration
        tracing::info!("Acquiring transcoding permit for migration item {}...", id);
        let _permit = crate::services::media::video_processing::get_ffmpeg_semaphore().acquire().await.ok();
        tracing::info!("Transcoding permit acquired for migration item {}.", id);

        if is_web_compatible {
            tracing::info!("Video {:?} is already H.264 MP4. Checking if we need faststart optimization...", file_path);
            
            let target_dir = "assets/uploads/gallery/";
            let temp_opt_path = format!("{}{}.opt.mp4", target_dir, id);

            let mut cmd = tokio::process::Command::new("ffmpeg");
            cmd.arg("-y").arg("-i").arg(disk_relative_path);
            
            if has_audio {
                cmd.args(["-vcodec", "copy", "-acodec", "copy", "-movflags", "faststart"]);
            } else {
                cmd.args(["-vcodec", "copy", "-an", "-movflags", "faststart"]);
            }
            cmd.arg(&temp_opt_path);

            let copy_pass = tokio::time::timeout(std::time::Duration::from_secs(30), cmd.output()).await;

            match copy_pass {
                Ok(Ok(out)) if out.status.success() => {
                    // Overwrite the original file atomically
                    if let Err(e) = fs::rename(&temp_opt_path, disk_relative_path).await {
                        tracing::error!("Failed to replace file with optimized version: {}", e);
                        let _ = fs::remove_file(&temp_opt_path).await;
                    } else {
                        tracing::info!("Applied faststart optimization to already-compatible video: {:?}", file_path);
                    }
                }
                _ => {
                    tracing::warn!("Faststart optimization pass failed or timed out. Keeping original file.");
                    let _ = fs::remove_file(&temp_opt_path).await;
                }
            }

            skipped_count += 1;
            continue;
        }

        tracing::info!("Transcoding video {:?} to H.264 MP4...", file_path);

        // Prepare new paths
        let target_dir = "assets/uploads/gallery/";
        let new_filename = format!("{}.mp4", id);
        let new_disk_path = format!("{}{}", target_dir, new_filename);
        let temp_disk_path = format!("{}{}.tmp.mp4", target_dir, id);

        // Build ffmpeg arguments for transcoding (with CRF 23)
        let mut cmd = tokio::process::Command::new("ffmpeg");
        cmd.arg("-y").arg("-i").arg(disk_relative_path);
        cmd.args(["-vcodec", "libx264", "-pix_fmt", "yuv420p", "-movflags", "faststart", "-crf", "23"]);

        if has_audio {
            cmd.args(["-acodec", "aac"]);
        } else {
            cmd.arg("-an");
        }

        cmd.arg(&temp_disk_path);

        // Run transcoding with timeout protection (120 seconds limit)
        let transcode_result = tokio::time::timeout(std::time::Duration::from_secs(120), cmd.output()).await;

        match transcode_result {
            Ok(Ok(out)) if out.status.success() => {
                tracing::info!("Transcoded successfully to temp file: {}", temp_disk_path);

                // Atomic step: verify the output MP4 with ffprobe
                let verify_codec_output = tokio::time::timeout(
                    std::time::Duration::from_secs(10),
                    tokio::process::Command::new("ffprobe")
                        .args([
                            "-v", "error",
                            "-select_streams", "v:0",
                            "-show_entries", "stream=codec_name",
                            "-of", "default=noprint_wrappers=1:nokey=1",
                            &temp_disk_path,
                        ])
                        .output(),
                )
                .await;

                let verified = match verify_codec_output {
                    Ok(Ok(out)) if out.status.success() => {
                        let codec = String::from_utf8_lossy(&out.stdout).trim().to_string();
                        codec == "h264"
                    }
                    _ => false,
                };

                if !verified {
                    tracing::error!("Verification failed for transcoded video: {}", temp_disk_path);
                    let _ = fs::remove_file(&temp_disk_path).await;
                    failed_count += 1;
                    continue;
                }

                // Move verified temp output to final disk path
                if let Err(e) = fs::rename(&temp_disk_path, &new_disk_path).await {
                    tracing::error!("Failed to rename temp file to {}: {}", new_disk_path, e);
                    let _ = fs::remove_file(&temp_disk_path).await;
                    failed_count += 1;
                    continue;
                }

                // Get file metadata size
                let metadata = match fs::metadata(&new_disk_path).await {
                    Ok(m) => m.len() as i64,
                    Err(_) => 0,
                };

                // Get new duration
                let duration_output = tokio::process::Command::new("ffprobe")
                    .args([
                        "-v", "error",
                        "-show_entries", "format=duration",
                        "-of", "default=noprint_wrappers=1:nokey=1",
                        &new_disk_path,
                    ])
                    .output()
                    .await;

                let duration_seconds = match duration_output {
                    Ok(out) if out.status.success() => {
                        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                        s.parse::<f64>().ok().map(|f| f.round() as i32)
                    }
                    _ => None,
                };

                // Generate new thumbnail from the new transcoded MP4
                let temp_thumb_path = format!("assets/uploads/temp/{}_thumb_temp.jpg", id);
                let _ = fs::create_dir_all("assets/uploads/temp/").await;
                
                let ffmpeg_thumb = tokio::time::timeout(
                    std::time::Duration::from_secs(15),
                    tokio::process::Command::new("ffmpeg")
                        .args([
                            "-y",
                            "-i", &new_disk_path,
                            "-ss", "00:00:01",
                            "-vframes", "1",
                            &temp_thumb_path,
                        ])
                        .output()
                )
                .await;

                let mut new_thumbnail_url = thumbnail_url.clone();

                if let Ok(Ok(out)) = ffmpeg_thumb {
                    if out.status.success() {
                        let final_thumb_filename = format!("{}_thumb.webp", id);
                        let final_thumb_disk_path = format!("{}{}", target_dir, final_thumb_filename);
                        let final_thumb_url = format!("/assets/uploads/gallery/{}", final_thumb_filename);

                        // Process and convert thumbnail to WebP using spawn_blocking
                        let t_thumb_path = temp_thumb_path.clone();
                        let f_thumb_disk_path = final_thumb_disk_path.clone();

                        let process_res = tokio::task::spawn_blocking(move || {
                            use image::{GenericImageView, ImageFormat};
                            let img = image::ImageReader::open(&t_thumb_path)
                                .map_err(|e| e.to_string())?
                                .with_guessed_format()
                                .map_err(|e| e.to_string())?
                                .decode()
                                .map_err(|e| e.to_string())?;

                            let (w, h) = img.dimensions();
                            let final_img = if w > 1024 || h > 1024 {
                                img.resize(1024, 1024, image::imageops::FilterType::Lanczos3)
                            } else {
                                img
                            };

                            final_img
                                .save_with_format(&f_thumb_disk_path, ImageFormat::WebP)
                                .map_err(|e| e.to_string())?;

                            let _ = std::fs::remove_file(&t_thumb_path);
                            Ok::<(), String>(())
                        })
                        .await;

                        if let Ok(Ok(())) = process_res {
                            new_thumbnail_url = Some(final_thumb_url);
                        } else {
                            let _ = fs::remove_file(&temp_thumb_path).await;
                        }
                    }
                }

                // Update database
                let new_file_url = format!("/assets/uploads/gallery/{}", new_filename);
                let update_res = sqlx::query(
                    r#"UPDATE vendor_gallery
                       SET file_url = $1, image_url = $1, thumbnail_url = $2, file_size = $3, duration_seconds = $4
                       WHERE id = $5"#
                )
                .bind(&new_file_url)
                .bind(&new_thumbnail_url)
                .bind(metadata)
                .bind(duration_seconds)
                .bind(id)
                .execute(db_pool)
                .await;

                match update_res {
                    Ok(_) => {
                        tracing::info!("Database updated successfully for gallery item {}", id);
                        
                        // ONLY now it is safe to delete the old file if it has a different path!
                        if disk_relative_path != new_disk_path {
                            if let Err(e) = fs::remove_file(disk_relative_path).await {
                                tracing::warn!("Could not remove old file {}: {}", disk_relative_path, e);
                            }
                        }
                        success_count += 1;
                    }
                    Err(e) => {
                        tracing::error!("Failed to update database for gallery item {}: {}", id, e);
                        let _ = fs::remove_file(&new_disk_path).await;
                        failed_count += 1;
                    }
                }
            }
            Ok(Ok(out)) => {
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                tracing::error!("ffmpeg transcoding failed for video {}: {}", disk_relative_path, stderr);
                let _ = fs::remove_file(&temp_disk_path).await;
                failed_count += 1;
            }
            _ => {
                tracing::error!("ffmpeg transcoding timed out or failed to run for video {}", disk_relative_path);
                let _ = fs::remove_file(&temp_disk_path).await;
                failed_count += 1;
            }
        }
    }

    tracing::info!(
        "Migration completed. Transcoded: {}, Skipped/Optimized: {}, Failed: {}",
        success_count,
        skipped_count,
        failed_count
    );

    Ok(())
}
