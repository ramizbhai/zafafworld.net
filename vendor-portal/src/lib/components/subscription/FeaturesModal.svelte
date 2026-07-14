<script lang="ts">
    import { Check } from 'lucide-svelte';
    import { getTierFeatures } from "../../services/subscription.service";

    let { i18n, subscriptionState } = $props<{ i18n: any, subscriptionState: any }>();
</script>

{#if subscriptionState.showFeaturesModal && subscriptionState.selectedFeaturesTier}
    <div class="modal modal-open">
        <div class="modal-box relative p-6 max-w-2xl bg-base-100 rounded-2xl shadow-xl">
            <button 
                class="btn btn-sm btn-circle absolute right-4 top-4 bg-base-200 hover:bg-base-300 border-none text-base-content"
                onclick={() => subscriptionState.showFeaturesModal = false}
            >✕</button>
            <h3 class="text-2xl font-bold mb-6 mt-2 text-center text-primary">
                {subscriptionState.selectedFeaturesTier.name.toUpperCase()} {i18n.locale === 'ar' ? 'مميزات الباقة' : 'Plan Features'}
            </h3>
            <ul class="tier-features features-modal-list">
                {#each getTierFeatures(subscriptionState.selectedFeaturesTier.name, i18n.locale) as feature}
                    <li>
                        <Check size={18} class="feature-icon" /> 
                        <span>{@html feature.text}</span>
                    </li>
                {/each}
            </ul>
        </div>
        <!-- svelte-ignore a11y_interactive_supports_focus -->
        <div 
            class="modal-backdrop bg-black/40 backdrop-blur-sm" 
            onclick={() => subscriptionState.showFeaturesModal = false}
            onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') subscriptionState.showFeaturesModal = false; }}
            role="button"
            tabindex="0"
            aria-label={i18n.locale === 'ar' ? 'إغلاق النافذة' : 'Close modal'}
        ></div>
    </div>
{/if}
