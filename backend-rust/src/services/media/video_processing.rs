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

// AVIF encoding (using pure-Rust ravif encoder)
fn encode_to_avif(img: &image::DynamicImage, quality: u32) -> Result<Vec<u8>, AppError> {
    let rgba8 = img.to_rgba8();
    let (width, height) = rgba8.dimensions();
    
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

fn compute_checksum(bytes: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

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

    // ── Check magic bytes and MIME type ──────────────────────────────────────
    let raw_bytes = tokio::fs::read(&temp_path).await.map_err(|e| {
        AppError::Internal(format!("Failed to read raw temp file: {}", e))
    })?;

    let inferred = infer::get(&raw_bytes).ok_or_else(|| {
        AppError::BadRequest("Failed to infer file format from magic bytes: unrecognized payload".to_string())
    })?;

    let mime = inferred.mime_type();
    let is_valid_mime = match mime {
        "video/mp4" | "video/webm" | "video/quicktime" | "application/mp4" | "video/x-msvideo" | "video/avi" | "video/msvideo" => true,
        _ => false,
    };

    if !is_valid_mime {
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(AppError::BadRequest(format!(
            "MIME check failed: type '{}' is not a supported video format. Only MP4, WEBM, MOV, and AVI media payloads are allowed.",
            mime
        )));
    }

    // ── Disk space check ──────────────────────────────────────────────────────
    let file_size = raw_bytes.len() as u64;
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
            "Video not compatible (codec={}, ext={}, pix_fmt={}) — running full transcode with audio normalization...",
            video_codec, upload_ext, pix_fmt
        );

        let mut cmd = tokio::process::Command::new("ffmpeg");
        cmd.kill_on_drop(true);
        cmd.arg("-y").arg("-i").arg(&temp_path)
            .args(["-vcodec", "libx264", "-pix_fmt", "yuv420p", "-movflags", "faststart", "-crf", "23"]);
        if has_audio {
            cmd.args(["-acodec", "aac", "-filter:a", "loudnorm"]);
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

    // ── Resolution extraction ────────────────────────────────────────────────
    let width_str = probe_single_field(&final_disk_path, &[
        "-v", "error", "-select_streams", "v:0",
        "-show_entries", "stream=width",
        "-of", "default=noprint_wrappers=1:nokey=1",
    ]).await;
    let width = width_str.trim().parse::<i32>().ok();

    let height_str = probe_single_field(&final_disk_path, &[
        "-v", "error", "-select_streams", "v:0",
        "-show_entries", "stream=height",
        "-of", "default=noprint_wrappers=1:nokey=1",
    ]).await;
    let height = height_str.trim().parse::<i32>().ok();

    // ── Bitrate extraction ───────────────────────────────────────────────────
    let bitrate_str = probe_single_field(&final_disk_path, &[
        "-v", "error", "-select_streams", "v:0",
        "-show_entries", "stream=bit_rate",
        "-of", "default=noprint_wrappers=1:nokey=1",
    ]).await;
    let mut bitrate = bitrate_str.trim().parse::<i64>().ok();
    if bitrate.is_none() {
        let fmt_bitrate_str = probe_single_field(&final_disk_path, &[
            "-v", "error",
            "-show_entries", "format=bit_rate",
            "-of", "default=noprint_wrappers=1:nokey=1",
        ]).await;
        bitrate = fmt_bitrate_str.trim().parse::<i64>().ok();
    }

    // ── Frame rate detection ─────────────────────────────────────────────────
    let fps_str = probe_single_field(&final_disk_path, &[
        "-v", "error", "-select_streams", "v:0",
        "-show_entries", "stream=r_frame_rate",
        "-of", "default=noprint_wrappers=1:nokey=1",
    ]).await;
    tracing::info!("Detected video frame rate: {}", fps_str);

    // ── Orientation detection ────────────────────────────────────────────────
    let rotation_str = probe_single_field(&final_disk_path, &[
        "-v", "error", "-select_streams", "v:0",
        "-show_entries", "stream_tags=rotate",
        "-of", "default=noprint_wrappers=1:nokey=1",
    ]).await;
    let orientation = rotation_str.trim().parse::<i32>().unwrap_or(0);

    // ── Duration probe ────────────────────────────────────────────────────────
    let duration_seconds = {
        let s = probe_single_field(&final_disk_path, &[
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
        ]).await;
        s.trim().parse::<f64>().ok().map(|f| f.round() as i32)
    };

    let final_poster_filename = format!("ZWI{}_poster.webp", temp_id);
    let final_poster_avif_filename = format!("ZWI{}_poster.avif", temp_id);
    let final_thumb_filename = format!("ZWI{}_thumb.webp", temp_id);
    let final_thumb_avif_filename = format!("ZWI{}_thumb.avif", temp_id);

    // ── Poster & Thumbnail extraction ──────────────────────────────────────────
    let temp_thumb_path = format!("{}{}_thumb_temp.jpg", TEMP_DIR, temp_id);
    let mut cmd_thumb = tokio::process::Command::new("ffmpeg");
    cmd_thumb.kill_on_drop(true);
    let ffmpeg_output = cmd_thumb
        .args(["-y", "-i", &final_disk_path, "-ss", "00:00:01", "-vframes", "1", &temp_thumb_path])
        .output()
        .await;

    let mut thumbnail_url = None;
    let mut poster_w = None;
    let mut poster_h = None;

    if let Ok(out) = ffmpeg_output {
        if out.status.success() {
            let final_poster_disk_path = format!("{}{}", target_dir, final_poster_filename);
            let final_poster_avif_disk_path = format!("{}{}", target_dir, final_poster_avif_filename);
            let final_thumb_disk_path = format!("{}{}", target_dir, final_thumb_filename);
            let final_thumb_url = format!("{}{}", url_prefix, final_thumb_filename);
            let final_thumb_avif_disk_path = format!("{}{}", target_dir, final_thumb_avif_filename);

            let t_thumb = temp_thumb_path.clone();
            let f_poster = final_poster_disk_path.clone();
            let f_poster_avif = final_poster_avif_disk_path.clone();
            let f_thumb = final_thumb_disk_path.clone();
            let f_thumb_avif = final_thumb_avif_disk_path.clone();

            let process_thumb = tokio::task::spawn_blocking(move || -> Result<(i32, i32, i32, i32, String, String, Vec<u8>, Vec<u8>), AppError> {
                let img = image::ImageReader::open(&t_thumb)
                    .map_err(|e| AppError::BadRequest(e.to_string()))?
                    .with_guessed_format()
                    .map_err(|e| AppError::BadRequest(e.to_string()))?
                    .decode()
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;

                // Poster (max 1024px)
                let poster_img = if img.width() > 1024 || img.height() > 1024 {
                    img.resize(1024, 1024, FilterType::Lanczos3)
                } else {
                    img.clone()
                };
                let p_w = poster_img.width() as i32;
                let p_h = poster_img.height() as i32;

                // Thumbnail (max 150px)
                let thumb_img = if img.width() > 150 || img.height() > 150 {
                    img.resize(150, 150, FilterType::Lanczos3)
                } else {
                    img.clone()
                };
                let t_w = thumb_img.width() as i32;
                let t_h = thumb_img.height() as i32;

                // Save WebP poster
                poster_img
                    .save_with_format(&f_poster, ImageFormat::WebP)
                    .map_err(|e| AppError::Internal(e.to_string()))?;

                // Save AVIF poster
                let avif_poster_bytes = encode_to_avif(&poster_img, 70)?;
                std::fs::write(&f_poster_avif, &avif_poster_bytes)
                    .map_err(|e| AppError::Internal(e.to_string()))?;

                // Save WebP thumbnail
                thumb_img
                    .save_with_format(&f_thumb, ImageFormat::WebP)
                    .map_err(|e| AppError::Internal(e.to_string()))?;

                // Save AVIF thumbnail
                let avif_thumb_bytes = encode_to_avif(&thumb_img, 70)?;
                std::fs::write(&f_thumb_avif, &avif_thumb_bytes)
                    .map_err(|e| AppError::Internal(e.to_string()))?;

                // Compute checksums
                let webp_poster_bytes = std::fs::read(&f_poster).map_err(|e| AppError::Internal(e.to_string()))?;
                let webp_poster_checksum = compute_checksum(&webp_poster_bytes);

                let webp_thumb_bytes = std::fs::read(&f_thumb).map_err(|e| AppError::Internal(e.to_string()))?;
                let webp_thumb_checksum = compute_checksum(&webp_thumb_bytes);

                let _ = std::fs::remove_file(&t_thumb);
                Ok((
                    p_w, p_h, t_w, t_h,
                    webp_poster_checksum, webp_thumb_checksum,
                    avif_poster_bytes, avif_thumb_bytes
                ))
            }).await;

            if let Ok(Ok((p_w, p_h, t_w, t_h, wp_checksum, wt_checksum, avif_poster_bytes, avif_thumb_bytes))) = process_thumb {
                thumbnail_url = Some(final_thumb_url);
                poster_w = Some(p_w);
                poster_h = Some(p_h);
                
                // Upload WebP poster
                if let Err(e) = minio.upload(&final_poster_disk_path, target_dir, &final_poster_filename, "image/webp", Some(temp_id), Some(p_w), Some(p_h), Some(&wp_checksum)).await {
                    tracing::error!("MinIO: failed to upload video WebP poster: {}", e);
                }
                
                // Upload AVIF poster
                let avif_poster_checksum = compute_checksum(&avif_poster_bytes);
                if let Err(e) = minio.upload(&final_poster_avif_disk_path, target_dir, &final_poster_avif_filename, "image/avif", Some(temp_id), Some(p_w), Some(p_h), Some(&avif_poster_checksum)).await {
                    tracing::error!("MinIO: failed to upload video AVIF poster: {}", e);
                }

                // Upload WebP thumbnail
                if let Err(e) = minio.upload(&final_thumb_disk_path, target_dir, &final_thumb_filename, "image/webp", Some(temp_id), Some(t_w), Some(t_h), Some(&wt_checksum)).await {
                    tracing::error!("MinIO: failed to upload video WebP thumbnail: {}", e);
                }
                
                // Upload AVIF thumbnail
                let avif_thumb_checksum = compute_checksum(&avif_thumb_bytes);
                if let Err(e) = minio.upload(&final_thumb_avif_disk_path, target_dir, &final_thumb_avif_filename, "image/avif", Some(temp_id), Some(t_w), Some(t_h), Some(&avif_thumb_checksum)).await {
                    tracing::error!("MinIO: failed to upload video AVIF thumbnail: {}", e);
                }
            }
        }
    }

    let _ = fs::remove_file(&temp_thumb_path).await;

    let metadata = fs::metadata(&final_disk_path)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let final_bytes = tokio::fs::read(&final_disk_path).await.unwrap_or_default();
    let final_checksum = compute_checksum(&final_bytes);

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

    if let Err(e) = minio.upload(
        &processed.disk_path,
        target_dir,
        &final_filename,
        &processed.mime_type,
        None,
        width,
        height,
        Some(&final_checksum),
    ).await {
        tracing::error!("MinIO: failed to upload video: {}", e);
        // Clean up thumbnails from MinIO if uploaded
        if processed.thumbnail_url.is_some() {
            let _ = minio.delete(&format!("{}{}", target_dir, final_poster_filename)).await;
            let _ = minio.delete(&format!("{}{}", target_dir, final_poster_avif_filename)).await;
            let _ = minio.delete(&format!("{}{}", target_dir, final_thumb_filename)).await;
            let _ = minio.delete(&format!("{}{}", target_dir, final_thumb_avif_filename)).await;
        }
        return Err(AppError::Internal(format!("MinIO: failed to upload video: {}", e)));
    }

    // ── Stage 2: Adaptive HLS Transcoding ──────────────────────────────────────
    let temp_hls_dir = format!("{}hls_{}/", TEMP_DIR, temp_id);
    let _ = fs::create_dir_all(&temp_hls_dir).await;

    let hls_start_time = std::time::Instant::now();
    let mut cmd_hls = tokio::process::Command::new("ffmpeg");
    cmd_hls.kill_on_drop(true);
    cmd_hls.arg("-y")
        .arg("-i").arg(&final_disk_path)
        .arg("-filter_complex").arg("[0:v]split=3[v1][v2][v3]; [v1]scale=w=1920:h=1080[v1out]; [v2]scale=w=1280:h=720[v2out]; [v3]scale=w=854:h=480[v3out]")
        .arg("-map").arg("[v1out]").arg("-c:v:0").arg("libx264").arg("-b:v:0").arg("3000k").arg("-maxrate:v:0").arg("3300k").arg("-bufsize:v:0").arg("6000k").arg("-r").arg("30").arg("-g").arg("60").arg("-keyint_min").arg("60").arg("-sc_threshold").arg("0")
        .arg("-map").arg("[v2out]").arg("-c:v:1").arg("libx264").arg("-b:v:1").arg("1500k").arg("-maxrate:v:1").arg("1650k").arg("-bufsize:v:1").arg("3000k").arg("-r").arg("30").arg("-g").arg("60").arg("-keyint_min").arg("60").arg("-sc_threshold").arg("0")
        .arg("-map").arg("[v3out]").arg("-c:v:2").arg("libx264").arg("-b:v:2").arg("800k").arg("-maxrate:v:2").arg("880k").arg("-bufsize:v:2").arg("1600k").arg("-r").arg("30").arg("-g").arg("60").arg("-keyint_min").arg("60").arg("-sc_threshold").arg("0");

    if has_audio {
        cmd_hls.arg("-map").arg("0:a").arg("-c:a:0").arg("aac").arg("-b:a:0").arg("128k")
            .arg("-map").arg("0:a").arg("-c:a:1").arg("aac").arg("-b:a:1").arg("128k")
            .arg("-map").arg("0:a").arg("-c:a:2").arg("aac").arg("-b:a:2").arg("96k")
            .arg("-var_stream_map").arg("v:0,a:0 v:1,a:1 v:2,a:2");
    } else {
        cmd_hls.arg("-var_stream_map").arg("v:0 v:1 v:2");
    }

    cmd_hls.args([
        "-f", "hls",
        "-hls_time", "6",
        "-hls_playlist_type", "vod",
        "-hls_segment_type", "fmp4",
        "-hls_segment_filename", &format!("{}ZWV{}_%v_%03d.m4s", temp_hls_dir, temp_id),
        "-master_pl_name", &format!("ZWV{}_master.m3u8", temp_id),
        &format!("{}ZWV{}_%v.m3u8", temp_hls_dir, temp_id)
    ]);

    let hls_transcode_res = tokio::time::timeout(
        std::time::Duration::from_secs(120),
        cmd_hls.output(),
    ).await;

    let hls_success = match hls_transcode_res {
        Ok(Ok(out)) if out.status.success() => {
            let elapsed = hls_start_time.elapsed().as_millis() as u64;
            crate::services::metrics::observe_encoding(elapsed);
            crate::services::metrics::observe_segment_gen(elapsed);
            crate::services::metrics::observe_playlist_gen(elapsed);
            tracing::info!("HLS transcode/segmentation succeeded in {} ms.", elapsed);
            true
        }
        Ok(Ok(out)) => {
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            tracing::error!("ffmpeg HLS transcoding failed: {}", stderr);
            crate::services::metrics::inc_hls_generation_failures();
            false
        }
        _ => {
            tracing::error!("ffmpeg HLS transcoding timed out or failed to execute.");
            crate::services::metrics::inc_hls_generation_failures();
            false
        }
    };

    if hls_success {
        // Upload all HLS files (playlists and segment fmp4 files) to MinIO
        if let Ok(mut entries) = fs::read_dir(&temp_hls_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.is_file() {
                    if let Some(filename_str) = path.file_name().and_then(|s| s.to_str()) {
                        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                        let mime_type = match extension {
                            "m3u8" => "application/vnd.apple.mpegurl",
                            "m4s" => "video/iso.segment",
                            "mp4" => "video/mp4",
                            _ => "application/octet-stream",
                        };

                        let file_bytes = tokio::fs::read(&path).await.unwrap_or_default();
                        let checksum = compute_checksum(&file_bytes);

                        if let Err(e) = minio.upload(
                            path.to_str().unwrap(),
                            target_dir,
                            filename_str,
                            mime_type,
                            Some(temp_id),
                            None,
                            None,
                            Some(&checksum),
                        ).await {
                            tracing::error!("MinIO: failed to upload HLS variant {}: {}", filename_str, e);
                        }
                    }
                }
            }
        }
    }

    let _ = fs::remove_dir_all(&temp_hls_dir).await;

    // Update db status and store complete metadata
    let _ = minio.update_upload_status(
        temp_id,
        "ready",
        None,
        Some(processed.file_size as i64),
        width,
        height,
        duration_seconds,
        Some("h264"),
        bitrate,
        Some(orientation),
        Some(&final_checksum),
    )
    .await;

    // Clean up local staging files (video and thumbnails) after successful upload
    if processed.thumbnail_url.is_some() {
        let final_poster_disk_path = format!("{}{}", target_dir, final_poster_filename);
        let _ = fs::remove_file(&final_poster_disk_path).await;

        let final_poster_avif_disk_path = format!("{}{}", target_dir, final_poster_avif_filename);
        let _ = fs::remove_file(&final_poster_avif_disk_path).await;

        let final_thumb_disk_path = format!("{}{}", target_dir, final_thumb_filename);
        let _ = fs::remove_file(&final_thumb_disk_path).await;

        let final_thumb_avif_disk_path = format!("{}{}", target_dir, final_thumb_avif_filename);
        let _ = fs::remove_file(&final_thumb_avif_disk_path).await;
    }
    let _ = fs::remove_file(&processed.disk_path).await;

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
