<script lang="ts">
    import { getI18n } from "$lib/i18n/i18n.svelte";
    import { Check } from "lucide-svelte";
    import type { WizardFinalState } from "../../features/vendor/wizard/wizardFinalState.svelte";

    let { state }: { state: WizardFinalState } = $props();
    const i18n = getI18n();

    function handleSubmit() {
        state.submitListing(i18n.locale);
    }
</script>

<div class="submit-bar">
    {#if state.validationError}
        <div class="error-banner mb-4 p-3 bg-red-50 text-red-600 border border-red-200 rounded-lg text-sm font-semibold">
            ⚠️ {state.validationError}
        </div>
    {/if}

    <button
        type="button"
        class="submit-btn"
        onclick={handleSubmit}
        disabled={state.isSubmitting}
    >
        {#if state.isSubmitting}
            {i18n.locale === "ar" ? "جاري الإنهاء..." : "Finalizing…"}
        {:else}
            <Check size={20} />
            {i18n.locale === "ar" ? "تأكيد الإرسال للمراجعة" : "Finalize Onboarding"}
        {/if}
    </button>
</div>

<style>
    .submit-bar {
        margin-top: 1.5rem;
    }

    .submit-btn {
        width: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 10px;
        background: var(--primary, #6366f1);
        color: white;
        border: none;
        padding: 16px;
        font-size: 1.1rem;
        font-weight: 700;
        border-radius: var(--radius-lg, 12px);
        cursor: pointer;
        transition: all 0.2s;
        box-shadow: 0 4px 14px rgba(99, 102, 241, 0.3);
    }

    .submit-btn:hover:not(:disabled) {
        background: var(--primary-dark, #4f46e5);
        transform: translateY(-2px);
        box-shadow: 0 6px 20px rgba(99, 102, 241, 0.4);
    }

    .submit-btn:disabled {
        opacity: 0.7;
        cursor: not-allowed;
        animation: spin-pulse 1.5s linear infinite;
    }

    @keyframes spin-pulse {
        0% { opacity: 0.7; }
        50% { opacity: 0.9; }
        100% { opacity: 0.7; }
    }
</style>
