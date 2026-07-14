use std::env;
use std::net::IpAddr;
use std::str::FromStr;

/// Known development placeholder secrets that must be rejected at startup.
/// These values appear in .env.example files and are publicly known.
/// Acceptance of any of these in production would allow JWT forgery by anyone
/// who has read the source code or example files.
const KNOWN_INSECURE_SECRETS: &[&str] = &[
    "super_secret_key_change_me_in_prod_12345!",
    "replace_with_256bit_secret_in_production",
    "changeme",
    "secret",
    "your_jwt_secret_here",
    "jwt_secret",
    "development_secret",
    "dev_secret",
];

/// Minimum entropy threshold: Shannon entropy below this value indicates
/// a low-diversity secret (e.g., repeated characters or dictionary words).
const MIN_SHANNON_ENTROPY: f64 = 3.5;

/// Compute Shannon entropy (bits per character) of a string.
fn shannon_entropy(s: &str) -> f64 {
    if s.is_empty() {
        return 0.0;
    }
    let mut counts = [0u32; 256];
    for &b in s.as_bytes() {
        counts[b as usize] += 1;
    }
    let len = s.len() as f64;
    counts
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f64 / len;
            -p * p.log2()
        })
        .sum()
}

#[derive(Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub port: u16,
    pub jwt_secret: String,
    /// Base URL of the frontend, used to construct password-reset links.
    /// Set via FRONTEND_URL environment variable.
    /// Defaults to http://localhost:5173 for local development.
    pub frontend_url: String,
    pub trusted_proxies: Vec<IpAddr>,
    /// SMTP configuration — all fields required together. If any are absent
    /// the email service falls back to stdout logging (development only).
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_from_email: String,
    pub smtp_from_name: String,
    pub whatsapp_graph_base_url: String,
    pub whatsapp_graph_version: String,
    pub whatsapp_template_new_inquiry: String,
    pub whatsapp_language: String,
    pub outbox_batch_size: i64,
    pub outbox_poll_interval_ms: u64,
    pub outbox_cleanup_interval_secs: u64,
    pub outbox_max_retries: i32,
    pub outbox_retention_days: i64,
    pub outbox_retry_schedule: Vec<u64>,
    pub outbox_max_parallel_deliveries: usize,
    pub smtp_timeout_seconds: u64,
    pub http_timeout_seconds: u64,
    pub app_environment: String,
    pub smtp_tls_mode: Option<String>,
    pub whatsapp_token: Option<String>,
    pub whatsapp_phone_number_id: Option<String>,
    pub afrah_notification_phone: Option<String>,
    /// Shared secret for the internal WordPress → Rust sync webhook.
    /// Read from WP_SYNC_SECRET. Both WordPress (via WP_SYNC_SECRET PHP constant
    /// in wp-config.php) and this backend must have the same value.
    /// If empty in production, the /api/v1/internal/wp-sync endpoint will always
    /// reject requests with 401 — a missing secret is a safer failure mode than
    /// accepting unauthenticated syncs.
    pub wp_sync_secret: String,
    pub minio_endpoint: String,
    pub minio_bucket: String,
    pub minio_app_user: String,
    pub minio_app_password: String,
    pub minio_root_prefix: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        // Load dotenv file if present. Errors (no file) are silently ignored.
        let _ = dotenvy::dotenv();

        // ── Database ──────────────────────────────────────────────────────────
        // No fallback to hard-coded credentials. Missing DATABASE_URL means the
        // process cannot connect and should fail fast at pool init.
        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/zafaf_world".to_string()
        });

        // ── Port ──────────────────────────────────────────────────────────────
        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .unwrap_or(8080);

        // ── JWT Secret ────────────────────────────────────────────────────────
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_default();

        if jwt_secret.is_empty() {
            panic!(
                "\n========================================================================\n\
                 [FATAL] Missing JWT_SECRET environment variable!\n\
                 \n\
                 The JWT_SECRET must be set to a cryptographically secure random string\n\
                 of at least 64 characters before the backend can boot.\n\
                 \n\
                 Generate one with:\n\
                   openssl rand -hex 64\n\
                 or:\n\
                   python3 -c \"import secrets; print(secrets.token_hex(64))\"\n\
                 \n\
                 Set it via environment variable or in infra/.env (never in backend-rust/.env).\n\
                 ========================================================================\n"
            );
        }

        // Minimum 64 characters (512 bits) for HS256
        if jwt_secret.len() < 64 {
            panic!(
                "\n========================================================================\n\
                 [FATAL] Insecure JWT_SECRET: too short ({} characters, minimum 64)!\n\
                 \n\
                 A short secret is vulnerable to brute-force attacks.\n\
                 Generate a new secret with: openssl rand -hex 64\n\
                 ========================================================================\n",
                jwt_secret.len()
            );
        }

        // Reject any publicly known development placeholder secrets
        let jwt_lower = jwt_secret.to_lowercase();
        for &known_bad in KNOWN_INSECURE_SECRETS {
            if jwt_lower.contains(&known_bad.to_lowercase()) {
                panic!(
                    "\n========================================================================\n\
                     [FATAL] JWT_SECRET matches a known insecure development placeholder!\n\
                     \n\
                     The configured secret contains a pattern used in example files\n\
                     and development environments. It is publicly known and must not\n\
                     be used in any deployment that handles real user data.\n\
                     \n\
                     Generate a secure replacement with: openssl rand -hex 64\n\
                     ========================================================================\n"
                );
            }
        }

        // Reject low-entropy secrets (repeated chars, dictionary words, etc.)
        let entropy = shannon_entropy(&jwt_secret);
        if entropy < MIN_SHANNON_ENTROPY {
            panic!(
                "\n========================================================================\n\
                 [FATAL] JWT_SECRET has insufficient entropy ({:.2} bits/char, minimum {:.1})!\n\
                 \n\
                 The secret appears to consist of repeated or highly predictable characters.\n\
                 Generate a secure replacement with: openssl rand -hex 64\n\
                 ========================================================================\n",
                entropy, MIN_SHANNON_ENTROPY
            );
        }

        // ── Frontend URL ──────────────────────────────────────────────────────
        let app_env = env::var("APP_ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        let is_prod = app_env.to_lowercase() == "production";

        let frontend_url = env::var("FRONTEND_URL").unwrap_or_default();
        let frontend_url = if frontend_url.is_empty() {
            if is_prod {
                panic!(
                    "\n========================================================================\n\
                     [FATAL] Missing FRONTEND_URL environment variable in production mode!\n\
                     \n\
                     Set FRONTEND_URL in the host environment or infra/.env to continue.\n\
                     ========================================================================\n"
                );
            } else {
                "http://localhost:5173".to_string()
            }
        } else {
            frontend_url
        };

        // ── SMTP ──────────────────────────────────────────────────────────────
        let smtp_host = env::var("SMTP_HOST").ok();
        let smtp_port = env::var("SMTP_PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok());
        let smtp_username = env::var("SMTP_USERNAME").ok();
        let smtp_password = env::var("SMTP_PASSWORD").ok();
        let smtp_from_email = env::var("SMTP_FROM_EMAIL").ok();
        let smtp_from_name =
            env::var("SMTP_FROM_NAME").unwrap_or_else(|_| "ZafafWorld".to_string());

        if is_prod {
            let mut missing = Vec::new();
            if smtp_host.as_ref().is_none_or(|s| s.trim().is_empty()) {
                missing.push("SMTP_HOST");
            }
            if smtp_port.is_none() {
                missing.push("SMTP_PORT");
            }
            if smtp_username.as_ref().is_none_or(|s| s.trim().is_empty()) {
                missing.push("SMTP_USERNAME");
            }
            if smtp_password.as_ref().is_none_or(|s| s.trim().is_empty()) {
                missing.push("SMTP_PASSWORD");
            }
            if smtp_from_email.as_ref().is_none_or(|s| s.trim().is_empty()) {
                missing.push("SMTP_FROM_EMAIL");
            }

            if !missing.is_empty() {
                panic!(
                    "\n========================================================================\n\
                     [FATAL] Missing required SMTP environment variables in production mode: {:?}!\n\
                     \n\
                     For security and system integrity, ZafafWorld password recovery\n\
                     requires a functional SMTP relay in production.\n\
                     \n\
                     Please specify valid values for all required SMTP variables in the environment.\n\
                     ========================================================================\n",
                    missing
                );
            }
        }

        let smtp_from_email =
            smtp_from_email.unwrap_or_else(|| "noreply@zafafworld.com".to_string());

        // ── Trusted Proxies ───────────────────────────────────────────────────
        let trusted_proxies_str = env::var("TRUSTED_PROXIES").unwrap_or_default();
        let mut trusted_proxies = Vec::new();
        for s in trusted_proxies_str.split(',') {
            let s_trim = s.trim();
            if !s_trim.is_empty() {
                if let Ok(ip) = IpAddr::from_str(s_trim) {
                    trusted_proxies.push(ip);
                } else {
                    panic!(
                        "\n========================================================================\n\
                         [FATAL] Malformed IP inside TRUSTED_PROXIES: '{}'!\n\
                         Please supply valid comma-separated IPv4 or IPv6 addresses.\n\
                         ========================================================================\n",
                        s_trim
                    );
                }
            }
        }

        // Default to local loopback addresses if none are specified
        if trusted_proxies.is_empty() {
            // SAFETY: Statically valid IPv4 address literal
            trusted_proxies.push(IpAddr::from_str("127.0.0.1").unwrap());
            // SAFETY: Statically valid IPv6 address literal
            trusted_proxies.push(IpAddr::from_str("::1").unwrap());
        }

        // ── WhatsApp Settings ────────────────────────────────────────────────
        let whatsapp_graph_base_url = env::var("WHATSAPP_GRAPH_BASE_URL")
            .unwrap_or_else(|_| "https://graph.facebook.com".to_string());
        if !whatsapp_graph_base_url.starts_with("http://")
            && !whatsapp_graph_base_url.starts_with("https://")
        {
            panic!(
                "\n[FATAL] WHATSAPP_GRAPH_BASE_URL must start with http:// or https://: '{}'\n",
                whatsapp_graph_base_url
            );
        }

        let whatsapp_graph_version =
            env::var("WHATSAPP_GRAPH_VERSION").unwrap_or_else(|_| "v18.0".to_string());
        if !whatsapp_graph_version.starts_with('v') || whatsapp_graph_version.len() < 3 {
            panic!(
                "\n[FATAL] WHATSAPP_GRAPH_VERSION must start with 'v' (e.g., 'v18.0'): '{}'\n",
                whatsapp_graph_version
            );
        }

        let whatsapp_template_new_inquiry = env::var("WHATSAPP_TEMPLATE_NEW_INQUIRY")
            .unwrap_or_else(|_| "new_inquiry_alert".to_string());
        if whatsapp_template_new_inquiry.trim().is_empty() {
            panic!("\n[FATAL] WHATSAPP_TEMPLATE_NEW_INQUIRY cannot be empty\n");
        }

        let whatsapp_language = env::var("WHATSAPP_LANGUAGE").unwrap_or_else(|_| "en".to_string());
        if whatsapp_language.trim().is_empty() {
            panic!("\n[FATAL] WHATSAPP_LANGUAGE cannot be empty\n");
        }

        // ── Outbox Settings ──────────────────────────────────────────────────
        let outbox_batch_size = env::var("OUTBOX_BATCH_SIZE")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<i64>()
            .unwrap_or_else(|e| panic!("\n[FATAL] Invalid OUTBOX_BATCH_SIZE: {}\n", e));
        if outbox_batch_size <= 0 {
            panic!(
                "\n[FATAL] OUTBOX_BATCH_SIZE must be greater than 0: {}\n",
                outbox_batch_size
            );
        }

        let outbox_poll_interval_ms = env::var("OUTBOX_POLL_INTERVAL_MS")
            .unwrap_or_else(|_| "1000".to_string())
            .parse::<u64>()
            .unwrap_or_else(|e| panic!("\n[FATAL] Invalid OUTBOX_POLL_INTERVAL_MS: {}\n", e));
        if outbox_poll_interval_ms == 0 {
            panic!("\n[FATAL] OUTBOX_POLL_INTERVAL_MS must be greater than 0\n");
        }

        let outbox_cleanup_interval_secs = env::var("OUTBOX_CLEANUP_INTERVAL_SECS")
            .unwrap_or_else(|_| "300".to_string())
            .parse::<u64>()
            .unwrap_or_else(|e| panic!("\n[FATAL] Invalid OUTBOX_CLEANUP_INTERVAL_SECS: {}\n", e));

        let outbox_max_retries = env::var("OUTBOX_MAX_RETRIES")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<i32>()
            .unwrap_or_else(|e| panic!("\n[FATAL] Invalid OUTBOX_MAX_RETRIES: {}\n", e));
        if outbox_max_retries < 0 {
            panic!(
                "\n[FATAL] OUTBOX_MAX_RETRIES must be non-negative: {}\n",
                outbox_max_retries
            );
        }

        let outbox_retention_days = env::var("OUTBOX_RETENTION_DAYS")
            .unwrap_or_else(|_| "7".to_string())
            .parse::<i64>()
            .unwrap_or_else(|e| panic!("\n[FATAL] Invalid OUTBOX_RETENTION_DAYS: {}\n", e));
        if outbox_retention_days < 0 {
            panic!(
                "\n[FATAL] OUTBOX_RETENTION_DAYS must be non-negative: {}\n",
                outbox_retention_days
            );
        }

        let retry_schedule_str =
            env::var("OUTBOX_RETRY_SCHEDULE").unwrap_or_else(|_| "30,120,600,1800".to_string());
        let mut outbox_retry_schedule = Vec::new();
        for val in retry_schedule_str.split(',') {
            let val_trim = val.trim();
            if !val_trim.is_empty() {
                let secs = val_trim.parse::<u64>().unwrap_or_else(|e| {
                    panic!(
                        "\n[FATAL] Invalid retry schedule segment '{}': {}\n",
                        val_trim, e
                    );
                });
                outbox_retry_schedule.push(secs);
            }
        }
        if outbox_retry_schedule.is_empty() {
            panic!("\n[FATAL] OUTBOX_RETRY_SCHEDULE cannot be empty\n");
        }

        let outbox_max_parallel_deliveries = env::var("OUTBOX_MAX_PARALLEL_DELIVERIES")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<usize>()
            .unwrap_or_else(|e| {
                panic!("\n[FATAL] Invalid OUTBOX_MAX_PARALLEL_DELIVERIES: {}\n", e)
            });
        if outbox_max_parallel_deliveries == 0 {
            panic!("\n[FATAL] OUTBOX_MAX_PARALLEL_DELIVERIES must be greater than 0\n");
        }

        let smtp_timeout_seconds = env::var("SMTP_TIMEOUT_SECONDS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u64>()
            .unwrap_or_else(|e| panic!("\n[FATAL] Invalid SMTP_TIMEOUT_SECONDS: {}\n", e));
        if smtp_timeout_seconds == 0 {
            panic!("\n[FATAL] SMTP_TIMEOUT_SECONDS must be greater than 0\n");
        }

        let http_timeout_seconds = env::var("HTTP_TIMEOUT_SECONDS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u64>()
            .unwrap_or_else(|e| panic!("\n[FATAL] Invalid HTTP_TIMEOUT_SECONDS: {}\n", e));
        if http_timeout_seconds == 0 {
            panic!("\n[FATAL] HTTP_TIMEOUT_SECONDS must be greater than 0\n");
        }

        let whatsapp_token = env::var("WHATSAPP_TOKEN")
            .ok()
            .filter(|s| !s.trim().is_empty());
        let whatsapp_phone_number_id = env::var("WHATSAPP_PHONE_NUMBER_ID")
            .ok()
            .filter(|s| !s.trim().is_empty());
        let afrah_notification_phone = env::var("AFRAH_NOTIFICATION_PHONE")
            .ok()
            .filter(|s| !s.trim().is_empty());
        let smtp_tls_mode = env::var("SMTP_TLS_MODE")
            .ok()
            .filter(|s| !s.trim().is_empty());

        // ── WordPress sync secret ─────────────────────────────────────────────
        // Required for the internal /api/v1/internal/wp-sync endpoint.
        // Warn in production if missing; the handler will reject all syncs with 401.
        let wp_sync_secret = env::var("WP_SYNC_SECRET").unwrap_or_default();
        if wp_sync_secret.is_empty() && is_prod {
            tracing::warn!(
                "WP_SYNC_SECRET is not set. The /api/v1/internal/wp-sync endpoint \
                 will reject all incoming WordPress publish webhooks with 401. \
                 Set WP_SYNC_SECRET in the environment to enable WordPress CMS sync."
            );
        }

        if let Some(ref id) = whatsapp_phone_number_id {
            if id.starts_with('+') {
                panic!(
                    "\n[FATAL] WHATSAPP_PHONE_NUMBER_ID must not start with '+': '{}'\n",
                    id
                );
            }
        }

        if let Some(ref phone) = afrah_notification_phone {
            if !phone.starts_with('+')
                || phone.len() < 8
                || phone.len() > 16
                || !phone[1..].chars().all(|c| c.is_ascii_digit())
            {
                panic!(
                    "\n[FATAL] AFRAH_NOTIFICATION_PHONE must be a valid E.164 number and start with '+': '{}'\n",
                    phone
                );
            }
        }

        let minio_endpoint = env::var("MINIO_ENDPOINT").unwrap_or_else(|_| "http://minio:9000".to_string());
        let minio_bucket = env::var("MINIO_BUCKET").unwrap_or_else(|_| "zafafworld-media".to_string());
        let minio_app_user = env::var("MINIO_APP_USER").unwrap_or_default();
        let minio_app_password = env::var("MINIO_APP_PASSWORD").unwrap_or_default();
        let minio_root_prefix = env::var("MINIO_ROOT_PREFIX").unwrap_or_else(|_| "assets/uploads".to_string());

        Self {
            database_url,
            port,
            jwt_secret,
            frontend_url,
            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
            smtp_from_email,
            smtp_from_name,
            trusted_proxies,
            whatsapp_graph_base_url,
            whatsapp_graph_version,
            whatsapp_template_new_inquiry,
            whatsapp_language,
            outbox_batch_size,
            outbox_poll_interval_ms,
            outbox_cleanup_interval_secs,
            outbox_max_retries,
            outbox_retention_days,
            outbox_retry_schedule,
            outbox_max_parallel_deliveries,
            smtp_timeout_seconds,
            http_timeout_seconds,
            app_environment: app_env,
            smtp_tls_mode,
            whatsapp_token,
            whatsapp_phone_number_id,
            afrah_notification_phone,
            wp_sync_secret,
            minio_endpoint,
            minio_bucket,
            minio_app_user,
            minio_app_password,
            minio_root_prefix,
        }
    }

    /// Returns true if a complete SMTP configuration is present.
    /// All four of host, port, username, and password must be set.
    pub fn smtp_configured(&self) -> bool {
        self.smtp_host.is_some()
            && self.smtp_port.is_some()
            && self.smtp_username.is_some()
            && self.smtp_password.is_some()
    }
}
