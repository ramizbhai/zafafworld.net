# Master System Design Document (MSDD)
## Greenfield Zero-Trust, High-SEO, Clean Modular Rust/Leptos SaaS Platform

**Status:** Modules 1–8 Locked · Module 9 Opened (Not Yet Designed) · Scaffolding Phase 1 (Workspace Setup) Complete & Verified
**Stack:** Full-Stack Rust (Leptos + Axum), Postgres, Stripe, `tracing`
**Scope of this document:** Conceptual architecture only. No implementation code.

---

## 1. Core Technical Foundations

### 1.1 Framework & Rendering Model
- **Leptos** (full-stack Rust, fine-grained reactive signals, no virtual DOM) compiling to server-side Rust (Axum) + client-side WASM from one codebase.
- **Streaming SSR** is the default rendering mode: HTML streams to the client as it becomes ready, optimizing Time-to-First-Byte and search-engine indexing.
- **Islands architecture** (`#[island]`) — not blanket micro-componentization — is the actual lever for minimizing client-side WASM payload. Static content stays inert HTML; only designated islands hydrate and run WASM.
- **Component granularity policy:** UI components are split along *semantic* boundaries (a meaningful functional unit — e.g. `SubmitButton`, `InvoiceRow` — not one component per DOM node). Domain/business logic, by contrast, is maximally isolated into small, independently testable Rust functions. This distinction was adopted specifically to prevent the original "every element is its own component" mandate from working against both compile times and root-cause traceability.

### 1.2 Internationalization & Semantic DOM (Arabic RTL / English LTR)
- **Locale resolution**, server-side, in strict precedence order:
  1. URL path prefix (`/ar/...`, `/en/...`) — authoritative.
  2. Cookie (prior preference) — fallback.
  3. `Accept-Language` header — final fallback.
  - **Conflict rule:** URL prefix always wins; the cookie is overwritten to match on any conflict.
- Resolved locale is injected into request context (`LocaleContext`) before rendering begins — never resolved client-side, to avoid hydration mismatches.
- The SSR shell's `<html dir="..." lang="...">` attributes are derived from this same resolved context in the same server pass that renders the rest of the page — direction and language can never disagree with the rendered strings.
- **Translation pipeline:** `leptos_i18n` compiles locale JSON/YAML into typed keys at build time. A missing or misspelled key is a compile error, not a runtime blank string. Adding a key never requires restructuring a component — only referencing it where needed.
- **Layout law:** Tailwind **logical properties only** (`ms-`, `me-`, `ps-`, `pe-`, `text-start`, `text-end`). Physical directional utilities (`ml-`, `mr-`, `pl-`, `pr-`, `left-`, `right-`) are prohibited.
  - **Enforcement:** a build-breaking CI gate (regex/grep check run during `cargo check`/`cargo test`) fails the pipeline the moment a physical directional utility appears. This is a compiler-adjacent gate, not a style-review convention.

---

## 2. Unified Request-Scoped Data Lifecycle (`core_types`)

A single internal crate/module, **`core_types`**, is the sole source of truth for every piece of request-scoped context. No layer (middleware, Leptos context engine, repository layer) is permitted to redefine or duplicate these types.

| Context Type | Origin | Populated By | Consumed By |
|---|---|---|---|
| `TraceContext` (ULID) | Generated at the outermost middleware layer, before routing/auth/locale | Axum/Actix entry middleware | Every `tracing::Span`, all repository calls, `<ErrorBoundary>` |
| `LocaleContext` | Resolved from URL → cookie → header | Locale-resolution middleware | i18n rendering, SSR shell `dir`/`lang`, locale-aware redirects |
| `UserContext` | Derived from validated opaque session cookie | Auth middleware | Server functions, UI personalization |
| `TenantContext` | Derived from validated session (never client-supplied) | Auth middleware | All Repository calls (mandatory parameter), RLS session variable |

**Fixed request pipeline ordering (locked):**

```
1. Trace ID generation (outermost)
2. Locale resolution
3. Auth / session validation → UserContext + TenantContext
4. Route / layout rendering
```

