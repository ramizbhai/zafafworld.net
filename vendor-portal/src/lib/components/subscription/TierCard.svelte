<script lang="ts">
    import { enhance } from '$app/forms';
    import { Check } from 'lucide-svelte';
    import { getTierTheme, getTierPrice, getTierFeatures, isRequested } from "../../services/subscription.service";

    let { tier, i18n, subscriptionState, handleEnhance } = $props<{ 
        tier: any, 
        i18n: any, 
        subscriptionState: any, 
        handleEnhance: () => any 
    }>();

    let theme = $derived(getTierTheme(tier.name));
    let isPopular = $derived(theme === 'vip');
    let requested = $derived(isRequested(tier.id, subscriptionState.requests));
    let isCurrent = $derived(subscriptionState.currentTierId === tier.id);
    let allFeatures = $derived(getTierFeatures(tier.name, i18n.locale));
    let visibleFeatures = $derived(allFeatures.slice(0, 4));
</script>

<div class="tier-card theme-{theme}" class:is-popular={isPopular}>
    <div class="glow-border"></div>
    {#if isPopular}
        <div class="popular-badge">MOST POPULAR</div>
    {/if}
    <div class="tier-badge-container">
        <span class="tier-badge">
            {#if theme === 'gold'}🏆{:else if theme === 'vip'}👑{:else if theme === 'diamond'}💎{:else}⭐{/if}
            {tier.name.toUpperCase()}
        </span>
    </div>
    
    <div class="tier-price">
        <span class="price-val">{getTierPrice(tier.name)} SAR</span>
        <span class="price-curr">YEARLY</span>
    </div>
    
    <ul class="tier-features">
        {#each visibleFeatures as feature}
            <li>
                <Check size={14} class="feature-icon" /> 
                <span>{@html feature.text}</span>
            </li>
        {/each}
    </ul>
    
    {#if allFeatures.length > 4}
        <button 
            type="button" 
            class="btn-read-more" 
            onclick={() => { subscriptionState.selectedFeaturesTier = tier; subscriptionState.showFeaturesModal = true; }}
        >
            {i18n.locale === 'ar' ? 'قراءة المزيد' : 'Read More'}
        </button>
    {/if}
    
    <div class="card-footer">
        {#if isCurrent}
            <button class="btn-upgrade current-plan" disabled>
                <Check size={16} />
                {i18n.locale === 'ar' ? 'باقتك الحالية' : 'Current Plan'}
            </button>
        {:else if requested}
            <button class="btn-upgrade requested" disabled>
                <Check size={16} />
                {i18n.locale === 'ar' ? 'تم الطلب' : 'Requested'}
            </button>
        {:else}
            <form method="POST" action="?/requestUpgrade" use:enhance={handleEnhance} class="upgrade-form" onsubmit={() => subscriptionState.submittingTierId = tier.id}>
                <input type="hidden" name="requested_tier_id" value={tier.id} />
                <button 
                    type="submit" 
                    class="btn-upgrade" 
                    disabled={subscriptionState.isSubmitting}
                >
                    {#if subscriptionState.isSubmitting && subscriptionState.submittingTierId === tier.id}
                        <span class="loading-spinner"></span>
                    {:else}
                        {i18n.locale === 'ar' ? 'طلب الترقية' : 'Request'}
                    {/if}
                </button>
            </form>
        {/if}
    </div>
</div>
