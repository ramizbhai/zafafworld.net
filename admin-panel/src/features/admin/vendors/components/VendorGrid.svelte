<script lang="ts">
    import { CheckCircle } from 'lucide-svelte';
    import VendorCard from './VendorCard.svelte';

    let { vendors, vendorsState, tiers } = $props<{ vendors: any[], vendorsState: any, tiers: any[] }>();
</script>

{#if vendors.length === 0}
    <div class="empty-state">
        <div class="empty-icon-wrapper">
            <CheckCircle size={48} class="check-icon" />
        </div>
        <h2>No Vendors Found</h2>
        <p>There are no profiles in this tab matching your search criteria.</p>
    </div>
{:else}
    <div class="curation-grid">
        {#each vendors as vendor (vendor.id)}
            <VendorCard {vendor} {vendorsState} {tiers} />
        {/each}
    </div>
{/if}
