use std::net::IpAddr;
use tokio::net::lookup_host;
use reqwest::Url;
use chrono::{DateTime, Utc};
use crate::errors::AppError;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CachedLocation {
    pub original_url: String,
    pub resolved_url: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
}

// ─── IP SECURITY & SSRF PROTECTION ──────────────────────────────────────────

pub fn is_safe_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => {
            let octets = ipv4.octets();
            // Loopback: 127.0.0.0/8
            if octets[0] == 127 {
                return false;
            }
            // Private: 10.0.0.0/8
            if octets[0] == 10 {
                return false;
            }
            // Private: 172.16.0.0/12
            if octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31 {
                return false;
            }
            // Private: 192.168.0.0/16
            if octets[0] == 192 && octets[1] == 168 {
                return false;
            }
            // Link-local: 169.254.0.0/16
            if octets[0] == 169 && octets[1] == 254 {
                return false;
            }
            // Multicast: 224.0.0.0/4
            if octets[0] >= 224 && octets[0] <= 239 {
                return false;
            }
            true
        }
        IpAddr::V6(ipv6) => {
            if ipv6.is_loopback() {
                return false;
            }
            let segments = ipv6.segments();
            // Unique local: fc00::/7
            let first_byte = (segments[0] >> 8) as u8;
            if (first_byte & 0xfe) == 0xfc {
                return false;
            }
            // Link-local: fe80::/10
            if (segments[0] & 0xffc0) == 0xfe80 {
                return false;
            }
            true
        }
    }
}

pub async fn is_url_safe(url_str: &str) -> bool {
    let parsed = match Url::parse(url_str) {
        Ok(u) => u,
        Err(_) => return false,
    };

    let host = match parsed.host_str() {
        Some(h) => h,
        None => return false,
    };

    // Google domain whitelist
    let host_lower = host.to_lowercase();
    let is_whitelisted = host_lower.ends_with("google.com")
        || host_lower.ends_with("goo.gl")
        || host_lower == "google.com"
        || host_lower == "goo.gl"
        || host_lower.ends_with("google.com.sa");

    if !is_whitelisted {
        return false;
    }

    // Resolve domain IPs asynchronously
    let port = parsed.port().unwrap_or(443);
    let addr_str = format!("{}:{}", host, port);
    if let Ok(mut addrs) = lookup_host(&addr_str).await {
        while let Some(addr) = addrs.next() {
            if !is_safe_ip(addr.ip()) {
                return false;
            }
        }
    } else {
        return false;
    }

    true
}

// ─── REDIRECT RESOLVER & HEAD/GET HYBRID REQUEST ─────────────────────────────

pub async fn resolve_redirect_custom(start_url: &str) -> Result<String, AppError> {
    let mut current_url = start_url.to_string();
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none()) // Custom redirect following
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to build HTTP client: {}", e)))?;

    for hop in 0..3 {
        // SSRF check
        if !is_url_safe(&current_url).await {
            tracing::warn!("[RESOLVER] SSRF block: URL {} is not whitelisted or points to an unsafe IP", current_url);
            return Err(AppError::BadRequest("Unsafe or forbidden URL host".to_string()));
        }

        // Try HEAD first
        tracing::info!("[RESOLVER] Hop {}: Sending HEAD request to {}", hop, current_url);
        let response_result = client.head(&current_url).send().await;
        
        let response = match response_result {
            Ok(resp) => {
                tracing::info!("[RESOLVER] HEAD success (status: {})", resp.status());
                resp
            }
            Err(e) => {
                // HEAD failure fallback to GET for this hop
                tracing::warn!("[RESOLVER] HEAD request failed: {}. Falling back to GET.", e);
                client.get(&current_url).send().await.map_err(|err| {
                    if err.is_timeout() {
                        tracing::error!("[RESOLVER] Outbound request timeout reached on {}", current_url);
                        AppError::Internal("Request timeout".to_string())
                    } else {
                        AppError::BadRequest(format!("Request failed: {}", err))
                    }
                })?
            }
        };

        if response.status().is_redirection() {
            if let Some(location) = response.headers().get(reqwest::header::LOCATION) {
                let loc_str = location.to_str().map_err(|_| {
                    AppError::BadRequest("Invalid Location header format".to_string())
                })?;
                
                // Resolve relative path redirects relative to base
                let base = Url::parse(&current_url).map_err(|_| {
                    AppError::BadRequest("Invalid base URL".to_string())
                })?;
                let resolved_url = base.join(loc_str).map_err(|_| {
                    AppError::BadRequest("Failed to resolve redirect Location path".to_string())
                })?;
                
                current_url = resolved_url.to_string();
                continue;
            }
        }
        
        // Reached destination URL
        return Ok(current_url);
    }
    
    tracing::warn!("[RESOLVER] Max redirect limit of 3 hops exceeded for {}", start_url);
    Err(AppError::BadRequest("Max redirect limit exceeded".to_string()))
}

