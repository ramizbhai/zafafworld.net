# ZafafWorld Database Migration & Schema Conventions

This document outlines the standard schema conventions and policies for the PostgreSQL database in the ZafafWorld platform. All future migrations MUST adhere to these rules to maintain referential integrity, security, and query performance.

---

## 1. Naming & Casing Conventions

### Tables & Columns
* **Casing**: All table and column names MUST be in lowercase `snake_case`.
* **Casing Enforcement**: Never use double quotes (`"TableName"`) when creating tables. Using PascalCase/CamelCase with double quotes forces the engine to preserve casing, which complicates SQL query writing and database client integrations.
* **Pluralization**: Table names should be pluralized (e.g. `users`, `vendors`, `core_bookings`) unless representing singular config/state tables (e.g., `admin_settings`).

### Keys & Indexes
* **Primary Keys**: Always name primary keys `id` (generally of type `UUID` with default `uuid_generate_v4()`).
* **Foreign Keys**: Column names for foreign keys should follow the `singular_table_name_id` convention (e.g. `vendor_id` referencing `vendors(id)`).
* **Index Naming**: All indexes should follow: `idx_<table_name>_<columns>` (e.g., `idx_vendors_city_id`).

---

## 2. Referential Integrity & Foreign Keys

* **Explicit Constraints**: Every foreign key reference column MUST have an explicit `FOREIGN KEY` constraint. Do not use unlinked UUID columns for referencing other database tables.
* **Cascade Rules**: Specify clean delete behaviors for every relation:
  * Use `ON DELETE CASCADE` if the child record's lifecycle depends directly on the parent (e.g. review attachments on review deletion).
  * Use `ON DELETE SET NULL` if the child record should persist when the parent is deleted (e.g. product city reference when a city is deleted).
* **Index Foreign Keys**: Always create a B-Tree index on any foreign key column to avoid sequential scan degradation on JOIN queries.

---

## 3. Audit Columns

Every table representing platform entities must include standard audit tracking columns:
```sql
created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
```
Ensure that tables use the generic trigger `touch_updated_at()` to automatically keep the `updated_at` timestamp synchronized on updates:
```sql
CREATE TRIGGER trg_touch_updated_at
    BEFORE UPDATE ON your_table
    FOR EACH ROW
    EXECUTE FUNCTION touch_updated_at();
```

---

## 4. Centralized File Uploads (MinIO)

* **Central Registry**: All upload references pointing to MinIO storage objects must be logged in the `uploaded_files` table:
```sql
CREATE TABLE public.uploaded_files (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    bucket_name VARCHAR(63) NOT NULL,
    object_key VARCHAR(1024) NOT NULL UNIQUE,
    file_name VARCHAR(255) NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    uploaded_by UUID REFERENCES public.global_users(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);
```
* **References**: Entity tables referencing uploaded files should include a foreign key pointing to `uploaded_files(id)` (named `file_id`) to ensure clean reference counting and cascading cleanup.

---

## 5. Security & Row Level Security (RLS)

* **Default RLS**: All user/client/vendor specific tables must enable RLS:
```sql
ALTER TABLE table_name ENABLE ROW LEVEL SECURITY;
ALTER TABLE table_name FORCE ROW LEVEL SECURITY;
```
* **Session Variables**: Access queries must use transaction-level session configurations for identifying current context:
  * `current_setting('app.current_user_id', true)`
  * `current_setting('app.current_user_role', true)`
* **Credentials**: Never hardcode role passwords inside migration files. Databases roles must be defined as `NOLOGIN`, with connection credentials injected via system environment parameters.

---

## 6. Production Readiness & Performance

* **Idempotency**: Always write migrations using `IF NOT EXISTS` or `DROP ... IF EXISTS` to make sure they are safe to re-run or recover from failure.
* **Non-Blocking Indexes**: In high-traffic environments, index additions on large tables should use `CREATE INDEX CONCURRENTLY` in independent transactions to avoid blocking reads/writes.

---

## 7. File Storage Architecture

### MinIO Client (Phase 3 update — 2026-07)

The MinIO S3-compatible client is now a **singleton** (`services/media/minio_client.rs`), constructed once at startup from environment variables and stored in `AppState.minio_client: Arc<MinioClient>`.

**Rules:**
- **Never** read `MINIO_*` environment variables inside a handler or per-request code path. Always use `state.minio_client`.
- **Every** successful `MinioClient::upload()` call SHOULD log to `uploaded_files` via `repositories::uploaded_files_repository::insert_upload()`.
- **Every** successful `MinioClient::delete()` or `delete_gallery_item()` call SHOULD remove the row via `repositories::uploaded_files_repository::delete_by_key()`.
- The `uploaded_files` table is the canonical source of truth for MinIO object inventory (§4 above).

### Media Sub-module Layout

```
src/services/media/
  mod.rs               — public surface, process_and_save_upload orchestrator
  minio_client.rs      — MinioClient singleton (upload, delete, delete_gallery_item)
  image_processing.rs  — single-decode WebP pipeline (5 variants, no per-call clones)
  video_processing.rs  — ffprobe probe + ffmpeg transcode pipeline with semaphore
  migration.rs         — one-time CLI video transcoding migration tool
```

---

## 8. Rust Release Profile

The production binary (`cargo build --release`) is configured in `Cargo.toml [profile.release]` with:
- `lto = true` — full link-time optimization
- `codegen-units = 1` — single codegen unit for maximum inlining
- `strip = true` — debug symbols stripped from final binary
- `panic = "abort"` — panics abort instead of unwind (safe: `CatchPanicLayer` catches them)

**Do not change these settings** without understanding the tradeoffs. In particular, `panic = "abort"` means the `Drop` impls of in-flight types are **not** called on panic — this is safe because all panics are caught by `CatchPanicLayer` before they propagate to any finalizer that would hold a DB transaction open.
