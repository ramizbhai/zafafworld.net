<script lang="ts">
    import { browser } from '$app/environment';
    import { get } from 'svelte/store';
    import { untrack } from 'svelte';
    import WizardLayout from '$lib/components/wizard/WizardLayout.svelte';
    import { listingStore } from '$lib/stores/listingStore';
    import { page } from '$app/stores';

    let { data, children } = $props();

    // Build a composite key from the server data so we can detect when the
    // server has fresher data than the in-memory store (e.g. the vendor
    // saved a step, navigated away, and came back — version changed on the
    // DB but the Svelte store still holds the old value).
    //
    // NOTE: The backend returns `version` inside `product.metadata.version`,
    // not at the top level of the product object.
    let serverVersion = $derived(data.product?.metadata?.version ?? 1);

    // Synchronous layout initialization on the client side.
    // This runs before any child pages/steps are instantiated,
    // ensuring their local `$state` bindings are pre-filled correctly.
    if (browser) {
        const initialProduct = untrack(() => data.product);
        const initialImages = untrack(() => data.listingImages);
        if (initialProduct) {
            // Always initialize on layout mount/refresh to clear any dirty or stale store state from other sessions or products.
            listingStore.initializeFromProduct(initialProduct, initialImages);
        }
    }

    // Re-initialize reactively if the product data updates on the server.
    // We untrack the store reads/writes to prevent infinite loops when the store is modified.
    $effect(() => {
        if (!data.product) return;

        const currentProductId = data.product.id;
        const currentVersion = serverVersion;

        untrack(() => {
            const store = get(listingStore);
            const needsInit =
                !store.productId ||
                store.productId !== currentProductId ||
                store.version !== currentVersion;

            if (needsInit) {
                listingStore.initializeFromProduct(data.product, data.listingImages);
            }
        });
    });
</script>

<WizardLayout basePath="/dashboard/products/{$page.params.id}/edit" isEditMode={true}>
    {@render children()}
</WizardLayout>
