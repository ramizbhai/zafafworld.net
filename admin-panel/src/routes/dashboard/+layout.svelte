<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { notificationStore } from '../../features/notifications/stores/notificationStore.svelte';
  import { FEATURE_FLAGS } from '../../core/config/featureFlags';
  import { wsClient } from '../../core/websocket/wsClient';
  import { getWsUrl } from '../../lib/utils/api';
  import { lang, t, dir } from '$lib/i18n/index.js';
  import { authStore } from '$lib/stores/auth.store.js';
  
  import { adminGlobalState } from '../../core/stores/adminGlobalState.svelte';
  import AdminSidebar from '$lib/components/layout/AdminSidebar.svelte';
  import AdminNavbar from '$lib/components/layout/AdminNavbar.svelte';
  import AdminNotificationPanel from '$lib/components/layout/AdminNotificationPanel.svelte';

  import '$lib/components/layout/styles.css';

  let { data, children } = $props<{ data: any, children: any }>();

  let currentPath = $derived(page.url.pathname);

  onMount(() => {
    if (window.innerWidth > 1024) {
      adminGlobalState.sidebarCollapsed = false;
    }
  });

  $effect(() => {
    if (currentPath && typeof window !== 'undefined' && window.innerWidth <= 1024) {
      adminGlobalState.sidebarCollapsed = true;
    }
  });

  $effect(() => {
    if (typeof window !== 'undefined') {
      const pollMessages = async () => {
        try {
          const res = await fetch('/api/v1/admin/messages/unread-count');
          if (res.ok) {
            const resData = await res.json();
            if (resData.status === 'success') {
              adminGlobalState.unreadChatsCount = resData.chats || 0;
              notificationStore.updateFromRest({ unreadCount: resData.inquiries || 0 });
            }
          }
        } catch (e) {
          console.error('Failed to poll unread messages', e);
        }
      };
      
      pollMessages();
      const intervalId = setInterval(pollMessages, 30000);
      (window as any).__updateAdminUnreadCounts = pollMessages;

      return () => clearInterval(intervalId);
    }
  });

  $effect(() => {
    if (typeof window !== 'undefined' && FEATURE_FLAGS.ENABLE_WS_NOTIFICATIONS) {
      notificationStore.initWsListener();
      wsClient.connect({ url: getWsUrl() });

      return () => {
        wsClient.destroy();
      };
    }
  });

  $effect(() => {
    if (data.user) {
        authStore.initialize(data.user);
    }
  });

  $effect(() => {
    if ($authStore.isInitialized && !$authStore.isAuthenticated) {
        goto('/login');
    }
  });

  $effect(() => {
    if (typeof document !== 'undefined') {
      document.documentElement.dir  = $dir;
      document.documentElement.lang = $lang;
    }
  });
</script>

<svelte:head>
  <title>ZafafWorld — لوحة التحكم الإدارية</title>
</svelte:head>

<div class="ambient-orb orb-gold" aria-hidden="true"></div>
<div class="ambient-orb orb-purple" aria-hidden="true"></div>

{#if !adminGlobalState.sidebarCollapsed}
  <button 
    type="button" 
    class="sidebar-mobile-backdrop" 
    onclick={() => adminGlobalState.sidebarCollapsed = true} 
    aria-label="Close sidebar"
  ></button>
{/if}

<div class="admin-layout" class:collapsed={adminGlobalState.sidebarCollapsed} dir={$dir as 'rtl' | 'ltr'}>
  <AdminSidebar {t} {data} />

  <div class="main-area">
    <AdminNavbar {t} {data} />
    <AdminNotificationPanel {t} {lang} {data} />

    <main class="page-content" id="main-content">
      <div class="content-inner fade-in">
        {@render children()}
      </div>
    </main>
  </div>
</div>
