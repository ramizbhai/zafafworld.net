<script lang="ts">
    import WizardLayout from '$lib/components/wizard/WizardLayout.svelte';
    import { listingStore } from '$lib/stores/listingStore';
    import { browser } from '$app/environment';
    import { get } from 'svelte/store';

    import { afterNavigate } from '$app/navigation';

    let { children } = $props();

    if (browser) {
        // Synchronous check on mount: If entering the /new wizard with a stale productId
        // that does NOT represent an active /new creation draft, reset to a clean slate.
        const store = get(listingStore);
        if (store.productId && !sessionStorage.getItem('zafaf_wiz_new_active')) {
            listingStore.reset();
        }

        afterNavigate((navigation) => {
            // Reset the store when navigating into the 'new' wizard from outside
            if (navigation.from) {
                const fromPath = navigation.from.url.pathname;
                const isFromOutside = !fromPath.startsWith('/dashboard/products/new');
                if (isFromOutside) {
                    listingStore.reset();
                }
            }
        });
    }
</script>

<WizardLayout basePath="/dashboard/products/new">
    {@render children()}
</WizardLayout>