// ─── COORDINATE EXTRACTION ──────────────────────────────────────────────────

pub fn extract_coords(url_str: &str) -> (Option<f64>, Option<f64>) {
    let mut lat = None;
    let mut lng = None;

    // Pattern 1: @latitude,longitude
    if let Some(at_idx) = url_str.find('@') {
        let remainder = &url_str[at_idx + 1..];
        let parts: Vec<&str> = remainder.split(',').collect();
        if parts.len() >= 2 {
            if let (Ok(parsed_lat), Ok(parsed_lng)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                if parsed_lat >= -90.0 && parsed_lat <= 90.0 && parsed_lng >= -180.0 && parsed_lng <= 180.0 {
                    lat = Some(parsed_lat);
                    lng = Some(parsed_lng);
                }
            }
        }
    }

    // Pattern 2: q=latitude,longitude or ll=latitude,longitude query string parameters
    if lat.is_none() {
        if let Ok(parsed_url) = Url::parse(url_str) {
            for (key, val) in parsed_url.query_pairs() {
                if key == "q" || key == "ll" {
                    let parts: Vec<&str> = val.split(',').collect();
                    if parts.len() >= 2 {
                        if let (Ok(parsed_lat), Ok(parsed_lng)) = (parts[0].trim().parse::<f64>(), parts[1].trim().parse::<f64>()) {
                            if parsed_lat >= -90.0 && parsed_lat <= 90.0 && parsed_lng >= -180.0 && parsed_lng <= 180.0 {
                                lat = Some(parsed_lat);
                                lng = Some(parsed_lng);
                            }
                        }
                    }
                }
            }
        }
    }

    (lat, lng)
}

// ─── CONCURRENT CACHING & DEDUPLICATION ──────────────────────────────────────

pub async fn resolve_location_with_cache(
    url_str: &str,
    app_state: &crate::state::AppState,
) -> Result<CachedLocation, AppError> {
    resolve_location_with_cache_impl(
        url_str,
        &app_state.location_cache,
        &app_state.active_location_requests,
    )
    .await
}

pub async fn resolve_location_with_cache_impl(
    url_str: &str,
    location_cache: &dashmap::DashMap<String, CachedLocation>,
    active_location_requests: &dashmap::DashMap<
        String,
        tokio::sync::broadcast::Sender<Result<CachedLocation, String>>,
    >,
) -> Result<CachedLocation, AppError> {
    let url_trimmed = url_str.trim().to_string();
    if url_trimmed.is_empty() {
        tracing::warn!("[RESOLVER] Received empty URL for resolution");
        return Err(AppError::BadRequest("URL cannot be empty".to_string()));
    }

    // Check SSRF format first
    if !is_url_safe(&url_trimmed).await {
        tracing::warn!("[RESOLVER] Invalid URL structure or untrusted domain: {}", url_trimmed);
        return Err(AppError::BadRequest("Invalid Google Maps URL domain".to_string()));
    }

    // 1. Check Cache
    if let Some(cached) = location_cache.get(&url_trimmed) {
        let now = Utc::now();
        let elapsed = now.signed_duration_since(cached.timestamp);
        let ttl_minutes = if cached.success { 15 } else { 5 };
        
        if elapsed.num_minutes() < ttl_minutes {
            tracing::info!("[RESOLVER] Cache HIT for original URL: {}", url_trimmed);
            return Ok(cached.clone());
        } else {
            // Stale entry cleanup
            drop(cached);
            location_cache.remove(&url_trimmed);
        }
    }

    // 2. Request Deduplication Flow
    let (tx, mut rx) = match active_location_requests.entry(url_trimmed.clone()) {
        dashmap::mapref::entry::Entry::Occupied(entry) => {
            let rx = entry.get().subscribe();
            (None, rx)
        }
        dashmap::mapref::entry::Entry::Vacant(entry) => {
            let (tx, rx) = tokio::sync::broadcast::channel(1);
            entry.insert(tx.clone());
            (Some(tx), rx)
        }
    };

    if let Some(broadcaster) = tx {
        tracing::info!("[RESOLVER] Cache MISS. Initiating active resolution for {}", url_trimmed);
        
        let resolve_result = resolve_redirect_custom(&url_trimmed).await;
        
        match resolve_result {
            Ok(resolved) => {
                let (lat, lng) = extract_coords(&resolved);
                let cached_entry = CachedLocation {
                    original_url: url_trimmed.clone(),
                    resolved_url: resolved,
                    latitude: lat,
                    longitude: lng,
                    timestamp: Utc::now(),
                    success: true,
                };
                
                location_cache.insert(url_trimmed.clone(), cached_entry.clone());
                tracing::info!("[RESOLVER] Resolver success. Coords extracted: {:?}", (lat, lng));
                
                let _ = broadcaster.send(Ok(cached_entry.clone()));
                active_location_requests.remove(&url_trimmed);
                
                Ok(cached_entry)
            }
            Err(e) => {
                let is_transient = match &e {
                    AppError::Internal(msg) => msg.contains("timeout") || msg.contains("500") || msg.contains("502") || msg.contains("503") || msg.contains("504"),
                    _ => false,
                };

                if !is_transient {
                    let failure_entry = CachedLocation {
                        original_url: url_trimmed.clone(),
                        resolved_url: "".to_string(),
                        latitude: None,
                        longitude: None,
                        timestamp: Utc::now(),
                        success: false,
                    };
                    location_cache.insert(url_trimmed.clone(), failure_entry.clone());
                    tracing::warn!("[RESOLVER] Resolver failure cached for 5 minutes: {:?}", e);
                }

                let _ = broadcaster.send(Err(format!("{:?}", e)));
                active_location_requests.remove(&url_trimmed);
                
                Err(e)
            }
        }
    } else {
        tracing::info!("[RESOLVER] Request deduplication active. Waiting on subscriber channel for {}", url_trimmed);
        match rx.recv().await {
            Ok(Ok(cached)) => Ok(cached),
            Ok(Err(err_msg)) => Err(AppError::BadRequest(err_msg)),
            Err(e) => Err(AppError::Internal(format!("Deduplication channel error: {}", e))),
        }
    }
}

