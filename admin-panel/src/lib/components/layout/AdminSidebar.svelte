<script lang="ts">
  import { authStore } from '$lib/stores/auth.store.js';
  import { adminGlobalState } from '../../../core/stores/adminGlobalState.svelte';
  import { RBACService } from '../../../core/auth/rbac.service';
  import { page } from '$app/state';
  import {
    LayoutDashboard, Building2, Users, CalendarCheck, CreditCard,
    BarChart3, Percent, HeadphonesIcon, Bell, ShieldCheck, FileText,
    MessageSquare, Newspaper, Megaphone, Activity, Settings,
    Globe, Sparkles, TrendingUp, Lock, Crown, X, LogOut
  } from 'lucide-svelte';

  let { t, data } = $props<{ t: any, data: any }>();

  let isSuperAdmin = $derived(RBACService.isSuperAdmin(data.user));
  let currentPath = $derived(page.url.pathname);

  // Navigation structure
  let navSections = $derived([
    {
      key: 'section.executive',
      items: [
        { href: '/dashboard',           icon: LayoutDashboard, key: 'nav.dashboard',     badge: null },
        { href: '/dashboard/analytics', icon: TrendingUp,      key: 'nav.analytics',     badge: null },
      ]
    },
    {
      key: 'section.moderation',
      items: [
        { href: '/dashboard/vendors',       icon: Building2,       key: 'nav.vendors',       badge: 'queue' },
        { href: '/dashboard/subscriptions', icon: Crown,           key: 'nav.subscriptions', badge: null },
        { href: '/dashboard/listings',      icon: FileText,        key: 'nav.listings',      badge: null },
        { href: '/dashboard/reviews',       icon: MessageSquare,   key: 'nav.reviews',       badge: null },
        { href: '/dashboard/moderation',    icon: ShieldCheck,     key: 'nav.moderation',    badge: null },
        { href: '/dashboard/messages',      icon: MessageSquare,   key: 'nav.messages',      badge: adminGlobalState.unreadChatsCount > 0 ? adminGlobalState.unreadChatsCount : null },
        { href: '/dashboard/afrah',         icon: Sparkles,        key: 'Afrah VIP Desk',    badge: null },
      ]
    },
    ...(isSuperAdmin ? [{
      key: 'section.users_mgmt',
      items: [
        { href: '/dashboard/users',      icon: Users,           key: 'nav.users',         badge: null },
        { href: '/dashboard/roles',      icon: Lock,            key: 'nav.roles',         badge: null },
      ]
    }] : []),
    {
      key: 'section.commerce',
      items: [
        { href: '/dashboard/bookings',     icon: CalendarCheck, key: 'nav.bookings',      badge: null },
        { href: '/dashboard/inquiries',    icon: MessageSquare, key: 'System Leads',      badge: adminGlobalState.unreadInquiriesCount > 0 ? adminGlobalState.unreadInquiriesCount : null },
        { href: '/dashboard/payments',     icon: CreditCard,    key: 'nav.payments',      badge: null },
        ...(isSuperAdmin ? [
          { href: '/dashboard/finance',      icon: BarChart3,     key: 'nav.finance',       badge: null },
          { href: '/dashboard/commissions',  icon: Percent,       key: 'nav.commissions',   badge: null },
        ] : []),
      ]
    },
    {
      key: 'section.support',
      items: [
        { href: '/dashboard/support',       icon: HeadphonesIcon, key: 'nav.support',     badge: 'urgent' },
        { href: '/dashboard/notifications', icon: Bell,           key: 'nav.notifications',badge: null },
      ]
    },
    {
      key: 'section.content',
      items: [
        { href: '/dashboard/blog/comments', icon: MessageSquare, key: 'nav.blog_comments', badge: null },
        { href: '/dashboard/cms',          icon: Newspaper,    key: 'nav.cms',            badge: null },
        { href: '/dashboard/discover',     icon: Globe,        key: 'nav.discover',       badge: null },
        { href: '/dashboard/marketing',    icon: Megaphone,    key: 'nav.marketing',      badge: null },
      ]
    },
    {
      key: 'section.system',
      items: [
        { href: '/dashboard/audit',        icon: FileText,     key: 'nav.audit',          badge: null },
        { href: '/dashboard/monitoring',   icon: Activity,     key: 'nav.monitoring',     badge: null },
        ...(isSuperAdmin ? [
          { href: '/dashboard/settings',     icon: Settings,     key: 'nav.settings',       badge: null },
        ] : [])
      ]
    },
  ]);

  function isActive(href: string): boolean {
    if (href === '/dashboard') return currentPath === '/dashboard';
    return currentPath.startsWith(href);
  }

  function getInitials(firstName: string, lastName: string): string {
    return `${firstName?.charAt(0) ?? ''}${lastName?.charAt(0) ?? ''}`.toUpperCase() || 'AD';
  }