This ordering exists so that failure handling (e.g., redirecting an unauthenticated user) can always render a **locale-correct** SSR redirect (`/login` vs `/ar/login`) without a client-side redirect flash or hydration mismatch.

All four context types are injected via `provide_context()` at the appropriate pipeline stage and retrieved downstream via `use_context::<T>()` — never re-derived or re-parsed by individual server functions or components.

---

## 3. Multi-Tenancy, Database & Billing

### 3.1 Data Isolation Model
- Single shared Postgres instance, shared schema. Every tenant-scoped table (`users`, `invoices`, `project_logs`, `subscriptions`, etc.) carries a mandatory `tenant_id UUID NOT NULL`, indexed as the leading column in composite indexes.
- **Defense-in-depth, two independent layers, both mandatory for v1:**
  1. **Repository Pattern (compile-time enforced):** no raw SQLx queries anywhere in server functions or domain handlers. All database access passes through per-entity repository functions whose signatures *require* `TenantContext` as a mandatory first argument — a query without tenant scope is a compile error.
  2. **Postgres Row-Level Security (RLS):** every tenant-scoped table has RLS policies bound to the session's active `tenant_id`, enforced at the database level independent of application code. This is deliberate redundancy: a bug in the repository layer cannot cross tenant boundaries because the database itself refuses the row.
- Zero-Trust posture: tenant scope is derived fresh from validated session context and re-verified on every single action — never cached across actions, never trusted from client input.

### 3.2 Billing (Stripe) — Local-First Entitlement Model
- Postgres holds a **local, derived cache** of subscription/entitlement state (`subscriptions` table keyed by `tenant_id`, `stripe_customer_id`, `stripe_subscription_id`, `plan_tier`, `status`, `current_period_end`).
- **Entitlement checks never call the Stripe API at request time.** All "is this tenant Enterprise tier?" checks resolve against the local table via the standard Repository layer.
- **The only writer to this table is the webhook handler** — no application code path may write `plan_tier` or `status` directly; only a confirmed Stripe event does.

### 3.3 Webhook Ingestion (Asynchronous, Transactional)
```
Stripe Event → Signature Verification (HMAC) → Store raw event
   into `stripe_webhook_events` (unique constraint on stripe_event_id)
   → Immediate 200 OK response
   → Background Tokio worker pool processes queue asynchronously
   → Tenant resolved from stripe_customer_id → tenant_id mapping
     (never trusted directly from event metadata)
   → Repository-layer transactional update, scoped to resolved tenant_id
```
- **Idempotency:** enforced via unique constraint on `stripe_event_id`; retried Stripe deliveries are no-ops on duplicate.
- **Authentication Exception:** this endpoint is formally documented as bypassing the Module 2 session-cookie authentication model entirely. It is secured exclusively by HMAC signature verification against the Stripe webhook signing secret. This exception is recorded explicitly in the architecture manifest so it is never mistaken for an unauthenticated or under-secured endpoint.

### 3.4 Checkout Flow (Client-Initiated)
- Client invokes a `#[server]` function → server derives `TenantContext`/`UserContext` from request context (never from client-supplied arguments) → creates a Stripe Checkout Session via `stripe-rust`, tagging `tenant_id` in session metadata for later webhook cross-reference → returns the hosted Checkout URL → client performs a top-level browser redirect.

---

## 4. Observability Framework

### 4.1 Trace Propagation
- A **ULID** `trace_id` is generated once, at the outermost middleware layer, for every request type uniformly: standard HTTP requests, isomorphic `#[server]` function calls, and the Stripe webhook endpoint.
- Propagated as `TraceContext` inside `core_types`, entered as a `tracing::Span` at the middleware layer — every downstream `tracing` call inherits it automatically through span context, with no manual threading through function signatures.
- Repository calls are wrapped in `#[tracing::instrument]`, logging query name/table plus the active `trace_id` (never raw SQL parameter values).

### 4.2 Redaction Layer (Mandatory, Subscriber-Level)
- A centralized `tracing-subscriber` field-filtering layer scrubs known-sensitive field names **before** anything reaches a log sink: raw SQL bound parameters, session cookie/JWT contents, Stripe API keys, and webhook signing secrets.
- This is enforced structurally at the subscriber level — not a per-call-site convention.

