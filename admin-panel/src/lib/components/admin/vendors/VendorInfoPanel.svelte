<script lang="ts">
    import { User, Mail, Phone, Globe, MapPin, Activity, Clock } from "lucide-svelte";
    import { formatDate } from "$lib/utils/adminFormatter.js";

    let { vendor } = $props<{ vendor: any }>();
</script>

<!-- Contact Info -->
<div class="info-card">
    <h3 class="card-title">
        <User size={15} /> Contact Information
    </h3>
    <div class="info-list">
        {#if vendor.email}
            <div class="info-row">
                <Mail size={14} /><span>{vendor.email}</span>
            </div>
        {/if}
        {#if vendor.phone}
            <div class="info-row">
                <Phone size={14} /><span>{vendor.phone}</span>
            </div>
        {/if}
        {#if vendor.website}
            <div class="info-row">
                <Globe size={14} />
                <a href={vendor.website} target="_blank" rel="noreferrer">
                    {vendor.website}
                </a>
            </div>
        {/if}
        {#if vendor.address_en}
            <div class="info-row">
                <MapPin size={14} /><span>{vendor.address_en}</span>
            </div>
        {/if}
    </div>
</div>

<!-- Description -->
{#if vendor.description_en}
    <div class="info-card">
        <h3 class="card-title"><Activity size={15} /> About</h3>
        <p class="description-text">{vendor.description_en}</p>
        {#if vendor.description_ar}
            <p class="description-text" dir="rtl">
                {vendor.description_ar}
            </p>
        {/if}
    </div>
{/if}

<!-- Timestamps -->
<div class="info-card">
    <h3 class="card-title"><Clock size={15} /> Timeline</h3>
    <div class="info-list">
        <div class="info-row">
            <span class="info-label">Registered:</span>
            <span>{formatDate(vendor.created_at)}</span>
        </div>
        <div class="info-row">
            <span class="info-label">Last Updated:</span>
            <span>{formatDate(vendor.updated_at)}</span>
        </div>
        <div class="info-row">
            <span class="info-label">Star Rating:</span>
            <span>
                {vendor.star_rating ? "⭐".repeat(vendor.star_rating) : "—"}
            </span>
        </div>
    </div>
</div>

<style>
    .info-card {
        background: var(--bg-elevated);
        border: 1px solid var(--glass-border);
        border-radius: 14px;
        padding: 18px;
    }
    .card-title {
        display: flex;
        align-items: center;
        gap: 7px;
        font-size: 0.82rem;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        color: #94a3b8;
        margin: 0 0 14px;
    }
    .info-list {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }
    .info-row {
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 0.82rem;
        color: #e2e8f0;
    }
    .info-row :global(svg) {
        color: #64748b;
        flex-shrink: 0;
    }
    .info-row a {
        color: #818cf8;
        text-decoration: none;
    }
    .info-row a:hover {
        text-decoration: underline;
    }
    .info-label {
        color: #64748b;
        font-weight: 600;
        min-width: 100px;
    }
    .description-text {
        font-size: 0.82rem;
        color: #94a3b8;
        line-height: 1.55;
        margin: 0 0 8px;
    }
    .description-text:last-child {
        margin-bottom: 0;
    }
</style>
