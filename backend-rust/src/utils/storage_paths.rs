use uuid::Uuid;

/// Normalizes the prefix by trimming leading and trailing slashes.
/// Example: "assets/uploads/" or "/assets/uploads" becomes "assets/uploads".
pub fn clean_prefix(prefix: &str) -> String {
    prefix.trim_matches('/').to_string()
}

/// Builds the local disk directory and prefix paths.
/// Returns (hierarchical_dir, hierarchical_prefix).
/// Example: ("assets/uploads/gallery/uuid/date/", "/assets/uploads/gallery/uuid/date/")
pub fn build_hierarchical_paths(root_prefix: &str, entity_type: &str, temp_id: &Uuid, date_str: &str) -> (String, String) {
    let clean = clean_prefix(root_prefix);
    let dir = format!("{}/{}/{}/{}/", clean, entity_type, temp_id, date_str);
    let prefix = format!("/{}/{}/{}/{}/", clean, entity_type, temp_id, date_str);
    (dir, prefix)
}

/// Normalizes MinIO/storage paths by stripping root prefixes (both dynamic and legacy).
pub fn normalize_key<'a>(key: &'a str, root_prefix: &str) -> &'a str {
    let clean_dynamic = clean_prefix(root_prefix);
    let clean_dynamic_with_slash = format!("{}/", clean_dynamic);
    
    key.strip_prefix(&clean_dynamic_with_slash)
        .or_else(|| key.strip_prefix("assets/uploads/"))
        .or_else(|| key.strip_prefix("/assets/uploads/"))
        .unwrap_or(key)
}

/// Security check: checks if a URL starts with the expected relative upload prefix (dynamic or legacy).
pub fn is_relative_upload(url: &str, root_prefix: &str) -> bool {
    let clean_dynamic = clean_prefix(root_prefix);
    let dynamic_prefix = format!("/{}/", clean_dynamic);
    
    url.starts_with(&dynamic_prefix)
        || url.starts_with("/assets/uploads/")
        || url.starts_with("/uploads/")
}
