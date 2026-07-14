<script lang="ts">
    import WizardLayout from '$lib/components/wizard/WizardLayout.svelte';
    import { listingStore } from '$lib/stores/listingStore';
    import { browser } from '$app/environment';
    import { get } from 'svelte/store';

    import { afterNavigate } from '$app/navigation';

    let { children } = $props();

    if (browser) {
        afterNavigate((navigation) => {
            // Reset the store ONLY when starting a genuinely fresh listing session.
            // We call listingStore.reset() only when entering the 'new' wizard from
            // any outside page path (i.e. not starting with '/dashboard/products/new').
            // We ignore initial page load/refreshes (where navigation.from is null) to preserve progress.
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