</script>

<aside class="sidebar" aria-label="Main navigation">
  <!-- Logo Zone -->
  <div class="sidebar-logo">
    <div style="display: flex; align-items: center; justify-content: space-between; width: 100%;">
      <div class="logo-mark">
        <div class="logo-icon">
          <Sparkles size={16} aria-hidden="true" />
        </div>
        {#if !adminGlobalState.sidebarCollapsed}
          <div class="logo-text">
            <span class="logo-gold">ZAFAF</span>
            <span class="logo-white">WORLD</span>
          </div>
        {/if}
      </div>
      {#if !adminGlobalState.sidebarCollapsed}
        <button 
          type="button" 
          class="sidebar-close-mobile" 
          onclick={() => adminGlobalState.sidebarCollapsed = true}
          aria-label="Close sidebar"
        >
          <X size={18} aria-hidden="true" />
        </button>
      {/if}
    </div>
    {#if !adminGlobalState.sidebarCollapsed}
      <div class="admin-suite-badge">
        <Lock size={9} aria-hidden="true" />
        <span>Admin Suite</span>
      </div>
    {/if}
  </div>

  <!-- Navigation -->
  <nav class="sidebar-nav" aria-label="Admin navigation">
    {#each navSections as section}
      {#if !adminGlobalState.sidebarCollapsed}
        <div class="nav-section-label">{t(section.key)}</div>
      {/if}
      {#each section.items as item}
        <a
          href={item.href}
          class="nav-item"
          class:active={isActive(item.href)}
          class:collapsed-item={adminGlobalState.sidebarCollapsed}
          title={adminGlobalState.sidebarCollapsed ? t(item.key) : undefined}
          aria-current={isActive(item.href) ? 'page' : undefined}
        >
          <div class="nav-icon-wrap" class:active-icon={isActive(item.href)}>
            <item.icon size={18} aria-hidden="true" />
          </div>
          {#if !adminGlobalState.sidebarCollapsed}
            <span class="nav-label">{t(item.key)}</span>
            {#if item.key === 'nav.vendors' && (data.dashboard?.pending_approvals_count ?? 0) > 0}
              <span class="nav-badge badge-purple-pill">{data.dashboard?.pending_approvals_count}</span>
            {:else if item.key === 'nav.reviews' && (data.dashboard?.pending_reviews_count ?? 0) > 0}
              <span class="nav-badge badge-purple-pill">{data.dashboard?.pending_reviews_count}</span>
            {:else if item.badge === 'queue'}
              <span class="nav-badge badge-warning-pill">Queue</span>
            {:else if item.badge === 'urgent'}
              <span class="nav-badge badge-danger-pill">Urgent</span>
            {:else if typeof item.badge === 'number'}
              <span class="nav-badge badge-purple-pill">{item.badge}</span>
            {/if}
            {#if isActive(item.href)}
              <div class="active-dot" aria-hidden="true"></div>
            {/if}
          {/if}
        </a>
      {/each}
      {#if !adminGlobalState.sidebarCollapsed}
        <div class="nav-section-gap" aria-hidden="true"></div>
      {/if}
    {/each}
  </nav>

  <!-- Profile Footer -->
  <div class="sidebar-footer">
    <div class="profile-card">
      <div class="profile-avatar">
        {getInitials(data.user?.first_name, data.user?.last_name)}
      </div>
      {#if !adminGlobalState.sidebarCollapsed}
        <div class="profile-info">
          <span class="profile-name">{data.user?.first_name ?? 'Admin'} {data.user?.last_name ?? ''}</span>
          <span class="profile-role">{t('profile.super_admin')}</span>
        </div>
        <button type="button" onclick={() => authStore.logout()} class="logout-btn" title={t('header.logout')}>
          <LogOut size={14} />
        </button>
      {/if}
    </div>
  </div>
</aside>
