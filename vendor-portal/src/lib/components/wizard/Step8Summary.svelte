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

    const CULTURAL_LABELS: Record<string, { en: string, ar: string }> = {
        prayer_room: { en: "Prayer Room / Musala", ar: "مصلى" },
        valet_parking: { en: "Valet Parking", ar: "صف السيارات" },
        bridal_suite: { en: "Bridal Suite", ar: "جناح العروس" },
        outdoor_garden: { en: "Outdoor Garden", ar: "حديقة خارجية" },
        external_catering_allowed: { en: "External Catering Allowed", ar: "ضيافة خارجية مسموحة" },
        halal_certified: { en: "Halal Certified", ar: "شهادة حلال" }
    };
    function getCulturalAttributeLabel(key: string): string {
        return i18n.locale === 'ar' ? (CULTURAL_LABELS[key]?.ar || key) : (CULTURAL_LABELS[key]?.en || key);
    }

    const FIELD_LABELS: Record<string, { en: string, ar: string }> = {
        men_capacity: { en: "Men Capacity", ar: "سعة قسم الرجال" },
        women_capacity: { en: "Women Capacity", ar: "سعة قسم النساء" },
        has_separate_entrances: { en: "Separate Entrances for Men & Women", ar: "مداخل منفصلة للرجال والنساء" },
        has_audio_link: { en: "Audio Link between Sections", ar: "ربط صوتي بين الأقسام" },
        max_events_per_day: { en: "Max Events per Day", ar: "الحد الأقصى للمناسبات في اليوم" },
        weekend_surcharge_sar: { en: "Weekend Surcharge (SAR)", ar: "رسوم إضافية لنهاية الأسبوع (ريال)" },
        private_pool: { en: "Private Pool Available", ar: "مسبح خاص متوفر" },
        in_house_catering: { en: "In-house Catering Available", ar: "بوفيه داخلي متوفر" },
        private_hall_available: { en: "Private Hall Available", ar: "قاعة خاصة متوفرة" },
        family_section: { en: "Family Section Available", ar: "قسم عائلي متوفر" },
        team_size: { en: "Number of staff", ar: "عدد الموظفين" },
        delivery_weeks: { en: "Delivery Time (Weeks)", ar: "مدة التسليم (بالأسابيع)" },
        women_section_coverage: { en: "Women Section Coverage Only", ar: "تغطية قسم النساء فقط" },
        drone_available: { en: "Drone Photography Available", ar: "تصوير طائرة (درون) متوفر" },
        highlight_reel: { en: "Highlight Reel Included", ar: "فيديو ملخص متضمن" },
        min_guests: { en: "Minimum Guests", ar: "الحد الأدنى للضيوف" },
        buffet_or_plated: { en: "Buffet or Plated Service", ar: "نوع الخدمة (بوفيه / تقديم أطباق)" },
        halal_certified_menu: { en: "Halal Certified Menu", ar: "قائمة طعام معتمدة حلال" },
        taste_testing: { en: "Taste Testing Session Available", ar: "جلسة تذوق متوفرة" },
        setup_cleanup: { en: "Setup & Cleanup Included", ar: "التجهيز والتنظيف متضمن" },
        rehearsal_count: { en: "Rehearsal Sessions", ar: "عدد جلسات البروفة" },
        tailoring_time_days: { en: "Tailoring Time (Days)", ar: "مدة الخياطة (بالأيام)" },
        bride_companions_count: { en: "Bride Companions Count", ar: "عدد مرافقات العروس" },
        address: { en: "Business Address", ar: "عنوان مقر العمل" },
        events_hosted_description: { en: "Events Hosted Description", ar: "وصف المناسبات المقدمة" },
        preparation_time_hours: { en: "Preparation Time (Hours)", ar: "مدة التحضير (بالساعات)" },
        vehicle_count: { en: "Number of Vehicles", ar: "عدد السيارات المتوفرة" }
    };
    function getCategoryAttributeLabel(key: string): string {
        return i18n.locale === 'ar' ? (FIELD_LABELS[key]?.ar || key) : (FIELD_LABELS[key]?.en || key);
    }

    const selectedFeaturesList = $derived.by(() => {
        const schema = $listingStore.schema;
        if (!schema || !schema.featureGroups) return [];
        const list: string[] = [];
        for (const [featId, checked] of Object.entries(fd.featuresSelection)) {
            if (checked === true || checked === 'true') {
                let foundLabel = "";
                for (const group of schema.featureGroups) {
                    const opt = group.options.find((o: any) => o.optionId === featId);
                    if (opt) {
                        foundLabel = i18n.locale === 'ar' 
                            ? (opt.titleAr || opt.labelAr || opt.optionId)
                            : (opt.titleEn || opt.labelEn || opt.optionId);
                        break;
                    }
                }
                list.push(foundLabel || featId);
            }
        }
        return list;
    });
    const hasSelectedFeatures = $derived(selectedFeaturesList.length > 0);
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

            <!-- GCC & Category Details Section -->
            {#if (fd.categoryAttributes && Object.keys(fd.categoryAttributes).length > 0) || (fd.culturalAttributes && Object.keys(fd.culturalAttributes).length > 0)}
                <div class="preview-desc-box">
                    <h4 class="text-sm font-bold text-gray-900 mb-3">
                        {i18n.locale === "ar" ? "تفاصيل ومواصفات الخدمة" : "Service Specifications"}
                    </h4>
                    <div class="specs-grid">
                        <!-- Render GCC attributes -->
                        {#each Object.entries(fd.culturalAttributes) as [key, val]}
                            {#if val === true || val === 'true'}
                                <div class="spec-tag">
                                    ✓ {getCulturalAttributeLabel(key)}
                                </div>
                            {/if}
                        {/each}

                        <!-- Render Category attributes -->
                        {#each Object.entries(fd.categoryAttributes) as [key, val]}
                            {#if val !== null && val !== undefined && val !== '' && val !== false}
                                <div class="spec-item">
                                    <span class="spec-label">{getCategoryAttributeLabel(key)}:</span>
                                    <span class="spec-val">
                                        {#if val === true || val === 'true'}
                                            {i18n.locale === 'ar' ? 'نعم' : 'Yes'}
                                        {:else if val === false || val === 'false'}
                                            {i18n.locale === 'ar' ? 'لا' : 'No'}
                                        {:else}
                                            {val}
                                        {/if}
                                    </span>
                                </div>
                            {/if}
                        {/each}
                    </div>
                </div>
            {/if}

            <!-- Selected Features Section -->
            {#if hasSelectedFeatures}
                <div class="preview-desc-box">
                    <h4 class="text-sm font-bold text-gray-900 mb-3">
                        {i18n.locale === "ar" ? "الميزات الإضافية المختارة" : "Selected Additional Features"}
                    </h4>
                    <div class="features-preview-tags">
                        {#each selectedFeaturesList as feature}
                            <div class="feature-tag">
                                ✦ {feature}
                            </div>
                        {/each}
                    </div>
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
                                    <video src={item.url.startsWith("http") ? item.url : getApiUrl(item.url)} class="w-full h-full object-cover">
                                        <track kind="captions" src="" label="English" srclang="en" default>
                                    </video>
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

    /* Specs & features preview layout styling */
    .specs-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
        gap: 12px;
        margin-top: 10px;
    }
    .spec-item {
        font-size: 0.9rem;
        color: #4b5563;
    }
    .spec-label {
        font-weight: 600;
        color: #1f2937;
    }
    .spec-val {
        margin-left: 4px;
    }
    .spec-tag {
        display: inline-flex;
        align-items: center;
        padding: 4px 10px;
        background: #f0fdf4;
        color: #16a34a;
        font-size: 0.85rem;
        font-weight: 500;
        border-radius: 6px;
        border: 1px solid #dcfce7;
    }
    .features-preview-tags {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
        margin-top: 10px;
    }
    .feature-tag {
        display: inline-flex;
        align-items: center;
        padding: 6px 12px;
        background: #eff6ff;
        color: #1d4ed8;
        font-size: 0.85rem;
        font-weight: 500;
        border-radius: 8px;
        border: 1px solid #dbeafe;
    }

    @media (max-width: 900px) {
        .preview-grid-layout {
            grid-template-columns: 1fr;
            gap: 1.5rem;
        }
    }
</style>
