<script lang="ts">
    import TierCard from "./TierCard.svelte";

    let { i18n, subscriptionState, handleEnhance } = $props<{ 
        i18n: any, 
        subscriptionState: any, 
        handleEnhance: () => any 
    }>();
</script>

<div class="tiers-grid animate-fade-in">
    {#each subscriptionState.tiers.filter((t: any) => t.priority_score > 25) as tier}
        <TierCard {tier} {i18n} {subscriptionState} {handleEnhance} />
    {/each}
</div>
