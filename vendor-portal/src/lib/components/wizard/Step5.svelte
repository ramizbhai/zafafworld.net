<script lang="ts">
    import { getContext, onMount } from "svelte";
    import { getI18n } from "$lib/i18n/i18n.svelte";
    import { listingStore } from "$lib/stores/listingStore";
    import { CATEGORY_GROUPS } from "$lib/constants/wizard";
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import { getApiUrl } from "$lib/utils/api";
    import { wizardFetch } from "$lib/utils/wizardFetch";
    import DynamicFieldRenderer from "./DynamicFieldRenderer.svelte";
    import DynamicFeatureGrid from "./DynamicFeatureGrid.svelte";
    import ListingSummaryReflection from "./ListingSummaryReflection.svelte";

    const i18n = getI18n();
    let { data } = $props<{ data: { sessionToken: string } }>();

    const wizard = getContext<{
        registerSubmitHandler: (handler: () => Promise<void>) => () => void;
        setCanContinue: (val: boolean) => void;
        setSubmitting: (val: boolean) => void;
    }>('wizard');

    const schema = $derived($listingStore.schema);
    const selectedCategory = $derived($listingStore.formData.selectedCategory);

    const selectedCategoryMeta = $derived.by(() => {
        for (const group of CATEGORY_GROUPS) {
            const found = group.items.find((i) => i.value === selectedCategory);
            if (found) return found;
        }
        return null;
    });

    $effect(() => {
        wizard.setCanContinue(true);
    });

    const EXCLUDED_FIELDS = [
        'hotel_name', 'hotelName',
        'city', 'city_id', 'cityId',
        'location_link', 'google_maps_url',
        'hotel_number', 'hotelNumber',
        'admin_number', 'adminNumber',
        'venue_id', 'venueId',
        'pictures_of_the_halls', 'hall_pictures',
        'female_team_available',
        'serving_staff_included'
    ];

    const totalVisibleFieldsCount = $derived(
        schema 
            ? schema.detailSections.reduce((acc: number, section: any) => {
                return acc + section.fields.filter((f: any) => !EXCLUDED_FIELDS.includes(f.fieldId)).length;
              }, 0)
            : 0
    );

    function retryFetchSchema() {
        listingStore.setSchemaError(null);
        listingStore.setSchema(null);
    }

    onMount(() => {
        const unregister = wizard.registerSubmitHandler(async () => {
            const isDirty = listingStore.isStepDirty(5, $listingStore);
            if (!isDirty) {
                listingStore.setHighestStep(5);
                await goto(`${$page.url.pathname.split("/step-")[0]}/step-6`);
                return;
            }

            wizard.setSubmitting(true);
            listingStore.setError("");

            try {
                const url = getApiUrl(`/api/v1/vendor/products/${$listingStore.productId}`);
                const payload = {
                    version: $listingStore.version,
                    attributes: $listingStore.formData.categoryAttributes,
                    featuresSelection: $listingStore.formData.featuresSelection,
                };

                const res = await wizardFetch(url, {
                    method: "PUT",
                    headers: {
                        "Content-Type": "application/json",
                        Authorization: `Bearer ${data.sessionToken}`,
                        "X-Trace-ID": listingStore.getTraceId(),
                    },
                    body: JSON.stringify(payload),
                });

                if (!res.ok) {
                    const err = await res.json().catch(() => ({}));
                    throw new Error(err.message || err.error || `Server error ${res.status}`);
                }

                const responseData = await res.json();
                if (responseData.product?.version) {
                    listingStore.setVersion(responseData.product.version);
                }

                listingStore.commitStepSave(5);
                listingStore.setHighestStep(5);
                await goto(`${$page.url.pathname.split("/step-")[0]}/step-6`);
            } catch (err: any) {
                listingStore.setError(err.message || "Failed to save details.");
            } finally {
                wizard.setSubmitting(false);
            }
        });
        return unregister;
    });
</script>

