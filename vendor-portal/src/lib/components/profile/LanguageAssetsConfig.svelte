<script lang="ts">
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import type { ProfileState } from '../../features/vendor/profileState.svelte.js';

    let { state }: { state: ProfileState } = $props();
    const i18n = getI18n();
</script>

<div class="split-grid">
    <!-- LEFT COLUMN: ENGLISH PORTFOLIO CONFIG -->
    <div class="config-panel left-panel">
        <div class="panel-header">
            <span class="panel-badge">EN</span>
            <h3 class="panel-title">🇬🇧 {i18n.t.pagesConfig.englishAsset}</h3>
        </div>
        <p class="panel-desc">{i18n.t.pagesConfig.englishAssetDesc}</p>

        <div class="form-field">
            <label for="name_en">{i18n.t.pagesConfig.officialTradeNameEn}</label>
            <input
                type="text"
                id="name_en"
                name="name_en"
                bind:value={state.name_en}
                placeholder={i18n.t.pagesConfig.officialTradeNameEnPl}
                required
            />
        </div>

        <div class="form-field">
            <label for="description_en">{i18n.t.pagesConfig.corporateOverviewEn}</label>
            <textarea
                id="description_en"
                name="description_en"
                bind:value={state.description_en}
                placeholder={i18n.t.pagesConfig.corporateOverviewEnPl}
                rows="8"
            ></textarea>
        </div>
    </div>

    <!-- RIGHT COLUMN: ARABIC PORTFOLIO CONFIG (RTL ALIGNED) -->
    <div class="config-panel right-panel">
        <div class="panel-header text-right justify-end">
            <h3 class="panel-title">🇸🇦 {i18n.t.pagesConfig.arabicAsset}</h3>
            <span class="panel-badge ar-badge">AR</span>
        </div>
        <p class="panel-desc text-right">{i18n.t.pagesConfig.arabicAssetDesc}</p>

        <div class="form-field text-right" dir="rtl">
            <label for="name_ar">{i18n.t.pagesConfig.officialTradeNameAr}</label>
            <input
                type="text"
                id="name_ar"
                name="name_ar"
                bind:value={state.name_ar}
                placeholder={i18n.t.pagesConfig.officialTradeNameArPl}
                required
            />
        </div>

        <div class="form-field text-right" dir="rtl">
            <label for="description_ar">{i18n.t.pagesConfig.corporateOverviewAr}</label>
            <textarea
                id="description_ar"
                name="description_ar"
                bind:value={state.description_ar}
                placeholder={i18n.t.pagesConfig.corporateOverviewArPl}
                rows="8"
            ></textarea>
        </div>
    </div>
</div>

<style>
    .split-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 3rem;
    }

    /* Border divider between panels */
    .split-grid > :first-child {
        border-inline-end: 1px solid var(--border-light);
        padding-inline-end: 3rem;
    }

    @media (max-width: 968px) {
        .split-grid {
            grid-template-columns: 1fr;
            gap: 2.5rem;
        }
        .split-grid > :first-child {
            border-inline-end: none;
            padding-inline-end: 0;
            border-bottom: 1px solid var(--border-light);
            padding-bottom: 2.5rem;
        }
    }

    .config-panel {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    .panel-header {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .justify-end {
        justify-content: flex-end;
    }

    .panel-badge {
        font-size: 0.6rem;
        font-weight: 800;
        background: var(--border-light);
        border: 1px solid var(--border);
        padding: 0.15rem 0.4rem;
        border-radius: 4px;
        color: var(--text-sec);
    }

    .ar-badge {
        border-color: rgba(217, 119, 6, 0.2);
        background: rgba(217, 119, 6, 0.05);
        color: var(--color-secondary);
    }

    .panel-title {
        margin: 0;
        font-size: 1.2rem;
        font-weight: 700;
        color: var(--text);
    }

    .panel-desc {
        margin: -0.5rem 0 0 0;
        font-size: 0.8rem;
        color: var(--text-sec);
        line-height: 1.4;
    }

    /* ─── FORM CONTROLS ──────────────────────────────────────────────────── */
    .form-field {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    label {
        font-size: 0.8rem;
        font-weight: 600;
        color: var(--text);
    }

    input, textarea {
        background: var(--white);
        border: 1.5px solid var(--border);
        border-radius: 8px;
        color: var(--text);
        font-size: 0.9rem;
        padding: 0.75rem 1rem;
        transition: border-color 0.2s ease, box-shadow 0.2s ease;
        width: 100%;
        box-sizing: border-box;
    }

    input:focus, textarea:focus {
        outline: none;
        border-color: var(--color-primary);
        box-shadow: 0 0 0 3px rgba(91, 33, 182, 0.1);
    }

    textarea {
        resize: vertical;
        font-family: inherit;
        line-height: 1.5;
    }

    .text-right {
        text-align: right;
    }

    .text-right label {
        font-family: 'Cairo', sans-serif;
    }

    .text-right input, .text-right textarea {
        font-family: 'Cairo', sans-serif;
    }
</style>
