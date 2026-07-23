//! MinIO object-storage client.
//!
//! Provides a thin wrapper around the `rust-s3` crate that reads credentials
//! once at startup (from `AppConfig`) rather than re-reading environment
//! variables on every call.
//!
//! # Usage
//!
//! ```no_run
//! // In main.rs, during AppState construction:
//! let minio_client = Arc::new(MinioClient::from_config(&app_config, db_pool.clone()));
//! ```
//!
//! Callers that previously invoked free functions (`upload_file_to_minio`,
//! `delete_file_from_minio`, `delete_gallery_item_from_minio`) should now
//! call the corresponding methods on `state.minio_client`.

use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::Region;

use sqlx::PgPool;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::repositories::uploaded_files_repository;

/// Singleton MinIO client.
///
/// Constructed once at startup and stored in `AppState` behind an `Arc`.
/// All methods are `&self` — the struct is intentionally stateless beyond
/// the config fields; S3 requests are stateless HTTP calls.
///
/// After a successful `upload`, the object key is automatically registered
/// in the `uploaded_files` table via `uploaded_files_repository::insert_upload`.
/// After a successful `delete`, the row is removed via `delete_by_key`.
#[derive(Clone)]
pub struct MinioClient {
    endpoint: String,
    bucket_name: String,
    app_user: String,
    app_password: String,
    root_prefix: String,
    /// DB pool for auto-registering uploads/deletes in `uploaded_files`.
    pool: PgPool,
}

impl MinioClient {
    /// Construct from `AppConfig` and a DB pool.
    ///
    /// The pool is used to keep `uploaded_files` in sync with MinIO without
    /// requiring callers to manage the registry separately.
    pub fn from_config(config: &AppConfig, pool: PgPool) -> Self {
        Self {
            endpoint: config.minio_endpoint.clone(),
            bucket_name: config.minio_bucket.clone(),
            app_user: config.minio_app_user.clone(),
            app_password: config.minio_app_password.clone(),
            root_prefix: config.minio_root_prefix.clone(),
            pool,
        }
    }

    pub fn root_prefix(&self) -> &str {
        &self.root_prefix
    }

    pub fn bucket(&self) -> Result<Bucket, String> {
        if self.app_user.is_empty() || self.app_password.is_empty() {
            return Err("MinIO credentials not configured".to_string());
        }

        let credentials = Credentials::new(
            Some(&self.app_user),
            Some(&self.app_password),
            None,
            None,
            None,
        )
        .map_err(|e| format!("Failed to create MinIO credentials: {}", e))?;

        let region = Region::Custom {
            region: "us-east-1".to_string(),
            endpoint: self.endpoint.clone(),
        };

        Bucket::new(&self.bucket_name, region, credentials)
            .map_err(|e| format!("Failed to initialize MinIO bucket context: {}", e))
            .map(|b| b.with_path_style())
    }

