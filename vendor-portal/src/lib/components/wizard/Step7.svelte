<script lang="ts">
    import { getI18n } from "$lib/i18n/i18n.svelte";
    import { Camera } from "lucide-svelte";
    import { listingStore } from "$lib/stores/listingStore";
    import { vendorStore } from "$lib/stores/vendorStore";
    import { WizardUploadState } from "../../features/vendor/wizard/wizardState.svelte";
    import GalleryDropzone from "./GalleryDropzone.svelte";
    import MediaPreviewGrid from "./MediaPreviewGrid.svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import { getApiUrl } from "$lib/utils/api";
    import { getContext } from "svelte";
    import { wizardFetch } from "$lib/utils/wizardFetch";

    let { data } = $props<{ data: { sessionToken: string } }>();
    const i18n = getI18n();
    const state = new WizardUploadState();

    const wizard = getContext<{
        registerSubmitHandler: (handler: () => Promise<void>) => () => void;
        setCanContinue: (val: boolean) => void;
        setSubmitting: (val: boolean) => void;
    }>('wizard');

    let maxCoverPhotos = $derived($vendorStore.policy_limits.cover_photos);
    let maxAdditionalPhotos = $derived($vendorStore.policy_limits.photos);
    let maxVideos = $derived($vendorStore.policy_limits.videos);

    $effect(() => {
        state.setSessionToken(data.sessionToken);
        // Init state on mount if it's currently empty, pulling from store
        if (state.coverItem === null && state.galleryItems.length === 0) {
            state.init($listingStore.formData.coverItem, $listingStore.formData.galleryItems || []);
        }
    });

    $effect(() => {
        wizard.setCanContinue(state.isValid);
        listingStore.updateFormData({ coverItem: state.coverItem, galleryItems: state.galleryItems });
    });

    $effect(() => {
        const unregister = wizard.registerSubmitHandler(async () => {
            if (state.isUploadingAny) {
                listingStore.setError("Wait for all uploads to finish before continuing.");
                return;
            }
            if (!state.isValid) {
                listingStore.setError("Cover image is required and must be fully uploaded.");
                return;
            }

            const isDirty = listingStore.isStepDirty(7, $listingStore);
            if (!isDirty) {
                listingStore.setHighestStep(7);
                await goto(`${$page.url.pathname.split("/step-")[0]}/step-8`);
                return;
            }

            wizard.setSubmitting(true);
            listingStore.setError("");

            try {
                listingStore.updateFormData({ coverItem: state.coverItem, galleryItems: state.galleryItems });

                const url = getApiUrl(`/api/v1/vendor/products/${$listingStore.productId}`);
                const payload = listingStore.getApiPayload($listingStore);

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

                listingStore.commitStepSave(7);
                listingStore.setHighestStep(7);
                await goto(`${$page.url.pathname.split("/step-")[0]}/step-8`);
            } catch (err: any) {
                listingStore.setError(err.message || "Failed to save gallery items.");
            } finally {
                wizard.setSubmitting(false);
            }
        });
        return unregister;
    });
</script>

<div class="step-pane fade-in">
    <div class="step-heading">
        <Camera class="step-icon" size={28} />
        <div>
            <h1>{i18n.t("listingsWizard.galleryMedia") || "Gallery Media"}</h1>
            <p>{i18n.t("listingsWizard.galleryMediaDesc") || "Upload high-quality photos and videos."}</p>
        </div>
    </div>

    {#if $listingStore.submitError}
        <div class="error-banner">{$listingStore.submitError}</div>
    {/if}

    <div class="form-card gallery-section split-layout">
        <div class="cover-side">
            <GalleryDropzone {state} />
        </div>
        <MediaPreviewGrid {state} {maxAdditionalPhotos} {maxVideos} />
    </div>
</div>

<style>
    .split-layout {
        display: grid;
        grid-template-columns: 60% 1fr;
        gap: 2rem;
        align-items: start;
    }
    .cover-side {
        display: flex;
        flex-direction: column;
    }
    @media (max-width: 768px) {
        .split-layout {
            grid-template-columns: 1fr;
            gap: 1.5rem;
        }
    }
</style>
