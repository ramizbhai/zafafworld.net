use crate::errors::AppError;

/// Translates any given text to a unified English string.
/// Currently uses a mock implementation returning the original text prefixed with [EN].
#[allow(dead_code)]
pub async fn translate_to_english(text: &str) -> Result<String, AppError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }

    // Basic heuristic: check if string contains Arabic characters
    let is_arabic = trimmed
        .chars()
        .any(|c| ('\u{0600}'..='\u{06FF}').contains(&c));

    if is_arabic {
        tracing::info!("Mock translating Arabic input to English: '{}'", trimmed);

        let mock_translation = format!("[EN] {}", trimmed);

        Ok(mock_translation)
    } else {
        // If it's already English, return as is
        Ok(trimmed.to_string())
    }
}
