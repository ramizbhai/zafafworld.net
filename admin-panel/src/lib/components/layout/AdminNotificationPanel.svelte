<script lang="ts">
  import { adminGlobalState } from '../../../core/stores/adminGlobalState.svelte';
  import { X } from 'lucide-svelte';

  let { t, lang, data } = $props<{ t: any, lang: any, data: any }>();

  let notifications = $derived(data.notifications || []);
  let unreadCount = $derived(data.unreadCount || 0);

  async function markAsRead(id: string) {
    try {
      const response = await fetch('/dashboard/notifications/mark-read', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ ids: [id] })
      });
      if (response.ok) {
        const { invalidateAll } = await import('$app/navigation');
        invalidateAll();
      }
    } catch (err) {
      console.error('Failed to mark notification as read:', err);
    }
  }

  async function markAllAsRead() {
    try {
      const response = await fetch('/dashboard/notifications/mark-read', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({})
      });
      if (response.ok) {
        const { invalidateAll } = await import('$app/navigation');
        invalidateAll();
      }
    } catch (err) {
      console.error('Failed to mark all notifications as read:', err);
    }
  }
</script>

{#if adminGlobalState.notifOpen}
  <div class="notif-backdrop" onclick={() => adminGlobalState.notifOpen = false} aria-hidden="true"></div>

  <div class="notif-panel" role="dialog" aria-label={t('header.notifications')}>
    <div class="notif-panel-header">
      <span class="notif-panel-title">{t('header.notifications')}</span>
      <div style="display: flex; gap: 8px; align-items: center;">
        {#if unreadCount > 0}
          <button 
            class="btn btn-outline btn-xs" 
            style="font-size: 10px; padding: 2px 6px;"
            onclick={markAllAsRead}
          >
            {lang === 'ar' ? 'تحديد الكل مقروء' : 'Mark all read'}
          </button>
        {/if}
        <button class="notif-close" onclick={() => adminGlobalState.notifOpen = false} aria-label="Close notifications">
          <X size={14} />
        </button>
      </div>
    </div>
    <div class="notif-list">
      {#each notifications as notif}
        <button 
          class="notif-item" 
          style="width: 100%; text-align: start; border: none; background: transparent; cursor: pointer; display: flex; align-items: flex-start; gap: 10px; padding: 12px 16px; border-bottom: 1px solid var(--glass-border);"
          class:unread={!notif.is_read}
          onclick={() => !notif.is_read && markAsRead(notif.id)}
        >
          <div 
            class="notif-type-dot" 
            class:type-approval={notif.event_type==='user_registered' || notif.event_type==='vendor_registered'} 
            class:type-payment={notif.event_type==='booking_received'} 
            class:type-support={notif.event_type==='inquiry_received'} 
            class:type-system={notif.event_type==='system_alert' || notif.event_type==='booking_cancelled' || notif.event_type==='review_received'}
          ></div>
          <div class="notif-content">
            <p class="notif-msg">
              {lang === 'ar' ? notif.message_ar : notif.message_en}
            </p>
            <span class="notif-time">{new Date(notif.created_at).toLocaleTimeString(lang, { hour: '2-digit', minute: '2-digit' })}</span>
          </div>
          {#if !notif.is_read}
            <div class="unread-dot" aria-label="Unread"></div>
          {/if}
        </button>
      {/each}
      {#if notifications.length === 0}
        <div style="padding: 24px; text-align: center; color: var(--text-ghost); font-size: 13px;">
          {lang === 'ar' ? 'لا توجد إشعارات' : 'No notifications'}
        </div>
      {/if}
    </div>
  </div>
{/if}
