<script lang="ts">
    import { page } from '$app/stores';
    import { setI18nContext } from '$lib/i18n/i18n.svelte';
    import { afterNavigate } from '$app/navigation';
    import { untrack } from 'svelte';
    import { Search } from 'lucide-svelte';
    
    import SubscriptionExpiredWall from '$lib/components/SubscriptionExpiredWall.svelte';
    import SubscriptionUpgradeModal from '$lib/components/SubscriptionUpgradeModal.svelte';
    
    import Topbar from '$lib/components/layout/Topbar.svelte';
    import Sidebar from '$lib/components/layout/Sidebar.svelte';
    import '$lib/components/layout/styles.css';

    import { isUserOwner, getUserName, getUserInitial, getVendorInitials } from '../../core/auth/sessionService';
    import { getNavItems, getBreadcrumbsAndTitle } from '$lib/services/navigation';
    import { createLayoutState } from '$lib/stores/layoutState.svelte';

    let { data, children } = $props();

    const i18n = setI18nContext(untrack(() => data.locale as any) || 'ar');
    
    $effect(() => {
        if (data.locale) i18n.locale = data.locale as any;
    });

    const layout = createLayoutState();

    $effect(() => {
        layout.initStreams(data);
    });

    let currentPath = $derived($page.url.pathname as string);
    let isOwner = $derived(isUserOwner(data.user));
    let userName = $derived(getUserName(data.user, i18n.locale as any));
    let userInitial = $derived(getUserInitial(data.user, i18n.locale as any));
    let vendorInitials = $derived(getVendorInitials(layout.vendor, i18n.locale as any));
    
    let navItems = $derived(getNavItems(i18n, isOwner));
    let meta = $derived(getBreadcrumbsAndTitle(i18n, $page.route.id ?? ''));

    afterNavigate(() => {
        layout.closeSidebar();
    });

    $effect(() => {
        if (currentPath.startsWith('/dashboard/statistics')) layout.statsExpanded = true;
    });
</script>

<svelte:window onclick={layout.handleWindowClick} />

{#if layout.gateLock === 'stopped'}
    <SubscriptionExpiredWall 
        sessionToken={data.sessionToken ?? ''} 
        vendorName={layout.vendor.name_en} 
        tiers={layout.tiers}
        currentTierId={layout.vendor.tier_id || layout.vendor.subscriptionTierId}
    />
{:else}
    <div class="app-shell" class:wizard-active={currentPath.includes('/products/') && (currentPath.includes('/new') || currentPath.includes('/edit'))}>
        <Topbar 
            {i18n}
            {meta}
            {layout}
            {userName}
            {userInitial}
            userEmail={data.user?.email ?? ''}
        />

        {#if layout.searchOpen}
            <div class="search-bar">
                <div class="search-bar-inner">
                    <Search size={16} class="search-bar-icon" />
                    <input
                        type="text"
                        class="search-bar-input"
                        placeholder={i18n.t.layout.searchPlaceholder}
                    />
                    <kbd class="search-kbd">ESC</kbd>
                </div>
            </div>
        {/if}

        {#if layout.sidebarOpen}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div class="sidebar-overlay" onclick={layout.closeSidebar} aria-hidden="true"></div>
        {/if}

        <Sidebar 
            {i18n}
            {layout}
            {navItems}
            {vendorInitials}
            {currentPath}
        />

        <main class="main-content">
            <div class="main-inner animate-fade-in">
                {@render children()}
            </div>
        </main>
    </div>
{/if}

<SubscriptionUpgradeModal tiers={layout.tiers} currentTierId={layout.vendor.tier_id || layout.vendor.subscriptionTierId} />
