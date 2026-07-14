<script lang="ts">
    import type { PageData, ActionData } from './$types';
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import { createProductsState } from '$lib/stores/productsState.svelte';

    import ProductsHeader from '$lib/components/products/ProductsHeader.svelte';
    import ProductsQuotaBar from '$lib/components/products/ProductsQuotaBar.svelte';
    import ProductGrid from '$lib/components/products/ProductGrid.svelte';
    import ProductDeleteModal from '$lib/components/products/ProductDeleteModal.svelte';

    import '$lib/components/products/styles.css';

    let { data, form } = $props<{ data: PageData; form: ActionData }>();

    const i18n = getI18n();
    const productsState = createProductsState(() => data);
</script>

<svelte:head>
    <title>{i18n.t.listings.title} — {i18n.t.common.appName}</title>
</svelte:head>

<div class="products-page" dir={i18n.locale === 'ar' ? 'rtl' : 'ltr'}>
    <ProductsHeader {i18n} {form} />
    
    <ProductsQuotaBar {productsState} />
    
    <ProductGrid {i18n} {productsState} {data} />

    <ProductDeleteModal {i18n} {productsState} />
</div>
