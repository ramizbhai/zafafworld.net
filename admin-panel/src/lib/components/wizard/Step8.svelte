<script lang="ts">
    import { getI18n } from "$lib/i18n/i18n.svelte";
    import { listingStore } from "$lib/stores/listingStore";
    import { GENDER_SECTIONS } from "$lib/constants/wizard";
    import { CATEGORY_GROUPS } from "$lib/constants/wizard";
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";

    const i18n = getI18n();
    let { data } = $props<{ data: { sessionToken: string } }>();

    $effect(() => {
        listingStore.setCanContinue(true);
    });

    const selectedCategoryMeta = $derived.by(() => {
        const cat = $listingStore.formData.selectedCategory;
        for (const group of CATEGORY_GROUPS) {
            const found = group.items.find((i: any) => i.value === cat);
            if (found) return found;
        }
        return null;
    });

    const coverItem = $derived($listingStore.formData.coverItem);
    const fd = $derived($listingStore.formData);

    const checklist = $derived([
        {
            label: i18n.locale === "ar" ? "العنوان بالعربية" : "Arabic Title",
            ok: !!fd.titleAr.trim(),
        },
        {
            label:
                i18n.locale === "ar" ? "العنوان بالإنجليزية" : "English Title",
            ok: !!fd.titleEn.trim(),
        },
        {
            label:
                i18n.locale === "ar" ? "الوصف بالعربية" : "Arabic Description",
            ok: fd.descriptionAr.length >= 2,
        },
        {
            label:
                i18n.locale === "ar"
                    ? "الوصف بالإنجليزية"
                    : "English Description",
            ok: fd.descriptionEn.length >= 2,
        },
        {
            label: i18n.locale === "ar" ? "تحديد السعر" : "Price set",
            ok:
                fd.priceOnInquiry ||
                (!!fd.basePriceSar && parseFloat(fd.basePriceSar) > 0),
        },
        {
            label: i18n.locale === "ar" ? "تصنيف الجنسين" : "Gender Section",
            ok: !!fd.genderSection,
        },
        {
            label: i18n.locale === "ar" ? "اختيار المدينة" : "City selected",
            ok: !!fd.selectedCityId,
        },
        {
            label: i18n.locale === "ar" ? "صورة الغلاف" : "Cover image",
            ok: !!coverItem && coverItem.status === "completed",
        },
        {
            label: i18n.locale === "ar" ? "معلومات المنسق" : "Coordinator info",
            ok: !!(
                fd.coordinatorPhone &&
                fd.coordinatorNameEn &&
                fd.coordinatorNameAr &&
                fd.coordinatorWhatsapp &&
                fd.coordinatorEmail
            ),
        },
    ]);

    const canSubmit = $derived(checklist.every((c) => c.ok));

    $effect(() => {
        listingStore.setSubmitHandler(async () => {
            if (!canSubmit) {
                listingStore.setError(
                    "Please complete all required fields before reviewing.",
                );
                return;
            }

            listingStore.setHighestStep(8);
            goto(`${$page.url.pathname.split("/step-")[0]}/step-9`);
        });
    });
</script>

<div class="step-pane fade-in">
    <div class="step-heading">
        <div class="step-icon-text">👁️</div>
        <div>
            <h1>
                {i18n.locale === "ar"
                    ? "معاينة الإعلان"
                    : "Preview Your Listing"}
            </h1>
            <p>
                {i18n.locale === "ar"
                    ? "راجع كل شيء قبل التقديم لموافقة الإدارة. يمكنك الرجوع إلى أي خطوة لإجراء تغييرات."
                    : "Review everything before submitting for admin approval. You can go back to any step to make changes."}
            </p>
        </div>
    </div>

    {#if $listingStore.submitError}
        <div class="error-banner">{$listingStore.submitError}</div>
    {/if}

    <div class="preview-card">
        {#if coverItem && coverItem.status === "completed"}
            <img
                src={coverItem.url.startsWith("http")
                    ? coverItem.url
                    : `${coverItem.url}`}
                alt="Cover"
                class="preview-cover"
            />
        {/if}
        <div class="preview-body">
            <span class="preview-category">
                {selectedCategoryMeta?.emoji}
                {i18n.locale === "ar"
                    ? selectedCategoryMeta?.ar
                    : selectedCategoryMeta?.en}
            </span>
            <h2 class="preview-title-en">{fd.titleEn || "—"}</h2>
            <h2 class="preview-title-ar">{fd.titleAr || "—"}</h2>
            <div class="preview-meta">
                {#if !fd.priceOnInquiry && fd.basePriceSar}
                    <span class="preview-price">
                        {i18n.locale === "ar" ? "تبدأ من " : "From "} ﷼{parseInt(
                            fd.basePriceSar,
                        ).toLocaleString(
                            i18n.locale === "ar" ? "ar-SA" : "en-SA",
                        )}
                        {i18n.locale === "ar" ? "ريال" : "SAR"}
                    </span>
                {:else}
                    <span class="preview-price inquiry">
                        {i18n.t("listingsWizard.priceOnInquiry") ||
                            "Price on Inquiry"}
                    </span>
                {/if}
                {#if fd.genderSection}
                    <span class="preview-gender">
                        {GENDER_SECTIONS.find(
                            (g) => g.value === fd.genderSection,
                        )?.icon}
                        {i18n.locale === "ar"
                            ? GENDER_SECTIONS.find(
                                  (g) => g.value === fd.genderSection,
                              )?.ar
                            : GENDER_SECTIONS.find(
                                  (g) => g.value === fd.genderSection,
                              )?.en}
                    </span>
                {/if}
            </div>
            {#if i18n.locale === "ar" ? fd.descriptionAr || fd.descriptionEn : fd.descriptionEn || fd.descriptionAr}
                <p class="preview-desc">
                    {(i18n.locale === "ar"
                        ? fd.descriptionAr || fd.descriptionEn
                        : fd.descriptionEn || fd.descriptionAr
                    ).slice(0, 300)}
                    {(i18n.locale === "ar"
                        ? fd.descriptionAr || fd.descriptionEn
                        : fd.descriptionEn || fd.descriptionAr
                    ).length > 300
                        ? "..."
                        : ""}
                </p>
            {/if}
        </div>
    </div>

    <div class="preview-checklist">
        <h3>
            {i18n.t("listingsWizard.listingCompleteness") ||
                "Listing Completeness"}
        </h3>
        {#each checklist as check}
            <div class="check-row">
                <span class={check.ok ? "check-ok" : "check-missing"}
                    >{check.ok ? "✓" : "○"}</span
                >
                {check.label}
            </div>
        {/each}
    </div>
</div>
