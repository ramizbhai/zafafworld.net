<script lang="ts">
    import { getI18n } from '$lib/i18n/i18n.svelte';
    let { clientError, form } = $props<{ clientError: string; form: any }>();
    const i18n = getI18n();
</script>

<div class="page-header">
    <div class="title-section">
        <span class="category-tag">{i18n.t.pagesConfig.configHub}</span>
        <h1>{i18n.t.pagesConfig.corporateProfile}</h1>
        <p class="subtitle">{i18n.t.pagesConfig.updateSubtitle}</p>
    </div>
</div>

{#if clientError}
    <div class="status-banner error-banner">
        <div class="banner-icon">⚠️</div>
        <div class="banner-text">
            <h3>{i18n.t.pagesConfig.validationAlert}</h3>
            <p>{clientError}</p>
        </div>
    </div>
{/if}

{#if form?.error}
    <div class="status-banner error-banner">
        <div class="banner-icon">⚠️</div>
        <div class="banner-text">
            {#if form.error.includes('conflict') || form.error.includes('another operator')}
                <h3>{i18n.t.pagesConfig.operationalConflict}</h3>
                <p>{i18n.t.pagesConfig.operationalConflictText}</p>
            {:else}
                <h3>{i18n.t.pagesConfig.syncFailure}</h3>
                <p>{form.error}</p>
            {/if}
        </div>
    </div>
{/if}

{#if form?.success}
    <div class="status-banner success-banner">
        <div class="banner-icon">✨</div>
        <div class="banner-text">
            <h3>{i18n.t.pagesConfig.registrySync}</h3>
            <p>{form.message ?? i18n.t.pagesConfig.registrySyncText}</p>
        </div>
    </div>
{/if}

<style>
    .page-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        border-bottom: 1px solid var(--border);
        padding-bottom: 1.5rem;
    }

    .title-section {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
    }

    .category-tag {
        font-size: 0.65rem;
        text-transform: uppercase;
        letter-spacing: 1.5px;
        font-weight: 700;
        color: var(--color-secondary);
    }

    .page-header h1 {
        margin: 0;
        font-size: 1.85rem;
        font-weight: 800;
        letter-spacing: -0.5px;
        color: var(--text);
    }

    .subtitle {
        margin: 0;
        font-size: 0.9rem;
        color: var(--text-sec);
        line-height: 1.4;
    }

    /* ─── STATUS NOTIFICATIONS ─────────────────────────────────────────────── */
    .status-banner {
        display: flex;
        align-items: center;
        gap: 1.25rem;
        padding: 1.25rem 1.75rem;
        border-radius: 10px;
        box-shadow: var(--shadow-md);
        animation: slide-down 0.3s cubic-bezier(0.16, 1, 0.3, 1);
        margin-bottom: 1.5rem;
    }

    @keyframes slide-down {
        from { transform: translateY(-10px); opacity: 0; }
        to { transform: translateY(0); opacity: 1; }
    }

    .error-banner {
        background: var(--color-danger-light);
        border: 1px solid var(--color-danger);
    }

    .success-banner {
        background: var(--color-success-light);
        border: 1px solid var(--color-success);
    }

    .banner-icon {
        font-size: 1.5rem;
        flex-shrink: 0;
    }

    .banner-text h3 {
        margin: 0 0 0.2rem 0;
        font-size: 0.95rem;
        font-weight: 700;
    }

    .error-banner h3 { color: var(--color-danger); }
    .success-banner h3 { color: var(--color-success); }

    .banner-text p {
        margin: 0;
        font-size: 0.8rem;
        color: var(--text-sec);
    }
</style>
