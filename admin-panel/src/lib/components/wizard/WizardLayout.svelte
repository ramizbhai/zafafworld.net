<script lang="ts">
    import './wizard.css';
    import { page } from '$app/stores';
    import { goto, beforeNavigate, invalidateAll } from '$app/navigation';
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import { listingStore } from '$lib/stores/listingStore';
    import { ArrowLeft, ArrowRight, Check } from 'lucide-svelte';

    const i18n = getI18n();
    let { basePath, children, isEditMode = false } = $props<{ basePath: string, children: any, isEditMode?: boolean }>();

    const TOTAL_STEPS = 9;

    const STEP_LABELS = $derived([
        i18n.t("listingsWizard.stepLabels.category") || "Category",
        i18n.t("listingsWizard.stepLabels.basicInfo") || "Basic Info",
        i18n.t("listingsWizard.stepLabels.description") || "Description",
        i18n.t("listingsWizard.stepLabels.culturalSetup") || "Cultural Setup",
        i18n.t("listingsWizard.stepLabels.details") || "Details",
        i18n.t("listingsWizard.stepLabels.coordinator") || "Coordinator",
        i18n.t("listingsWizard.stepLabels.gallery") || "Gallery",
        i18n.t("listingsWizard.stepLabels.preview") || "Preview",
        i18n.t("listingsWizard.stepLabels.submit") || "Submit",
    ]);

    let currentStep = $derived(parseInt(($page.params as any).stepId || '1') || 1);

    // Navigation Guard
    beforeNavigate((navigation) => {
        if (navigation.to?.route.id?.includes('step-[stepId]')) {
            const targetStep = parseInt((navigation.to.params as any)?.stepId || '1');
            const maxAllowed = $listingStore.highestCompletedStep + 1;
            
            if (targetStep > maxAllowed) {
                navigation.cancel();
                goto(`${basePath}/step-${maxAllowed}`);
            }
        }
    });

    // Handle hard refresh / store cleared:
    // - NEW flow: no productId on step > 1 → redirect to step-1 so a new product is created
    // - EDIT flow: no productId on step > 1 → call invalidateAll() to re-trigger the SSR
    //   layout loader which will re-fetch the product and re-initialize the store.
    $effect(() => {
        if (currentStep > 1 && !$listingStore.productId) {
            if (isEditMode) {
                // Trigger the server-side layout load to re-hydrate the store
                invalidateAll();
            } else {
                // New flow: store was cleared, start over
                goto(`${basePath}/step-1`, { replaceState: true });
            }
        }
    });
</script>

<div class="wizard-shell" dir={i18n.locale === 'ar' ? 'rtl' : 'ltr'}>
    <header class="wizard-header">
        <a href="/dashboard/products" class="back-link">
            {#if i18n.locale === 'ar'}
                <ArrowRight size={18} />
            {:else}
                <ArrowLeft size={18} />
            {/if}
            {i18n.t("listingsWizard.backToListings") || "Back to Listings"}
        </a>

        <div class="wizard-progress">
            {#each STEP_LABELS as label, i}
                <div class="progress-step" class:done={i + 1 < currentStep} class:active={i + 1 === currentStep} class:pending={i + 1 > currentStep}>
                    <div class="step-dot">
                        {#if i + 1 < currentStep}
                            <Check size={12} />
                        {:else}
                            {i + 1}
                        {/if}
                    </div>
                    <span class="step-label">{label}</span>
                </div>
                {#if i < STEP_LABELS.length - 1}
                    <div class="progress-line" class:filled={i + 1 < currentStep}></div>
                {/if}
            {/each}
        </div>

        <span class="step-counter">
            {(i18n.t("listingsWizard.stepCounter") || "Step {step} of {total}").replace("{step}", String(currentStep)).replace("{total}", String(TOTAL_STEPS))}
        </span>
    </header>

    <main class="wizard-content">
        {@render children()}
    </main>

    <footer class="wizard-footer">
        <button class="btn-secondary" onclick={() => { if(currentStep > 1) goto(`${basePath}/step-${currentStep - 1}`) }} disabled={currentStep === 1 || $listingStore.isSubmitting}>
            {#if i18n.locale === 'ar'}
                <ArrowRight size={18} />
            {:else}
                <ArrowLeft size={18} />
            {/if}
            {i18n.locale === 'ar' ? 'السابق' : 'Back'}
        </button>

        <div class="footer-center">
            {#each Array(TOTAL_STEPS) as _, i}
                <div
                    class="dot-nav"
                    class:active={i + 1 === currentStep}
                    class:done={i + 1 < currentStep}
                ></div>
            {/each}
        </div>

        {#if currentStep < TOTAL_STEPS}
            <button
                class="btn-primary"
                onclick={() => listingStore.submitCurrentStep()}
                disabled={$listingStore.isSubmitting || !$listingStore.canContinue}
            >
                {#if $listingStore.isSubmitting}
                    <span class="spinner">⏳</span>
                {:else}
                    {currentStep === TOTAL_STEPS - 1 ? (i18n.locale === 'ar' ? "مراجعة" : "Review") : (i18n.locale === 'ar' ? "متابعة" : "Continue")}
                    {#if i18n.locale === 'ar'}
                        <ArrowLeft size={18} />
                    {:else}
                        <ArrowRight size={18} />
                    {/if}
                {/if}
            </button>
        {:else}
            <div></div>
        {/if}
    </footer>
</div>
