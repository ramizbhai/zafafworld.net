<script lang="ts">
    import { Plus } from 'lucide-svelte';
    import { goto } from '$app/navigation';
    import ProductCard from './ProductCard.svelte';

    let { i18n, productsState, data } = $props<{ i18n: any, productsState: any, data: any }>();

    function handleAddNew() {
        goto('/dashboard/products/new');
    }
</script>

{#if data.products.length === 0}
    <div class="empty-state">
        <div class="empty-icon">🏛️</div>
        <h2>{i18n.t.listings.emptyTitle}</h2>
        <p>{i18n.t.listings.emptyDesc}</p>
        <button class="btn-add-lg" onclick={handleAddNew}>
            <Plus size={18} /> {i18n.t.listings.addFirst}
        </button>
    </div>
{:else}
    <div class="products-grid">
        {#each data.products as product (product.id)}
            <ProductCard {product} {i18n} {productsState} {data} />
        {/each}
    </div>
{/if}
