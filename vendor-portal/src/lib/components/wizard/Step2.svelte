<script lang="ts">
    import { getContext, onMount } from "svelte";
    import { getI18n } from "$lib/i18n/i18n.svelte";
    import { listingStore } from "$lib/stores/listingStore";
    import { Building2 } from "lucide-svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import { getApiUrl } from "$lib/utils/api";
    import { wizardFetch } from "$lib/utils/wizardFetch";

    const i18n = getI18n();
    let { data } = $props<{ data: { sessionToken: string, cities: any[] } }>();

    const wizard = getContext<{
        registerSubmitHandler: (handler: () => Promise<void>) => () => void;
        setCanContinue: (val: boolean) => void;
        setSubmitting: (val: boolean) => void;
    }>('wizard');

    const isMapsUrlValid = $derived(
        $listingStore.formData.googleMapsUrl.trim().startsWith("http://") ||
            $listingStore.formData.googleMapsUrl.trim().startsWith("https://")
            ? $listingStore.formData.googleMapsUrl.includes("google.com/maps") ||
                  $listingStore.formData.googleMapsUrl.includes("maps.google.com") ||
                  $listingStore.formData.googleMapsUrl.includes("maps.app.goo.gl") ||
                  $listingStore.formData.googleMapsUrl.includes("goo.gl/maps")
            : false,
    );

    const mapEmbedUrl = $derived(
        isMapsUrlValid
            ? $listingStore.formData.latitude.trim() && $listingStore.formData.longitude.trim()
                ? `https://maps.google.com/maps?q=${$listingStore.formData.latitude.trim()},${$listingStore.formData.longitude.trim()}&t=&z=15&ie=UTF8&iwloc=&output=embed`
                : `https://maps.google.com/maps?q=${encodeURIComponent($listingStore.formData.googleMapsUrl)}&t=&z=13&ie=UTF8&iwloc=&output=embed`
            : "",
    );


    const cities = $derived(
        (data.cities || []).map((c: any) => ({
            id: c.id,
            nameAr: c.name_ar || "",
            nameEn: c.name_en || "",
        })),
    );

    onMount(() => {
        const unregister = wizard.registerSubmitHandler(async () => {
            const hasTitle =
                $listingStore.formData.titleAr.trim().length > 0 || $listingStore.formData.titleEn.trim().length > 0;
            const hasPrice =
                $listingStore.formData.priceOnInquiry ||
                ($listingStore.formData.basePriceSar !== "" && parseFloat($listingStore.formData.basePriceSar) > 0);
            const hasCity = !!$listingStore.formData.selectedCityId;
            const hasMap = $listingStore.formData.googleMapsUrl.trim().length > 0 && isMapsUrlValid;

            if (!hasTitle || !hasPrice || !hasCity || !hasMap) {
                listingStore.setError(
                    "Please fill all required fields correctly.",
                );
                return;
            }

            // Dirty check bypass
            const isDirty = listingStore.isStepDirty(2, $listingStore);
            if (!isDirty) {
                listingStore.setHighestStep(2);
                await goto(`${$page.url.pathname.split("/step-")[0]}/step-3`);
                return;
            }

            wizard.setSubmitting(true);
            listingStore.setError("");

            try {
                const url = getApiUrl(`/api/v1/vendor/products/${$listingStore.productId}`);
                const payload = {
                    version: $listingStore.version, // Use dynamic version for Optimistic Concurrency Control
                    titleAr: $listingStore.formData.titleAr || null,
                    titleEn: $listingStore.formData.titleEn || null,
                    basePriceSar: $listingStore.formData.priceOnInquiry
                        ? null
                        : parseFloat($listingStore.formData.basePriceSar),
                    priceOnInquiry: $listingStore.formData.priceOnInquiry,
                    depositPercentage: $listingStore.formData.depositPercentage,
                    cityId: $listingStore.formData.selectedCityId || null,
                    googleMapsUrl: $listingStore.formData.googleMapsUrl.trim() || null,
                    latitude: $listingStore.formData.latitude.trim() ? parseFloat($listingStore.formData.latitude) : null,
                    longitude: $listingStore.formData.longitude.trim() ? parseFloat($listingStore.formData.longitude) : null,
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
                await goto(`${$page.url.pathname.split("/step-")[0]}/step-3`);
            } catch (err: any) {
                listingStore.setError(
                    err.message || "Failed to save basic info.",
                );
            } finally {
                wizard.setSubmitting(false);
            }
        });

        // Store subscription for dynamic Google Maps URL coordinate resolution
        let lastResolvedUrl = "";
        const unsubscribeStore = listingStore.subscribe((state) => {
            const rawUrl = state.formData.googleMapsUrl.trim();
            const isValid = rawUrl.startsWith("http://") || rawUrl.startsWith("https://")
                ? rawUrl.includes("google.com/maps") ||
                  rawUrl.includes("maps.google.com") ||
                  rawUrl.includes("maps.app.goo.gl") ||
                  rawUrl.includes("goo.gl/maps")
                : false;

            if (rawUrl && isValid && rawUrl !== lastResolvedUrl) {
                lastResolvedUrl = rawUrl;
                console.log("[RESOLVER] Store subscribe triggered resolution for:", rawUrl);
                
                wizardFetch(getApiUrl("/api/v1/vendor/products/resolve-location"), {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                        Authorization: `Bearer ${data.sessionToken}`,
                    },
                    body: JSON.stringify({ url: rawUrl }),
                })
                .then((res) => {
                    if (res.ok) return res.json();
                    throw new Error("Resolution failed");
                })
                .then((resData) => {
                    // Prevent race conditions from keypress overlap: read current store URL
                    let currentUrl = "";
                    const unsub = listingStore.subscribe((s) => { currentUrl = s.formData.googleMapsUrl.trim(); });
                    unsub();

                    if (resData && resData.success && resData.originalUrl === currentUrl) {
                        if (resData.latitude !== null && resData.latitude !== undefined &&
                            resData.longitude !== null && resData.longitude !== undefined) {
                            
                            console.log("[RESOLVER] Resolution success. Lat:", resData.latitude, "Lng:", resData.longitude);
                            listingStore.updateFormData({
                                latitude: String(resData.latitude),
                                longitude: String(resData.longitude)
                            });
                        }
                    }
                })
                .catch((err) => {
                    console.error("[RESOLVER] Frontend resolution error:", err);
                });
            }
        });

        return () => {
            unregister();
            unsubscribeStore();
        };
    });

    let isValid = $derived(
        $listingStore.formData.titleEn.trim().length > 0 &&
        $listingStore.formData.titleAr.trim().length > 0 &&
        ($listingStore.formData.priceOnInquiry || String($listingStore.formData.basePriceSar).trim().length > 0) &&
        $listingStore.formData.selectedCityId.trim().length > 0 &&
        $listingStore.formData.googleMapsUrl.trim().length > 0
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
                    bind:value={$listingStore.formData.titleEn}
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
                    bind:value={$listingStore.formData.titleAr}
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
                            bind:value={$listingStore.formData.basePriceSar}
                            placeholder={i18n.t(
                                "listingsWizard.basePricePlaceholder",
                            ) || "Enter amount"}
                            min="0"
                            disabled={$listingStore.formData.priceOnInquiry}
                            class:disabled={$listingStore.formData.priceOnInquiry}
                        />
                    </div>
                    <label class="toggle-label">
                        <input type="checkbox" bind:checked={$listingStore.formData.priceOnInquiry} />
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
                        <select id="city" bind:value={$listingStore.formData.selectedCityId}>
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
                    <span class="hint">({$listingStore.formData.depositPercentage}%)</span>
                </label>
                <input
                    id="deposit"
                    type="range"
                    bind:value={$listingStore.formData.depositPercentage}
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
                    bind:value={$listingStore.formData.googleMapsUrl}
                    placeholder={i18n.t("listingsWizard.googleMapsPlaceholder") || "https://google.com/maps/..."}
                    class:invalid={$listingStore.formData.googleMapsUrl.trim() && !isMapsUrlValid}
                />
                {#if $listingStore.formData.googleMapsUrl.trim() && !isMapsUrlValid}
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
                        bind:value={$listingStore.formData.latitude}
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
                        bind:value={$listingStore.formData.longitude}
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
