<script lang="ts">
    import { getContext, onMount, onDestroy } from "svelte";
    import { getI18n } from "$lib/i18n/i18n.svelte";
    import { listingStore } from "$lib/stores/listingStore";
    import UnifiedDescriptionBuilder from "$lib/components/UnifiedDescriptionBuilder.svelte";
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

    let descriptionAr = $state($listingStore.formData.descriptionAr);
    let descriptionEn = $state($listingStore.formData.descriptionEn);

    let isValid = $derived(
        descriptionAr.trim().length >= 50 || descriptionEn.trim().length >= 50
    );

    $effect(() => {
        wizard.setCanContinue(isValid);
    });

    let metaTitleAr = $state($listingStore.formData.metaTitleAr);
    let metaTitleEn = $state($listingStore.formData.metaTitleEn);
    let metaDescriptionAr = $state($listingStore.formData.metaDescriptionAr);
    let metaDescriptionEn = $state($listingStore.formData.metaDescriptionEn);

    let titleAr = $derived($listingStore.formData.titleAr);
    let titleEn = $derived($listingStore.formData.titleEn);

    // Sync state on unmount (synchronous safety net)
    onDestroy(() => {
        listingStore.updateFormData({
            descriptionAr,
            descriptionEn,
            metaTitleAr,
            metaTitleEn,
            metaDescriptionAr,
            metaDescriptionEn,
        });
    });

    // 300ms debounced background sync
    $effect(() => {
        const _watch = descriptionAr + descriptionEn + metaTitleAr + metaTitleEn +
                       metaDescriptionAr + metaDescriptionEn;
        const timer = setTimeout(() => {
            listingStore.updateFormData({
                descriptionAr, descriptionEn, metaTitleAr, metaTitleEn,
                metaDescriptionAr, metaDescriptionEn,
            });
        }, 300);
        return () => clearTimeout(timer);
    });

    onMount(() => {
        const unregister = wizard.registerSubmitHandler(async () => {
            if (
                descriptionAr.trim().length < 50 &&
                descriptionEn.trim().length < 50
            ) {
                listingStore.setError(
                    "At least one description must be 50 characters or longer.",
                );
                return;
            }

            // Sync state immediately before API call
            listingStore.updateFormData({
                descriptionAr,
                descriptionEn,
                metaTitleAr,
                metaTitleEn,
                metaDescriptionAr,
                metaDescriptionEn,
            });

            // Dirty check bypass
            const isDirty = listingStore.isStepDirty(3, $listingStore);
            if (!isDirty) {
                listingStore.setHighestStep(3);
                goto(`${$page.url.pathname.split("/step-")[0]}/step-4`);
                return;
            }

            wizard.setSubmitting(true);
            listingStore.setError("");

            try {
                const url = getApiUrl(`/api/v1/vendor/products/${$listingStore.productId}`);
                const payload = {
                    version: $listingStore.version,
                    descriptionAr: descriptionAr || null,
                    descriptionEn: descriptionEn || null,
                    metaTitleAr: metaTitleAr || null,
                    metaTitleEn: metaTitleEn || null,
                    metaDescriptionAr: metaDescriptionAr || null,
                    metaDescriptionEn: metaDescriptionEn || null,
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

                listingStore.commitStepSave(3);
                listingStore.setHighestStep(3);
                goto(`${$page.url.pathname.split("/step-")[0]}/step-4`);
            } catch (err: any) {
                listingStore.setError(
                    err.message || "Failed to save description.",
                );
            } finally {
                wizard.setSubmitting(false);
            }
        });
        return unregister;
    });
</script>

<div class="step-pane fade-in">
    <div class="step-heading">
        <div class="step-icon-text">📝</div>
        <div>
            <h1>
                {i18n.locale === "ar" ? "وصف الإعلان وتحسين محركات البحث" : "Listing Description & SEO"}
            </h1>
            <p>
                {i18n.locale === "ar"
                    ? "أضف فقرات وصف، صور، وخرائط بمرونة باللغتين العربية والإنجليزية، وقم بإعداد الميتا الخاص بك للوصول للمزيد من الزوار."
                    : "Add description blocks, images, and maps flexibly in both Arabic and English, and configure your Meta tags to reach more visitors."}
            </p>
        </div>
    </div>

    {#if $listingStore.submitError}
        <div class="error-banner">{$listingStore.submitError}</div>
    {/if}

    <div class="form-card">
        <UnifiedDescriptionBuilder
            bind:descriptionAr
            bind:descriptionEn
            bind:metaTitleAr
            bind:metaTitleEn
            bind:metaDescriptionAr
            bind:metaDescriptionEn
            {titleAr}
            {titleEn}
        />
    </div>
</div>
