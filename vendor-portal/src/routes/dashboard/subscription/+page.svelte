<script lang="ts">
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import { invalidateAll } from '$app/navigation';
    import { createSubscriptionState } from '$lib/stores/subscriptionState.svelte';

    import SubscriptionHeader from '$lib/components/subscription/SubscriptionHeader.svelte';
    import SubscriptionTabs from '$lib/components/subscription/SubscriptionTabs.svelte';
    import SubscriptionBanners from '$lib/components/subscription/SubscriptionBanners.svelte';
    import SubscriptionOverview from '$lib/components/subscription/SubscriptionOverview.svelte';
    import SubscriptionUpgrade from '$lib/components/subscription/SubscriptionUpgrade.svelte';
    import FeaturesModal from '$lib/components/subscription/FeaturesModal.svelte';

    import '$lib/components/subscription/styles.css';

    let { data } = $props();

    const i18n = getI18n();
    const subscriptionState = createSubscriptionState(() => data);

    function handleEnhance() {
        subscriptionState.isSubmitting = true;
        subscriptionState.requestResult = { success: false, error: '' };
        
        return async ({ result, update }: any) => {
            subscriptionState.isSubmitting = false;
            subscriptionState.submittingTierId = '';
            
            if (result.type === 'success') {
                subscriptionState.requestResult = { success: true, error: '' };
                await invalidateAll();
                setTimeout(() => {
                    subscriptionState.requestResult = { success: false, error: '' };
                }, 3000);
            } else if (result.type === 'error' || result.type === 'failure') {
                subscriptionState.requestResult = { 
                    success: false, 
                    error: result.data?.message || (i18n.locale === 'ar' ? 'حدث خطأ. حاول مرة أخرى.' : 'An error occurred. Please try again.') 
                };
            }
            await update();
        };
    }
</script>

<svelte:head>
    <title>{i18n.locale === 'ar' ? 'الباقات والاشتراكات' : 'Subscriptions'} — ZafafWorld</title>
</svelte:head>

<div class="subscription-page" dir={i18n.locale === 'ar' ? 'rtl' : 'ltr'}>
    <SubscriptionHeader {i18n} />
    
    <SubscriptionTabs {i18n} {subscriptionState} />

    <div class="page-content">
        <SubscriptionBanners {i18n} {subscriptionState} />

        {#if subscriptionState.activeTab === 'overview'}
            <SubscriptionOverview {i18n} {subscriptionState} />
        {/if}
        
        {#if subscriptionState.activeTab === 'upgrade'}
            <SubscriptionUpgrade {i18n} {subscriptionState} {handleEnhance} />
        {/if}
    </div>
</div>

<FeaturesModal {i18n} {subscriptionState} />
