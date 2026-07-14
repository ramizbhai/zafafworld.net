import { untrack } from 'svelte';

export function createProductsState(getData: () => any) {
    let deletingId = $state<string | null>(null);
    let togglingId = $state<string | null>(null);
    let productToDelete = $state<any>(null);

    let maxProducts = $state(1);
    let vendorTierId = $state('');

    $effect(() => {
        const data = getData();
        if (data?.streamed?.telemetry) {
            data.streamed.telemetry.then((res: any) => {
                const limits = res?.data?.vendor?.policy_limits;
                if (limits && limits.max_products !== undefined) {
                    untrack(() => {
                        maxProducts = limits.max_products;
                    });
                }
                untrack(() => {
                    vendorTierId = res?.data?.vendor?.tier_id ?? '';
                });
            });
        }
    });

    let usedProducts = $derived.by(() => {
        const data = getData();
        return data?.products?.filter((p: any) => p.status === 'active' || p.status === 'pending_approval').length || 0;
    });

    let quotaPercentage = $derived(maxProducts < 0 ? 0 : Math.min(100, (usedProducts / maxProducts) * 100));

    return {
        get deletingId() { return deletingId; },
        set deletingId(v) { deletingId = v; },
        
        get togglingId() { return togglingId; },
        set togglingId(v) { togglingId = v; },
        
        get productToDelete() { return productToDelete; },
        set productToDelete(v) { productToDelete = v; },
        
        get maxProducts() { return maxProducts; },
        get vendorTierId() { return vendorTierId; },
        get usedProducts() { return usedProducts; },
        get quotaPercentage() { return quotaPercentage; },
    };
}