// ─── UNIT TESTS ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use dashmap::DashMap;
    use std::sync::Arc;
    use chrono::Duration;

    #[test]
    fn test_is_safe_ip() {
        // Unsafe local/private IPs
        assert!(!is_safe_ip("127.0.0.1".parse().unwrap()));
        assert!(!is_safe_ip("10.0.0.1".parse().unwrap()));
        assert!(!is_safe_ip("172.16.2.2".parse().unwrap()));
        assert!(!is_safe_ip("192.168.1.1".parse().unwrap()));
        assert!(!is_safe_ip("169.254.1.1".parse().unwrap()));
        assert!(!is_safe_ip("224.0.0.1".parse().unwrap()));
        assert!(!is_safe_ip("::1".parse().unwrap()));
        assert!(!is_safe_ip("fe80::1".parse().unwrap()));
        assert!(!is_safe_ip("fc00::1".parse().unwrap()));

        // Safe public IPs
        assert!(is_safe_ip("8.8.8.8".parse().unwrap()));
        assert!(is_safe_ip("1.1.1.1".parse().unwrap()));
        assert!(is_safe_ip("2606:4700:4700::1111".parse().unwrap()));
    }

    #[tokio::test]
    async fn test_is_url_safe() {
        // Whitelisted domains
        assert!(is_url_safe("https://maps.google.com").await);
        assert!(is_url_safe("https://google.com/maps").await);
        assert!(is_url_safe("https://maps.app.goo.gl/tB3a8B6u").await);
        assert!(is_url_safe("https://goo.gl/maps/8fRj3").await);
        assert!(is_url_safe("https://google.com.sa/maps").await);

        // Forbidden domains
        assert!(!is_url_safe("https://attacker.com").await);
        assert!(!is_url_safe("http://127.0.0.1:8080").await);
    }

    #[test]
    fn test_extract_coords() {
        // Test @lat,lng format
        let url_at = "https://www.google.com/maps/place/Riyadh/@24.7136,46.6753,15z/data=...";
        let (lat1, lng1) = extract_coords(url_at);
        assert_eq!(lat1, Some(24.7136));
        assert_eq!(lng1, Some(46.6753));

        // Test q=lat,lng format
        let url_q = "https://www.google.com/maps?q=20.5,70.5&z=10";
        let (lat2, lng2) = extract_coords(url_q);
        assert_eq!(lat2, Some(20.5));
        assert_eq!(lng2, Some(70.5));

        // Test ll=lat,lng format
        let url_ll = "https://maps.google.com/?ll=25.0,45.0";
        let (lat3, lng3) = extract_coords(url_ll);
        assert_eq!(lat3, Some(25.0));
        assert_eq!(lng3, Some(45.0));

        // Missing/Invalid coordinates
        let url_empty = "https://maps.google.com/maps/place/Riyadh";
        let (lat4, lng4) = extract_coords(url_empty);
        assert_eq!(lat4, None);
        assert_eq!(lng4, None);
    }

    #[tokio::test]
    async fn test_cache_miss_and_hit() {
        let cache = Arc::new(DashMap::new());
        let active_reqs = Arc::new(DashMap::new());

        let url = "https://maps.google.com/maps/place/Riyadh/@24.7136,46.6753,15z";
        
        // Cache miss resolves URL (mocking resolves by hitting google)
        let res1 = resolve_location_with_cache_impl(url, &cache, &active_reqs).await.unwrap();
        assert!(res1.success);
        assert_eq!(res1.latitude, Some(24.7136));
        assert_eq!(res1.longitude, Some(46.6753));

        // Verify it was added to cache
        assert!(cache.contains_key(url));

        // Modify cache entry to verify Cache Hit behavior (return cached item)
        if let Some(mut entry) = cache.get_mut(url) {
            entry.latitude = Some(99.9);
        }

        let res2 = resolve_location_with_cache_impl(url, &cache, &active_reqs).await.unwrap();
        assert_eq!(res2.latitude, Some(99.9)); // Confirm hit returned modified cached item
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = Arc::new(DashMap::new());
        let active_reqs = Arc::new(DashMap::new());
        let url = "https://maps.google.com/maps/place/Riyadh/@24.7136,46.6753,15z";

        // Pre-populate stale success cache (16 minutes ago)
        let stale_entry = CachedLocation {
            original_url: url.to_string(),
            resolved_url: url.to_string(),
            latitude: Some(24.7136),
            longitude: Some(46.6753),
            timestamp: Utc::now() - Duration::minutes(16),
            success: true,
        };
        cache.insert(url.to_string(), stale_entry);

        // Resolving should evict stale cache and perform resolution
        let res = resolve_location_with_cache_impl(url, &cache, &active_reqs).await.unwrap();
        assert!(res.success);
        // Timestamp should be updated to fresh
        let fresh = cache.get(url).unwrap();
        assert!(Utc::now().signed_duration_since(fresh.timestamp).num_minutes() < 1);
    }

    #[tokio::test]
    async fn test_failure_cache() {
        let cache = Arc::new(DashMap::new());
        let active_reqs = Arc::new(DashMap::new());
        let url = "https://maps.google.com/failed_url";

        // Pre-populate failure cache (success = false, timestamp = now)
        let failure_entry = CachedLocation {
            original_url: url.to_string(),
            resolved_url: "".to_string(),
            latitude: None,
            longitude: None,
            timestamp: Utc::now(),
            success: false,
        };
        cache.insert(url.to_string(), failure_entry);

        // Resolving this URL should hit the failure cache and return it immediately (Cache Hit)
        let res = resolve_location_with_cache_impl(url, &cache, &active_reqs).await.unwrap();
        assert!(!res.success);

        // Modify timestamp to simulate a stale failure entry (6 minutes ago)
        if let Some(mut entry) = cache.get_mut(url) {
            entry.timestamp = Utc::now() - Duration::minutes(6);
        }

        // Resolving it again should evict the stale entry and trigger a cache miss
        let res2 = resolve_location_with_cache_impl(url, &cache, &active_reqs).await;
        // The real request will resolve (maps.google.com/failed_url returns 200/404, but no coords)
        // Meaning it resolves successfully with success=true and latitude=None
        if let Ok(fresh) = res2 {
            assert!(fresh.success);
            assert_eq!(fresh.latitude, None);
        }
    }

    #[tokio::test]
    async fn test_concurrent_requests_deduplication() {
        let cache = Arc::new(DashMap::new());
        let active_reqs = Arc::new(DashMap::new());
        let url = "https://maps.google.com/maps/place/Riyadh/@24.7136,46.6753,15z";

        // Fire multiple concurrent requests for the same URL
        let cache_clone1 = cache.clone();
        let active_clone1 = active_reqs.clone();
        let task1 = tokio::spawn(async move {
            resolve_location_with_cache_impl(url, &cache_clone1, &active_clone1).await
        });

        let cache_clone2 = cache.clone();
        let active_clone2 = active_reqs.clone();
        let task2 = tokio::spawn(async move {
            resolve_location_with_cache_impl(url, &cache_clone2, &active_clone2).await
        });

        let (r1, r2) = tokio::join!(task1, task2);
        let res1 = r1.unwrap().unwrap();
        let res2 = r2.unwrap().unwrap();

        // Both should get identical results and the URL should be removed from active requests map
        assert_eq!(res1.resolved_url, res2.resolved_url);
        assert_eq!(res1.latitude, Some(24.7136));
        assert!(!active_reqs.contains_key(url));
    }
}

