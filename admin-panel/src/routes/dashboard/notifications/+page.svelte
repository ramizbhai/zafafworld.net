<script lang="ts">
  import { t, lang } from '$lib/i18n/index.js';
  import { Bell, Plus, Send, CheckCircle, X, Megaphone, Eye } from 'lucide-svelte';
  import { enhance } from '$app/forms';

  let { data } = $props();

  let showModal = $state(false);
  let isSubmitting = $state(false);
  let showSuccess = $state(false);
  let errorMessage = $state('');

  // Form input states
  let titleAr = $state('');
  let titleEn = $state('');
  let messageAr = $state('');
  let messageEn = $state('');
  let targetAudience = $state('all'); // 'all' or specific vendor ID

  // Dynamic search and filter
  let searchQuery = $state('');
  let audienceFilter = $state('all'); // 'all', 'broadcast', 'vendor'

  let filteredNotifications = $derived.by(() => {
    let list = data.notifications || [];

    // Filter by type / audience
    if (audienceFilter === 'broadcast') {
      list = list.filter((n: any) => !n.target_vendor_id);
    } else if (audienceFilter === 'vendor') {
      list = list.filter((n: any) => n.target_vendor_id);
    }

    // Filter by search query
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase().trim();
      list = list.filter((n: any) => 
        n.message_en.toLowerCase().includes(q) || 
        n.message_ar.includes(q) ||
        (n.audience_en && n.audience_en.toLowerCase().includes(q)) ||
        (n.audience_ar && n.audience_ar.includes(q))
      );
    }

    return list;
  });

  // Calculate dynamic stats
  let totalSent = $derived(data.notifications?.length || 0);
  let broadcastCount = $derived(data.notifications?.filter((n: any) => !n.target_vendor_id).length || 0);
  let targetedCount = $derived(data.notifications?.filter((n: any) => n.target_vendor_id).length || 0);
  let unreadCount = $derived(data.notifications?.filter((n: any) => !n.is_read).length || 0);

  function fmtDate(d: string | null) {
    if (!d) return '—';
    return new Date(d).toLocaleDateString($lang === 'ar' ? 'ar-SA' : 'en-US', { 
      month: 'short', 
      day: 'numeric', 
      hour: '2-digit', 
      minute: '2-digit' 
    });
  }
</script>

