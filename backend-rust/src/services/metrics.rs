// Zero-dependency, lock-free metrics counters using Rust atomics.
// All counters are process-global, reset on process restart.
// Exposed via GET /api/v1/metrics in Prometheus text format.

use std::sync::atomic::{AtomicU64, Ordering};

// ── Media upload counters ──────────────────────────────────────────────────────
static UPLOAD_SUCCESS_TOTAL: AtomicU64 = AtomicU64::new(0);
static UPLOAD_FAILED_TOTAL: AtomicU64 = AtomicU64::new(0);

// ── Listing lifecycle counters ─────────────────────────────────────────────────
static LISTING_CREATE_TOTAL: AtomicU64 = AtomicU64::new(0);
static LISTING_SUBMIT_TOTAL: AtomicU64 = AtomicU64::new(0);
static LISTING_EDIT_TOTAL: AtomicU64 = AtomicU64::new(0);

// ── Validation and quota counters ─────────────────────────────────────────────
static VALIDATION_FAILED_TOTAL: AtomicU64 = AtomicU64::new(0);
static QUOTA_BLOCK_TOTAL: AtomicU64 = AtomicU64::new(0);

// ── Increment helpers ─────────────────────────────────────────────────────────
pub fn inc_upload_success() {
    UPLOAD_SUCCESS_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn inc_upload_failed() {
    UPLOAD_FAILED_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn inc_listing_create() {
    LISTING_CREATE_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn inc_listing_submit() {
    LISTING_SUBMIT_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn inc_listing_edit() {
    LISTING_EDIT_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn inc_validation_failed() {
    VALIDATION_FAILED_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn inc_quota_block() {
    QUOTA_BLOCK_TOTAL.fetch_add(1, Ordering::Relaxed);
}

/// Serializes all counters to Prometheus text exposition format.
pub fn render_prometheus_text() -> String {
    let mut out = String::with_capacity(512);

    out.push_str("# HELP upload_success_total Total number of successful file uploads\n");
    out.push_str("# TYPE upload_success_total counter\n");
    out.push_str(&format!("upload_success_total {}\n", UPLOAD_SUCCESS_TOTAL.load(Ordering::Relaxed)));

    out.push_str("# HELP upload_failed_total Total number of failed file uploads\n");
    out.push_str("# TYPE upload_failed_total counter\n");
    out.push_str(&format!("upload_failed_total {}\n", UPLOAD_FAILED_TOTAL.load(Ordering::Relaxed)));

    out.push_str("# HELP listing_create_total Total number of listings created\n");
    out.push_str("# TYPE listing_create_total counter\n");
    out.push_str(&format!("listing_create_total {}\n", LISTING_CREATE_TOTAL.load(Ordering::Relaxed)));

    out.push_str("# HELP listing_submit_total Total number of listings submitted for approval\n");
    out.push_str("# TYPE listing_submit_total counter\n");
    out.push_str(&format!("listing_submit_total {}\n", LISTING_SUBMIT_TOTAL.load(Ordering::Relaxed)));

    out.push_str("# HELP listing_edit_total Total number of listing update/edit operations\n");
    out.push_str("# TYPE listing_edit_total counter\n");
    out.push_str(&format!("listing_edit_total {}\n", LISTING_EDIT_TOTAL.load(Ordering::Relaxed)));

    out.push_str("# HELP validation_failed_total Total number of validation failures\n");
    out.push_str("# TYPE validation_failed_total counter\n");
    out.push_str(&format!("validation_failed_total {}\n", VALIDATION_FAILED_TOTAL.load(Ordering::Relaxed)));

    out.push_str("# HELP quota_block_total Total requests blocked by subscription quota\n");
    out.push_str("# TYPE quota_block_total counter\n");
    out.push_str(&format!("quota_block_total {}\n", QUOTA_BLOCK_TOTAL.load(Ordering::Relaxed)));

    out
}
