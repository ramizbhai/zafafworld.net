<script lang="ts">
    let { i18n, subscriptionState } = $props<{ i18n: any, subscriptionState: any }>();
</script>

<div class="tabs-container">
    <button 
        class="tab-btn" 
        class:active={subscriptionState.activeTab === 'overview'} 
        onclick={() => subscriptionState.activeTab = 'overview'}
    >
        {i18n.locale === 'ar' ? 'باقتي' : 'My Subscription'}
    </button>
    <button 
        class="tab-btn" 
        class:active={subscriptionState.activeTab === 'upgrade'} 
        onclick={() => subscriptionState.activeTab = 'upgrade'}
    >
        {i18n.locale === 'ar' ? 'ترقية الباقة' : 'Upgrade Plans'}
    </button>
</div>
