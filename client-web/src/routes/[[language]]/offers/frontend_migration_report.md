# Frontend Migration Report: Listing Promotion System

## 1. Migration Overview

The frontend codebase has been successfully migrated from the legacy `vendor_offers` system to the new, production-ready **Listing Promotion System**. All frontend connections to the old, unmoderated database tables have been decommissioned and replaced by connections to the new system, which acts as the Single Source of Truth (SSOT).

This transition has been accomplished without any alterations to the database schema, core business rules, or the backend promotion lifecycle.

---

## 2. Component Migration Summary

### 2.1 Vendor Portal (`vendor-portal`)
* **API Redirection:** Replaced `/api/v1/vendor/offers` with `/api/v1/vendor/promotions` for all CRUD operations.
* **Target Listings Selection:** Replaced the un-targeted single field layout with a multi-select checkbox list that pulls the vendor's active listings via `/api/v1/vendor/products`, enforcing that at least one listing must be targeted.
* **Promotion Schedule:** Replaced the single `valid_until` date with two `datetime-local` selectors (`start_at` and `end_at`), validating that `end_at > start_at`.
* **State Operations:** Integrated action triggers for `Pause`, `Resume`, and `Duplicate` linking directly to `/api/v1/vendor/promotions/:id/pause`, `/resume`, and `/duplicate` respectively.
* **Badges & Inclusions:** Added input fields for localized descriptions and ribbon text (`badge_text_en`/`badge_text_ar`) to display special promotions labels.
* **Validation & Overlap Errors:** Implemented validation for discount range (5% - 90%) and mapped HTTP 409 database overlap warnings to user-facing alerts.

### 2.2 Admin Panel (`admin-panel`)
* **API Redirection:** Replaced `/api/v1/admin/marketing/campaigns` with `/api/v1/admin/promotions`.
* **Pending Queue:** Transformed the read-only campaigns board into an interactive moderation queue, allowing admins to filter promotions by status (Pending, Approved, Rejected).
* **Approve / Reject Dialogs:** Added inline moderation buttons. Rejecting a campaign triggers a modal asking for a **Rejection Reason**, which is submitted to `/api/v1/admin/promotions/:id/reject`.
* **Bulk Moderation:** Implemented check-boxes on table rows allowing admins to perform **Bulk Approve** and **Bulk Reject** (requesting a bulk reason) in parallel loops.
* **Target Listings Preview:** Rendered the targeted listing UUIDs in a detailed details modal for every promotion, using the newly added backend payload `listing_ids`.

### 2.3 Client Website (`client-web`)
* **API Redirection:** Connected listings catalog loaders to `/api/v1/public/promotions`.
* **Promotional Ribbon & Price Slasher:** Modified `ListingCard.svelte` to read matched active promotions, displaying a localized ribbon tag on the cover image and the original price slashed next to the new discounted price.
* **Offers Catalog Page (`/offers`):** Created a premium landing page listing all currently active promotions, sorted by vendor subscription rank.
* **Promotion Detail Page (`/offers/[id]`):** Created a detail view featuring banner images, localized description panels, real-time countdown clocks (days, hours, minutes, seconds), and cards of the targeted halls/services.
