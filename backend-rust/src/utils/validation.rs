use crate::errors::AppError;

/// Centralized password validator implementing **Policy C (Adaptive Complexity)**.
///
/// Validates incoming user password payloads across all registration, reset, and creation endpoints.
/// Enforces strict byte-based limits to prevent silent truncation collisions in Bcrypt, while
/// allowing passphrase usability for highly secure passwords.
///
/// Limits:
/// * **Minimum Length:** 8 bytes.
/// * **Maximum Length:** 72 bytes.
/// * **Adaptive Complexity:**
///   * If the password is between **8 and 11 bytes**: Requires at least one lowercase letter,
///     one uppercase letter, and one digit or special character.
///   * If the password is **12 bytes or longer**: Waives character complexity rules to encourage
///     passphrases (e.g. `correct horse battery staple`).
pub fn validate_password(password: &str) -> Result<(), AppError> {
    let bytes_len = password.len();

    // Strict boundaries corresponding to Bcrypt blowfish block sizes
    if bytes_len < 8 {
        return Err(AppError::BadRequest(
            "Password must be at least 8 bytes long".to_string(),
        ));
    }
    if bytes_len > 72 {
        return Err(AppError::BadRequest(
            "Password must not exceed 72 bytes".to_string(),
        ));
    }

    // Adaptive Complexity: enforced only for shorter keys (8-11 bytes)
    if bytes_len < 12 {
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_uppercase = password.chars().any(|c| c.is_uppercase());

        // JUSTIFICATION FOR is_numeric() OVER is_ascii_digit():
        // ZafafWorld is a bilingual platform serving English and Arabic speaking markets.
        // `is_numeric()` checks any Unicode numeric digit (including Arabic-Indic numerals '١', '٢', '٣'),
        // whereas `is_ascii_digit()` is restricted to '0'-'9'.
        // Using `is_numeric()` prevents legitimate internationalized/Arabic characters from failing
        // the complexity checks, supporting localized secure passwords without friction.
        let has_digit = password
            .chars()
            .any(|c| c.is_numeric() || c.is_ascii_punctuation());

        if !has_lowercase || !has_uppercase || !has_digit {
            return Err(AppError::BadRequest(
                "Passwords between 8 and 11 bytes must contain at least one lowercase letter, one uppercase letter, and one digit or special character.".to_string()
            ));
        }
    }

    Ok(())
}

use axum::{
    async_trait,
    extract::{FromRequest, Request},
    Json,
};
use serde::de::DeserializeOwned;
use validator::Validate;

pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|err| AppError::BadRequest(format!("Failed to deserialize request body: {}", err)))?;

        value.validate().map_err(AppError::Validation)?;

        Ok(ValidatedJson(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_password_length_limits() {
        assert!(validate_password("short").is_err());
        assert!(validate_password(&"a".repeat(73)).is_err());
        assert!(validate_password("Pass1234").is_ok());
    }

    #[test]
    fn test_validate_password_adaptive_complexity() {
        // Short password (8-11 bytes) needs lowercase, uppercase, digit/special
        assert!(validate_password("lowercase").is_err());
        assert!(validate_password("UPPERCASE").is_err());
        assert!(validate_password("LowerUpper").is_err());
        assert!(validate_password("LowerUpper1").is_ok());

        // Long passphrase (>= 12 bytes) waives complexity
        assert!(validate_password("longpassphrase").is_ok());
    }
}
