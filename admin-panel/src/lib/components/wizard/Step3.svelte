<script lang="ts">
    import { getI18n } from "$lib/i18n/i18n.svelte";
    import { listingStore } from "$lib/stores/listingStore";
    import UnifiedDescriptionBuilder from "$lib/components/UnifiedDescriptionBuilder.svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import { env } from "$env/dynamic/public";

    const i18n = getI18n();
    let { data } = $props<{ data: { sessionToken: string } }>();
    const apiBase =
        env.PUBLIC_API_URL && env.PUBLIC_API_URL.includes("localhost")
            ? ""
            : env.PUBLIC_API_URL || "";

    let descriptionAr = $state($listingStore.formData.descriptionAr);
    let descriptionEn = $state($listingStore.formData.descriptionEn);

    $effect(() => {
        listingStore.setCanContinue(true);
    });

    let metaTitleAr = $state($listingStore.formData.metaTitleAr);
    let metaTitleEn = $state($listingStore.formData.metaTitleEn);
    let metaDescriptionAr = $state($listingStore.formData.metaDescriptionAr);
    let metaDescriptionEn = $state($listingStore.formData.metaDescriptionEn);

    let titleAr = $derived($listingStore.formData.titleAr);
    let titleEn = $derived($listingStore.formData.titleEn);

    $effect(() => {
        listingStore.updateFormData({
            descriptionAr,
            descriptionEn,
            metaTitleAr,
            metaTitleEn,
            metaDescriptionAr,
            metaDescriptionEn,
        });

        listingStore.setSubmitHandler(async () => {
            if (
                descriptionAr.trim().length < 50 &&
                descriptionEn.trim().length < 50
            ) {
                listingStore.setError(
                    "At least one description must be 50 characters or longer.",
                );
                return;
            }

            listingStore.setSubmitting(true);
            listingStore.setError("");

            try {
                const url = `${apiBase}/api/v1/vendor/products/${$listingStore.productId}`;
                const payload = {
                    version: $listingStore.version,
                    descriptionAr: descriptionAr || null,
                    descriptionEn: descriptionEn || null,
                    metaTitleAr: metaTitleAr || null,
                    metaTitleEn: metaTitleEn || null,
                    metaDescriptionAr: metaDescriptionAr || null,
                    metaDescriptionEn: metaDescriptionEn || null,
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

                listingStore.setHighestStep(3);
                goto(`${$page.url.pathname.split("/step-")[0]}/step-4`);
            } catch (err: any) {
                listingStore.setError(
                    err.message || "Failed to save description.",
                );
            } finally {
                listingStore.setSubmitting(false);
            }
        });
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
