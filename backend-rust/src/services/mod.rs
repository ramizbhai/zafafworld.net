pub mod category_schema;
pub mod email;
pub mod media; // services/media/ directory — see media/mod.rs for layout
pub mod metrics;
pub mod outbox_worker;
pub mod translation;
pub mod whatsapp;
// wp_cache_sync removed 2026-07-16 — WordPress retired, worker was a no-op
