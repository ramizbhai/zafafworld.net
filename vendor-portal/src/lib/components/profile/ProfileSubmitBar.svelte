<script lang="ts">
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import type { ProfileState } from '../../features/vendor/profileState.svelte.js';

    let { state }: { state: ProfileState } = $props();
    const i18n = getI18n();
</script>

<div class="form-actions">
    <button type="submit" disabled={state.isSubmitting} class="save-btn">
        {#if state.isSubmitting}
            <span class="micro-spinner"></span> {i18n.t.pagesConfig.synchronizingProfile}
        {:else}
            {i18n.t.pagesConfig.publishUpdates}
        {/if}
    </button>
</div>

<style>
    /* ─── ACTIONS BAR ────────────────────────────────────────────────────── */
    .form-actions {
        display: flex;
        justify-content: flex-end;
        border-top: 1px solid var(--border-light);
        padding-top: 2rem;
        margin-top: 2.5rem;
    }

    .save-btn {
        background: linear-gradient(135deg, var(--color-secondary) 0%, var(--color-secondary) 100%);
        border: none;
        border-radius: 8px;
        color: var(--white);
        font-weight: 700;
        font-size: 0.95rem;
        padding: 0.85rem 2rem;
        cursor: pointer;
        display: inline-flex;
        align-items: center;
        gap: 0.5rem;
        transition: transform 0.2s ease, box-shadow 0.2s ease, filter 0.2s ease;
        box-shadow: var(--shadow-gold);
    }

    .save-btn:hover:not(:disabled) {
        transform: translateY(-2px);
        box-shadow: 0 6px 20px rgba(217, 119, 6, 0.40);
        filter: brightness(1.05);
    }

    .save-btn:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }

    .micro-spinner {
        display: inline-block;
        width: 1rem;
        height: 1rem;
        border: 2px solid rgba(255, 255, 255, 0.2);
        border-top-color: var(--white);
        border-radius: 50%;
        animation: spin 0.8s linear infinite;
    }

    @keyframes spin {
        to { transform: rotate(360deg); }
    }
</style>
