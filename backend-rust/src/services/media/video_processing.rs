//! Video processing pipeline (ffprobe + ffmpeg).
//!
//! Handles the video branch of [`super::process_and_save_upload`]:
//! - 5 sequential ffprobe calls to inspect codec, pixel format, audio, container
//! - Faststart-optimized copy pass for already-compatible H.264 MP4 files
//! - Full transcode (libx264 + AAC) for other formats
//! - Thumbnail extraction and WebP conversion
//! - MinIO upload for the final video and thumbnail

use crate::errors::AppError;
use crate::services::media::minio_client::MinioClient;
use crate::services::media::{ProcessedMedia, TEMP_DIR};
use image::imageops::FilterType;
use image::ImageFormat;
use std::sync::OnceLock;
use tokio::fs;
use tokio::sync::Semaphore;
use uuid::Uuid;

// ── Semaphore ─────────────────────────────────────────────────────────────────

pub static FFMPEG_SEMAPHORE: OnceLock<Semaphore> = OnceLock::new();

pub fn get_ffmpeg_semaphore() -> &'static Semaphore {
    FFMPEG_SEMAPHORE.get_or_init(|| Semaphore::new(2))
}

// ── Public entry point ────────────────────────────────────────────────────────

/// Process an already-streamed temporary video file.
///
/// Probes the video, applies faststart optimization or full transcode as
/// needed, extracts a WebP thumbnail, and uploads both to MinIO.
pub async fn process_video(
    temp_path: String,
    temp_id: Uuid,
    original_file_name: &str,
    target_dir: &str,
    url_prefix: &str,
    minio: &MinioClient,
) -> Result<ProcessedMedia, AppError> {
    let final_filename = format!("ZWV{}.mp4", temp_id);
    let final_disk_path = format!("{}{}", target_dir, final_filename);
    let final_url = format!("{}{}", url_prefix, final_filename);

    // ── Disk space check ──────────────────────────────────────────────────────
    let file_size = tokio::fs::metadata(&temp_path).await.map(|m| m.len()).unwrap_or(0);
    if let Some(available_space) = super::get_available_disk_space(target_dir).await {
        let required_space = file_size * 3;
        let safe_margin = 500 * 1024 * 1024; // 500 MB
        if available_space < required_space || available_space < safe_margin {
            let _ = tokio::fs::remove_file(&temp_path).await;
            return Err(AppError::Internal(format!(
                "Insufficient disk space to process video (Available: {} MB, Required: {} MB)",
                available_space / (1024 * 1024),
                std::cmp::max(required_space, safe_margin) / (1024 * 1024)
            )));
        }
    }

    // ── Extension inference ───────────────────────────────────────────────────
    let upload_ext = std::path::Path::new(original_file_name)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    // ── ffprobe: video codec ──────────────────────────────────────────────────
    let video_codec = probe_single_field(&temp_path, &[
        "-v", "error", "-select_streams", "v:0",
        "-show_entries", "stream=codec_name",
        "-of", "default=noprint_wrappers=1:nokey=1",
    ]).await.to_lowercase();

    // ── ffprobe: pixel format ─────────────────────────────────────────────────
    let pix_fmt = probe_single_field(&temp_path, &[
        "-v", "error", "-select_streams", "v:0",
        "-show_entries", "stream=pix_fmt",
        "-of", "default=noprint_wrappers=1:nokey=1",
    ]).await.to_lowercase();

    // ── ffprobe: audio stream presence ───────────────────────────────────────
    let has_audio = {
        let out = probe_single_field(&temp_path, &[
            "-v", "error", "-select_streams", "a",
            "-show_entries", "stream=index",
            "-of", "default=noprint_wrappers=1:nokey=1",
        ]).await;
        !out.trim().is_empty()
    };

    // ── ffprobe: audio codec (if audio exists) ────────────────────────────────
    let audio_codec = if has_audio {
        probe_single_field(&temp_path, &[
            "-v", "error", "-select_streams", "a:0",
            "-show_entries", "stream=codec_name",
            "-of", "default=noprint_wrappers=1:nokey=1",
        ]).await.to_lowercase()
    } else {
        "aac".to_string()
    };

    // ── ffprobe: container format ─────────────────────────────────────────────
    let container_format = probe_single_field(&temp_path, &[
        "-v", "error",
        "-show_entries", "format=format_name",
        "-of", "default=noprint_wrappers=1:nokey=1",
    ]).await.to_lowercase();

    let is_mp4_container = container_format.contains("mp4") || container_format.contains("mov");

    let is_web_compatible = upload_ext == "mp4"
        && is_mp4_container
        && video_codec == "h264"
        && pix_fmt == "yuv420p"
        && audio_codec == "aac";

    // ── Acquire ffmpeg semaphore ───────────────────────────────────────────────
    tracing::info!("Acquiring video transcoding/remuxing permit...");
    let _permit = get_ffmpeg_semaphore().acquire().await.ok();
    tracing::info!("Video transcoding/remuxing permit acquired.");

    let mut processed_successfully = false;

    if is_web_compatible {
        tracing::info!("Uploaded video is already web compatible — applying faststart optimization...");
        let mut cmd = tokio::process::Command::new("ffmpeg");
        cmd.kill_on_drop(true);
        cmd.arg("-y").arg("-i").arg(&temp_path);
        if has_audio {
            cmd.args(["-vcodec", "copy", "-acodec", "copy", "-movflags", "faststart"]);
        } else {
            cmd.args(["-vcodec", "copy", "-an", "-movflags", "faststart"]);
        }
        cmd.arg(&final_disk_path);

        let copy_pass = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            cmd.output(),
        ).await;

        match copy_pass {
            Ok(Ok(out)) if out.status.success() => {
                tracing::info!("Faststart copy pass succeeded.");
                processed_successfully = true;
            }
            _ => {
                tracing::warn!("Faststart copy pass failed — falling back to rename.");
                if let Err(e) = fs::rename(&temp_path, &final_disk_path).await {
                    tracing::error!("Rename fallback failed: {}", e);
                } else {
                    processed_successfully = true;
                }
            }
        }
    } else {
        tracing::info!(
            "Video not compatible (codec={}, ext={}, pix_fmt={}) — running full transcode...",
            video_codec, upload_ext, pix_fmt
        );

        let mut cmd = tokio::process::Command::new("ffmpeg");
        cmd.kill_on_drop(true);
        cmd.arg("-y").arg("-i").arg(&temp_path)
            .args(["-vcodec", "libx264", "-pix_fmt", "yuv420p", "-movflags", "faststart", "-crf", "23"]);
        if has_audio {
            cmd.args(["-acodec", "aac"]);
        } else {
            cmd.arg("-an");
        }
        cmd.arg(&final_disk_path);

        let transcode_res = tokio::time::timeout(
            std::time::Duration::from_secs(120),
            cmd.output(),
        ).await;

        match transcode_res {
            Ok(Ok(out)) if out.status.success() => {
                tracing::info!("Transcoding pipeline succeeded.");
                processed_successfully = true;
            }
            Ok(Ok(out)) => {
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                tracing::error!("ffmpeg transcoding failed: {}", stderr);
            }
            _ => {
                tracing::error!("ffmpeg transcoding timed out or failed to execute.");
            }
        }
    }

    // ── Cleanup temp file ─────────────────────────────────────────────────────
    let _ = tokio::fs::remove_file(&temp_path).await;

    if !processed_successfully {
        let _ = tokio::fs::remove_file(&final_disk_path).await;
        return Err(AppError::BadRequest(
            "Failed to process video file: transcoding/optimization error".to_string(),
        ));
    }

    // ── Verify output is H.264 ────────────────────────────────────────────────
    let codec_check = probe_single_field(&final_disk_path, &[
        "-v", "error", "-select_streams", "v:0",
        "-show_entries", "stream=codec_name",
        "-of", "default=noprint_wrappers=1:nokey=1",
    ]).await;

    if codec_check.trim() != "h264" {
        let _ = tokio::fs::remove_file(&final_disk_path).await;
        return Err(AppError::BadRequest(
            "Failed to process video file: codec verification failed".to_string(),
        ));
    }

    // ── Duration probe ────────────────────────────────────────────────────────
    let duration_seconds = {
        let s = probe_single_field(&final_disk_path, &[
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
        ]).await;
        s.trim().parse::<f64>().ok().map(|f| f.round() as i32)
    };

    // ── Thumbnail extraction ──────────────────────────────────────────────────
    let temp_thumb_path = format!("{}{}_thumb_temp.jpg", TEMP_DIR, temp_id);
    let mut cmd_thumb = tokio::process::Command::new("ffmpeg");
    cmd_thumb.kill_on_drop(true);
    let ffmpeg_output = cmd_thumb
        .args(["-y", "-i", &final_disk_path, "-ss", "00:00:01", "-vframes", "1", &temp_thumb_path])
        .output()
        .await;

    let mut thumbnail_url = None;

    if let Ok(out) = ffmpeg_output {
        if out.status.success() {
            let final_thumb_filename = format!("ZWI{}_thumb.webp", temp_id);
            let final_thumb_disk_path = format!("{}{}", target_dir, final_thumb_filename);
            let final_thumb_url = format!("{}{}", url_prefix, final_thumb_filename);

            let t_thumb = temp_thumb_path.clone();
            let f_thumb = final_thumb_disk_path.clone();

            let process_thumb = tokio::task::spawn_blocking(move || -> Result<(), AppError> {
                let img = image::ImageReader::open(&t_thumb)
                    .map_err(|e| AppError::BadRequest(e.to_string()))?
                    .with_guessed_format()
                    .map_err(|e| AppError::BadRequest(e.to_string()))?
                    .decode()
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;

                let final_img = if img.width() > 1024 || img.height() > 1024 {
                    img.resize(1024, 1024, FilterType::Lanczos3)
                } else {
                    img
                };

                final_img
                    .save_with_format(&f_thumb, ImageFormat::WebP)
                    .map_err(|e| AppError::Internal(e.to_string()))?;

                let _ = std::fs::remove_file(&t_thumb);
                Ok(())
            }).await;

            if let Ok(Ok(())) = process_thumb {
                thumbnail_url = Some(final_thumb_url);
                if let Err(e) = minio.upload(&final_thumb_disk_path, target_dir, &final_thumb_filename, "image/webp", Some(temp_id)).await {
                    tracing::error!("MinIO: failed to upload video thumbnail: {}", e);
                }
            }
        }
    }

    let _ = fs::remove_file(&temp_thumb_path).await;

    let metadata = fs::metadata(&final_disk_path)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let processed = ProcessedMedia {
        id: temp_id,
        file_name: original_file_name.to_string(),
        file_url: final_url,
        file_size: metadata.len() as usize,
        mime_type: "video/mp4".to_string(),
        disk_path: final_disk_path.clone(),
        media_type: "video".to_string(),
        thumbnail_url,
        duration_seconds,
        status: "ready".to_string(),
    };

    if let Err(e) = minio.upload(&processed.disk_path, target_dir, &final_filename, &processed.mime_type, None).await {
        tracing::error!("MinIO: failed to upload video: {}", e);
        return Err(AppError::Internal(format!("MinIO: failed to upload video: {}", e)));
    }

    Ok(processed)
}

// ── Internal helper ───────────────────────────────────────────────────────────

/// Run a single ffprobe invocation (10-second timeout) and return stdout as
/// a trimmed string. Returns an empty string on any error.
async fn probe_single_field(path: &str, args: &[&str]) -> String {
    let mut all_args = args.to_vec();
    all_args.push(path);

    let mut cmd = tokio::process::Command::new("ffprobe");
    cmd.kill_on_drop(true);
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        cmd.args(&all_args).output(),
    ).await;

    match result {
        Ok(Ok(out)) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }
        _ => String::new(),
    }
}
