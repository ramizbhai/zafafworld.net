use sqlx::{Postgres, Transaction};
use uuid::Uuid;
use crate::errors::AppError;
use crate::services::media::minio_client::MinioClient;
use tracing::{info, warn};

pub struct StorageDeletionService;

impl StorageDeletionService {
    /// Collects file references from vendor_gallery and listing_promotions, 
    /// and enqueues them in the database transaction.
    pub async fn queue_deletion(
        tx: &mut Transaction<'_, Postgres>,
        product_id: Uuid,
        vendor_id: Uuid,
    ) -> Result<(), AppError> {
        // 1. Gather all file_ids, file_paths and image_urls from vendor_gallery
        let gallery_records: Vec<(Option<Uuid>, Option<String>, String)> = sqlx::query_as(
            "SELECT file_id, file_path, image_url 
             FROM public.vendor_gallery 
             WHERE product_id = $1 AND vendor_id = $2"
        )
        .bind(product_id)
        .bind(vendor_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        // 2. Gather custom banner file details from listing_promotions
        let promo_records: Vec<(Option<Uuid>, Option<String>)> = sqlx::query_as(
            "SELECT file_id, custom_banner_image_url 
             FROM public.listing_promotions 
             WHERE listing_id = $1 AND vendor_id = $2"
        )
        .bind(product_id)
        .bind(vendor_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        // Extract original file_ids and URLs/paths
        let mut original_file_ids: Vec<Uuid> = Vec::new();
        let mut local_paths: Vec<String> = Vec::new();

        for (file_id, file_path, _image_url) in gallery_records {
            if let Some(fid) = file_id {
                original_file_ids.push(fid);
            }
            if let Some(path) = file_path {
                local_paths.push(path);
            }
        }

        for (file_id, _banner_url) in promo_records {
            if let Some(fid) = file_id {
                original_file_ids.push(fid);
            }
        }

        if original_file_ids.is_empty() {
            // Nothing to queue for deletion
            return Ok(());
        }

        // 3. Query public.uploaded_files using parent_id/id relations to find all exact registry entries!
        let registry_files: Vec<(Uuid, String)> = sqlx::query_as(
            "SELECT id, object_key 
             FROM public.uploaded_files 
             WHERE id = ANY($1) OR parent_id = ANY($1)"
        )
        .bind(&original_file_ids)
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let mut file_ids_to_delete: Vec<Uuid> = Vec::new();
        let mut object_keys: Vec<String> = Vec::new();
        for (fid, key) in registry_files {
            file_ids_to_delete.push(fid);
            object_keys.push(key);
        }

        if file_ids_to_delete.is_empty() {
            return Ok(());
        }

        // 4. Enqueue the deletion task in the database
        sqlx::query(
            "INSERT INTO public.storage_deletion_queue (file_ids, object_keys, local_paths, status)
             VALUES ($1, $2, $3, 'pending')"
        )
        .bind(&file_ids_to_delete)
        .bind(&object_keys)
        .bind(&local_paths)
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Executes the physical storage and local disk deletion.
    pub async fn execute_cleanup(
        minio_client: &MinioClient,
        keys: &[String],
        local_paths: &[String],
    ) -> Result<(), String> {
        let bucket = minio_client.bucket()?;

        let mut tasks = Vec::new();
        for key in keys {
            let bucket = bucket.clone();
            let key = key.clone();
            tasks.push(async move {
                if let Err(e) = bucket.delete_object(&key).await {
                    warn!("MinIO: failed to delete key {}: {}", key, e);
                    Err(e.to_string())
                } else {
                    info!("MinIO: successfully deleted key {}", key);
                    Ok(())
                }
            });
        }

        let minio_delete_results = futures_util::future::join_all(tasks).await;

        // Check if there are any errors in S3 deletions
        let s3_errors: Vec<String> = minio_delete_results
            .into_iter()
            .filter_map(|r| r.err())
            .collect();

        if !s3_errors.is_empty() {
            return Err(format!("Some S3 deletions failed: {:?}", s3_errors));
        }

        // Clean up disk paths
        for path in local_paths {
            if let Err(e) = tokio::fs::remove_file(path).await {
                if e.kind() != std::io::ErrorKind::NotFound {
                    warn!("Disk: failed to delete file {}: {}", path, e);
                }
            } else {
                info!("Disk: successfully deleted file {}", path);
            }
        }

        Ok(())
    }
}
