<script lang="ts">
    import { getI18n } from "$lib/i18n/i18n.svelte";
    import { GENDER_SECTIONS, CATEGORY_GROUPS } from "$lib/constants/wizard";
    import { getApiUrl } from "$lib/utils/api";
    import { listingStore } from "$lib/stores/listingStore";

    let { data, checklist } = $props<{ data: { cities: any[] }, checklist: { label: string, ok: boolean }[] }>();
    const i18n = getI18n();

    const fd = $derived($listingStore.formData);
    const coverItem = $derived($listingStore.formData.coverItem);

    const selectedCategoryMeta = $derived.by(() => {
        const cat = fd.selectedCategory;
        for (const group of CATEGORY_GROUPS) {
            const found = group.items.find((i: any) => i.value === cat);
            if (found) return found;
        }
        return null;
    });
</script>

<div class="preview-grid-layout">
    <!-- Main Content: Storefront Details (Left Side) -->
    <div class="preview-storefront">
        {#if coverItem && coverItem.status === "completed"}
            <img
                src={coverItem.url.startsWith("http")
                    ? coverItem.url
                    : getApiUrl(coverItem.url)}
                alt="Cover"
                class="preview-banner"
            />
        {:else}
            <div class="preview-placeholder-banner">
                📸
            </div>
        {/if}

        <div class="preview-details">
            <span class="preview-cat-badge">
                {selectedCategoryMeta?.emoji || "🏷️"}
                {i18n.locale === "ar"
                    ? selectedCategoryMeta?.ar
                    : selectedCategoryMeta?.en}
            </span>

            <h1 class="text-2xl font-bold text-gray-900 mb-1">{fd.titleEn || "—"}</h1>
            <h2 class="text-xl font-semibold text-gray-700 mb-4">{fd.titleAr || "—"}</h2>

            <div class="flex gap-4 flex-wrap text-sm text-gray-500 mb-6">
                {#if fd.selectedCityId}
                    <div class="flex items-center gap-1">
                        📍 {data.cities?.find((c: any) => c.id === fd.selectedCityId)?.[i18n.locale === 'ar' ? 'name_ar' : 'name_en'] || "Selected City"}
                    </div>
                {/if}
                {#if fd.genderSection}
                    <div class="flex items-center gap-1">
                        {GENDER_SECTIONS.find((g) => g.value === fd.genderSection)?.icon || "⚧"}
                        {i18n.locale === "ar"
                            ? GENDER_SECTIONS.find((g) => g.value === fd.genderSection)?.ar
                            : GENDER_SECTIONS.find((g) => g.value === fd.genderSection)?.en}
                    </div>
                {/if}
            </div>

            <!-- Arabic & English Descriptions -->
            {#if fd.descriptionAr}
                <div class="preview-desc-box">
                    <h4>الوصف العربي</h4>
                    <div class="preview-desc-text">{fd.descriptionAr}</div>
                </div>
            {/if}

            {#if fd.descriptionEn}
                <div class="preview-desc-box">
                    <h4>English Description</h4>
                    <div class="preview-desc-text">{fd.descriptionEn}</div>
                </div>
            {/if}

            <!-- Gallery Photos Preview -->
            {#if fd.galleryItems && fd.galleryItems.length > 0}
                <div class="gallery-preview-section border-t border-gray-100 pt-6">
                    <h4 class="text-sm font-bold text-gray-900 mb-3">Gallery Photos</h4>
                    <div class="gallery-preview-grid">
                        {#each fd.galleryItems as item}
                            <div class="gallery-preview-item">
                                {#if item.mediaType === 'video'}
                                    <video src={item.url.startsWith("http") ? item.url : getApiUrl(item.url)} class="w-full h-full object-cover"></video>
                                {:else}
                                    <img
                                        src={item.url.startsWith("http")
                                            ? item.url
                                            : getApiUrl(item.url)}
                                        alt="Gallery Thumb"
                                    />
                                {/if}
                            </div>
                        {/each}
                    </div>
                </div>
            {/if}
        </div>
    </div>

    <!-- Sidebar Widgets (Right Side) -->
    <div class="preview-sidebar">
        <!-- Price Card -->
        <div class="sidebar-widget price-widget">
            <h4 class="widget-title">{i18n.locale === "ar" ? "السعر الأساسي" : "Base Pricing"}</h4>
            {#if !fd.priceOnInquiry && fd.basePriceSar}
                <div class="widget-value">
                    ﷼{parseInt(fd.basePriceSar).toLocaleString(i18n.locale === "ar" ? "ar-SA" : "en-SA")}
                    <span class="text-xs text-gray-500 font-normal">SAR</span>
                </div>
            {:else}
                <div class="widget-value inquiry">
                    {i18n.t("listingsWizard.priceOnInquiry") || "Price on Inquiry"}
                </div>
            {/if}
        </div>

        <!-- Coordinator Card -->
        <div class="sidebar-widget">
            <h4 class="widget-title">{i18n.locale === "ar" ? "منسق الخدمة" : "Service Coordinator"}</h4>
            {#if fd.coordinatorNameEn || fd.coordinatorNameAr}
                <div class="text-sm font-semibold text-gray-800">{i18n.locale === 'ar' ? fd.coordinatorNameAr : fd.coordinatorNameEn}</div>
                <div class="text-xs text-gray-500 mt-1">📞 {fd.coordinatorPhone || "—"}</div>
                <div class="text-xs text-gray-500 mt-0.5">💬 WhatsApp: {fd.coordinatorWhatsapp || "—"}</div>
                <div class="text-xs text-gray-500 mt-0.5">✉️ {fd.coordinatorEmail || "—"}</div>
            {:else}
                <div class="text-sm text-gray-400 italic">No coordinator assigned</div>
            {/if}
        </div>

        <!-- Completeness Checklist -->
        <div class="sidebar-widget">
            <h4 class="widget-title">{i18n.t("listingsWizard.listingCompleteness") || "Listing Completeness"}</h4>
            <div class="mt-2">
                {#each checklist as check}
                    <div class="check-row">
                        <span class={check.ok ? "check-ok" : "check-missing"}>
                            {check.ok ? "✓" : "✕"}
                        </span>
                        <span class="text-xs text-gray-700">{check.label}</span>
                    </div>
                {/each}
            </div>
        </div>
    </div>
</div>

<style>
    .preview-grid-layout {
        display: grid;
        grid-template-columns: 1fr 340px;
        gap: 2rem;
        align-items: start;
        margin-top: 1.5rem;
    }

    .preview-storefront {
        background: white;
        border-radius: var(--radius-lg, 12px);
        border: 1px solid var(--border, #e5e7eb);
        overflow: hidden;
    }

    .preview-banner {
        width: 100%;
        height: 300px;
        object-fit: cover;
        background: #f3f4f6;
    }

    .preview-placeholder-banner {
        width: 100%;
        height: 300px;
        display: flex;
        align-items: center;
        justify-content: center;
        background: linear-gradient(135deg, #e0e7ff 0%, #ede9fe 100%);
        color: #6366f1;
        font-size: 3rem;
    }

    .preview-details {
        padding: 24px;
    }

    .preview-cat-badge {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        padding: 6px 12px;
        background: #f5f3ff;
        color: #6d28d9;
        font-weight: 600;
        font-size: 0.85rem;
        border-radius: 9999px;
        margin-bottom: 12px;
    }

    .preview-desc-box {
        margin-top: 1.5rem;
        padding-top: 1.5rem;
        border-top: 1px solid #f3f4f6;
    }

    .preview-desc-box h4 {
        margin: 0 0 10px 0;
        font-size: 1rem;
        color: var(--text-dark, #1f2937);
        font-weight: 600;
    }

    .preview-desc-text {
        font-size: 0.95rem;
        line-height: 1.6;
        color: #4b5563;
        white-space: pre-wrap;
    }

    .gallery-preview-section {
        margin-top: 2rem;
    }

    .gallery-preview-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
        gap: 12px;
        margin-top: 10px;
    }

    .gallery-preview-item {
        aspect-ratio: 1;
        border-radius: 8px;
        overflow: hidden;
        border: 1px solid #e5e7eb;
    }

    .gallery-preview-item img, .gallery-preview-item video {
        width: 100%;
        height: 100%;
        object-fit: cover;
    }

    .preview-sidebar {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    .sidebar-widget {
        background: white;
        border-radius: 12px;
        border: 1px solid #e5e7eb;
        padding: 20px;
    }

    .price-widget {
        border-top: 4px solid #6366f1;
    }

    .widget-title {
        font-size: 0.85rem;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: #9ca3af;
        font-weight: 700;
        margin: 0 0 8px 0;
    }

    .widget-value {
        font-size: 1.5rem;
        font-weight: 800;
        color: #111827;
    }

    .widget-value.inquiry {
        color: #6b7280;
        font-size: 1.25rem;
        font-weight: 700;
    }

    .check-row {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 10px 0;
        border-bottom: 1px solid #f3f4f6;
        font-size: 0.9rem;
    }

    .check-row:last-child {
        border-bottom: none;
    }

    .check-ok {
        color: #10b981;
        font-weight: bold;
    }

    .check-missing {
        color: #ef4444;
        font-weight: bold;
    }

    @media (max-width: 900px) {
        .preview-grid-layout {
            grid-template-columns: 1fr;
            gap: 1.5rem;
        }
    }
</style>
