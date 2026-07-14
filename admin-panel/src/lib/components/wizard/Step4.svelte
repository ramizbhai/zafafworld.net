<script lang="ts">
    import { getI18n } from "$lib/i18n/i18n.svelte";
    import { listingStore } from "$lib/stores/listingStore";
    import { GENDER_SECTIONS } from "$lib/constants/wizard";
    import { Check } from "lucide-svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import { env } from "$env/dynamic/public";

    const i18n = getI18n();
    let { data } = $props<{ data: { sessionToken: string } }>();
    const apiBase =
        env.PUBLIC_API_URL && env.PUBLIC_API_URL.includes("localhost")
            ? ""
            : env.PUBLIC_API_URL || "";

    let genderSection = $state($listingStore.formData.genderSection);
    let culturalAttributes = $state($listingStore.formData.culturalAttributes);
    let selectedCategory = $derived($listingStore.formData.selectedCategory);

    $effect(() => {
        listingStore.updateFormData({ genderSection, culturalAttributes });
    });

    let isValid = $derived(!!genderSection);
    $effect(() => {
        listingStore.setCanContinue(isValid);
    });

    $effect(() => {

        listingStore.setSubmitHandler(async () => {
            if (!genderSection) {
                listingStore.setError("Please select a gender section setup.");
                return;
            }

            listingStore.setSubmitting(true);
            listingStore.setError("");

            try {
                const url = `${apiBase}/api/v1/vendor/products/${$listingStore.productId}`;
                const payload = {
                    version: $listingStore.version,
                    genderSection: genderSection || null,
                    culturalAttributes: culturalAttributes,
                };

                const res = await fetch(url, {
                    method: "PUT",
                    headers: {
                        "Content-Type": "application/json",
                        Authorization: `Bearer ${data.sessionToken}`,
                    },
                    body: JSON.stringify(payload),
                });

                if (!res.ok) {
                    const err = await res.json().catch(() => ({}));
                    throw new Error(err.error || `Server error ${res.status}`);
                }

                const responseData = await res.json();
                if (responseData.product?.version) {
                    listingStore.setVersion(responseData.product.version);
                }

                listingStore.setHighestStep(4);
                goto(`${$page.url.pathname.split("/step-")[0]}/step-5`);
            } catch (err: any) {
                listingStore.setError(
                    err.message || "Failed to save cultural settings.",
                );
            } finally {
                listingStore.setSubmitting(false);
            }
        });
    });
</script>

<div class="step-pane fade-in">
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
                    : "These settings are critical for the Saudi and GCC market. Couples filter by gender setup first before anything else."}
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
                        class:selected={genderSection === gs.value}
                        onclick={() => (genderSection = gs.value)}
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
                        {#if genderSection === gs.value}
                            <div class="tile-check">
                                <Check size={14} />
                            </div>
                        {/if}
                    </button>
                {/each}
            </div>
        </div>

        {#if ["wedding-palace", "hotel-venue", "villa-resort", "restaurant-event", "outdoor-garden", "chalet"].includes(selectedCategory)}
            <hr class="form-divider" />
            <div class="form-group">
                <span class="amenity-label"
                    >{i18n.locale === "ar"
                        ? "المرافق والخدمات الرئيسية"
                        : "Key Amenities"}</span
                >
                <div class="amenity-checks">
                    {#each [{ key: "prayer_room", en: "Prayer Room / Musala", ar: "مصلى" }, { key: "valet_parking", en: "Valet Parking", ar: "صف السيارات" }, { key: "bridal_suite", en: "Bridal Suite", ar: "جناح العروس" }, { key: "outdoor_garden", en: "Outdoor Garden", ar: "حديقة خارجية" }, { key: "external_catering_allowed", en: "External Catering Allowed", ar: "ضيافة خارجية مسموحة" }, { key: "halal_certified", en: "Halal Certified", ar: "شهادة حلال" }] as amenity}
                        <label class="amenity-check-label">
                            <input
                                type="checkbox"
                                bind:checked={culturalAttributes[amenity.key]}
                            />
                            <span class="check-label"
                                >{i18n.locale === "ar"
                                    ? amenity.ar
                                    : amenity.en}</span
                            >
                        </label>
                    {/each}
                </div>
            </div>
        {/if}
    </div>
</div>
