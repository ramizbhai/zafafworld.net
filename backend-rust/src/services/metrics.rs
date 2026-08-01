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

// ── Video Adaptive Streaming (HLS) counters ───────────────────────────────────
static ACTIVE_STREAMS: AtomicU64 = AtomicU64::new(0);
static SEGMENT_GENERATION_TIME_MS_SUM: AtomicU64 = AtomicU64::new(0);
static SEGMENT_GENERATION_TIME_MS_COUNT: AtomicU64 = AtomicU64::new(0);
static PLAYLIST_GENERATION_TIME_MS_SUM: AtomicU64 = AtomicU64::new(0);
static PLAYLIST_GENERATION_TIME_MS_COUNT: AtomicU64 = AtomicU64::new(0);
static STREAM_ERRORS_TOTAL: AtomicU64 = AtomicU64::new(0);
static BUFFER_EVENTS_TOTAL: AtomicU64 = AtomicU64::new(0);
static ENCODING_DURATION_MS_SUM: AtomicU64 = AtomicU64::new(0);
static ENCODING_DURATION_MS_COUNT: AtomicU64 = AtomicU64::new(0);
static HLS_GENERATION_FAILURES_TOTAL: AtomicU64 = AtomicU64::new(0);

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

// HLS specific increments
pub fn inc_active_streams() {
    ACTIVE_STREAMS.fetch_add(1, Ordering::Relaxed);
}

pub fn dec_active_streams() {
    let val = ACTIVE_STREAMS.load(Ordering::Relaxed);
    if val > 0 {
        ACTIVE_STREAMS.fetch_sub(1, Ordering::Relaxed);
    }
}

pub fn observe_segment_gen(duration_ms: u64) {
    SEGMENT_GENERATION_TIME_MS_SUM.fetch_add(duration_ms, Ordering::Relaxed);
    SEGMENT_GENERATION_TIME_MS_COUNT.fetch_add(1, Ordering::Relaxed);
}

pub fn observe_playlist_gen(duration_ms: u64) {
    PLAYLIST_GENERATION_TIME_MS_SUM.fetch_add(duration_ms, Ordering::Relaxed);
    PLAYLIST_GENERATION_TIME_MS_COUNT.fetch_add(1, Ordering::Relaxed);
}

pub fn inc_stream_errors() {
    STREAM_ERRORS_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn inc_buffer_events() {
    BUFFER_EVENTS_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn observe_encoding(duration_ms: u64) {
    ENCODING_DURATION_MS_SUM.fetch_add(duration_ms, Ordering::Relaxed);
    ENCODING_DURATION_MS_COUNT.fetch_add(1, Ordering::Relaxed);
}

pub fn inc_hls_generation_failures() {
    HLS_GENERATION_FAILURES_TOTAL.fetch_add(1, Ordering::Relaxed);
}

/// Serializes all counters to Prometheus text exposition format.
pub fn render_prometheus_text() -> String {
    let mut out = String::with_capacity(1024);

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

    // Video adaptive streaming
    out.push_str("# HELP active_streams Total number of concurrent client HLS streaming sessions\n");
    out.push_str("# TYPE active_streams gauge\n");
    out.push_str(&format!("active_streams {}\n", ACTIVE_STREAMS.load(Ordering::Relaxed)));

    out.push_str("# HELP segment_generation_time_ms_sum Cumulative duration in ms of HLS segment generation\n");
    out.push_str("# TYPE segment_generation_time_ms_sum counter\n");
    out.push_str(&format!("segment_generation_time_ms_sum {}\n", SEGMENT_GENERATION_TIME_MS_SUM.load(Ordering::Relaxed)));

    out.push_str("# HELP segment_generation_time_ms_count Total number of HLS segments generated\n");
    out.push_str("# TYPE segment_generation_time_ms_count counter\n");
    out.push_str(&format!("segment_generation_time_ms_count {}\n", SEGMENT_GENERATION_TIME_MS_COUNT.load(Ordering::Relaxed)));

    out.push_str("# HELP playlist_generation_time_ms_sum Cumulative duration in ms of HLS playlist generation\n");
    out.push_str("# TYPE playlist_generation_time_ms_sum counter\n");
    out.push_str(&format!("playlist_generation_time_ms_sum {}\n", PLAYLIST_GENERATION_TIME_MS_SUM.load(Ordering::Relaxed)));

    out.push_str("# HELP playlist_generation_time_ms_count Total number of HLS playlists generated\n");
    out.push_str("# TYPE playlist_generation_time_ms_count counter\n");
    out.push_str(&format!("playlist_generation_time_ms_count {}\n", PLAYLIST_GENERATION_TIME_MS_COUNT.load(Ordering::Relaxed)));

    out.push_str("# HELP stream_errors_total Cumulative stream play errors reported by client browsers\n");
    out.push_str("# TYPE stream_errors_total counter\n");
    out.push_str(&format!("stream_errors_total {}\n", STREAM_ERRORS_TOTAL.load(Ordering::Relaxed)));

    out.push_str("# HELP buffer_events_total Cumulative client playback stall and buffer events reported by client browsers\n");
    out.push_str("# TYPE buffer_events_total counter\n");
    out.push_str(&format!("buffer_events_total {}\n", BUFFER_EVENTS_TOTAL.load(Ordering::Relaxed)));

    out.push_str("# HELP encoding_duration_ms_sum Cumulative duration in ms of full video transcode encoding\n");
    out.push_str("# TYPE encoding_duration_ms_sum counter\n");
    out.push_str(&format!("encoding_duration_ms_sum {}\n", ENCODING_DURATION_MS_SUM.load(Ordering::Relaxed)));

    out.push_str("# HELP encoding_duration_ms_count Total number of full video transcode encodings run\n");
    out.push_str("# TYPE encoding_duration_ms_count counter\n");
    out.push_str(&format!("encoding_duration_ms_count {}\n", ENCODING_DURATION_MS_COUNT.load(Ordering::Relaxed)));

    out.push_str("# HELP hls_generation_failures_total Total number of HLS adaptive streaming generation failures\n");
    out.push_str("# TYPE hls_generation_failures_total counter\n");
    out.push_str(&format!("hls_generation_failures_total {}\n", HLS_GENERATION_FAILURES_TOTAL.load(Ordering::Relaxed)));

    out
}
