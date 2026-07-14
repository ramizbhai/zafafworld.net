//! Pure-Rust, zero-dependency string scrubbing engine.
//!
//! Provides a single entry-point `sanitize_str` that:
//!   1. Strips all angle-bracket HTML/script tag fragments from the input string.
//!   2. Enforces a hard character ceiling (not byte ceiling) via `max_len`.
//!
//! This module intentionally has no external crate dependencies. The character
//! scan loop is O(n) and allocates exactly one intermediate `String`.

/// Scrubs a free-text string of HTML/script injection vectors and enforces a
/// declarative character limit.
///
/// # Strategy
/// - Any sequence beginning with `<` and ending with `>` is silently dropped,
///   regardless of content. This neutralises `<script>`, `<img onerror=...>`,
///   `<svg onload=...>`, `<iframe src=...>`, and all similar tag patterns.
/// - After stripping, the result is truncated to `max_len` Unicode scalar values
///   (characters), not bytes. Truncation always lands on a clean char boundary.
///
/// # Arguments
/// * `input`   - Raw user-supplied string slice.
/// * `max_len` - Maximum number of Unicode characters to retain.
///
/// # Returns
/// A new, heap-allocated `String` that is safe to bind to a parameterized query.
pub fn sanitize_str(input: &str, max_len: usize) -> String {
    // Pre-allocate conservatively: we will never exceed min(input.len(), max_len).
    let reserve = input.len().min(max_len.saturating_mul(4)); // 4 bytes/char worst-case UTF-8
    let mut result = String::with_capacity(reserve);

    let mut in_tag = false;

    for ch in input.chars() {
        match ch {
            '<' => {
                // Begin tag suppression — discard this character and everything
                // until (and including) the matching '>'.
                in_tag = true;
            }
            '>' => {
                // Close tag suppression — discard this closing delimiter.
                in_tag = false;
            }
            _ if !in_tag => {
                result.push(ch);
            }
            _ => {
                // Inside tag body — silently discard.
            }
        }
    }

    // Enforce character ceiling on a clean Unicode scalar boundary.
    if result.chars().count() > max_len {
        result = result.chars().take(max_len).collect();
    }

    result
}

/// Convenience wrapper for `Option<String>` fields — sanitizes the inner value
/// if present, returning `None` if the original was `None`.
pub fn sanitize_opt(input: &Option<String>, max_len: usize) -> Option<String> {
    input.as_deref().map(|s| sanitize_str(s, max_len))
}

/// Declarative character-limit table for all free-text fields written to the
/// database. Adjust constants here to tighten or relax per-field limits without
/// modifying handler logic.
#[allow(dead_code)]
pub mod limits {
    /// Short human names: first_name, last_name, customer_name, staff name.
    pub const NAME_SHORT: usize = 100;

    /// Long corporate names: company name AR/EN, venue name AR/EN.
    pub const NAME_LONG: usize = 255;

    /// Email addresses.
    pub const EMAIL: usize = 255;

    /// Phone numbers — international format including country code and separators.
    pub const PHONE: usize = 30;

    /// Rich description fields: description_ar/en, amenities, address AR/EN.
    pub const DESCRIPTION: usize = 2_000;

    /// Free-text message body: inquiry message, special_requests.
    pub const MESSAGE: usize = 1_000;

    /// URL strings: image_url.
    pub const URL: usize = 512;

    /// Short caption text: gallery caption.
    pub const CAPTION: usize = 255;

    /// WhatsApp template body text.
    pub const TEMPLATE_BODY: usize = 2_000;

    /// Template name, offer title, package name (short label fields).
    pub const LABEL: usize = 255;

    /// URL slug strings: city_slug, vendor slug.
    pub const SLUG: usize = 100;

    /// Taxonomy strings: category, event_type.
    pub const CATEGORY: usize = 100;

    /// Each individual tag string inside a `Vec<String>` tag array.
    pub const TAG_ITEM: usize = 50;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_script_tag() {
        let input = "Hello <script>alert('xss')</script> World";
        assert_eq!(sanitize_str(input, 1000), "Hello alert('xss') World");
    }

    #[test]
    fn strips_img_onerror() {
        let input = r#"Nice photo <img src=x onerror="steal()"> caption"#;
        assert_eq!(sanitize_str(input, 1000), "Nice photo  caption");
    }

    #[test]
    fn truncates_at_char_boundary() {
        let input = "abcdefghij";
        assert_eq!(sanitize_str(input, 5), "abcde");
    }

    #[test]
    fn handles_empty_string() {
        assert_eq!(sanitize_str("", 100), "");
    }

    #[test]
    fn sanitize_opt_none() {
        let input: Option<String> = None;
        assert_eq!(sanitize_opt(&input, 100), None);
    }

    #[test]
    fn sanitize_opt_some() {
        let input = Some("<b>bold</b>".to_string());
        assert_eq!(sanitize_opt(&input, 100), Some("bold".to_string()));
    }

    #[test]
    fn strips_nested_tags() {
        let input = "A <div class='x'><span>inner</span></div> B";
        assert_eq!(sanitize_str(input, 1000), "A inner B");
    }
}
