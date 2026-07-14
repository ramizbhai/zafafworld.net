<script lang="ts">
    import { getContext, onMount, onDestroy } from "svelte";
    import { getI18n } from "$lib/i18n/i18n.svelte";
    import { listingStore } from "$lib/stores/listingStore";
    import { Building2 } from "lucide-svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import { getApiUrl } from "$lib/utils/api";
    import { wizardFetch } from "$lib/utils/wizardFetch";

    const i18n = getI18n();
    let { data } = $props<{ data: { sessionToken: string } }>();

    const wizard = getContext<{
        registerSubmitHandler: (handler: () => Promise<void>) => () => void;
        setCanContinue: (val: boolean) => void;
        setSubmitting: (val: boolean) => void;
    }>('wizard');

    let titleEn = $state($listingStore.formData.titleEn);
    let titleAr = $state($listingStore.formData.titleAr);
    let basePriceSar = $state($listingStore.formData.basePriceSar);
    let priceOnInquiry = $state($listingStore.formData.priceOnInquiry);
    let depositPercentage = $state($listingStore.formData.depositPercentage);
    let selectedCityId = $state($listingStore.formData.selectedCityId);
    let googleMapsUrl = $state($listingStore.formData.googleMapsUrl);
    let latitude = $state($listingStore.formData.latitude);
    let longitude = $state($listingStore.formData.longitude);

    const isMapsUrlValid = $derived(
        googleMapsUrl.trim().startsWith("http://") ||
            googleMapsUrl.trim().startsWith("https://")
            ? googleMapsUrl.includes("google.com/maps") ||
                  googleMapsUrl.includes("maps.google.com") ||
                  googleMapsUrl.includes("maps.app.goo.gl") ||
                  googleMapsUrl.includes("goo.gl/maps")
            : false,
    );

    const mapEmbedUrl = $derived(
        isMapsUrlValid
            ? `https://maps.google.com/maps?q=${encodeURIComponent(googleMapsUrl)}&t=&z=13&ie=UTF8&iwloc=&output=embed`
            : "",
    );

    const cities = $derived(
        (data.cities || []).map((c: any) => ({
            id: c.id,
            nameAr: c.name_ar || "",
            nameEn: c.name_en || "",
        })),
    );

    // Sync state on unmount (synchronous safety net)
    onDestroy(() => {
        listingStore.updateFormData({
            titleEn,
            titleAr,
            basePriceSar,
            priceOnInquiry,
            depositPercentage,
            selectedCityId,
            googleMapsUrl,
            latitude,
            longitude,
        });
    });

    // 300ms debounced background sync — keeps store in sync as user types
    // so navigation away mid-step doesn't lose data
    $effect(() => {
        // Reactive reads — changes to any of these trigger the effect
        const _watch = titleEn + titleAr + basePriceSar + String(priceOnInquiry) +
                       String(depositPercentage) + selectedCityId + googleMapsUrl + latitude + longitude;
        const timer = setTimeout(() => {
            listingStore.updateFormData({
                titleEn, titleAr, basePriceSar, priceOnInquiry,
                depositPercentage, selectedCityId, googleMapsUrl, latitude, longitude,
            });
        }, 300);
        return () => clearTimeout(timer);
    });

    onMount(() => {
        const unregister = wizard.registerSubmitHandler(async () => {
            const hasTitle =
                titleAr.trim().length > 0 || titleEn.trim().length > 0;
            const hasPrice =
                priceOnInquiry ||
                (basePriceSar !== "" && parseFloat(basePriceSar) > 0);
            const hasCity = !!selectedCityId;
            const hasMap = googleMapsUrl.trim().length > 0 && isMapsUrlValid;

            if (!hasTitle || !hasPrice || !hasCity || !hasMap) {
                listingStore.setError(
                    "Please fill all required fields correctly.",
                );
                return;
            }

            // Sync state immediately before API call
            listingStore.updateFormData({
                titleEn,
                titleAr,
                basePriceSar,
                priceOnInquiry,
                depositPercentage,
                selectedCityId,
                googleMapsUrl,
                latitude,
                longitude,
            });

            // Dirty check bypass
            const isDirty = listingStore.isStepDirty(2, $listingStore);
            if (!isDirty) {
                listingStore.setHighestStep(2);
                goto(`${$page.url.pathname.split("/step-")[0]}/step-3`);
                return;
            }

            wizard.setSubmitting(true);
            listingStore.setError("");

            try {
                const url = getApiUrl(`/api/v1/vendor/products/${$listingStore.productId}`);
                const payload = {
                    version: $listingStore.version, // Use dynamic version for Optimistic Concurrency Control
                    titleAr: titleAr || null,
                    titleEn: titleEn || null,
                    basePriceSar: priceOnInquiry
                        ? null
                        : parseFloat(basePriceSar),
                    priceOnInquiry: priceOnInquiry,
                    depositPercentage: depositPercentage,
                    cityId: selectedCityId || null,
                    googleMapsUrl: googleMapsUrl.trim() || null,
                    latitude: latitude.trim() ? parseFloat(latitude) : null,
                    longitude: longitude.trim() ? parseFloat(longitude) : null,
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

                listingStore.commitStepSave(2);
                listingStore.setHighestStep(2);
                goto(`${$page.url.pathname.split("/step-")[0]}/step-3`);
            } catch (err: any) {
                listingStore.setError(
                    err.message || "Failed to save basic info.",
                );
            } finally {
                wizard.setSubmitting(false);
            }
        });
        return unregister;
    });

    let isValid = $derived(
        titleEn.trim().length > 0 &&
        titleAr.trim().length > 0 &&
        (priceOnInquiry || String(basePriceSar).trim().length > 0) &&
        selectedCityId.trim().length > 0 &&
        googleMapsUrl.trim().length > 0
    );

    $effect(() => {
        wizard.setCanContinue(isValid);
    });
</script>

<div class="step-pane fade-in">
    <div class="step-heading">
        <Building2 class="step-icon" size={28} />
        <div>
            <h1>
                {i18n.locale === "ar"
                    ? "معلومات الإعلان الأساسية"
                    : "Basic Listing Information"}
            </h1>
            <p>
                {i18n.locale === "ar"
                    ? "أدخل عنوان إعلانك باللغتين العربية والإنجليزية، وحدد السعر، واختر مدينتك."
                    : "Enter your listing title in both Arabic and English, set pricing, and choose your city."}
            </p>
        </div>
    </div>

    {#if $listingStore.submitError}
        <div class="error-banner">{$listingStore.submitError}</div>
    {/if}

    <div class="form-card split-layout">
        <!-- Left Column: Basic Details -->
        <div class="form-column">
            <h3 class="column-title">
                {i18n.locale === "ar" ? "المعلومات الأساسية" : "Basic Details"}
            </h3>

            <div class="form-group">
                <label for="title-en">
                    {i18n.t("listingsWizard.englishTitle") || "English Title"}
                    <span class="required">*</span>
                </label>
                <input
                    id="title-en"
                    type="text"
                    bind:value={titleEn}
                    placeholder="e.g. Al Nour Wedding Palace — Riyadh"
                    maxlength={200}
                />
            </div>

            <div class="form-group">
                <label for="title-ar">
                    {i18n.t("listingsWizard.arabicTitle") || "Arabic Title"}
                    <span class="required">*</span>
                </label>
                <input
                    id="title-ar"
                    type="text"
                    bind:value={titleAr}
                    placeholder="مثال: قصر النور للأفراح — الرياض"
                    maxlength={200}
                    dir="rtl"
                    class="rtl-input"
                />
            </div>

            <div class="form-row">
                <div class="form-group">
                    <label for="price">
                        {i18n.t("listingsWizard.basePriceLabel") || "Base Price"}
                        <span class="required">*</span>
                    </label>
                    <div class="input-prefix">
                        <span class="prefix">﷼</span>
                        <input
                            id="price"
                            type="number"
                            bind:value={basePriceSar}
                            placeholder={i18n.t(
                                "listingsWizard.basePricePlaceholder",
                            ) || "Enter amount"}
                            min="0"
                            disabled={priceOnInquiry}
                            class:disabled={priceOnInquiry}
                        />
                    </div>
                    <label class="toggle-label">
                        <input type="checkbox" bind:checked={priceOnInquiry} />
                        {i18n.t("listingsWizard.priceOnInquiry") ||
                            "Price on Inquiry"}
                    </label>
                </div>

                <div class="form-group">
                    <label for="city">
                        {i18n.t("listingsWizard.city") || "City"}
                        <span class="required">*</span>
                    </label>
                    <div class="select-wrapper">
                        <select id="city" bind:value={selectedCityId}>
                            <option value="">
                                — {i18n.t("listingsWizard.selectCity") || "Select City"} —
                            </option>
                            {#each cities as city}
                                <option value={city.id}>
                                    {i18n.locale === "ar" ? city.nameAr : city.nameEn}
                                </option>
                            {/each}
                        </select>
                    </div>
                </div>
            </div>

            <div class="form-group">
                <label for="deposit">
                    {i18n.t("listingsWizard.depositPercentage") || "Deposit"}
                    <span class="hint">({depositPercentage}%)</span>
                </label>
                <input
                    id="deposit"
                    type="range"
                    bind:value={depositPercentage}
                    min="10"
                    max="100"
                    step="5"
                />
                <div
                    class="range-labels"
                    style="position: relative; height: 16px; margin-top: 4px;"
                >
                    <span style="position: absolute; left: 0%; transform: translateX(0%); font-size: 0.75rem; color: var(--text-muted);">10%</span>
                    <span style="position: absolute; left: 16.67%; transform: translateX(-50%); font-size: 0.75rem; color: var(--text-muted);">25%</span>
                    <span style="position: absolute; left: 44.44%; transform: translateX(-50%); font-size: 0.75rem; color: var(--text-muted);">50%</span>
                    <span style="position: absolute; left: 100%; transform: translateX(-100%); font-size: 0.75rem; color: var(--text-muted);">100%</span>
                </div>
            </div>
        </div>

        <!-- Right Column: Location Details -->
        <div class="form-column location-column">
            <h3 class="column-title">
                🗺️ {i18n.locale === "ar" ? "تفاصيل الموقع" : "Location Details"}
            </h3>

            <p class="hint-text" style="font-size: 0.8rem; color: var(--text-muted);">
                {i18n.locale === "ar"
                    ? "ضع رابط خرائط جوجل والإحداثيات الاختيارية لعرض موقعك للعملاء."
                    : "Provide the Google Maps URL and optional coordinates to show clients where you are located."}
            </p>

            <div class="form-group">
                <label for="google_maps_url">
                    {i18n.t("listingsWizard.googleMapsUrl") || "Google Maps URL"}
                    <span class="required">*</span>
                </label>
                <input
                    id="google_maps_url"
                    type="url"
                    bind:value={googleMapsUrl}
                    placeholder={i18n.t("listingsWizard.googleMapsPlaceholder") || "https://google.com/maps/..."}
                    class:invalid={googleMapsUrl.trim() && !isMapsUrlValid}
                />
                {#if googleMapsUrl.trim() && !isMapsUrlValid}
                    <div class="validation-error">
                        {i18n.locale === "ar"
                            ? "يجب أن يكون رابط خرائط جوجل صالحاً (مثال: https://goo.gl/maps/...)"
                            : "Must be a valid Google Maps link (e.g., https://goo.gl/maps/...)"}
                    </div>
                {/if}
            </div>

            <div class="form-row">
                <div class="form-group">
                    <label for="latitude"
                        >{i18n.t("listingsWizard.latitude") || "Latitude"} ({i18n.locale ===
                        "ar"
                            ? "اختياري"
                            : "Optional"})</label
                    >
                    <input
                        id="latitude"
                        type="text"
                        bind:value={latitude}
                        placeholder="e.g. 24.7136"
                    />
                </div>
                <div class="form-group">
                    <label for="longitude"
                        >{i18n.t("listingsWizard.longitude") || "Longitude"} ({i18n.locale ===
                        "ar"
                            ? "اختياري"
                            : "Optional"})</label
                    >
                    <input
                        id="longitude"
                        type="text"
                        bind:value={longitude}
                        placeholder="e.g. 46.6753"
                    />
                </div>
            </div>

            {#if isMapsUrlValid && mapEmbedUrl}
                <div
                    style="margin-top: 0.5rem; border: 1px solid rgba(0,0,0,0.08); border-radius: 12px; overflow: hidden; height: 200px; background: #fafafa;"
                >
                    <iframe
                        title="Google Maps Location Preview"
                        width="100%"
                        height="100%"
                        style="border:0;"
                        loading="lazy"
                        allowfullscreen
                        referrerpolicy="no-referrer-when-downgrade"
                        src={mapEmbedUrl}
                    ></iframe>
                </div>
            {/if}
        </div>
    </div>

<style>
    .split-layout {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 32px;
        align-items: start;
    }
    .form-column {
        display: flex;
        flex-direction: column;
        gap: 20px;
    }
    .location-column {
        background: #f8fafc;
        padding: 24px;
        border-radius: 12px;
        border: 1px solid #e2e8f0;
    }
    .column-title {
        font-size: 1rem;
        font-weight: 700;
        color: #334155;
        margin: 0;
        border-bottom: 2px solid #e2e8f0;
        padding-bottom: 8px;
        margin-bottom: 4px;
    }
    
    @media (max-width: 768px) {
        .split-layout {
            grid-template-columns: 1fr;
            gap: 24px;
        }
        .location-column {
            padding: 16px;
        }
    }
</style>
    </div>
