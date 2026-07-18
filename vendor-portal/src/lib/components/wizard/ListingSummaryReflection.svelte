<script lang="ts">
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import { listingStore } from '$lib/stores/listingStore';
    import { page } from '$app/stores';
    import { MapPin, Link as LinkIcon, Lock, ChevronRight, ChevronLeft } from 'lucide-svelte';

    const i18n = getI18n();

    // Pull from page.data.cities
    const cities = $derived($page.data.cities || []);
    const selectedCity = $derived(cities.find((c: any) => c.id === $listingStore.formData.selectedCityId));
    
    const cityName = $derived(
        selectedCity 
            ? (i18n.locale === 'ar' ? (selectedCity.name_ar || selectedCity.nameAr || "") : (selectedCity.name_en || selectedCity.nameEn || ""))
            : ""
    );

    const title = $derived(
        i18n.locale === 'ar' 
            ? ($listingStore.formData.titleAr || $listingStore.formData.titleEn || "")
            : ($listingStore.formData.titleEn || $listingStore.formData.titleAr || "")
    );

    const step2Path = $derived(
        $page.url.pathname.includes('/step-')
            ? $page.url.pathname.split("/step-")[0] + "/step-2"
            : ""
    );
</script>

<div class="summary-reflection-card">
    <div class="card-header">
        <div class="title-lock-group">
            <Lock size={12} class="lock-icon" />
            <h4 class="card-title">{title || (i18n.locale === 'ar' ? 'بدون عنوان' : 'Untitled Listing')}</h4>
        </div>
        {#if step2Path}
            <a href={step2Path} class="edit-link">
                <span>{i18n.locale === 'ar' ? 'تعديل في خطوة 2' : 'Edit in step 2'}</span>
                {#if i18n.locale === 'ar'}
                    <ChevronLeft size={12} class="arrow-icon" />
                {:else}
                    <ChevronRight size={12} class="arrow-icon" />
                {/if}
            </a>
        {/if}
    </div>
    <div class="card-body">
        {#if cityName}
            <div class="info-item">
                <MapPin size={14} class="info-icon" />
                <span class="info-label">{i18n.locale === 'ar' ? 'المدينة:' : 'City:'}</span>
                <span class="info-pill">{cityName}</span>
            </div>
        {/if}
        {#if $listingStore.formData.googleMapsUrl}
            <div class="info-item">
                <LinkIcon size={14} class="info-icon" />
                <span class="info-label">{i18n.locale === 'ar' ? 'رابط الخريطة:' : 'Google Maps:'}</span>
                <span class="info-pill maps-pill">
                    <a href={$listingStore.formData.googleMapsUrl} target="_blank" rel="noopener noreferrer" class="maps-link">
                        {$listingStore.formData.googleMapsUrl}
                    </a>
                </span>
            </div>
        {/if}
    </div>
</div>

<style>
    .summary-reflection-card {
        background: #f3f4f6;
        border: 1px solid var(--border, #e5e7eb);
        border-radius: 12px;
        padding: 16px 20px;
        margin-bottom: 24px;
    }
    .card-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 12px;
        flex-wrap: wrap;
        gap: 8px;
    }
    .title-lock-group {
        display: flex;
        align-items: center;
        gap: 6px;
    }
    .card-title {
        font-size: 0.85rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: var(--text-muted, #6b7280);
        margin: 0;
    }
    .edit-link {
        display: flex;
        align-items: center;
        gap: 2px;
        font-size: 0.75rem;
        color: var(--primary, #6c3fa0);
        text-decoration: none;
        font-weight: 600;
        transition: opacity 0.15s;
    }
    .edit-link:hover {
        opacity: 0.8;
        text-decoration: underline;
    }
    .card-body {
        display: flex;
        flex-direction: row;
        flex-wrap: wrap;
        gap: 16px;
    }
    @media (max-width: 640px) {
        .card-body {
            flex-direction: column;
            gap: 8px;
        }
    }
    .info-item {
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 0.85rem;
        color: var(--text, #1a1a2e);
        min-width: 0;
    }
    .info-label {
        font-weight: 500;
        color: var(--text-muted, #6b7280);
        flex-shrink: 0;
    }
    .info-pill {
        background: white;
        border: 1px solid var(--border, #e5e7eb);
        border-radius: 9999px;
        padding: 2px 10px;
        font-size: 0.8rem;
        font-weight: 600;
        color: var(--text, #1a1a2e);
        display: inline-flex;
        max-width: 100%;
        box-shadow: 0 1px 2px rgba(0, 0, 0, 0.02);
    }
    .maps-pill {
        padding: 2px 10px;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        display: inline-block;
    }
    .maps-link {
        color: var(--primary, #6c3fa0);
        text-decoration: none;
    }
    .maps-link:hover {
        text-decoration: underline;
    }
</style>