### 4.3 Log Sink (v1)
- **Structured JSON logs to stdout**, paired with a lightweight Postgres logging table with a **GIN index on `trace_id`** for instant diagnostic lookup.
- Chosen to keep v1 infrastructure thin while allowing future routing to an external aggregation platform (Loki/Elastic) without application code changes.

### 4.4 Client-Facing Diagnostics
- On failure, `<ErrorBoundary>` displays the `trace_id` as an opaque, locale-invariant diagnostic code, wrapped in localized (Arabic/English) surrounding text via `leptos_i18n`.
- The `trace_id` is returned explicitly as a field in the failed server function's error response — never inferred or generated client-side — guaranteeing the ID the user sees matches the one in server logs.

---

## 5. Cross-Cutting Architectural Laws (Summary)

1. **UI granularity is semantic; domain logic granularity is maximal.**
2. **`core_types`** is the single source of truth for `TraceContext`, `LocaleContext`, `UserContext`, `TenantContext` — no duplication across middleware, Leptos context, or repository layers.
3. **Locale → Auth → Layout** is the fixed request pipeline order, system-wide.
4. **No raw SQLx queries** outside the Repository layer, anywhere in the codebase.
5. **RLS + Repository pattern** operate as independent, redundant tenant-isolation layers.
6. **Stripe is authoritative for billing; Postgres is authoritative for entitlement reads.** Only the webhook handler writes subscription state.
7. **Tailwind logical properties only** — physical directional utilities fail CI.
8. **Every request gets exactly one `trace_id`**, generated once, propagated everywhere, redacted before storage, surfaced to users as an opaque diagnostic code.

---

## 6. Multi-App Workspace Topology (Locked)

### 6.1 Workspace Layout
Single Rust workspace, 7 crates (scaffolded and verified — `cargo check --workspace` / `cargo test --workspace` pass):

| Crate | Type | Responsibility |
|---|---|---|
| `zafaf_core_types` | Library | `UserContext`, `TenantContext`, `LocaleContext`, `TraceContext`, shared domain enums |
| `zafaf_domain` | Library | Repository layer (RLS-aware), webhook handling, Stripe integration, business rules — the sole boundary where SQLx queries are permitted |
| `ci_gates` | Library + test | Hosts the `cargo test`-integrated RTL/LTR lint gate (§1.2) |
| `client_portal` | Leptos bin | Zero-auth, max-Islands, public browsing + Inquiry Form → `zafafworld.net` |
| `vendor_portal` | Leptos bin | Session-authed dashboard, analytics, subscription management → `vendor.zafafworld.net` |
| `admin_portal` | Leptos bin | Session-authed, high-privilege supervision → `admin.zafafworld.net` |
| `api_gateway` | Axum bin, zero-UI | Stripe webhook HMAC ingestion + notification job-table triggering, isolated from all 3 UI portals' deploy/traffic lifecycles → `api.zafafworld.net` |

**Resolution:** `zafaf_domain` is a shared library, not a gateway process. All 4 apps (`client_portal`, `vendor_portal`, `admin_portal`, `api_gateway`) compile and deploy as **4 fully independent binaries/processes**, guaranteeing that a public client-traffic spike, an admin action, or a Stripe webhook burst can never affect one another's availability.

### 6.1.1 Deployment Domain Topology (Locked)
| Domain | Binary | Notes |
|---|---|---|
| `zafafworld.net` | `client_portal` | Public, zero-auth, highest traffic, SEO-critical |
| `vendor.zafafworld.net` | `vendor_portal` | Session-authed |
| `admin.zafafworld.net` | `admin_portal` | Session-authed, high-privilege |
| `api.zafafworld.net` | `api_gateway` | Zero-UI. Stripe webhook ingestion + notification triggering only. **Locked.** |

### 6.2 Design Note — Why a 6th Crate Was Added
The original 5-crate proposal defined `zafaf_core_types` as types-only, leaving no shared home for repository implementations, RLS-aware queries, or webhook logic across three separate frontend crates. `zafaf_domain` was introduced specifically to preserve the "no raw SQLx outside the Repository layer" law across a multi-app topology.

