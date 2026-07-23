<script lang="ts">
    import './wizard.css';
    import { page } from '$app/stores';
    import { goto, beforeNavigate, invalidateAll } from '$app/navigation';
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import { listingStore } from '$lib/stores/listingStore';
    import { uiStore } from '$lib/stores/ui.svelte';
    import { ArrowLeft, ArrowRight, Check } from 'lucide-svelte';
    import { getSchemaForCategory } from "$lib/constants/wizardSchemas";
    import { getApiUrl } from '$lib/utils/api';
    import { wizardFetch } from '$lib/utils/wizardFetch';

    import { get, derived } from 'svelte/store';
    import { setContext, untrack } from 'svelte';


    const i18n = getI18n();
    let { basePath, children, isEditMode = false } = $props<{ basePath: string, children: any, isEditMode?: boolean }>();

    const TOTAL_STEPS = 9;

    const STEP_LABELS = $derived([
        i18n.t("listingsWizard.stepLabels.category") || "Category",
        i18n.t("listingsWizard.stepLabels.basicInfo") || "Basic Info",
        i18n.t("listingsWizard.stepLabels.description") || "Description",
        i18n.t("listingsWizard.stepLabels.cultural") || "Cultural Specs",
        i18n.t("listingsWizard.stepLabels.details") || "Details",
        i18n.t("listingsWizard.stepLabels.coordinator") || "Coordinator",
        i18n.t("listingsWizard.stepLabels.gallery") || "Gallery",
        i18n.t("listingsWizard.stepLabels.preview") || "Preview",
        i18n.t("listingsWizard.stepLabels.submission") || "Submit"
    ]);

    let currentStep = $derived(parseInt($page.params.stepId || '1') || 1);

    // Layout local context states
    let submitHandler = $state<(() => Promise<void>) | null>(null);
    let canContinue = $state(true);
    let isSubmitting = $state(false);

    setContext('wizard', {
        get submitHandler() { return submitHandler; },
        get canContinue() { return canContinue; },
        get isSubmitting() { return isSubmitting; },
        registerSubmitHandler: (handler: () => Promise<void>) => {
            submitHandler = handler;
            return () => {
                if (submitHandler === handler) {
                    submitHandler = null;
                }
            };
        },
        setCanContinue: (val: boolean) => {
            canContinue = val;
        },
        setSubmitting: (val: boolean) => {
            isSubmitting = val;
            uiStore.setLoading(val);
        }
    });

    const isNotApplicable = (stepNum: number) => {
        const schema = $listingStore.schema;
        const status = schema?.stepOverrides?.[String(stepNum)]?.status;
        return status === 'not_applicable' || status === 'NotApplicable';
    };

    const getReason = (stepNum: number) => {
        const schema = $listingStore.schema;
        return schema?.stepOverrides?.[String(stepNum)]?.reasonEn;
    };

    let activeFetchCategory = $state<string | null>(null);

    // Derived trigger store to isolate effect dependencies
    const schemaLoaderTrigger = derived(listingStore, $store => {
        const category = $store.formData.selectedCategory || '';
        const hasSchema = $store.schema ? 'yes' : 'no';
        const hasError = $store.schemaError ? 'yes' : 'no';
        return `${category}:${hasSchema}:${hasError}`;
    });

    // Backend schema resolution effect
    $effect(() => {
        const trigger = $schemaLoaderTrigger;
        const [category, hasSchema, hasError] = trigger.split(':');
        if (!category) return;

        untrack(() => {
            const schema = get(listingStore).schema;
            if (schema && schema.categoryId === category) {
                return;
            }

            // Return early if error is already set (prevents retry loop on failures)
            if (get(listingStore).schemaError) {
                return;
            }

            listingStore.setSchemaLoading(true);
            listingStore.setSchemaError(null);
            
            const sessionToken = $page.data.sessionToken;
            const url = getApiUrl(`/api/v1/vendor/wizard-schema/${category}`);
            
            const headers: Record<string, string> = {
                "X-Trace-ID": listingStore.getTraceId(),
            };
            if (sessionToken) {
                headers["Authorization"] = `Bearer ${sessionToken}`;
            }

            wizardFetch(url, { headers })
            .then(async (res) => {
                if (!res.ok) {
                    const err = await res.json().catch(() => ({}));
                    throw new Error(err.message || err.error || `Server error ${res.status}`);
                }
                const schemaData = await res.json();
                
                // Discard the response if user switched categories mid-flight
                if (get(listingStore).formData.selectedCategory === category) {
                    listingStore.setSchema(schemaData);
                    listingStore.setSchemaLoading(false);
                }
            })
            .catch((err) => {
                console.error("Failed to load backend wizard schema, falling back to local schema:", err);
                try {
                    const schemaData = getSchemaForCategory(category);
                    if (schemaData && get(listingStore).formData.selectedCategory === category) {
                        listingStore.setSchema(schemaData);
                        listingStore.setSchemaLoading(false);
                        return;
                    }
                } catch (_) {}
                
                if (get(listingStore).formData.selectedCategory === category) {
                    listingStore.setSchemaError(err.message || "Failed to load schema");
                    listingStore.setSchemaLoading(false);
                }
            });
        });
    });

    // Automatically skip not_applicable steps on direct loads/refreshes
    $effect(() => {
        const schema = $listingStore.schema;
        if (schema && schema.stepOverrides && isNotApplicable(currentStep)) {
            let nextStep = currentStep;
            while (nextStep <= TOTAL_STEPS && isNotApplicable(nextStep)) {
                nextStep++;
            }
            if (nextStep <= TOTAL_STEPS) {
                goto(`${basePath}/step-${nextStep}`, { replaceState: true });
            } else {
                let prevStep = currentStep;
                while (prevStep >= 1 && isNotApplicable(prevStep)) {
                    prevStep--;
                }
                if (prevStep >= 1) {
                    goto(`${basePath}/step-${prevStep}`, { replaceState: true });
                }
            }
        }
    });

    // Navigation Guard — handles redirection for all Back/Continue navigation
    beforeNavigate((navigation) => {
        if (isSubmitting) return;
        if (navigation.to?.route.id?.includes('step-[stepId]')) {
            const targetStep = parseInt(navigation.to.params?.stepId || '1');
            const storeVal = get(listingStore);
            const schema = storeVal.schema;
            const maxAllowed = storeVal.highestCompletedStep + 1;

            let resolvedTarget = targetStep;
            if (targetStep > maxAllowed) {
                resolvedTarget = maxAllowed;
            }

            if (schema && schema.stepOverrides) {
                const direction = resolvedTarget >= currentStep ? 'forward' : 'backward';
                let finalStep = resolvedTarget;

                if (direction === 'forward') {
                    while (finalStep <= TOTAL_STEPS) {
                        const status = schema.stepOverrides[String(finalStep)]?.status;
                        if (status === 'not_applicable' || status === 'NotApplicable') {
                            finalStep++;
                        } else {
                            break;
                        }
                    }
                    if (finalStep > TOTAL_STEPS) {
                        finalStep = TOTAL_STEPS;
                    }
                } else {
                    while (finalStep >= 1) {
                        const status = schema.stepOverrides[String(finalStep)]?.status;
                        if (status === 'not_applicable' || status === 'NotApplicable') {
                            finalStep--;
                        } else {
                            break;
                        }
                    }
                    if (finalStep < 1) {
                        finalStep = 1;
                    }
                }

                if (finalStep !== targetStep) {
                    navigation.cancel();
                    if (finalStep > maxAllowed) {
                        listingStore.setHighestStep(finalStep - 1);
                    }
                    goto(`${basePath}/step-${finalStep}`);
                    return;
                }
            } else if (resolvedTarget !== targetStep) {
                navigation.cancel();
                goto(`${basePath}/step-${resolvedTarget}`);
                return;
            }
        }
    });

    // Handle hard refresh / store cleared
    $effect(() => {
        if (currentStep > 1 && !$listingStore.productId && !isSubmitting) {
            if (isEditMode) {
                invalidateAll();
            } else {
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
                <div
                    class="progress-step"
                    class:done={i + 1 < currentStep && !isNotApplicable(i + 1)}
                    class:active={i + 1 === currentStep}
                    class:pending={i + 1 > currentStep && !isNotApplicable(i + 1)}
                    class:not-applicable={isNotApplicable(i + 1)}
                    title={isNotApplicable(i + 1) ? (getReason(i + 1) || "Not applicable") : ""}
                >
                    <div class="step-dot">
                        {#if isNotApplicable(i + 1)}
                            —
                        {:else if i + 1 < currentStep}
                            <Check size={12} />
                        {:else}
                            {i + 1}
                        {/if}
                    </div>
                    <span class="step-label">{label}</span>
                </div>
                {#if i < STEP_LABELS.length - 1}
                    <div 
                        class="progress-line" 
                        class:filled={i + 1 < currentStep && !isNotApplicable(i + 1) && !isNotApplicable(i + 2)}
                        class:not-applicable={isNotApplicable(i + 1) || isNotApplicable(i + 2)}
                    ></div>
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
        <button class="btn-secondary" onclick={() => { if(currentStep > 1) goto(`${basePath}/step-${currentStep - 1}`) }} disabled={currentStep === 1 || isSubmitting}>
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
                    class:done={i + 1 < currentStep && !isNotApplicable(i + 1)}
                    class:not-applicable={isNotApplicable(i + 1)}
                ></div>
            {/each}
        </div>

        {#if currentStep < TOTAL_STEPS}
            <button
                class="btn-primary"
                onclick={async () => { if (submitHandler && !isSubmitting) await submitHandler(); }}
                disabled={isSubmitting || !canContinue}
            >
                {currentStep === TOTAL_STEPS - 1 ? (i18n.locale === 'ar' ? "مراجعة" : "Review") : (i18n.locale === 'ar' ? "متابعة" : "Continue")}
                {#if i18n.locale === 'ar'}
                    <ArrowLeft size={18} />
                {:else}
                    <ArrowRight size={18} />
                {/if}
            </button>
        {:else}
            <div></div>
        {/if}
    </footer>
</div>
