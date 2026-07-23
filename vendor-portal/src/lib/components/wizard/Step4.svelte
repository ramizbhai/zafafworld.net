<script lang="ts">
    import { getContext, onMount } from "svelte";
    import { getI18n } from "$lib/i18n/i18n.svelte";
    import { listingStore } from "$lib/stores/listingStore";
    import { GENDER_SECTIONS } from "$lib/constants/wizard";
    import { Check } from "lucide-svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import { getApiUrl } from "$lib/utils/api";
    import { wizardFetch } from "$lib/utils/wizardFetch";
    import DynamicFeatureGrid from "./DynamicFeatureGrid.svelte";

    const i18n = getI18n();
    let { data } = $props<{ data: { sessionToken: string } }>();

    const wizard = getContext<{
        registerSubmitHandler: (handler: () => Promise<void>) => () => void;
        setCanContinue: (val: boolean) => void;
        setSubmitting: (val: boolean) => void;
    }>('wizard');

    const schema = $derived($listingStore.schema);

    // Static definition of GCC cultural amenities group for DynamicFeatureGrid
    const culturalGroup = {
        groupId: "cultural",
        titleEn: "Key Amenities",
        titleAr: "المرافق والخدمات الرئيسية",
        options: [
            { optionId: "prayer_room", labelEn: "Prayer Room / Musala", labelAr: "مصلى" },
            { optionId: "valet_parking", labelEn: "Valet Parking", labelAr: "صف السيارات" },
            { optionId: "bridal_suite", labelEn: "Bridal Suite", labelAr: "جناح العروس" },
            { optionId: "outdoor_garden", labelEn: "Outdoor Garden", labelAr: "حديقة خارجية" },
            { optionId: "external_catering_allowed", labelEn: "External Catering Allowed", labelAr: "ضيافة خارجية مسموحة" },
            { optionId: "halal_certified", labelEn: "Halal Certified", labelAr: "شهادة حلال" }
        ]
    };

    let isValid = $derived(!!$listingStore.formData.genderSection);
    $effect(() => {
        wizard.setCanContinue(isValid);
    });

    onMount(() => {
        const unregister = wizard.registerSubmitHandler(async () => {
            if (!$listingStore.formData.genderSection) {
                listingStore.setError("Please select a gender section setup.");
                return;
            }

            const isDirty = listingStore.isStepDirty(4, $listingStore);
            if (!isDirty) {
                listingStore.setHighestStep(4);
                await goto(`${$page.url.pathname.split("/step-")[0]}/step-5`);
                return;
            }

            wizard.setSubmitting(true);
            listingStore.setError("");

            try {
                const url = getApiUrl(`/api/v1/vendor/products/${$listingStore.productId}`);
                const payload = {
                    version: $listingStore.version,
                    genderSection: $listingStore.formData.genderSection || null,
                    culturalAttributes: $listingStore.formData.culturalAttributes,
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

                listingStore.commitStepSave(4);
                listingStore.setHighestStep(4);
                await goto(`${$page.url.pathname.split("/step-")[0]}/step-5`);
            } catch (err: any) {
                listingStore.setError(
                    err.message || "Failed to save cultural settings.",
                );
            } finally {
                wizard.setSubmitting(false);
            }
        });
        return unregister;
    });
</script>

<div class="step-pane fade-in">
    {#if schema && schema.usesCulturalSettings}
        <div class="step-heading">
            <div class="step-icon-text">🕌</div>
            <div>
                <h1>
                    {i18n.locale === "ar"
                        ? "التهيئة الثقافية والاجتماعية"
                        : "GCC Cultural Settings"}
                </h1>
                <p>
                    {i18n.locale === "ar"
                        ? "هذه الإعدادات مهمة جداً للسوق المحلي. يبحث العرسان عن تهيئة قسم الجنسين أولاً قبل أي شيء آخر."
                        : "Keep your details updated. Couples filter by gender setup first before anything else."}
                </p>
            </div>
        </div>

        {#if $listingStore.submitError}
            <div class="error-banner">{$listingStore.submitError}</div>
        {/if}

        <div class="form-card">
            <div class="form-group">
                <p class="field-label" style="margin-bottom: 0;">
                    {i18n.t("listingsWizard.genderSection") ||
                        "Gender Section Setup"} <span class="required">*</span>
                </p>
                <p class="field-hint">
                    {i18n.locale === "ar"
                        ? "كيف يتم ترتيب وتوزيع الجنسين في الخدمة/القاعة؟"
                        : "How does your service/venue handle gender separation?"}
                </p>
                <div class="gender-grid">
                    {#each GENDER_SECTIONS as gs}
                        <button
                            type="button"
                            class="gender-tile"
                            class:selected={$listingStore.formData.genderSection === gs.value}
                            onclick={() => ($listingStore.formData.genderSection = gs.value)}
                        >
                            <span class="gender-icon">{gs.icon}</span>
                            <div class="gender-labels">
                                <span class="gender-en"
                                    >{i18n.locale === "ar" ? gs.ar : gs.en}</span
                                >
                                <span class="gender-desc">
                                    {i18n.locale === "ar"
                                        ? gs.value === "dual_parallel"
                                            ? "قسمين منفصلين (رجال ونساء)"
                                            : gs.value === "women_only"
                                              ? "مخصص لحفلات النساء فقط"
                                              : gs.value === "family"
                                                ? "عائلي / مختلط"
                                                : gs.value === "men_only"
                                                  ? "مخصص لحفلات الرجال فقط"
                                                  : gs.value === "both_sections"
                                                    ? "يتوفر كلا الخيارين"
                                                    : gs.value === "not_applicable"
                                                      ? "غير محدد"
                                                      : ""
                                        : gs.description}
                                </span>
                            </div>
                            {#if $listingStore.formData.genderSection === gs.value}
                                <div class="tile-check">
                                    <Check size={14} />
                                </div>
                            {/if}
                        </button>
                    {/each}
                </div>
            </div>

            <hr class="form-divider" />
            
            <DynamicFeatureGrid group={culturalGroup} storeKey="culturalAttributes" />
        </div>
    {:else}
        <div class="step-placeholder">
            {i18n.locale === "ar" ? "جاري تحميل الإعدادات الثقافية..." : "Loading cultural settings..."}
        </div>
    {/if}
</div>
