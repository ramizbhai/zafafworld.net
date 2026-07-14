use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

#[derive(Clone, Debug, serde::Serialize)]
pub struct ErrorDiagnostic {
    pub sql_error: Option<String>,
    pub filesystem_error: Option<String>,
    pub stack_trace: Option<String>,
}

#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub status: String,
    pub error_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    pub timestamp: String,
}

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    TooManyRequests(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    PaymentRequired(String, serde_json::Value),
    Database(String),
    Internal(String),
    Status(StatusCode, String),
    Validation(validator::ValidationErrors),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let mut diagnostic = ErrorDiagnostic {
            sql_error: None,
            filesystem_error: None,
            stack_trace: None,
        };

        // Capture stack trace for database and internal errors.
        // In release builds this is DISABLED — `Backtrace::force_capture()` calls
        // `backtrace::trace()` which acquires a global mutex and unwinds the stack,
        // adding ~500µs–5ms per error on every DB failure under load.
        // In debug/test builds the full backtrace is still captured for diagnostics.
        let capture_stack = matches!(&self, AppError::Database(_) | AppError::Internal(_));
        if capture_stack {
            #[cfg(debug_assertions)]
            {
                diagnostic.stack_trace =
                    Some(format!("{}", std::backtrace::Backtrace::force_capture()));
            }
        }

        match &self {
            AppError::Database(err) => {
                diagnostic.sql_error = Some(err.clone());
            }
            AppError::Internal(err)
                if (err.contains("IO error")
                    || err.contains("filesystem")
                    || err.contains("os error")
                    || err.contains("permission")) =>
            {
                diagnostic.filesystem_error = Some(err.clone());
            }
            _ => {}
        }

        let errors_val = match &self {
            AppError::Validation(errs) => {
                Some(serde_json::to_value(errs).unwrap_or(serde_json::json!({})))
            }
            _ => None,
        };

        let (status, error_type, message, meta) = match self {
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                "BAD_REQUEST".to_string(),
                msg,
                None,
            ),
            AppError::TooManyRequests(msg) => (
                StatusCode::TOO_MANY_REQUESTS,
                "TOO_MANY_REQUESTS".to_string(),
                msg,
                None,
            ),
            AppError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED".to_string(),
                msg,
                None,
            ),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, "FORBIDDEN".to_string(), msg, None),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND".to_string(), msg, None),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT".to_string(), msg, None),
            AppError::PaymentRequired(msg, meta) => (
                StatusCode::PAYMENT_REQUIRED,
                "PAYMENT_REQUIRED".to_string(),
                msg,
                Some(meta),
            ),
            AppError::Database(err) => {
                tracing::error!("Database error occurred: {}", err);
                if err.contains("Overlap conflict") {
                    (
                        StatusCode::CONFLICT,
                        "PROMOTION_OVERLAP".to_string(),
                        "One or more selected listings already have an overlapping promotion.".to_string(),
                        None,
                    )
                } else {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "DATABASE_ERROR".to_string(),
                        "A database operation failed to execute cleanly.".to_string(),
                        None,
                    )
                }
            }
            AppError::Internal(err) => {
                tracing::error!("Internal server error occurred: {}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_SERVER_ERROR".to_string(),
                    "An unexpected error occurred.".to_string(),
                    None,
                )
            }
            AppError::Status(code, msg) => {
                let error_type = match code {
                    StatusCode::TOO_MANY_REQUESTS => "TOO_MANY_REQUESTS".to_string(),
                    StatusCode::FORBIDDEN => "FORBIDDEN".to_string(),
                    StatusCode::UNAUTHORIZED => "UNAUTHORIZED".to_string(),
                    StatusCode::NOT_FOUND => "NOT_FOUND".to_string(),
                    StatusCode::CONFLICT => "CONFLICT".to_string(),
                    StatusCode::BAD_REQUEST => "BAD_REQUEST".to_string(),
                    _ => "ERROR".to_string(),
                };
                (code, error_type, msg, None)
            }
            AppError::Validation(_errs) => {
                crate::services::metrics::inc_validation_failed();
                (
                    StatusCode::BAD_REQUEST,
                    "VALIDATION_ERROR".to_string(),
                    "Validation failed".to_string(),
                    None,
                )
            }
        };

        // If validation errors exist, use them. Otherwise, use custom meta if present.
        let final_errors = errors_val.or(meta);

        // Compute optional code BEFORE moving error_type into ErrorResponse, so
        // we avoid the clone that was previously required by the borrow.
        let code = if error_type == "PROMOTION_OVERLAP" {
            Some("PROMOTION_OVERLAP".to_string())
        } else {
            None
        };

        let body = Json(ErrorResponse {
            status: "error".to_string(),
            error_type,  // moved — no clone
            code,
            message,
            errors: final_errors,
            request_id: None,  // Patched in by logging middleware
            timestamp: chrono::Utc::now().to_rfc3339(),
        });

        let mut response = (status, body).into_response();
        response.extensions_mut().insert(diagnostic);
        response
    }
}

