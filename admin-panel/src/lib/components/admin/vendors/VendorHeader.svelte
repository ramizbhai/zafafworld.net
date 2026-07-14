<script lang="ts">
    import { CheckCircle, MapPin } from "lucide-svelte";
    import { getStatusColor } from "$lib/utils/adminFormatter.js";

    let { vendor, productsCount } = $props<{ vendor: any, productsCount: number }>();
</script>

<div class="vendor-header">
    <div class="vendor-avatar">
        {vendor.name_en?.[0] || 'V'}
    </div>
    <div class="vendor-headline">
        <div class="name-row">
            <h1>{vendor.name_en}</h1>
            {#if vendor.is_verified}
                <span class="verified-chip">
                    <CheckCircle size={12} /> Verified
                </span>
            {/if}
            {#if vendor.is_featured}
                <span class="featured-chip">⭐ Featured</span>
            {/if}
        </div>
        <p class="name-ar" dir="rtl">{vendor.name_ar}</p>
        <div class="meta-chips">
            <span class="chip chip-category">{vendor.category}</span>
            <span
                class="chip"
                style="background:rgba(0,0,0,0.2);color:{getStatusColor(
                    vendor.status,
                )};border-color:{getStatusColor(vendor.status)}40"
            >
                {vendor.status}
            </span>
            <span class="chip chip-sub">{vendor.subscription_status}</span>
            <span class="chip chip-city">
                <MapPin size={11} /> {vendor.city_name_en}
            </span>
        </div>
    </div>
    <div class="vendor-stats">
        <div class="stat-box">
            <span class="stat-val">{vendor.review_count}</span>
            <span class="stat-label">Reviews</span>
        </div>
        <div class="stat-box">
            <span class="stat-val">
                {vendor.avg_rating?.toFixed(1) ?? "—"}
            </span>
            <span class="stat-label">Avg Rating</span>
        </div>
        <div class="stat-box">
            <span class="stat-val">{productsCount}</span>
            <span class="stat-label">Listings</span>
        </div>
    </div>
</div>

<style>
    .vendor-header {
        display: flex;
        align-items: flex-start;
        gap: 20px;
        background: var(--bg-elevated);
        border: 1px solid var(--glass-border);
        border-radius: 16px;
        padding: 24px;
    }
    .vendor-avatar {
        width: 64px;
        height: 64px;
        border-radius: 16px;
        background: linear-gradient(135deg, #a855f7, #7c3aed);
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 1.8rem;
        font-weight: 800;
        color: #fff;
        flex-shrink: 0;
    }
    .vendor-headline {
        flex: 1;
        min-width: 0;
    }
    .name-row {
        display: flex;
        align-items: center;
        gap: 10px;
        flex-wrap: wrap;
        margin-bottom: 2px;
    }
    .name-row h1 {
        font-size: 1.5rem;
        font-weight: 800;
        color: var(--text-primary);
        margin: 0;
        letter-spacing: -0.3px;
    }
    .verified-chip {
        display: inline-flex;
        align-items: center;
        gap: 4px;
        font-size: 0.65rem;
        font-weight: 700;
        padding: 2px 8px;
        border-radius: 20px;
        background: rgba(16, 185, 129, 0.15);
        color: #34d399;
        border: 1px solid rgba(16, 185, 129, 0.25);
    }
    .featured-chip {
        font-size: 0.65rem;
        font-weight: 700;
        padding: 2px 8px;
        border-radius: 20px;
        background: rgba(251, 191, 36, 0.15);
        color: #fbbf24;
        border: 1px solid rgba(251, 191, 36, 0.25);
    }
    .name-ar {
        font-size: 0.9rem;
        color: #94a3b8;
        margin: 2px 0 10px;
        font-family: system-ui;
    }
    .meta-chips {
        display: flex;
        flex-wrap: wrap;
        gap: 6px;
    }
    .chip {
        display: inline-flex;
        align-items: center;
        gap: 4px;
        font-size: 0.65rem;
        font-weight: 700;
        padding: 3px 9px;
        border-radius: 6px;
        border: 1px solid transparent;
        text-transform: capitalize;
    }
    .chip-category {
        background: rgba(168, 85, 247, 0.1);
        color: #c084fc;
        border-color: rgba(168, 85, 247, 0.2);
    }
    .chip-sub {
        background: rgba(59, 130, 246, 0.1);
        color: #60a5fa;
        border-color: rgba(59, 130, 246, 0.2);
    }
    .chip-city {
        background: rgba(100, 116, 139, 0.1);
        color: #94a3b8;
        border-color: rgba(100, 116, 139, 0.15);
    }
    .vendor-stats {
        display: flex;
        gap: 16px;
        flex-shrink: 0;
    }
    .stat-box {
        display: flex;
        flex-direction: column;
        align-items: center;
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.06);
        border-radius: 10px;
        padding: 12px 20px;
        min-width: 72px;
    }
    .stat-val {
        font-size: 1.4rem;
        font-weight: 800;
        color: var(--text-primary);
    }
    .stat-label {
        font-size: 0.65rem;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        color: #64748b;
        font-weight: 600;
    }

    @media (max-width: 900px) {
        .vendor-header {
            flex-direction: column;
        }
        .vendor-stats {
            flex-direction: row;
        }
    }
</style>
