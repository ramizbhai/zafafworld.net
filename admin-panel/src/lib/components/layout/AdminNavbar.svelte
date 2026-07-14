<script lang="ts">
  import { adminGlobalState } from '../../../core/stores/adminGlobalState.svelte';
  import { lang } from '$lib/i18n/index.js';
  import { Menu, ChevronLeft, Search, Globe, Bell } from 'lucide-svelte';

  let { t, data } = $props<{ t: any, data: any }>();

  function getInitials(firstName: string, lastName: string): string {
    return `${firstName?.charAt(0) ?? ''}${lastName?.charAt(0) ?? ''}`.toUpperCase() || 'AD';
  }
</script>

<header class="top-navbar">
  <div class="navbar-start">
    <!-- Sidebar toggle -->
    <button
      class="sidebar-toggle"
      onclick={() => adminGlobalState.sidebarCollapsed = !adminGlobalState.sidebarCollapsed}
      aria-label="Toggle sidebar"
      aria-expanded={!adminGlobalState.sidebarCollapsed}
    >
      {#if adminGlobalState.sidebarCollapsed}
        <Menu size={18} aria-hidden="true" />
      {:else}
        <ChevronLeft size={18} aria-hidden="true" />
      {/if}
    </button>

    <!-- System status pill -->
    <div class="system-status">
      <div class="status-dot status-live" role="status" aria-label="System is live and active"></div>
      <span class="status-label-text">{t('header.live')}</span>
    </div>

    <!-- Search bar -->
    <div class="navbar-search" role="search">
      <Search size={15} aria-hidden="true" class="search-ico" />
      <input
        type="search"
        placeholder={t('header.search_placeholder')}
        bind:value={adminGlobalState.searchQuery}
        aria-label={t('header.search_placeholder')}
      />
      <kbd class="search-kbd">⌘K</kbd>
    </div>
  </div>

  <div class="navbar-end">
    <!-- DB Status -->
    <div class="db-pill" role="status" aria-label="Database: PostgreSQL is operational">
      <div class="db-dot"></div>
      <span>PostgreSQL</span>
    </div>

    <!-- Language Toggle -->
    <button
      class="lang-toggle"
      onclick={() => lang.toggle()}
      aria-label="Switch language"
      title={t('header.lang_switch')}
    >
      <Globe size={15} aria-hidden="true" />
      <span>{t('header.lang_switch')}</span>
    </button>

    <!-- Notification Bell -->
    <button
      class="notif-btn"
      onclick={() => adminGlobalState.notifOpen = !adminGlobalState.notifOpen}
      aria-label="{t('header.notifications')} ({data.unreadCount || 0} unread)"
      aria-expanded={adminGlobalState.notifOpen}
    >
      <Bell size={17} aria-hidden="true" />
      {#if (data.unreadCount || 0) > 0}
        <span class="notif-badge-count">
          {data.unreadCount}
        </span>
      {/if}
    </button>

    <!-- Admin Avatar -->
    <div class="admin-avatar-btn">
      <div class="admin-avatar-circle">
        {getInitials(data.user?.first_name, data.user?.last_name)}
      </div>
    </div>
  </div>
</header>