### 6.3 Functional Flow Mapping
- **Client → Inquiry Form:** anonymous write; tenant scope resolved from the URL route (vendor slug/ID) rather than session — a documented exception to the standard `TenantContext` derivation rule, alongside the existing Stripe webhook exception (§3.3). Downstream notifications (email, WhatsApp) route through the same Postgres job-table worker pattern as Stripe webhooks, rather than a second async mechanism.
- **Vendor → Subscription tiers + Free Trial:** extends the existing local-first entitlement model (§3.2) with a `trial` value in `plan_tier`, still exclusively webhook-driven.
- **Vendor → Product Draft/Pending/Approved pipeline:** a `status` enum with role-gated transitions, enforced as an authorization rule inside `zafaf_domain`.
- **Admin → verification switchboard:** cross-tenant reads under an explicit, audited elevated-permission pattern — not a `TenantContext` bypass. RLS policies require an explicit admin-role carve-out to avoid a second, unaudited access path.

### 6.4 Resolutions (Locked)
1. `zafaf_domain` is a shared library; 4 independent deployment binaries (`client_portal`, `vendor_portal`, `admin_portal`, `api_gateway`). **Locked.**
2. `zafaf_domain` 7th-crate-total split (including `ci_gates`). **Locked.**
3. Notification delivery (email/WhatsApp) reuses the Module 4 Postgres job-table/worker pattern exclusively — concretely, the existing `notification_outbox` table (discovered in the reference schema audit, §8.5). **Locked.**
4. `api_gateway` deployed at `api.zafafworld.net` as the 4th deployment target for Stripe webhook ingestion + notification triggering, decoupled from the 3 UI portals. **Locked.**

---

## 7. Reference Metadata & Media Pipeline

### 7.1 Reference Metadata (Lookup Taxonomies)
- **Law:** all primary lookup taxonomies — `categories`, `venue_types`, `amenities`, `countries`, `cities` — are keyed by indexed UUID primary identifiers. String/slug matching is prohibited at the relational (foreign-key/join) boundary.
- A human-readable `slug` column may still exist per row for URL/SEO purposes, but it is display-only — never used as a join key or lookup key at the database boundary.

### 7.2 Object Storage — MinIO (S3-Compatible)
- **Decision:** self-hosted MinIO, accessed exclusively via `aws-sdk-s3` (async). This is the correct and most resilient path for this stack — no more-integrated native Rust alternative exists for this use case, and this choice keeps a clean migration path to managed S3 later without code changes.
- **Host path (`/var/lib/zafafworld/`)** is confirmed as MinIO's own backing data volume on the host. `zafaf_domain` and all three frontend crates are strictly prohibited from direct filesystem writes to this path — all application-to-storage connectivity is exclusively via `aws-sdk-s3`. **Locked.**

### 7.3 Asynchronous Image Optimization Pipeline
```
Vendor uploads raw image → stored to MinIO via presigned/direct upload
   → job enqueued in the existing Postgres job-table pattern (§3.3)
   → background Tokio worker:
       - generates WebP + AVIF variants
       - generates multi-scale responsive variants
       - writes resulting asset URLs back via zafaf_domain repository layer
   → client_portal serves optimized variants via <picture>/srcset, never the raw original
```
Reuses the same worker/job-table infrastructure locked for Stripe webhook processing — no second background-processing mechanism introduced.

### 7.4 Video — HLS Segmentation Pipeline
```
Vendor uploads raw video → stored to MinIO (raw, unpublished)
   → job enqueued in the same Postgres job-table pattern
   → background worker transcodes to HLS segments + .m3u8 master playlist
   → client_portal consumes via an HLS-compatible player, never serves the raw file
```
- Raw video files are never served directly to end users — only the segmented HLS output, satisfying the "no monolithic static video" requirement while reusing existing async infrastructure.

---

## 8. Database Schema & RLS Engine Specification (Locked)

Derived from a structural audit of the reference migration set (6 legacy migration files). Findings below reflect actual bugs and anti-patterns identified in that audit, corrected per locked directives.

