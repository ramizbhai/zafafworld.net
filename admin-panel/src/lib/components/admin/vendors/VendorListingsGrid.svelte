<script lang="ts">
    import { Building2, Package, CheckCircle, XCircle } from "lucide-svelte";
    import { enhance } from "$app/forms";
    import {
        getStatusColor,
        genderLabel,
        capacity,
        categoryLabel,
        isVenueListing,
        formatDate,
    } from "$lib/utils/adminFormatter.js";
    import { RBACService, type User } from "../../../../core/auth/rbac.service.js";
    import type { VendorAdminState } from "../../../../core/stores/adminState.svelte.js";

    let { products, user, state } = $props<{ products: any[], user: User, state: VendorAdminState }>();
</script>

<div class="halls-section">
    <div class="halls-header">
        <div>
            <h3 class="section-title">
                <Package size={16} /> Listings
            </h3>
            <p class="section-sub">
                {products.length} listing{products.length !== 1 ? "s" : ""} configured
            </p>
        </div>
        <div class="hall-legend">
            <span class="legend-item"><span class="legend-dot" style="background:#10b981"></span>Active</span>
            <span class="legend-item"><span class="legend-dot" style="background:#f59e0b"></span>Pending</span>
            <span class="legend-item"><span class="legend-dot" style="background:#ef4444"></span>Suspended</span>
        </div>
    </div>

    {#if products.length === 0}
        <div class="no-halls">
            <Building2 size={32} />
            <p>No listings have been created by this vendor yet.</p>
        </div>
    {:else}
        <div class="halls-list">
            {#each products as product}
                <div class="hall-card">
                    <div class="hall-header">
                        <div class="hall-title-group">
                            <span
                                class="hall-status-dot"
                                style="background:{getStatusColor(product.status)}"
                            ></span>
                            <div>
                                <h4 class="hall-name">{product.title}</h4>
                            </div>
                        </div>
                        <div class="hall-badges">
                            {#if product.attributes?.gender_section}
                                <span class="badge badge-gender">{genderLabel(product.attributes.gender_section)}</span>
                            {:else}
                                <span class="badge badge-service">✂️ Service</span>
                            {/if}
                            <span
                                class="badge"
                                style="color:{getStatusColor(product.status)};border-color:{getStatusColor(product.status)}40;background:{getStatusColor(product.status)}10"
                            >
                                {product.status}
                            </span>
                        </div>
                    </div>

                    <div class="hall-meta-grid">
                        {#if isVenueListing(product)}
                            <div class="hall-meta-item">
                                <span class="meta-label">Gender Section</span>
                                <span class="meta-val">{genderLabel(product.attributes?.gender_section)}</span>
                            </div>
                            <div class="hall-meta-item">
                                <span class="meta-label">Capacity</span>
                                <span class="meta-val">👥 {capacity(product)}</span>
                            </div>
                        {/if}
                        {#if product.base_price_sar}
                            <div class="hall-meta-item">
                                <span class="meta-label">Base Price</span>
                                <span class="meta-val price-val">
                                    {Math.round(product.base_price_sar).toLocaleString()} SAR
                                </span>
                            </div>
                        {/if}
                        {#if product.attributes?.area_sqm && isVenueListing(product)}
                            <div class="hall-meta-item">
                                <span class="meta-label">Area</span>
                                <span class="meta-val">📐 {product.attributes.area_sqm} m²</span>
                            </div>
                        {/if}
                        <div class="hall-meta-item">
                            <span class="meta-label">Type</span>
                            <span class="meta-val listing-type" class:is-venue={isVenueListing(product)}>
                                {isVenueListing(product) ? "🏛️ Venue" : "✂️ Service"}
                            </span>
                        </div>
                        <div class="hall-meta-item">
                            <span class="meta-label">Category</span>
                            <span class="meta-val">{categoryLabel(product.product_category)}</span>
                        </div>
                        <div class="hall-meta-item">
                            <span class="meta-label">Available</span>
                            <span class="meta-val">{product.is_available ? "✅ Yes" : "❌ No"}</span>
                        </div>
                        <div class="hall-meta-item">
                            <span class="meta-label">Added</span>
                            <span class="meta-val">{formatDate(product.created_at)}</span>
                        </div>
                    </div>

                    {#if product.description}
                        <p class="hall-desc">{product.description}</p>
                    {/if}

                    {#if RBACService.canModerateListings(user)}
                        <!-- Hall moderation actions -->
                        <form
                            method="POST"
                            action="?/updateProductStatus"
                            use:enhance={() => {
                                state.isSubmittingProduct = product.id;
                                return async ({ update }) => {
                                    state.isSubmittingProduct = null;
                                    await update();
                                };
                            }}
                            class="hall-actions"
                        >
                            <input type="hidden" name="product_id" value={product.id} />
                            <input type="hidden" name="reason" value={state.productReason} />

                            {#if product.status !== "active"}
                                <button
                                    type="submit"
                                    name="status"
                                    value="active"
                                    class="hall-btn hall-btn-approve"
                                    disabled={state.isSubmittingProduct === product.id}
                                >
                                    <CheckCircle size={12} />
                                    {state.isSubmittingProduct === product.id ? "Activating…" : "Activate"}
                                </button>
                            {/if}

                            {#if product.status !== "suspended"}
                                <button
                                    type="submit"
                                    name="status"
                                    value="suspended"
                                    class="hall-btn hall-btn-suspend"
                                    disabled={state.isSubmittingProduct === product.id}
                                >
                                    <XCircle size={12} />
                                    {state.isSubmittingProduct === product.id ? "Suspending…" : "Suspend"}
                                </button>
                            {/if}

                            {#if product.status !== "archived"}
                                <button
                                    type="submit"
                                    name="status"
                                    value="archived"
                                    class="hall-btn hall-btn-archive"
                                    disabled={state.isSubmittingProduct === product.id}
                                >
                                    Archive
                                </button>
                            {/if}

                            {#if product.status === "active" && RBACService.canPromoteListing(user)}
                                <button
                                    type="button"
                                    class="hall-btn hall-btn-promote"
                                    style="background: var(--gold-subtle); color: var(--gold-deep); border-color: var(--gold-border);"
                                    onclick={(e) => {
                                        e.preventDefault();
                                        state.openPromoteModal(product);
                                    }}
                                >
                                    ⭐ Promote
                                </button>
                            {/if}
                        </form>
                    {/if}
                </div>
            {/each}
        </div>
    {/if}
</div>

<style>
    .halls-header {
        display: flex;
        align-items: flex-start;
        justify-content: space-between;
        gap: 12px;
        margin-bottom: 16px;
        padding-bottom: 14px;
        border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    }
    .section-title {
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 0.95rem;
        font-weight: 700;
        color: var(--text-primary);
        margin: 0;
    }
    .section-sub {
        font-size: 0.75rem;
        color: #64748b;
        margin: 4px 0 0;
    }
    .hall-legend {
        display: flex;
        gap: 12px;
        align-items: center;
        flex-shrink: 0;
    }
    .legend-item {
        display: flex;
        align-items: center;
        gap: 5px;
        font-size: 0.68rem;
        color: #64748b;
        font-weight: 600;
    }
    .legend-dot {
        width: 7px;
        height: 7px;
        border-radius: 50%;
    }
    .no-halls {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 10px;
        padding: 2rem;
        text-align: center;
        color: #475569;
    }
    .no-halls p {
        margin: 0;
        font-size: 0.85rem;
    }
    .halls-list {
        display: flex;
        flex-direction: column;
        gap: 14px;
    }
    .hall-card {
        background: var(--bg-raised);
        border: 1px solid var(--glass-border);
        border-radius: 12px;
        padding: 16px;
        display: flex;
        flex-direction: column;
        gap: 12px;
        transition: border-color 0.2s;
    }
    .hall-card:hover {
        border-color: rgba(168, 85, 247, 0.2);
    }
    .hall-header {
        display: flex;
        align-items: flex-start;
        justify-content: space-between;
        gap: 12px;
    }
    .hall-title-group {
        display: flex;
        align-items: flex-start;
        gap: 10px;
    }
    .hall-status-dot {
        width: 9px;
        height: 9px;
        border-radius: 50%;
        flex-shrink: 0;
        margin-top: 5px;
    }
    .hall-name {
        font-size: 0.9rem;
        font-weight: 700;
        color: var(--text-primary);
        margin: 0;
    }
    .hall-badges {
        display: flex;
        flex-wrap: wrap;
        gap: 5px;
    }
    .badge {
        display: inline-block;
        font-size: 0.6rem;
        font-weight: 700;
        padding: 2px 7px;
        border-radius: 5px;
        border: 1px solid;
        text-transform: capitalize;
        white-space: nowrap;
    }
    .badge-gender {
        background: rgba(168, 85, 247, 0.1);
        color: #c084fc;
        border-color: rgba(168, 85, 247, 0.2);
    }
    .badge-service {
        background: rgba(34, 197, 94, 0.08);
        color: #4ade80;
        border-color: rgba(34, 197, 94, 0.2);
    }
    .listing-type {
        font-size: 0.78rem;
    }
    .listing-type.is-venue {
        color: #c084fc;
    }
    .hall-meta-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
        gap: 8px;
    }
    .hall-meta-item {
        display: flex;
        flex-direction: column;
        gap: 2px;
    }
    .meta-label {
        font-size: 0.62rem;
        text-transform: uppercase;
        letter-spacing: 0.4px;
        color: #475569;
        font-weight: 700;
    }
    .meta-val {
        font-size: 0.8rem;
        font-weight: 600;
        color: #e2e8f0;
    }
    .price-val {
        color: #a855f7;
    }
    .hall-desc {
        font-size: 0.78rem;
        color: #64748b;
        line-height: 1.5;
        margin: 0;
        padding-top: 8px;
        border-top: 1px solid rgba(255, 255, 255, 0.05);
    }
    .hall-actions {
        display: flex;
        gap: 8px;
        flex-wrap: wrap;
        padding-top: 8px;
        border-top: 1px solid rgba(255, 255, 255, 0.05);
    }
    .hall-btn {
        display: inline-flex;
        align-items: center;
        gap: 5px;
        font-size: 0.72rem;
        font-weight: 700;
        padding: 5px 12px;
        border-radius: 7px;
        border: 1px solid transparent;
        cursor: pointer;
        transition: all 0.15s;
    }
    .hall-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }
    .hall-btn-approve {
        background: rgba(16, 185, 129, 0.1);
        color: #34d399;
        border-color: rgba(16, 185, 129, 0.2);
    }
    .hall-btn-approve:hover:not(:disabled) {
        background: rgba(16, 185, 129, 0.2);
    }
    .hall-btn-suspend {
        background: rgba(239, 68, 68, 0.1);
        color: #f87171;
        border-color: rgba(239, 68, 68, 0.2);
    }
    .hall-btn-suspend:hover:not(:disabled) {
        background: rgba(239, 68, 68, 0.2);
    }
    .hall-btn-archive {
        background: rgba(100, 116, 139, 0.1);
        color: #94a3b8;
        border-color: rgba(100, 116, 139, 0.2);
    }
    .hall-btn-archive:hover:not(:disabled) {
        background: rgba(100, 116, 139, 0.2);
    }
</style>
