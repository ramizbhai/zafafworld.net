export function createSubscriptionState(getData: () => any) {
    let activeTab = $state<'overview' | 'upgrade'>('overview');
    
    let isSubmitting = $state(false);
    let submittingTierId = $state('');
    let requestResult = $state({ success: false, error: '' });
    
    let showFeaturesModal = $state(false);
    let selectedFeaturesTier = $state<any>(null);

    let tiers = $state<any[]>([]);
    let currentTierId = $state('');
    let vendorLimits = $state<any>(null);
    let expirationDate = $state<any>(null);

    let requests = $derived(getData().requests || []);
    let activeTier = $derived(tiers.find((t: any) => t.id === currentTierId));

    $effect(() => {
        const data = getData();
        if (data?.streamed?.tiers) {
            data.streamed.tiers.then((res: any) => {
                tiers = res?.tiers || res || [];
            });
        }
        if (data?.streamed?.telemetry) {
            data.streamed.telemetry.then((res: any) => {
                const v = res?.data?.vendor;
                currentTierId = v?.tier_id || v?.subscriptionTierId || '';
                vendorLimits = v?.policy_limits;
                expirationDate = v?.subscription_expires_at;
            });
        }
    });

    return {
        get activeTab() { return activeTab; },
        set activeTab(v) { activeTab = v; },
        
        get isSubmitting() { return isSubmitting; },
        set isSubmitting(v) { isSubmitting = v; },
        
        get submittingTierId() { return submittingTierId; },
        set submittingTierId(v) { submittingTierId = v; },
        
        get requestResult() { return requestResult; },
        set requestResult(v) { requestResult = v; },
        
        get showFeaturesModal() { return showFeaturesModal; },
        set showFeaturesModal(v) { showFeaturesModal = v; },
        
        get selectedFeaturesTier() { return selectedFeaturesTier; },
        set selectedFeaturesTier(v) { selectedFeaturesTier = v; },
        
        get tiers() { return tiers; },
        get currentTierId() { return currentTierId; },
        get vendorLimits() { return vendorLimits; },
        get expirationDate() { return expirationDate; },
        get requests() { return requests; },
        get activeTier() { return activeTier; }
    };
}
