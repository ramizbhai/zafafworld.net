<script lang="ts">
    import { ChevronDown, TrendingUp, BarChart2, HelpCircle, LogOut } from 'lucide-svelte';
    import { goto } from '$app/navigation';
    import { isNavItemActive } from '$lib/services/navigation';

    let {
        i18n,
        layout,
        navItems,
        vendorInitials,
        currentPath
    } = $props<{
        i18n: any;
        layout: any;
        navItems: any[];
        vendorInitials: string;
        currentPath: string;
    }>();

    function handleLogout() {
        goto('/logout');
    }
</script>

<aside class="sidebar" class:open={layout.sidebarOpen} aria-label="Navigation">
    <!-- Vendor mini-card -->
    <div class="sidebar-vendor">
        <div class="sidebar-vendor-avatar">{vendorInitials}</div>
        <div class="sidebar-vendor-info">
            <span class="sidebar-vendor-name">
                {i18n.locale === 'ar' ? layout.vendor.name_ar : layout.vendor.name_en}
            </span>
            <span class="sidebar-vendor-status" class:active={layout.vendor.status === 'active'}>
                {layout.vendor.status === 'active'
                    ? i18n.t.layout.activeLive
                    : layout.vendor.status === 'suspended'
                        ? i18n.t.layout.suspended
                        : i18n.t.layout.banned
                }
            </span>
        </div>
    </div>

    <div class="sidebar-divider"></div>

    <!-- Navigation -->
    <nav class="sidebar-nav">
        {#each navItems as item}
            {@const active = isNavItemActive(item, currentPath)}
            <a
                class="nav-item"
                class:active
                href={item.href}
            >
                <span class="nav-icon">
                    <item.icon size={17} />
                </span>
                <span class="nav-label">{item.label}</span>
                {#if active}
                    <span class="nav-active-dot"></span>
                {/if}
            </a>
        {/each}

        <!-- Statistics expandable -->
        <button
            class="nav-item stats-toggle"
            class:active={currentPath.startsWith('/dashboard/statistics')}
            onclick={() => layout.statsExpanded = !layout.statsExpanded}
        >
            <span class="nav-icon"><TrendingUp size={17} /></span>
            <span class="nav-label">{i18n.t.nav.statistics}</span>
            <ChevronDown size={14} class="nav-chevron" style="transform: rotate({layout.statsExpanded ? '180deg' : '0deg'}); transition: transform 0.2s ease;" />
        </button>

        {#if layout.statsExpanded}
            <div class="sub-nav">
                <a
                    class="nav-item sub-item"
                    class:active={currentPath === '/dashboard/statistics/page'}
                    href="/dashboard/statistics/page"
                >
                    <span class="nav-icon"><BarChart2 size={15} /></span>
                    <span class="nav-label">{i18n.t.nav.statisticsPage}</span>
                </a>
                <a
                    class="nav-item sub-item"
                    class:active={currentPath === '/dashboard/statistics/competitors'}
                    href="/dashboard/statistics/competitors"
                >
                    <span class="nav-icon"><BarChart2 size={15} /></span>
                    <span class="nav-label">{i18n.t.nav.statisticsCompetitors}</span>
                </a>
            </div>
        {/if}
    </nav>

    <!-- Bottom sidebar -->
    <div class="sidebar-bottom">
        <div class="sidebar-divider"></div>
        <button class="nav-item help-item">
            <span class="nav-icon"><HelpCircle size={17} /></span>
            <span class="nav-label">{i18n.t.common.help}</span>
        </button>
        <button class="nav-item logout-item" onclick={handleLogout}>
            <span class="nav-icon"><LogOut size={17} /></span>
            <span class="nav-label">{i18n.t.common.logout}</span>
        </button>
    </div>
</aside>