<div class="fade-in">
  {#if showSuccess}
    <div class="alert alert-success" style="position: fixed; top: 20px; right: 20px; z-index: 1000; box-shadow: 0 4px 12px rgba(0,0,0,0.15); display: flex; align-items: center; gap: 8px;">
      <CheckCircle size={18} />
      <span>{$lang === 'ar' ? 'تم إرسال الإشعار بنجاح' : 'Notification sent successfully'}</span>
    </div>
  {/if}

  <div class="page-header">
    <div class="page-header-left">
      <h1 class="page-title">{$t('nav.notifications')}</h1>
      <p class="page-subtitle">{$lang === 'ar' ? 'إدارة وإرسال الإشعارات للمستخدمين والموردين' : 'Manage and send notifications to users and vendors'}</p>
    </div>
    <button class="btn btn-gold btn-sm" onclick={() => showModal = true}>
      <Plus size={14} /> {$lang === 'ar' ? 'إشعار جديد' : 'New Notification'}
    </button>
  </div>

  <!-- Telemetry Row -->
  <div class="notif-stats">
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'إجمالي المُرسَل' : 'Total Sent'}</span>
      <span class="mini-stat-value">{totalSent}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'الإعلانات العامة' : 'Broadcasts'}</span>
      <span class="mini-stat-value text-gold">{broadcastCount}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'إشعارات الموردين' : 'Targeted Alerts'}</span>
      <span class="mini-stat-value" style="color:var(--info)">{targetedCount}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'غير مقروءة' : 'Unread'}</span>
      <span class="mini-stat-value text-success">{unreadCount}</span>
    </div>
  </div>

  <!-- Filters Area -->
  <div class="card" style="margin-bottom: 20px; padding: 12px 16px; display: flex; align-items: center; gap: 12px; flex-wrap: wrap;">
    <div class="form-group" style="margin: 0; flex: 1; min-width: 200px;">
      <input 
        class="form-input" 
        type="text" 
        bind:value={searchQuery} 
        placeholder={$lang === 'ar' ? 'بحث في محتوى الإشعار...' : 'Search notification content...'} 
        aria-label="Search notifications"
      />
    </div>
    <div class="form-group" style="margin: 0; min-width: 150px;">
      <select class="form-select" bind:value={audienceFilter} aria-label="Filter by audience">
        <option value="all">{$lang === 'ar' ? 'جميع الجماهير' : 'All Audiences'}</option>
        <option value="broadcast">{$lang === 'ar' ? 'عام (بث)' : 'General Broadcasts'}</option>
        <option value="vendor">{$lang === 'ar' ? 'موردين مستهدفين' : 'Targeted Vendors'}</option>
      </select>
    </div>
  </div>

  <!-- Notification Logs Table -->
  <div class="table-container">
    <div class="table-head-bar"><span class="table-title">{$lang === 'ar' ? 'سجل الإشعارات' : 'Notification History'}</span></div>
    <div class="table-scroll">
      <table>
        <thead>
          <tr>
            <th>ID</th>
            <th>{$lang === 'ar' ? 'الجمهور المستهدف' : 'Target Audience'}</th>
            <th>{$lang === 'ar' ? 'المحتوى (عربي)' : 'Content (Arabic)'}</th>
            <th>{$lang === 'ar' ? 'المحتوى (إنجليزي)' : 'Content (English)'}</th>
            <th>{$lang === 'ar' ? 'تاريخ الإرسال' : 'Sent At'}</th>
            <th>{$t('common.status')}</th>
          </tr>
        </thead>
        <tbody>
          {#each filteredNotifications as n}
            <tr>
              <td><span class="mono" style="font-size:11.5px; color:var(--text-tertiary)">{n.id.substring(0,8)}</span></td>
              <td style="font-weight: 600;">{$lang === 'ar' ? n.audience_ar : n.audience_en}</td>
              <td style="max-width:250px; font-size:13px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap" title={n.message_ar}>{n.message_ar}</td>
              <td style="max-width:250px; font-size:13px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap" title={n.message_en}>{n.message_en}</td>
              <td class="text-muted" style="font-size:12px">{fmtDate(n.created_at)}</td>
              <td>
                <span class={n.is_read ? 'badge badge-dot badge-muted' : 'badge badge-dot badge-success'}>
                  {n.is_read ? ($lang === 'ar' ? 'مقروء' : 'Read') : ($lang === 'ar' ? 'نشط/غير مقروء' : 'Active/Unread')}
                </span>
              </td>
            </tr>
          {:else}
            <tr>
              <td colspan="6" style="text-align: center; padding: 40px; color: var(--text-ghost)">
                <Bell size={32} style="margin: 0 auto 12px; color: var(--text-ghost)" />
                <p>{$lang === 'ar' ? 'لا توجد إشعارات مسجلة تطابق الفلاتر' : 'No notifications logged matching filters'}</p>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
</div>

<!-- Send Notification Modal -->
{#if showModal}
  <div
    class="modal-backdrop"
    role="button"
    tabindex="-1"
    onclick={() => showModal = false}
    onkeydown={(e) => { if (e.key === 'Escape') showModal = false; }}
    style="position: fixed; inset: 0; background: rgba(0,0,0,0.5); backdrop-filter: blur(4px); display: flex; align-items: center; justify-content: center; z-index: 1000;"
  >
    <div
      class="modal-card card"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
      style="width: 100%; max-width: 550px; border: 1px solid var(--glass-border); background: var(--bg-surface); padding: 24px; border-radius: 12px; position: relative;"
    >
      <button class="btn-icon" onclick={() => showModal = false} style="position: absolute; top: 16px; right: 16px;" aria-label="Close modal">
        <X size={18} />
      </button>

      <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 20px;">
        <Megaphone size={20} style="color: var(--gold)" />
        <h2 style="font-size: 18px; font-weight: 800; margin: 0;">{$lang === 'ar' ? 'إرسال إشعار جديد' : 'Send New Notification'}</h2>
      </div>

      {#if errorMessage}
        <div class="alert alert-danger" style="margin-bottom: 16px;">{errorMessage}</div>
      {/if}

      <form method="POST" action="?/sendNotification" use:enhance={() => {
        isSubmitting = true;
        errorMessage = '';
        return async ({ result, update }) => {
          isSubmitting = false;
          if (result.type === 'success') {
            showSuccess = true;
            showModal = false;
            titleAr = '';
            titleEn = '';
            messageAr = '';
            messageEn = '';
            targetAudience = 'all';
            setTimeout(() => { showSuccess = false; }, 3000);
          } else {
            errorMessage = $lang === 'ar' ? 'فشل إرسال الإشعار' : 'Failed to send notification';
            setTimeout(() => { errorMessage = ''; }, 3000);
          }
          update({ reset: false });
        };
      }}>
        <div class="form-group">
          <label class="form-label" for="target-audience">{$lang === 'ar' ? 'الجمهور المستهدف' : 'Target Audience'}</label>
          <select class="form-select" id="target-audience" name="targetAudience" bind:value={targetAudience}>
            <option value="all">{$lang === 'ar' ? 'جميع المستخدمين والموردين (عام)' : 'All Users & Vendors (Broadcast)'}</option>
            <optgroup label={$lang === 'ar' ? 'مورد محدد' : 'Specific Vendor'}>
              {#each data.vendors || [] as vendor}
                <option value={vendor.id}>
                  {$lang === 'ar' ? vendor.name_ar : vendor.name_en}
                </option>
              {/each}
            </optgroup>
          </select>
        </div>

        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 12px;">
          <div class="form-group">
            <label class="form-label" for="title-ar">{$lang === 'ar' ? 'العنوان (عربي)' : 'Title (Arabic)'}</label>
            <input class="form-input" id="title-ar" name="titleAr" bind:value={titleAr} type="text" placeholder="مثال: تحديث النظام" />
          </div>
          <div class="form-group">
            <label class="form-label" for="title-en">{$lang === 'ar' ? 'العنوان (إنجليزي)' : 'Title (English)'}</label>
            <input class="form-input" id="title-en" name="titleEn" bind:value={titleEn} type="text" placeholder="e.g., System Update" />
          </div>
        </div>

        <div class="form-group">
          <label class="form-label" for="message-ar">{$lang === 'ar' ? 'الرسالة (عربي)' : 'Message (Arabic)'}</label>
          <textarea class="form-input" id="message-ar" name="messageAr" bind:value={messageAr} rows="3" required placeholder="محتوى الإشعار باللغة العربية..."></textarea>
        </div>

        <div class="form-group">
          <label class="form-label" for="message-en">{$lang === 'ar' ? 'الرسالة (إنجليزي)' : 'Message (English)'}</label>
          <textarea class="form-input" id="message-en" name="messageEn" bind:value={messageEn} rows="3" required placeholder="Notification body in English..."></textarea>
        </div>

        <div style="display: flex; justify-content: flex-end; gap: 10px; margin-top: 20px;">
          <button type="button" class="btn btn-muted btn-sm" onclick={() => showModal = false}>
            {$lang === 'ar' ? 'إلغاء' : 'Cancel'}
          </button>
          <button type="submit" class="btn btn-gold btn-sm" disabled={isSubmitting}>
            <Send size={14} />
            {isSubmitting ? ($lang === 'ar' ? 'جاري الإرسال...' : 'Sending...') : ($lang === 'ar' ? 'إرسال الآن' : 'Send Now')}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<style>
  .notif-stats { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; margin-bottom: 20px; }
  .mini-stat { padding: 16px 18px; display: flex; flex-direction: column; gap: 6px; }
  .mini-stat-label { font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.4px; color: var(--text-ghost); }
  .mini-stat-value { font-size: 22px; font-weight: 800; letter-spacing: -0.4px; }
  @media (max-width: 900px) { .notif-stats { grid-template-columns: repeat(2, 1fr); } }
</style>