### 8.1 Reference Metadata — Normalized to UUID
- `countries`: migrated from string PK (`'sa'`, `'ae'`, …) to a proper UUID PK. The former code is demoted to a unique, display-only `iso_code` column. Dependent tables (`cities.country_id`, `categories.available_countries`) are refactored to reference the UUID.
- `cities`, `districts`: already UUID-keyed — unchanged.
- `categories`: UUID PK added (was PK'd directly on `slug`). `slug` becomes a unique, non-PK column for SEO URLs only.
- `amenities` (new lookup table): UUID-keyed, replacing the legacy bare `text[]` on `vendors` and the previously orphaned, unjoined `features` catalog. A `vendor_amenities` / `product_amenities` join table connects vendors/products to this lookup.
- `vendor_products.product_category` → `category_id UUID REFERENCES categories(id)`. The legacy 26-value hardcoded `CHECK ... = ANY(ARRAY[...])` constraint is eliminated entirely — adding a category becomes a data insert, never a schema migration.

### 8.2 Tenant Identity — Standardized (Bug Fix)
**Finding:** the reference schema used `vendor_id` inconsistently as a FK to two different tables — `vendors.id` (business profile) in `vendor_gallery`/`vendor_products`/`vendor_wallets`, but `global_users.id` (login account) in `core_bookings`/`lead_inquiries`/`vendor_reviews`. These are distinct UUID spaces linked only via `vendors.user_id`.

**Correction (locked):** `vendor_id` means `vendors.id`, exclusively, everywhere in the schema. Every FK previously pointing at `global_users.id` under that column name is corrected. `TenantContext.tenant_id` always resolves to `vendors.id`.

### 8.3 RLS Session Variable — Direct, Not Recomputed Per Row
Reference policies resolved tenant ownership via a correlated subquery per check (`vendor_id = (SELECT vendors.id FROM vendors WHERE user_id = current_setting(...))`). Locked model: `app.current_tenant_id` is set directly from the already-resolved `TenantContext` at auth-middleware time; RLS policies compare `vendor_id = current_setting('app.current_tenant_id')::uuid` directly — same isolation guarantee, no per-row subquery cost.

### 8.4 Lead/Inquiry Consolidation
- `lead_inquiries` and `vendor_inquiries` (the genuine duplicate pair — same entity, overlapping columns/status vocab) are merged into a single `lead_inquiries` table with one canonical status enum. `vendor_inquiry_admin_notes` and `vendor_inquiry_management` are re-pointed at the consolidated table.
- `afrah_inquiries` (anonymous platform contact form — no vendor/client scope) and `assistant_inquiries` (client concierge channel — no vendor scope) remain **separate tables**, since they are structurally distinct entities, not duplicates of the vendor-inquiry flow.
- The zero-auth `client_portal` Inquiry Form (§6.3) writes to the consolidated `lead_inquiries`, with `tenant_id` derived via `TenantContext::from_url_route` — the one legitimate `UrlRouteDerived` write path in the schema.

### 8.5 Job-Table Models — Reused From Reference Schema
- `notification_outbox` (found in the reference audit, already matching the locked job-table pattern: status enum, retry count, `next_retry_at`, `channel_delivery` jsonb) is adopted as the single generic delivery queue for Inquiry notifications, Stripe-triggered notifications, and media-pipeline completion signals — no second queue mechanism introduced.
- `stripe_webhook_events` (per Module 4) remains a separate table for raw event storage + idempotency; successful processing enqueues into `notification_outbox` rather than dispatching directly.
- Image/video processing jobs (Module 7) get their own dedicated job table (distinct payload shape) but share the same worker-pool pattern.

### 8.6 Dropped From the Legacy Schema
- `csrf_tokens`: dropped entirely. Redundant given the locked `SameSite=Strict` opaque-session model (Module 2), which already mitigates CSRF for same-origin `#[server]` function calls.

### 8.7 Carried Forward As-Is
RLS policies for `client_budgets` (client-side wedding budget tracking — distinct from Module 9's vendor-facing financial ledger), conversations/messages, and blog content already follow the correct `tenant_id/user_id OR admin` shape from the reference audit and carry forward directly once `vendor_id` is standardized (§8.2). *(Note: `vendor_wallets`, `escrow_accounts`, `invoices`, and `payment_intents` — previously listed here — are removed; they belong exclusively to Module 9, which is out of scope for this generation pass.)*

---

### 8.8 Lookup Tables — Detailed Layout

**`countries`**
| Column | Type | Notes |
|---|---|---|
| `id` | UUID, PK | New surrogate key |
| `iso_code` | varchar(10), UNIQUE | Was the PK; demoted to display-only lookup (`'sa'`, `'ae'`, …) |
| `slug` | varchar(100), UNIQUE | SEO/URL only |
| `name_ar`, `name_en` | varchar(100) | |
| `currency` | varchar(10) | |
| `created_at` | timestamptz | |

**`cities`** — unchanged shape, FK corrected: `country_id UUID REFERENCES countries(id)` (was referencing the old varchar PK).

**`districts`** — unchanged (`city_id UUID REFERENCES cities(id)`), no anti-pattern found here.

**`categories`**
| Column | Type | Notes |
|---|---|---|
| `id` | UUID, PK | New surrogate key (was PK'd on `slug`) |
| `slug` | varchar(100), UNIQUE | Demoted — display/URL only, never a join key |
| `name_ar`, `name_en`, `description_ar`, `description_en` | text/varchar | unchanged |
| `parent_group` | `category_parent_group` enum | unchanged |
| `icon_name`, `emoji`, `priority`, `is_active`, `sort_order`, `launch_phase` | unchanged | |
| `available_countries` | **`uuid[]`** | Corrected per today's UUID Array Law — array of `countries.id`, not ISO code strings |

**`category_schemas`** — FK corrected: `category_id UUID REFERENCES categories(id)` (was `category_slug` string FK). All other columns (schema_json, searchable_fields, capacity_mode, etc.) unchanged.

**`amenities`** (new lookup table, replacing the orphaned `features` table + free-text `vendors.amenities[]`)
| Column | Type | Notes |
|---|---|---|
| `id` | UUID, PK | |
| `name_ar`, `name_en` | varchar | |
| `category_id` | UUID, REFERENCES `categories(id)` | Which category this amenity is relevant to (nullable — some amenities are cross-category, e.g. "valet parking") |
| `input_type` | varchar | Preserved from legacy `features.input_type` (`boolean`, etc.) |
| `created_at`, `updated_at` | timestamptz | |

**`vendor_amenities`** (new join table)
| Column | Type | Notes |
|---|---|---|
| `vendor_id` | UUID, REFERENCES `vendors(id)` | |
| `amenity_id` | UUID, REFERENCES `amenities(id)` | |
| `value` | jsonb (nullable) | For non-boolean amenity types (e.g. a numeric capacity) |
| PK | `(vendor_id, amenity_id)` composite | |

### 8.9 Unified Tenant Identity — Detailed Layout

**`global_users`** — carried forward largely unchanged from the reference schema (it was already well-formed): `id UUID PK`, `email` (lowercase-enforced), `password_hash`, `domain_type user_domain_enum` (`Client`/`Vendor`/`Admin`), `scopes varchar[]`, status/lockout fields, `token_valid_after` (supports instant revocation alongside the opaque session store). This table remains the single identity record regardless of portal.

**`vendors`** (this table *is* "tenant" — `TenantContext.tenant_id` always means `vendors.id`)
| Column | Type | Notes |
|---|---|---|
| `id` | UUID, PK | **The tenant identity.** Every other table's `vendor_id` FK points here, exclusively (§8.2 correction). |
| `user_id` | UUID, UNIQUE, REFERENCES `global_users(id)` | Login account — distinct from tenant identity. |
| `name_ar`, `name_en`, `slug` | unchanged | |
| `city_id` | UUID REFERENCES `cities(id)` | unchanged |
| `subscription_tier_id` | UUID REFERENCES `subscription_tiers(id)` | unchanged |
| `verification_level`, `status`, `is_verified`, `is_featured` | unchanged | |
| *(removed)* `category` varchar, `amenities` text[] | — | Replaced by `vendor_amenities` join (§8.8) and per-product `category_id` (§8.1) |

**`subscriptions`** (new — this is the local-first entitlement cache from MSDD §3.2, made concrete)
| Column | Type | Notes |
|---|---|---|
| `id` | UUID, PK | |
| `tenant_id` | UUID, UNIQUE, REFERENCES `vendors(id)` | One active subscription row per tenant |
| `stripe_customer_id` | varchar, nullable | Null for `trial` tier, which has no Stripe object yet |
| `stripe_subscription_id` | varchar, nullable | |
| `plan_tier` | enum: `trial`, `free`, `gold`, `vip`, `diamond` | `trial` added per the vendor Free Trial requirement (§6.3); mirrors legacy `subscription_tiers` naming |
| `status` | varchar | Mirrors Stripe subscription status vocabulary |
| `current_period_end` | timestamptz, nullable | |
| `policy_limits`, `features` | jsonb | Carried forward from legacy `subscription_tiers` (max_products, max_photos, etc.) |
| `updated_at` | timestamptz | **Written exclusively by the webhook handler** (§3.2/§8.5) — no other code path writes this row |

**`vendor_products`**
| Column | Type | Notes |
|---|---|---|
| `id` | UUID, PK | |
| `vendor_id` | UUID, REFERENCES `vendors(id)` | Standardized (§8.2) |
| `category_id` | UUID, REFERENCES `categories(id)` | Replaces `product_category` varchar + CHECK constraint (§8.1) |
| `city_id` | UUID, REFERENCES `cities(id)` | unchanged |
| `slug`, `title_ar`, `title_en`, `description_ar`, `description_en` | unchanged | |
| `status` | enum: `draft`, `pending_approval`, `active`, `rejected`, `suspended`, `archived` | unchanged, matches the Draft/Pending/Approved pipeline (§6.3) |
| `base_price_sar`, `price_on_inquiry`, `deposit_percentage` | unchanged | |
| `attributes`, `features_selection`, `cultural_attributes` | jsonb | unchanged — category-specific dynamic fields per `category_schemas` |
| `quality_score`, `total_capacity`, `searchable_amenities` | unchanged | (searchable_amenities remains a denormalized `text[]` cache for full-text search performance — read-only projection of `vendor_amenities`, not the source of truth) |

### 8.10 Consolidated Inquiries & Job Outbox — Detailed Layout

**`lead_inquiries`** (unified — merges legacy `lead_inquiries` + `vendor_inquiries`)
| Column | Type | Notes |
|---|---|---|
| `id` | UUID, PK | |
| `tenant_id` | UUID, REFERENCES `vendors(id)` | Standardized name/target (was `vendor_id` pointing at `global_users.id` in the legacy `lead_inquiries`) |
| `tenant_origin` | enum: `session_derived`, `url_route_derived` | Persists which `TenantOrigin` produced this row — auditable, matches `zafaf_core_types::TenantOrigin` |
| `client_id` | UUID, nullable, REFERENCES `global_users(id)` | Null for anonymous zero-auth submissions |
| `product_id` | UUID, nullable, REFERENCES `vendor_products(id)` | |
| `city_id` | UUID, REFERENCES `cities(id)` | |
| `customer_name`, `phone`, `email` | varchar, nullable | Required when `client_id` is null (anonymous path) |
| `wedding_date` / `event_date` | date | |
| `guest_count` | integer, nullable | From `vendor_inquiries` |
| `message` | text | |
| `conversation_id` | UUID, nullable | From `vendor_inquiries` — links to messaging thread once a vendor replies |
| `status` | unified enum: `new`, `viewed`, `pending`, `negotiation`, `replied`, `done`, `paid`, `rejected`, `declined`, `unreachable`, `expired`, `closed` | Superset of both legacy status vocabularies |
| `resolution_note` | text, nullable | |
| `created_at`, `updated_at` | timestamptz | |

`vendor_inquiry_admin_notes` and `vendor_inquiry_management` are re-pointed at `lead_inquiries.id` (column rename only, shape unchanged).

**`notification_outbox`** (adopted as-is from the reference audit — already correct)
| Column | Type | Notes |
|---|---|---|
| `id` | UUID, PK | |
| `event_type` | varchar | e.g. `inquiry.created`, `stripe.subscription_updated` |
| `aggregate_type`, `aggregate_id` | varchar / UUID | Polymorphic reference to the source row (a `lead_inquiries.id`, a `subscriptions.id`, etc.) |
| `payload` | jsonb | Channel-agnostic event data |
| `status` | enum: `PENDING`, `PROCESSING`, `DELIVERED`, `FAILED`, `RETRYING` | |
| `attempt_count`, `next_retry_at`, `last_attempt_at` | int / timestamptz | |
| `channel_delivery` | jsonb | Per-channel delivery status (email sent, WhatsApp sent, etc. — one row can fan out to multiple channels) |
| `error_message` | text, nullable | |

### 8.11 RLS Policy Pattern — Declarative Layout

Standard tenant-isolation policy shape, applied uniformly to every tenant-scoped table (`vendor_products`, `vendor_gallery`, `vendor_packages`, `vendor_amenities`, `lead_inquiries`, `subscriptions`, etc.):

```
CREATE POLICY <table>_tenant_isolation ON <table>
    USING (
        tenant_id = NULLIF(current_setting('app.current_tenant_id', true), '')::uuid
        OR current_setting('app.current_user_role', true) = 'admin'
    );
```

Key properties of this pattern (locked, §8.3):
- **Direct comparison, no subquery.** `app.current_tenant_id` is set once per session/request from the already-resolved `TenantContext` — the policy never re-derives tenant identity from `global_users` at check time.
- **`NULLIF(..., '')` guard** preserves the legacy pattern's safe handling of an unset session variable (evaluates to `NULL`, which fails the comparison closed rather than throwing a cast error).
- **Admin carve-out is explicit and separate** — a second condition, not a bypass of the tenant check — so admin cross-tenant access is always visibly audited as its own clause, never silently implied.
- **Public-read tables** (`categories`, `amenities`, `countries`, `cities`, `subscription_tiers`) use a simpler `FOR SELECT USING (true)` policy paired with an admin-only modify policy — matching the legacy `catalog_select_*` / `catalog_modify_*` pattern, which was already correct.
- Anonymous zero-auth inserts (client Inquiry Form → `lead_inquiries`) require a distinct `FOR INSERT WITH CHECK (true)` policy, since no `app.current_tenant_id` exists for that session — the tenant scoping there happens at the application layer via `TenantContext::from_url_route`, not at the RLS layer. This is the one deliberate, documented gap between RLS and Repository-layer enforcement, consistent with the `UrlRouteDerived` exception already locked.

---

## 9. Escrow, Financial Ledger & Transaction Trust Engine (Opened — Not Yet Designed)

Formally decoupled from Module 4 (which covers only vendor *subscription* billing via Stripe Checkout Sessions). This module scopes the marketplace-side financial infrastructure discovered in the reference audit: `escrow_accounts`, `payment_intents`, `invoices`, `payout_requests`, `vendor_wallets`.

This is materially different from Module 4's billing model — it involves holding client funds in escrow against a booking, releasing to vendor wallets, and paying vendors out — which likely requires Stripe Connect (a different primitive than Checkout Sessions) rather than simple subscription billing, plus its own Zero-Trust posture around money movement (dispute handling, payout authorization, ledger integrity).

**Status:** scope acknowledged and isolated from Module 4; full conceptual design not yet drafted. Awaiting direction to begin Module 9 intake.

---

## Open Items Carried Forward (Not Yet Decided)

None blocking Modules 1–8 — all resolved and locked.

**Module 9** (Escrow, Financial Ledger & Transaction Trust Engine) is opened but not yet designed — awaiting intake direction.

---

*This document is the authoritative architectural reference for scaffolding. Modules 1–8 are locked; Phase 1 workspace scaffolding is complete and verified (`cargo check --workspace`, `cargo test --workspace` passing). Awaiting either Module 9 intake or the next scaffolding phase directive (e.g., database migration files reflecting §8).*
