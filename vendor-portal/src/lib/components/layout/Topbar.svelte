<script lang="ts">
    import { Search, Bell, Globe, ChevronDown, LogOut, Menu, X } from 'lucide-svelte';
    import { goto } from '$app/navigation';

    let { 
        i18n, 
        meta,
        layout,
        userName, 
        userInitial,
        userEmail
    } = $props<{
        i18n: any;
        meta: any;
        layout: any;
        userName: string;
        userInitial: string;
        userEmail: string;
    }>();

    let unreadCount = $derived(layout.notifications.filter((n: any) => n.unread).length);

    function handleLogout() {
        goto('/logout');
    }
</script>

<header class="topbar">
    <!-- Logo -->
    <a href="/dashboard" class="logo-container">
        <img src="/logo.webp" alt="ZafafWorld" class="logo-image" />
        <div class="logo-text-stack">
            <span class="logo-title">{i18n.locale === 'ar' ? 'زفاف' : 'ZAFAF'}</span>
            <span class="logo-subtitle">{i18n.locale === 'ar' ? 'وورلد' : 'WORLD'}</span>
        </div>
        <span class="badge">{i18n.t.layout.vendorHub}</span>
    </a>

    <!-- Center: page title + breadcrumb -->
    <div class="topbar-center">
        <h1 class="topbar-title">{meta.title}</h1>
        {#if meta.crumbs?.length > 1}
            <nav class="breadcrumb" aria-label="breadcrumb">
                {#each meta.crumbs as crumb, i}
                    {#if i > 0}<span class="bc-sep">›</span>{/if}
                    {#if i === meta.crumbs.length - 1}
                        <span class="bc-active">{crumb.label}</span>
                    {:else}
                        <a href={'href' in crumb ? crumb.href : '#'} class="bc-link">{crumb.label}</a>
                    {/if}
                {/each}
            </nav>
        {/if}
    </div>

    <!-- Right/Left controls -->
    <div class="topbar-actions">
        <!-- Search toggle -->
        <button
            class="topbar-btn search-toggle"
            onclick={() => layout.searchOpen = !layout.searchOpen}
            aria-label="Search"
            title="Search"
        >
            <Search size={17} />
        </button>

        <!-- Notifications -->
        <div class="notif-wrap">
            <button
                class="topbar-btn notif-btn"
                onclick={() => { layout.notifOpen = !layout.notifOpen; layout.userMenuOpen = false; layout.langMenuOpen = false; }}
                aria-label={i18n.t.layout.notifications}
            >
                <Bell size={17} />
                {#if unreadCount > 0}
                    <span class="notif-badge">{unreadCount}</span>
                {/if}
            </button>

            {#if layout.notifOpen}
                <div class="notif-panel">
                    <div class="notif-header">
                        <span class="notif-title">{i18n.t.layout.notifications}</span>
                        <span class="notif-count">{i18n.interpolate(i18n.t.layout.newCount, { count: unreadCount })}</span>
                    </div>
                    <div class="notif-list">
                        {#if layout.notifications.length === 0}
                            <div class="notif-item" style="justify-content: center; color: var(--text-sec); font-size: 13px;">
                                {i18n.t.layout.noNotifications}
                            </div>
                        {:else}
                            {#each layout.notifications as n}
                                <div class="notif-item" class:unread={n.unread}>
                                    <span class="notif-item-icon">{n.icon || '🔔'}</span>
                                    <div class="notif-item-body">
                                        <div class="notif-item-title">{n.title}</div>
                                        <div class="notif-item-desc">{n.desc}</div>
                                    </div>
                                    <span class="notif-item-time">{n.time}</span>
                                </div>
                            {/each}
                        {/if}
                    </div>
                    <div class="notif-footer">
                        <button class="btn-link">{i18n.t.layout.viewAll}</button>
                    </div>
                </div>
            {/if}
        </div>

        <!-- Language switcher -->
        <div class="lang-menu-wrap">
            <button
                class="lang-btn"
                onclick={() => { layout.langMenuOpen = !layout.langMenuOpen; layout.userMenuOpen = false; layout.notifOpen = false; }}
                aria-label="Language"
            >
                <Globe size={15} />
                <span>{i18n.locale === 'ar' ? 'عربي' : 'EN'}</span>
                <ChevronDown size={12} />
            </button>
            {#if layout.langMenuOpen}
                <div class="dropdown-panel lang-dropdown">
                    <button
                        class="dropdown-item"
                        class:active={i18n.locale === 'ar'}
                        onclick={() => {
                            i18n.locale = 'ar';
                            layout.langMenuOpen = false;
                        }}
                    >
                        <span class="dropdown-item-flag">🇸🇦</span>
                        العربية
                    </button>
                    <button
                        class="dropdown-item"
                        class:active={i18n.locale === 'en'}
                        onclick={() => {
                            i18n.locale = 'en';
                            layout.langMenuOpen = false;
                        }}
                    >
                        <span class="dropdown-item-flag">🇺🇸</span>
                        English
                    </button>
                </div>
            {/if}
        </div>

        <!-- User menu -->
        <div class="user-menu-wrap">
            <button
                class="user-chip"
                onclick={() => { layout.userMenuOpen = !layout.userMenuOpen; layout.langMenuOpen = false; layout.notifOpen = false; }}
            >
                <div class="user-avatar">{userInitial}</div>
                <div class="user-info">
                    <span class="user-name">{userName}</span>
                    <span class="user-role">{i18n.t.users.roleAdmin}</span>
                </div>
                <ChevronDown size={13} class="chevron-icon" />
            </button>

            {#if layout.userMenuOpen}
                <div class="dropdown-panel user-dropdown">
                    <div class="dropdown-user-info">
                        <div class="dropdown-avatar">{userInitial}</div>
                        <div>
                            <div class="dropdown-user-name">{userName}</div>
                            <div class="dropdown-user-email">{userEmail}</div>
                        </div>
                    </div>
                    <div class="dropdown-divider"></div>
                    <button class="dropdown-item danger" onclick={handleLogout}>
                        <LogOut size={14} />
                        {i18n.t.common.logout}
                    </button>
                </div>
            {/if}
        </div>

        <!-- Mobile hamburger -->
        <button class="menu-toggle" onclick={layout.toggleSidebar} aria-label="Toggle menu">
            {#if layout.sidebarOpen}
                <X size={20} />
            {:else}
                <Menu size={20} />
            {/if}
        </button>
    </div>
</header>
