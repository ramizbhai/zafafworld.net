<script lang="ts">
    import { onMount } from "svelte";
    import { getI18n } from "$lib/i18n/i18n.svelte";
    import { getApiUrl } from "$lib/utils/api";
    import { ui } from "$lib/stores/ui.store";

    let { featuresSelection = $bindable({}) } = $props<{
        featuresSelection: Record<string, any>;
    }>();

    const i18n = getI18n();

    interface Feature {
        id: string;
        nameEn: string;
        nameAr: string;
        category: string;
        inputType: string;
    }

    let allFeatures = $state<Feature[]>([]);
    let fetchError = $state("");

    onMount(async () => {
        ui.setLoading(true);
        try {
            const res = await fetch(getApiUrl("/api/v1/features"));
            if (!res.ok) {
                throw new Error("Failed to load features");
            }
            const data = await res.json();
            if (data.status === "success") {
                allFeatures = data.data;
            } else {
                throw new Error(data.message || "Failed to load features");
            }
        } catch (err: any) {
            fetchError = err.message;
        } finally {
            ui.setLoading(false);
        }
    });

    const categories = $derived(
        [...new Set(allFeatures.map(f => f.category))]
    );

    function getFeaturesByCategory(category: string) {
        return allFeatures.filter(f => f.category === category);
    }
</script>

<div class="features-selection-wrapper">
    {#if fetchError}
        <div class="error-state">
            <p>{fetchError}</p>
        </div>
    {:else}
        {#each categories as category}
            <div class="feature-category">
                <h3 class="category-title">{category === 'General Info' ? (i18n.locale === 'ar' ? 'معلومات عامة' : 'General Info') : (i18n.locale === 'ar' ? 'المرافق والخدمات' : 'Amenities')}</h3>
                <div class="features-grid">
                    {#each getFeaturesByCategory(category) as feature}
                        <div class="feature-item">
                            {#if feature.inputType === 'boolean'}
                                <label class="checkbox-label">
                                    <input 
                                        type="checkbox" 
                                        bind:checked={featuresSelection[feature.id]} 
                                    />
                                    <span>{i18n.locale === 'ar' ? feature.nameAr : feature.nameEn}</span>
                                </label>
                            {:else if feature.inputType === 'number'}
                                <div class="input-group">
                                    <label for="feature-{feature.id}">{i18n.locale === 'ar' ? feature.nameAr : feature.nameEn}</label>
                                    <input 
                                        id="feature-{feature.id}"
                                        type="number" 
                                        bind:value={featuresSelection[feature.id]} 
                                        class="text-input"
                                        placeholder="0"
                                    />
                                </div>
                            {:else}
                                <div class="input-group">
                                    <label for="feature-{feature.id}">{i18n.locale === 'ar' ? feature.nameAr : feature.nameEn}</label>
                                    <input 
                                        id="feature-{feature.id}"
                                        type="text" 
                                        bind:value={featuresSelection[feature.id]} 
                                        class="text-input"
                                    />
                                </div>
                            {/if}
                        </div>
                    {/each}
                </div>
            </div>
        {/each}
    {/if}
</div>

<style>
    .features-selection-wrapper {
        display: flex;
        flex-direction: column;
        gap: 24px;
        width: 100%;
    }

    .error-state {
        padding: 20px;
        text-align: center;
        color: #6b7280;
    }

    .error-state {
        color: #ef4444;
    }

    .feature-category {
        background: #ffffff;
        border-radius: 12px;
        padding: 20px;
        border: 1px solid #e5e7eb;
        box-shadow: 0 1px 3px rgba(0,0,0,0.05);
    }

    .category-title {
        font-size: 1.125rem;
        font-weight: 600;
        color: #111827;
        margin-bottom: 16px;
        padding-bottom: 12px;
        border-bottom: 1px solid #f3f4f6;
    }

    .features-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
        gap: 16px;
    }

    .feature-item {
        display: flex;
        flex-direction: column;
        justify-content: center;
    }

    .checkbox-label {
        display: flex;
        align-items: center;
        gap: 8px;
        cursor: pointer;
        font-size: 0.95rem;
        color: #374151;
        user-select: none;
    }

    .checkbox-label input[type="checkbox"] {
        width: 18px;
        height: 18px;
        accent-color: #fca5a5; /* matching theme */
        border-radius: 4px;
        cursor: pointer;
    }

    .input-group {
        display: flex;
        flex-direction: column;
        gap: 6px;
    }

    .input-group label {
        font-size: 0.9rem;
        font-weight: 500;
        color: #374151;
    }

    .text-input {
        padding: 8px 12px;
        border: 1px solid #d1d5db;
        border-radius: 6px;
        font-size: 0.95rem;
        transition: all 0.2s;
    }

    .text-input:focus {
        outline: none;
        border-color: #fca5a5;
        box-shadow: 0 0 0 3px rgba(252, 165, 165, 0.2);
    }
</style>