// Implement standard conversions
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

impl From<bcrypt::BcryptError> for AppError {
    fn from(err: bcrypt::BcryptError) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        AppError::Validation(err)
    }
}

/// Derives a sanitized, user-facing conflict message from a PostgreSQL 23505 error message string.
/// PostgreSQL's `.message()` for unique violations reads:
///   'duplicate key value violates unique constraint "constraint_name"'
/// We match on constraint name fragments to produce domain-specific responses
/// without leaking raw internal constraint names or column values into API responses.
fn extract_conflict_detail(message: &str) -> String {
    let msg_lower = message.to_lowercase();
    if msg_lower.contains("email") {
        "An account with this email address is already registered.".to_string()
    } else if msg_lower.contains("booking_number") {
        "This booking reference already exists.".to_string()
    } else if msg_lower.contains("idx_unique_active_booking")
        || (msg_lower.contains("vendor_id") && msg_lower.contains("wedding_date"))
    {
        "This venue is already booked for the selected date.".to_string()
    } else if msg_lower.contains("idx_vendor_gallery_single_cover")
        || msg_lower.contains("is_cover")
    {
        "This vendor already has a cover image. Unset the existing cover before assigning a new one.".to_string()
    } else if msg_lower.contains("client_id") {
        "A record for this client already exists.".to_string()
    } else {
        "This record conflicts with an existing entry. Please review your input.".to_string()
    }
}

/// Maps a raw `sqlx::Error` to a domain-appropriate `AppError` by inspecting the
/// PostgreSQL SQLSTATE code before falling back to the generic `Database` variant.
///
/// SQLSTATE codes handled:
/// - `23505` — Unique constraint violation → `AppError::Conflict`
/// - `23503` — Foreign key violation → `AppError::BadRequest`
/// - `23514` — Check constraint violation → `AppError::BadRequest`
/// - `23502` — Not-null constraint violation → `AppError::BadRequest`
pub fn map_db_error(err: sqlx::Error) -> AppError {
    if let Some(db_err) = err.as_database_error() {
        match db_err.code().as_deref() {
            Some("23505") => {
                // .message() is on the DatabaseError trait; .detail() is postgres-specific only.
                // We derive the sanitized conflict description from the constraint message text.
                AppError::Conflict(extract_conflict_detail(db_err.message()))
            }
            Some("23503") => AppError::BadRequest(
                "A required reference does not exist. Please verify your input data.".to_string(),
            ),
            Some("23514") => AppError::BadRequest(format!(
                "The submitted value was rejected by a database integrity rule: {}",
                db_err.message()
            )),
            Some("23502") => AppError::BadRequest(
                "A required field was missing from the submission.".to_string(),
            ),
            _ => AppError::Database(err.to_string()),
        }
    } else {
        AppError::Database(err.to_string())
    }
}