    /// Normalize a local disk path prefix to a MinIO object key prefix.
    ///
    /// Strips `assets/uploads/` or `/assets/uploads/` from the front so
    /// that the object key tree mirrors the gallery sub-directory.
    fn normalize_dir<'a>(&self, dir: &'a str) -> &'a str {
        crate::utils::storage_paths::normalize_key(dir, &self.root_prefix)
    }

    /// Upload a file from local disk to MinIO and register it in `uploaded_files`.
    ///
    /// * `disk_path`   — absolute or relative path to the local file.
    /// * `target_dir`  — destination directory (may include `assets/uploads/` prefix).
    /// * `filename`    — object filename within `target_dir`.
    /// * `mime_type`   — MIME type for the Content-Type header.
    ///
    /// On success, inserts a row into `uploaded_files` automatically.
    pub async fn upload(
        &self,
        disk_path: &str,
        target_dir: &str,
        filename: &str,
        mime_type: &str,
        parent_id: Option<Uuid>,
    ) -> Result<(), String> {
        let bucket = self.bucket()?;

        let data = tokio::fs::read(disk_path)
            .await
            .map_err(|e| format!("Failed to read local file {}: {}", disk_path, e))?;

        let clean_dir = self.normalize_dir(target_dir);
        let key = format!("{}{}", clean_dir, filename);

        let response = bucket
            .put_object_with_content_type(&key, &data, mime_type)
            .await
            .map_err(|e| format!("Failed to put object in MinIO bucket: {}", e))?;

        if response.status_code() != 200 {
            return Err(format!(
                "MinIO responded with non-200 status: {} (body: {})",
                response.status_code(),
                String::from_utf8_lossy(response.bytes())
            ));
        }

        tracing::info!(
            "MinIO: uploaded {} → bucket={} key={}",
            disk_path,
            self.bucket_name,
            key
        );

        // Auto-register in `uploaded_files` table. Failure is logged but does
        // not fail the upload — the object is already in MinIO at this point.
        let file_size = data.len() as i64;
        if let Err(e) = uploaded_files_repository::insert_upload(
            &self.pool,
            &self.bucket_name,
            &key,
            filename,
            file_size,
            mime_type,
            None,  // uploaded_by: None (caller can pass user_id in a future iteration)
            parent_id,
        ).await {
            tracing::warn!("MinIO: upload succeeded but failed to register in uploaded_files: key={} err={}", key, e);
        }

        Ok(())
    }

    /// Pre-register a file upload in `uploaded_files` with `processing` status.
    pub async fn insert_processing_record(
        &self,
        id: Uuid,
        target_dir: &str,
        filename: &str,
        mime_type: &str,
        uploaded_by: Option<Uuid>,
        parent_id: Option<Uuid>,
    ) -> Result<(), String> {
        let clean_dir = self.normalize_dir(target_dir);
        let key = format!("{}{}", clean_dir, filename);
        
        uploaded_files_repository::insert_upload_with_status(
            &self.pool,
            id,
            &self.bucket_name,
            &key,
            filename,
            0,
            mime_type,
            uploaded_by,
            "processing",
            parent_id,
        )
        .await
        .map_err(|e| format!("Failed to insert processing record: {}", e))?;

        Ok(())
    }

    /// Update status of an upload.
    pub async fn update_upload_status(
        &self,
        id: Uuid,
        status: &str,
        error_message: Option<&str>,
        file_size: Option<i64>,
    ) -> Result<(), String> {
        uploaded_files_repository::update_status(&self.pool, id, status, error_message, file_size)
            .await
            .map_err(|e| format!("Failed to update upload status: {}", e))?;

        Ok(())
    }

    /// Delete a single object from MinIO by its raw key.
    ///
    /// The key may optionally include an `assets/uploads/` prefix — it will be
    /// stripped automatically for consistency with [`upload`].
    pub async fn delete(&self, key: &str) -> Result<(), String> {
        let bucket = self.bucket()?;

        let clean_key = crate::utils::storage_paths::normalize_key(key, &self.root_prefix);

        let response = bucket
            .delete_object(clean_key)
            .await
            .map_err(|e| format!("Failed to delete object from MinIO: {}", e))?;

        if response.status_code() != 204 && response.status_code() != 200 {
            return Err(format!(
                "MinIO responded with non-204/200 status: {} (body: {})",
                response.status_code(),
                String::from_utf8_lossy(response.bytes())
            ));
        }

        tracing::info!(
            "MinIO: deleted key={} from bucket={}",
            clean_key,
            self.bucket_name
        );

        // Auto-remove from `uploaded_files` registry. Failure is logged but does
        // not fail the delete — the object is already gone from MinIO.
        if let Err(e) = uploaded_files_repository::delete_by_key(&self.pool, clean_key).await {
            tracing::warn!("MinIO: delete succeeded but failed to remove from uploaded_files: key={} err={}", clean_key, e);
        }

        Ok(())
    }

    /// Delete all MinIO objects associated with a gallery item.
    ///
    /// For images this includes the original, thumb, card, medium, and large
    /// variants. For videos it includes the video file and thumbnail.
    pub async fn delete_gallery_item(&self, file_url: &str, media_type: &str) {
        let path_str = crate::utils::storage_paths::normalize_key(file_url, &self.root_prefix);

        let keys_to_delete: Vec<String> = if let Some(dot_idx) = path_str.rfind('.') {
            let base_key = &path_str[..dot_idx];
            let ext = &path_str[dot_idx..];

            if media_type == "video" {
                vec![
                    path_str.to_string(),
                    format!("{}_thumb.webp", base_key),
                ]
            } else {
                let mut keys = vec![
                    format!("{}{}", base_key, ext),
                    format!("{}_thumb.webp", base_key),
                    format!("{}_card.webp", base_key),
                    format!("{}_medium.webp", base_key),
                    format!("{}_large.webp", base_key),
                ];
                if ext != ".webp" {
                    keys.push(format!("{}.webp", base_key));
                }
                keys
            }
        } else {
            vec![path_str.to_string()]
        };

        for key in keys_to_delete {
            if let Err(e) = self.delete(&key).await {
                tracing::warn!("MinIO: could not delete object '{}': {}", key, e);
            }
        }
    }
}
