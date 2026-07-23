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
) -> Result<Uuid, sqlx::Error> {
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO public.uploaded_files
            (bucket_name, object_key, file_name, file_size, mime_type, uploaded_by, status, parent_id)
        VALUES ($1, $2, $3, $4, $5, $6, 'ready', $7)
        ON CONFLICT (object_key) DO UPDATE
            SET file_size   = EXCLUDED.file_size,
                mime_type   = EXCLUDED.mime_type,
                file_name   = EXCLUDED.file_name,
                status      = 'ready',
                parent_id   = COALESCE(EXCLUDED.parent_id, public.uploaded_files.parent_id)
        RETURNING id
        "#,
    )
    .bind(bucket_name)
    .bind(object_key)
    .bind(file_name)
    .bind(file_size)
    .bind(mime_type)
    .bind(uploaded_by)
    .bind(parent_id)
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
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO public.uploaded_files
            (id, bucket_name, object_key, file_name, file_size, mime_type, uploaded_by, status, parent_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT (object_key) DO UPDATE
            SET file_size   = EXCLUDED.file_size,
                mime_type   = EXCLUDED.mime_type,
                file_name   = EXCLUDED.file_name,
                status      = EXCLUDED.status,
                parent_id   = COALESCE(EXCLUDED.parent_id, public.uploaded_files.parent_id)
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
    .bind(parent_id)
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
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE public.uploaded_files
        SET status = $2,
            error_message = $3,
            file_size = COALESCE($4, file_size)
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(status)
    .bind(error_message)
    .bind(file_size)
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
