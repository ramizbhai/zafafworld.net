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

    pub fn pool(&self) -> &PgPool {
        &self.pool
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

    /// Perform a deep health check of MinIO object storage.
    ///
    /// Validates service availability, bucket accessibility, write, read, and delete capability.
    pub async fn health_check(&self) -> Result<(), String> {
        let bucket = self.bucket()?;
        let test_key = format!("{}.health_probe", self.normalize_dir(""));

        // 1. Write probe
        let put_res = bucket
            .put_object_with_content_type(&test_key, b"probe", "text/plain")
            .await
            .map_err(|e| format!("Storage write probe failed: {}", e))?;

        if put_res.status_code() != 200 {
            return Err(format!("Storage write probe returned HTTP {}", put_res.status_code()));
        }

        // 2. Read probe
        let get_res = bucket
            .get_object(&test_key)
            .await
            .map_err(|e| format!("Storage read probe failed: {}", e))?;

        if get_res.status_code() != 200 {
            let _ = bucket.delete_object(&test_key).await; // Best effort cleanup
            return Err(format!("Storage read probe returned HTTP {}", get_res.status_code()));
        }

        if &get_res.bytes()[..] != b"probe" {
            let _ = bucket.delete_object(&test_key).await;
            return Err("Storage read probe returned corrupted data".to_string());
        }

        // 3. Delete probe
        let del_res = bucket
            .delete_object(&test_key)
            .await
            .map_err(|e| format!("Storage delete probe failed: {}", e))?;

        if del_res.status_code() != 204 && del_res.status_code() != 200 {
            return Err(format!("Storage delete probe returned HTTP {}", del_res.status_code()));
        }

        Ok(())
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
        width: Option<i32>,
        height: Option<i32>,
        checksum: Option<&str>,
    ) -> Result<(), String> {
        let clean_dir = self.normalize_dir(target_dir);
        let key = format!("{}{}", clean_dir, filename);
        let operation_id = Uuid::new_v4();

        tracing::info!(
            target: "storage",
            "Upload Started: operation_id={} filename={} bucket={} key={}",
            operation_id, filename, self.bucket_name, key
        );

        let bucket = self.bucket().map_err(|e| {
            tracing::error!(
                target: "storage",
                "Upload Failed: operation_id={} bucket={} key={} operation=get_bucket error={}",
                operation_id, self.bucket_name, key, e
            );
            e
        })?;

        let data = tokio::fs::read(disk_path)
            .await
            .map_err(|e| {
                let err_msg = format!("Failed to read local file {}: {}", disk_path, e);
                tracing::error!(
                    target: "storage",
                    "Upload Failed: operation_id={} bucket={} key={} operation=read_local_file error={}",
                    operation_id, self.bucket_name, key, err_msg
                );
                err_msg
            })?;

        tracing::info!(
            target: "storage",
            "Upload Completed (Local Read): operation_id={} filename={} file_size={} mime_type={}",
            operation_id, filename, data.len(), mime_type
        );

        let response = bucket
            .put_object_with_content_type(&key, &data, mime_type)
            .await
            .map_err(|e| {
                let err_msg = format!("Failed to put object in MinIO bucket: {}", e);
                tracing::error!(
                    target: "storage",
                    "Upload Failed: operation_id={} bucket={} key={} operation=s3_put error={}",
                    operation_id, self.bucket_name, key, err_msg
                );
                err_msg
            })?;

        if response.status_code() != 200 {
            let err_msg = format!(
                "MinIO responded with non-200 status: {} (body: {})",
                response.status_code(),
                String::from_utf8_lossy(response.bytes())
            );
            tracing::error!(
                target: "storage",
                "Upload Failed: operation_id={} bucket={} key={} operation=s3_put_status error={}",
                operation_id, self.bucket_name, key, err_msg
            );
            return Err(err_msg);
        }

        tracing::info!(
            target: "storage",
            "Object Stored: operation_id={} bucket={} key={} file_size={}",
            operation_id, self.bucket_name, key, data.len()
        );

        // Auto-register in `uploaded_files` table. Failure is logged but does
        // not fail the upload — the object is already in MinIO at this point.
        let file_size = data.len() as i64;
        match uploaded_files_repository::insert_upload(
            &self.pool,
            &self.bucket_name,
            &key,
            filename,
            file_size,
            mime_type,
            None,  // uploaded_by
            parent_id,
            width,
            height,
            checksum,
        ).await {
            Ok(db_id) => {
                tracing::info!(
                    target: "storage",
                    "Metadata Updated: operation_id={} id={} bucket={} key={} status=ready",
                    operation_id, db_id, self.bucket_name, key
                );
            }
            Err(e) => {
                tracing::warn!(
                    target: "storage",
                    "MinIO: upload succeeded but failed to register in uploaded_files: key={} err={}",
                    key, e
                );
            }
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
            None,
            None,
            None,
        )
        .await
        .map_err(|e| {
            tracing::error!(
                target: "storage",
                "Metadata Update Failed: id={} bucket={} key={} operation=insert_processing error={}",
                id, self.bucket_name, key, e
            );
            format!("Failed to insert processing record: {}", e)
        })?;

        tracing::info!(
            target: "storage",
            "Metadata Updated: id={} bucket={} key={} status=processing",
            id, self.bucket_name, key
        );

        Ok(())
    }

    /// Update status of an upload.
    pub async fn update_upload_status(
        &self,
        id: Uuid,
        status: &str,
        error_message: Option<&str>,
        file_size: Option<i64>,
        width: Option<i32>,
        height: Option<i32>,
        duration_seconds: Option<i32>,
        codec: Option<&str>,
        bitrate: Option<i64>,
        orientation: Option<i32>,
        checksum: Option<&str>,
    ) -> Result<(), String> {
        uploaded_files_repository::update_status(
            &self.pool,
            id,
            status,
            error_message,
            file_size,
            width,
            height,
            duration_seconds,
            codec,
            bitrate,
            orientation,
            checksum,
        )
        .await
        .map_err(|e| {
            tracing::error!(
                target: "storage",
                "Metadata Update Failed: id={} status={} operation=update_status error={}",
                id, status, e
            );
            format!("Failed to update upload status: {}", e)
        })?;

        tracing::info!(
            target: "storage",
            "Metadata Updated: id={} status={}",
            id, status
        );

        Ok(())
    }

    /// Delete a single object from MinIO by its raw key.
    ///
    /// The key may optionally include an `assets/uploads/` prefix — it will be
    /// stripped automatically for consistency with [`upload`].
    pub async fn delete(&self, key: &str) -> Result<(), String> {
        let clean_key = crate::utils::storage_paths::normalize_key(key, &self.root_prefix);
        let operation_id = Uuid::new_v4();

        let bucket = self.bucket().map_err(|e| {
            tracing::error!(
                target: "storage",
                "Delete Failed: operation_id={} bucket={} key={} operation=get_bucket error={}",
                operation_id, self.bucket_name, clean_key, e
            );
            e
        })?;

        let response = bucket
            .delete_object(clean_key)
            .await
            .map_err(|e| {
                let err_msg = format!("Failed to delete object from MinIO: {}", e);
                tracing::error!(
                    target: "storage",
                    "Delete Failed: operation_id={} bucket={} key={} operation=s3_delete error={}",
                    operation_id, self.bucket_name, clean_key, err_msg
                );
                err_msg
            })?;

        if response.status_code() != 204 && response.status_code() != 200 {
            let err_msg = format!(
                "MinIO responded with non-204/200 status: {} (body: {})",
                response.status_code(),
                String::from_utf8_lossy(response.bytes())
            );
            tracing::error!(
                target: "storage",
                "Delete Failed: operation_id={} bucket={} key={} operation=s3_delete_status error={}",
                operation_id, self.bucket_name, clean_key, err_msg
            );
            return Err(err_msg);
        }

        tracing::info!(
            target: "storage",
            "Object Deleted: operation_id={} bucket={} key={}",
            operation_id, self.bucket_name, clean_key
        );

        // Auto-remove from `uploaded_files` registry. Failure is logged but does
        // not fail the delete — the object is already gone from MinIO.
        if let Err(e) = uploaded_files_repository::delete_by_key(&self.pool, clean_key).await {
            tracing::warn!(
                target: "storage",
                "MinIO: delete succeeded but failed to remove from uploaded_files: key={} err={}",
                clean_key, e
            );
        } else {
            tracing::info!(
                target: "storage",
                "Metadata Updated (Delete): operation_id={} bucket={} key={} status=deleted",
                operation_id, self.bucket_name, clean_key
            );
        }

        Ok(())
    }

    /// Delete all MinIO objects associated with a gallery item.
    ///
    /// For images this includes the original, thumb, card, medium, and large
    /// variants. For videos it includes the video file and thumbnail.
    pub async fn delete_gallery_item(&self, file_url: &str, media_type: &str) {
        use sqlx::Row;
        let path_str = crate::utils::storage_paths::normalize_key(file_url, &self.root_prefix).to_string();
        let mut keys_to_delete = vec![path_str.clone()];

        // Query both uploaded_files and uploaded_file_variants for this key's ID
        if let Ok(Some(row)) = sqlx::query("SELECT id FROM public.uploaded_files WHERE object_key = $1")
            .bind(&path_str)
            .fetch_optional(&self.pool)
            .await
        {
            if let Ok(parent_id) = row.try_get::<Uuid, _>("id") {
                // Find all variant keys
                if let Ok(variants) = sqlx::query("SELECT object_key FROM public.uploaded_file_variants WHERE uploaded_file_id = $1")
                    .bind(parent_id)
                    .fetch_all(&self.pool)
                    .await
                {
                    for var in variants {
                        if let Ok(k) = var.try_get::<String, _>("object_key") {
                            keys_to_delete.push(k);
                        }
                    }
                }
            }
        }

        // Fallback checks if db is out-of-sync or empty
        if keys_to_delete.len() == 1 {
            if let Some(dot_idx) = path_str.rfind('.') {
                let base_key = &path_str[..dot_idx];
                let ext = &path_str[dot_idx..];

                if media_type == "video" {
                    keys_to_delete.push(format!("{}_thumb.webp", base_key));
                    keys_to_delete.push(format!("{}_thumb.avif", base_key));
                    keys_to_delete.push(format!("{}_poster.webp", base_key));
                    keys_to_delete.push(format!("{}_poster.avif", base_key));
                    keys_to_delete.push(format!("{}_master.m3u8", base_key));
                    keys_to_delete.push(format!("{}_1080p.m3u8", base_key));
                    keys_to_delete.push(format!("{}_720p.m3u8", base_key));
                    keys_to_delete.push(format!("{}_480p.m3u8", base_key));
                } else {
                    keys_to_delete.push(format!("{}_thumb.webp", base_key));
                    keys_to_delete.push(format!("{}_card.webp", base_key));
                    keys_to_delete.push(format!("{}_medium.webp", base_key));
                    keys_to_delete.push(format!("{}_large.webp", base_key));
                    if ext != ".webp" {
                        keys_to_delete.push(format!("{}.webp", base_key));
                    }
                }
            }
        }

        for key in keys_to_delete {
            if let Err(e) = self.delete(&key).await {
                tracing::warn!("MinIO: could not delete object '{}': {}", key, e);
            }
        }
    }
}
