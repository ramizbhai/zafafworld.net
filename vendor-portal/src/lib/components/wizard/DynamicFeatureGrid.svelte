<script lang="ts">
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import { listingStore } from '$lib/stores/listingStore';

    const i18n = getI18n();

    let { group, storeKey = 'featuresSelection' } = $props<{
        group: {
            groupId: string;
            titleEn: string;
            titleAr: string;
            options: Array<{
                optionId?: string; // from backend wizard-schema (RawDbFeature optionId)
                value?: string;    // legacy/options format
                titleEn?: string;
                titleAr?: string;
                labelEn?: string;
                labelAr?: string;
            }>;
        },
        storeKey?: 'culturalAttributes' | 'featuresSelection';
    }>();

    const title = $derived(i18n.locale === 'ar' ? group.titleAr : group.titleEn);

    function isChecked(optId: string): boolean {
        if (!storeKey) return false;
        const formGroup = ($listingStore.formData as any)[storeKey] as Record<string, any> | undefined;
        const val = formGroup?.[optId];
        return val === true || val === 'true';
    }

    function handleCheckboxChange(optId: string, checked: boolean) {
        if (!storeKey) return;
        const formGroup = ($listingStore.formData as any)[storeKey] as Record<string, any> | undefined;
        const current = { ...(formGroup || {}) };
        current[optId] = checked;
        listingStore.updateFormData({ [storeKey]: current });
    }
</script>

<div class="features-group-card">
    <h3 class="group-title">{title}</h3>
    <div class="features-checkbox-grid">
        {#each group.options as option}
            {@const optionId = option.optionId || option.value || ''}
            {@const optLabel = i18n.locale === 'ar' 
                ? (option.titleAr || option.labelAr || optionId)
                : (option.titleEn || option.labelEn || optionId)
            }
            <label class="feature-checkbox-label">
                <input
                    type="checkbox"
                    checked={isChecked(optionId)}
                    onchange={(e) => handleCheckboxChange(optionId, (e.target as HTMLInputElement).checked)}
                />
                <span class="checkbox-text">{optLabel}</span>
            </label>
        {/each}
    </div>
</div>

<style>
    .features-group-card {
        background: white;
        border: 1px solid var(--border, #e5e7eb);
        border-radius: 12px;
        padding: 20px;
        margin-bottom: 24px;
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
    }
    .group-title {
        font-size: 0.8rem;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: var(--text-muted, #6b7280);
        margin-top: 0;
        margin-bottom: 16px;
        padding-bottom: 0;
    }
    .features-checkbox-grid {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: 12px 24px;
    }
    .feature-checkbox-label {
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 0.9rem;
        color: var(--text, #1a1a2e);
        cursor: pointer;
        padding: 4px 0;
    }
    .feature-checkbox-label input {
        width: 16px;
        height: 16px;
        cursor: pointer;
        accent-color: var(--primary, #6c3fa0);
    }
    @media (max-width: 640px) {
        .features-checkbox-grid {
            grid-template-columns: 1fr;
            gap: 10px;
        }
    }
</style>
