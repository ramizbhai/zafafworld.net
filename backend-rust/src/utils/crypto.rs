use crate::errors::AppError;

pub async fn hash_password(password: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || bcrypt::hash(&password, 12))
        .await
        .map_err(|err| AppError::Internal(format!("Crypto thread pool panic: {}", err)))?
        .map_err(|err| AppError::Internal(format!("Hash generation failure: {}", err)))
}

pub async fn verify_password(password: String, hash: String) -> Result<bool, AppError> {
    tokio::task::spawn_blocking(move || bcrypt::verify(&password, &hash))
        .await
        .map_err(|err| AppError::Internal(format!("Crypto thread pool panic: {}", err)))?
        .map_err(|err| AppError::Internal(format!("Hash verification failure: {}", err)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hash_and_verify_password() {
        let password = "SecretPassword123!".to_string();
        let hash = hash_password(password.clone()).await.unwrap(); println!("TEST_HASH={}", hash);
        assert!(!hash.is_empty());

        let valid = verify_password(password, hash.clone()).await.unwrap();
        assert!(valid);

        let invalid = verify_password("WrongPassword!".to_string(), hash)
            .await
            .unwrap();
        assert!(!invalid);
    }
}