<div class="step-pane fade-in">
    <ListingSummaryReflection />

    <div class="step-heading">
        <div class="step-icon-text">
            {selectedCategoryMeta?.emoji || "📋"}
        </div>
        <div>
            <h1>
                {i18n.locale === "ar"
                    ? selectedCategoryMeta?.ar
                    : selectedCategoryMeta?.en}
                {i18n.locale === "ar" ? " تفاصيل" : " Details"}
            </h1>
            <p>
                {i18n.locale === "ar"
                    ? `يرجى ملء التفاصيل المحددة لـ ${selectedCategoryMeta?.ar} التي يبحث عنها العملاء عند التصفح.`
                    : `Fill in the specific details for your ${selectedCategoryMeta?.en?.toLowerCase() || "listing"} that clients look for when browsing.`}
            </p>
        </div>
    </div>

    {#if $listingStore.submitError}
        <div class="error-banner">{$listingStore.submitError}</div>
    {/if}

    <div class="form-card">
        {#if $listingStore.schemaLoading}
            <div class="skeleton-card">
                <div class="skeleton-title animate-pulse"></div>
                <div class="skeleton-grid">
                    <div class="skeleton-field animate-pulse"></div>
                    <div class="skeleton-field animate-pulse"></div>
                </div>
            </div>
            <div class="skeleton-card" style="margin-top: 24px;">
                <div class="skeleton-title animate-pulse"></div>
                <div class="skeleton-grid">
                    <div class="skeleton-field animate-pulse"></div>
                    <div class="skeleton-field animate-pulse"></div>
                </div>
            </div>
        {:else if $listingStore.schemaError}
            <div class="inline-error-retry">
                <p class="error-text">{$listingStore.schemaError}</p>
                <button type="button" class="btn-retry" onclick={retryFetchSchema}>
                    {i18n.locale === 'ar' ? 'إعادة محاولة التحميل' : 'Retry Loading'}
                </button>
            </div>
        {:else if schema}
            {#if totalVisibleFieldsCount > 0 && totalVisibleFieldsCount < 3}
                <!-- Sparse category compact layout -->
                <p class="compact-info-message">
                    {i18n.locale === "ar" 
                        ? "نحتاج فقط إلى الأساسيات لهذه الفئة — وسنطلب منك المزيد بمجرد إضافة الصور." 
                        : "We need just the essentials for this category — we'll ask for more once you add photos."}
                </p>
                <div class="section-card compact-section-card">
                    <h3 class="section-title">
                        {i18n.locale === "ar" ? "التفاصيل الأساسية" : "Essential Details"}
                    </h3>
                    <div class="form-grid">
                        {#each schema.detailSections as section}
                            {#each section.fields.filter((f: any) => !EXCLUDED_FIELDS.includes(f.fieldId)) as field}
                                <DynamicFieldRenderer fieldId={field.fieldId} required={field.required} visibleWhen={field.visibleWhen} />
                            {/each}
                        {/each}
                    </div>
                </div>
                <hr class="form-divider" />
            {:else if totalVisibleFieldsCount >= 3}
                <!-- Dense category multi-card layout -->
                {#each schema.detailSections as section}
                    {@const visibleFields = section.fields.filter((f: any) => !EXCLUDED_FIELDS.includes(f.fieldId))}
                    {#if visibleFields.length > 0}
                        <div class="section-card">
                            <h3 class="section-title">
                                {i18n.locale === "ar" ? section.titleAr : section.titleEn}
                            </h3>
                            <div class="form-grid">
                                {#each visibleFields as field}
                                    <DynamicFieldRenderer fieldId={field.fieldId} required={field.required} visibleWhen={field.visibleWhen} />
                                {/each}
                            </div>
                        </div>
                        <hr class="form-divider" />
                    {/if}
                {/each}
            {:else}
                <!-- totalVisibleFieldsCount === 0 -->
                <p class="compact-info-message">
                    {i18n.locale === "ar" 
                        ? "نحتاج فقط إلى الأساسيات لهذه الفئة — وسنطلب منك المزيد بمجرد إضافة الصور." 
                        : "We need just the essentials for this category — we'll ask for more once you add photos."}
                </p>
            {/if}

            <!-- Render feature groups dynamically -->
            {#if schema.featureGroups && schema.featureGroups.length > 0}
                <div class="features-header">
                    <h3 class="additional-features-title">
                        {i18n.locale === "ar"
                            ? "الميزات الإضافية (اختياري)"
                            : "Additional Features (Optional)"}
                    </h3>
                </div>
                {#each schema.featureGroups as group}
                    <DynamicFeatureGrid group={group} storeKey="featuresSelection" />
                {/each}
            {/if}

            <!-- Fallback if no specific details or features are returned in schema -->
            {#if totalVisibleFieldsCount === 0 && (!schema.featureGroups || schema.featureGroups.length === 0)}
                <p class="step-placeholder">
                    ✅ {i18n.t("listingsWizard.noSpecificDetails") || "No specific details required for this category."}
                </p>
            {/if}
        {/if}
    </div>
</div>

<style>
    .section-card {
        background: white;
        border: 1px solid var(--border, #e5e7eb);
        border-radius: 12px;
        padding: 20px;
        margin-bottom: 24px;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
    }
    .section-title {
        font-size: 0.8rem;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: var(--text-muted, #6b7280);
        margin-top: 0;
        margin-bottom: 16px;
    }
    .form-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
        gap: 16px 20px;
    }
    .features-header {
        margin-top: 32px;
        margin-bottom: 16px;
    }
    .additional-features-title {
        font-size: 0.95rem;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: var(--text, #1a1a2e);
        margin: 0;
    }
    .compact-info-message {
        font-size: 0.85rem;
        color: var(--primary, #6c3fa0);
        background: rgba(108, 63, 160, 0.05);
        border: 1px solid rgba(108, 63, 160, 0.1);
        border-radius: 8px;
        padding: 12px 16px;
        margin-bottom: 20px;
        line-height: 1.45;
        font-weight: 500;
    }
    .skeleton-card {
        padding: 20px;
        background: white;
        border: 1px solid var(--border, #e5e7eb);
        border-radius: 12px;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
    }
    .skeleton-title {
        height: 14px;
        width: 120px;
        background: #f3f4f6;
        border-radius: 4px;
        margin-bottom: 20px;
    }
    .skeleton-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 16px;
    }
    .skeleton-field {
        height: 38px;
        background: #f3f4f6;
        border-radius: 8px;
    }
    .inline-error-retry {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 12px;
        padding: 24px;
        background: #fef2f2;
        border: 1px solid #fecaca;
        border-radius: 12px;
        text-align: center;
    }
    .error-text {
        font-size: 0.9rem;
        color: #b91c1c;
        margin: 0;
        font-weight: 500;
    }
    .btn-retry {
        padding: 8px 16px;
        background: var(--primary, #6c3fa0);
        color: white;
        border: none;
        border-radius: 8px;
        font-size: 0.85rem;
        font-weight: 600;
        cursor: pointer;
        transition: opacity 0.15s;
        box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
    }
    .btn-retry:hover {
        opacity: 0.9;
    }
    .animate-pulse {
        animation: pulse 1.5s infinite ease-in-out;
    }
    @keyframes pulse {
        0%, 100% { opacity: 0.6; }
        50% { opacity: 1; }
    }
</style>
