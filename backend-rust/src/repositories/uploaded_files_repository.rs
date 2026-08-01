use sqlx::PgPool;
use uuid::Uuid;

pub struct UploadedFileDto {
    pub id: Uuid,
    #[allow(dead_code)]
    pub bucket_name: String,
    pub object_key: String,
    pub file_name: String,
    pub file_size: i64,
    pub mime_type: String,
    pub status: String,
    pub error_message: Option<String>,
}

/// Retrieve an upload record by its ID.
pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Option<UploadedFileDto>, sqlx::Error> {
    let row_opt = sqlx::query(
        r#"
        SELECT id, bucket_name, object_key, file_name, file_size, mime_type, status, error_message
        FROM public.uploaded_files
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row_opt {
        use sqlx::Row;
        Ok(Some(UploadedFileDto {
            id: row.get("id"),
            bucket_name: row.get("bucket_name"),
            object_key: row.get("object_key"),
            file_name: row.get("file_name"),
            file_size: row.get("file_size"),
            mime_type: row.get("mime_type"),
            status: row.get("status"),
            error_message: row.get("error_message"),
        }))
    } else {
        Ok(None)
    }
}

/// Insert a new variant row into `uploaded_file_variants`.
pub async fn insert_variant(
    pool: &PgPool,
    uploaded_file_id: Uuid,
    format: &str,
    variant: &str,
    size_bytes: i64,
    object_key: &str,
    width: Option<i32>,
    height: Option<i32>,
    checksum: Option<&str>,
) -> Result<Uuid, sqlx::Error> {
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO public.uploaded_file_variants
            (uploaded_file_id, format, variant, size_bytes, object_key, width, height, checksum)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (object_key) DO UPDATE
            SET size_bytes = EXCLUDED.size_bytes,
                width      = COALESCE(EXCLUDED.width, uploaded_file_variants.width),
                height     = COALESCE(EXCLUDED.height, uploaded_file_variants.height),
                checksum   = COALESCE(EXCLUDED.checksum, uploaded_file_variants.checksum)
        RETURNING id
        "#
    )
    .bind(uploaded_file_id)
    .bind(format)
    .bind(variant)
    .bind(size_bytes)
    .bind(object_key)
    .bind(width)
    .bind(height)
    .bind(checksum)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

/// Insert a new `uploaded_files` row after a successful MinIO put.
///
/// Returns the newly-created row ID.
pub async fn insert_upload(
    pool: &PgPool,
    bucket_name: &str,
    object_key: &str,
    file_name: &str,
    file_size: i64,
    mime_type: &str,
    uploaded_by: Option<Uuid>,
    parent_id: Option<Uuid>,
    width: Option<i32>,
    height: Option<i32>,
    checksum: Option<&str>,
) -> Result<Uuid, sqlx::Error> {
    if let Some(pid) = parent_id {
        let format = if mime_type == "image/avif" {
            "avif"
        } else if mime_type == "image/webp" {
            "webp"
        } else if mime_type == "application/x-mpegURL" || mime_type == "application/vnd.apple.mpegurl" || file_name.ends_with(".m3u8") {
            "hls"
        } else if mime_type == "video/mp4" || mime_type == "video/iso.segment" || file_name.ends_with(".m4s") || file_name.ends_with(".mp4") {
            "fmp4"
        } else {
            "raw"
        };

        let variant = if file_name.contains("_thumb.") {
            "thumb"
        } else if file_name.contains("_poster.") {
            "poster"
        } else if file_name.contains("_card.") {
            "card"
        } else if file_name.contains("_medium.") {
            "medium"
        } else if file_name.contains("_large.") {
            "large"
        } else if file_name.contains("_raw.") {
            "raw"
        } else if file_name.contains("_master.") {
            "master"
        } else if file_name.contains("_1080p.") {
            "1080p"
        } else if file_name.contains("_720p.") {
            "720p"
        } else if file_name.contains("_480p.") {
            "480p"
        } else if file_name.contains("_1080p_") || file_name.contains("_720p_") || file_name.contains("_480p_") || file_name.contains("_init") {
            "segment"
        } else {
            "original"
        };

        return insert_variant(pool, pid, format, variant, file_size, object_key, width, height, checksum).await;
    }

    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO public.uploaded_files
            (bucket_name, object_key, file_name, file_size, mime_type, uploaded_by, status, width, height, checksum)
        VALUES ($1, $2, $3, $4, $5, $6, 'ready', $7, $8, $9)
        ON CONFLICT (object_key) DO UPDATE
            SET file_size   = EXCLUDED.file_size,
                mime_type   = EXCLUDED.mime_type,
                file_name   = EXCLUDED.file_name,
                status      = 'ready',
                width       = COALESCE(EXCLUDED.width, uploaded_files.width),
                height      = COALESCE(EXCLUDED.height, uploaded_files.height),
                checksum    = COALESCE(EXCLUDED.checksum, uploaded_files.checksum)
        RETURNING id
        "#,
    )
    .bind(bucket_name)
    .bind(object_key)
    .bind(file_name)
    .bind(file_size)
    .bind(mime_type)
    .bind(uploaded_by)
    .bind(width)
    .bind(height)
    .bind(checksum)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

/// Insert a new `uploaded_files` row with a specific status and pre-determined ID.
pub async fn insert_upload_with_status(
    pool: &PgPool,
    id: Uuid,
    bucket_name: &str,
    object_key: &str,
    file_name: &str,
    file_size: i64,
    mime_type: &str,
    uploaded_by: Option<Uuid>,
    status: &str,
    parent_id: Option<Uuid>,
    width: Option<i32>,
    height: Option<i32>,
    checksum: Option<&str>,
) -> Result<(), sqlx::Error> {
    if let Some(pid) = parent_id {
        let format = if mime_type == "image/avif" {
            "avif"
        } else if mime_type == "image/webp" {
            "webp"
        } else if mime_type == "application/x-mpegURL" || mime_type == "application/vnd.apple.mpegurl" || file_name.ends_with(".m3u8") {
            "hls"
        } else if mime_type == "video/mp4" || mime_type == "video/iso.segment" || file_name.ends_with(".m4s") || file_name.ends_with(".mp4") {
            "fmp4"
        } else {
            "raw"
        };

        let variant = if file_name.contains("_thumb.") {
            "thumb"
        } else if file_name.contains("_poster.") {
            "poster"
        } else if file_name.contains("_card.") {
            "card"
        } else if file_name.contains("_medium.") {
            "medium"
        } else if file_name.contains("_large.") {
            "large"
        } else if file_name.contains("_raw.") {
            "raw"
        } else if file_name.contains("_master.") {
            "master"
        } else if file_name.contains("_1080p.") {
            "1080p"
        } else if file_name.contains("_720p.") {
            "720p"
        } else if file_name.contains("_480p.") {
            "480p"
        } else if file_name.contains("_1080p_") || file_name.contains("_720p_") || file_name.contains("_480p_") || file_name.contains("_init") {
            "segment"
        } else {
            "original"
        };

        let _ = insert_variant(pool, pid, format, variant, file_size, object_key, width, height, checksum).await?;
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO public.uploaded_files
            (id, bucket_name, object_key, file_name, file_size, mime_type, uploaded_by, status, width, height, checksum)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (object_key) DO UPDATE
            SET file_size   = EXCLUDED.file_size,
                mime_type   = EXCLUDED.mime_type,
                file_name   = EXCLUDED.file_name,
                status      = EXCLUDED.status,
                width       = COALESCE(EXCLUDED.width, uploaded_files.width),
                height      = COALESCE(EXCLUDED.height, uploaded_files.height),
                checksum    = COALESCE(EXCLUDED.checksum, uploaded_files.checksum)
        "#,
    )
    .bind(id)
    .bind(bucket_name)
    .bind(object_key)
    .bind(file_name)
    .bind(file_size)
    .bind(mime_type)
    .bind(uploaded_by)
    .bind(status)
    .bind(width)
    .bind(height)
    .bind(checksum)
    .execute(pool)
    .await?;

    Ok(())
}

/// Update status and optional columns of an existing upload.
pub async fn update_status(
    pool: &PgPool,
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
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE public.uploaded_files
        SET status = $2,
            error_message = $3,
            file_size = COALESCE($4, file_size),
            width = COALESCE($5, width),
            height = COALESCE($6, height),
            duration_seconds = COALESCE($7, duration_seconds),
            codec = COALESCE($8, codec),
            bitrate = COALESCE($9, bitrate),
            orientation = COALESCE($10, orientation),
            checksum = COALESCE($11, checksum)
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(status)
    .bind(error_message)
    .bind(file_size)
    .bind(width)
    .bind(height)
    .bind(duration_seconds)
    .bind(codec)
    .bind(bitrate)
    .bind(orientation)
    .bind(checksum)
    .execute(pool)
    .await?;

    Ok(())
}

/// Remove the `uploaded_files` row for a given object key.
///
/// Returns `true` if a row was deleted, `false` if the key was not found.
pub async fn delete_by_key(pool: &PgPool, object_key: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "DELETE FROM public.uploaded_files WHERE object_key = $1",
    )
    .bind(object_key)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}
