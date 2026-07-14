<script lang="ts">
    import { getI18n } from "$lib/i18n/i18n.svelte";
    import type { WizardFinalState } from "../../features/vendor/wizard/wizardFinalState.svelte";

    let { state }: { state: WizardFinalState } = $props();
    const i18n = getI18n();

    function handleCheckboxChange(e: Event) {
        const input = e.target as HTMLInputElement;
        state.agreedToTerms = input.checked;
        if (state.validationError === "You must agree to the Terms & Conditions." && state.agreedToTerms) {
            state.validationError = null;
        }
    }
</script>

<div class="terms-container">
    <label class="terms-label">
        <input 
            type="checkbox" 
            class="terms-checkbox" 
            checked={state.agreedToTerms}
            onchange={handleCheckboxChange}
            disabled={state.isSubmitting}
        />
        <span class="terms-text">
            {#if i18n.locale === "ar"}
                أوافق على <a href="/terms" target="_blank" class="terms-link">الشروط والأحكام</a> الخاصة بالمنصة وأن جميع المعلومات المقدمة صحيحة.
            {:else}
                I agree to the <a href="/terms" target="_blank" class="terms-link">Terms & Conditions</a> of the platform and confirm all provided information is accurate.
            {/if}
        </span>
    </label>
</div>

<style>
    .terms-container {
        margin: 1.5rem 0;
        padding: 1.5rem;
        background: #f9fafb;
        border: 1px solid #e5e7eb;
        border-radius: 8px;
    }
    
    .terms-label {
        display: flex;
        align-items: flex-start;
        gap: 12px;
        cursor: pointer;
    }
    
    .terms-checkbox {
        margin-top: 4px;
        width: 18px;
        height: 18px;
        accent-color: #6366f1;
        cursor: pointer;
    }
    
    .terms-checkbox:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .terms-text {
        font-size: 0.95rem;
        color: #4b5563;
        line-height: 1.5;
    }
    
    .terms-link {
        color: #6366f1;
        text-decoration: underline;
        font-weight: 600;
    }
</style>
