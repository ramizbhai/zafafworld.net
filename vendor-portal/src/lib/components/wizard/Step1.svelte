<script lang="ts">
    import { onMount } from "svelte";
    import { getContext } from "svelte";
    import { getI18n } from "$lib/i18n/i18n.svelte";
    import { listingStore } from "$lib/stores/listingStore";
    import { CATEGORY_GROUPS } from "$lib/constants/wizard";
    import { Sparkles, Check } from "lucide-svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import { checkSubscriptionQuota } from "$lib/utils/subscriptionGuard";
    import { triggerUpgrade } from "$lib/stores/upgradeStore";
    import { getApiUrl } from "$lib/utils/api";
    import { wizardFetch } from "$lib/utils/wizardFetch";

    const i18n = getI18n();
    let { data } = $props<{ data: { sessionToken: string } }>();

    const wizard = getContext<{
        registerSubmitHandler: (handler: () => Promise<void>) => () => void;
        setCanContinue: (val: boolean) => void;
        setSubmitting: (val: boolean) => void;
    }>('wizard');

    // Reactive store bindings
    let selectedCategory = $derived($listingStore.formData.selectedCategory);

    function selectCategory(cat: string) {
        listingStore.updateFormData({ selectedCategory: cat });
    }

    let isValid = $derived(!!selectedCategory);
    $effect(() => {
        wizard.setCanContinue(isValid);
    });

    onMount(() => {
        const unregister = wizard.registerSubmitHandler(async () => {
            if (!selectedCategory) return;

            // Dirty check bypass
            const isDirty = listingStore.isStepDirty(1, $listingStore);
            if ($listingStore.productId && !isDirty) {
                listingStore.setHighestStep(1);
                await goto(`${$page.url.pathname.split("/step-")[0]}/step-2`);
                return;
            }

            wizard.setSubmitting(true);
            listingStore.setError("");

            try {
                // Determine API URL (POST if new, PUT if existing draft)
                const url = $listingStore.productId
                    ? getApiUrl(`/api/v1/vendor/products/${$listingStore.productId}`)
                    : getApiUrl(`/api/v1/vendor/products`);

                const method = $listingStore.productId ? "PUT" : "POST";

                const payload = {
                    productCategory: selectedCategory,
                    version: $listingStore.productId ? $listingStore.version : undefined,
                };

                const { blocked, response: res } = await checkSubscriptionQuota(async () => {
                    return await wizardFetch(url, {
                        method,
                        headers: {
                            "Content-Type": "application/json",
                            Authorization: `Bearer ${data.sessionToken}`,
                            "X-Trace-ID": listingStore.getTraceId(),
                        },
                        body: JSON.stringify(payload),
                    });
                });

                if (blocked) {
                    triggerUpgrade(
                        'products', 
                        $page.data.telemetry?.vendor?.tier_id ?? '', 
                        i18n.locale === 'ar' ? 'لقد وصلت إلى الحد الأقصى لعدد الإعلانات.' : 'Subscription quota limit reached.'
                    );
                    listingStore.setError("Subscription quota limit reached.");
                    wizard.setSubmitting(false);
                    return;
                }

                if (!res.ok) {
                    const err = await res.json().catch(() => ({}));
                    throw new Error(err.message || err.error || `Server error ${res.status}`);
                }

                const responseData = await res.json();

                const newProductId = !$listingStore.productId && responseData.id;
                // If it's a POST, save the generated product ID and mark active /new session flag
                if (newProductId) {
                    listingStore.setProductId(responseData.id);
                    if (typeof window !== 'undefined') {
                        sessionStorage.setItem('zafaf_wiz_new_active', 'true');
                    }
                }
                
                if (responseData.product?.version) {
                    listingStore.setVersion(responseData.product.version);
                }

                listingStore.commitStepSave(1);
                listingStore.setHighestStep(1);

                // If we were editing an existing product AND the category actually changed,
                // wipe any attributes/features/cultural data from the old category.
                // The backend also clears these on its side when category_changed is true,
                // but we must also clear the client store so that Steps 4/5 don't re-send
                // stale old-category data in a subsequent PUT.
                if ($listingStore.productId && isDirty) {
                    listingStore.clearCategoryDependentData();
                }

                const targetId = responseData.id || $listingStore.productId;
                if ($page.url.pathname.includes('/new') && targetId) {
                    // Route Promotion: transition from new flow to edit flow
                    await goto(`/dashboard/products/${targetId}/edit/step-2`);
                } else {
                    await goto(`${$page.url.pathname.split("/step-")[0]}/step-2`);
                }
            } catch (err: any) {
                listingStore.setError(
                    err.message || "Failed to save category.",
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
        <Sparkles class="step-icon" size={28} />
        <div>
            <h1>
                {i18n.t("listingsWizard.whatsCategory") ||
                    "What's your category?"}
            </h1>
            <p>
                {i18n.t("listingsWizard.categoryDesc") ||
                    "Select the category that best describes your service."}
            </p>
        </div>
    </div>

    {#if $listingStore.submitError}
        <div class="error-banner">{$listingStore.submitError}</div>
    {/if}

    <div class="category-groups">
        {#each CATEGORY_GROUPS as group}
            <div class="group-section">
                <h3 class="group-title">
                    <span class="group-emoji">{group.emoji}</span>
                    {i18n.locale === "ar" ? group.labelAr : group.labelEn}
                </h3>
                <div class="category-grid">
                    {#each group.items as cat}
                        <button
                            type="button"
                            class="category-tile"
                            class:selected={selectedCategory === cat.value}
                            onclick={() => selectCategory(cat.value)}
                        >
                            <span class="tile-emoji">{cat.emoji}</span>
                            <span class="tile-en"
                                >{i18n.locale === "ar" ? cat.ar : cat.en}</span
                            >
                            {#if selectedCategory === cat.value}
                                <div class="tile-check">
                                    <Check size={14} />
                                </div>
                            {/if}
                        </button>
                    {/each}
                </div>
            </div>
        {/each}
    </div>
</div>

