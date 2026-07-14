<script lang="ts">
  import { t, lang } from '$lib/i18n/index.js';
  import { Search, MessageSquare, Clock, AlertCircle, CheckCircle2 } from 'lucide-svelte';
  import { enhance } from '$app/forms';

  interface Props {
    data: {
      inquiries: any[];
      error?: string;
    };
  }

  let { data }: Props = $props();

  let statusFilter = $state('all');
  let search = $state('');

  let filtered = $derived(data.inquiries.filter(tk => {
    const q = search.toLowerCase();
    const matchesSearch = !search || 
      tk.id.toLowerCase().includes(q) || 
      tk.message.toLowerCase().includes(q) || 
      tk.client_email.toLowerCase().includes(q);
    const matchesStatus = statusFilter === 'all' || tk.status === statusFilter;
    return matchesSearch && matchesStatus;
  }));

  function statusClass(s: string) {
    if (s === 'pending') return 'badge badge-dot badge-warning';
    if (s === 'resolved') return 'badge badge-dot badge-success';
    return 'badge badge-dot badge-muted';
  }

  function statusLabel(s: string) {
    const map: Record<string, string> = {
      pending: $lang === 'ar' ? 'معلق' : 'Pending',
      resolved: $lang === 'ar' ? 'محلول' : 'Resolved'
    };
    return map[s] ?? s;
  }

  function formatDate(d: string): string {
    if (!d) return '-';
    return new Date(d).toLocaleDateString($lang === 'ar' ? 'ar-SA' : 'en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  let pendingCount = $derived(data.inquiries.filter(i => i.status === 'pending').length);
  let resolvedCount = $derived(data.inquiries.filter(i => i.status === 'resolved').length);
</script>

<div class="fade-in">
  <div class="page-header">
    <div class="page-header-left">
      <h1 class="page-title">{$t('nav.support')}</h1>
      <p class="page-subtitle">{$lang === 'ar' ? 'إدارة استفسارات المساعد الذكي والعملاء' : 'Manage AI Assistant inquiries and customer support'}</p>
    </div>
  </div>

  {#if data.error}
    <div class="notice-banner error">
      <AlertCircle size={18} class="notice-icon" />
      <div class="notice-text">{data.error}</div>
    </div>
  {/if}

  <div class="supp-stats">
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'إجمالي الاستفسارات' : 'Total Inquiries'}</span>
      <span class="mini-stat-value" style="color: var(--gold)">{data.inquiries.length}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'معلقة' : 'Pending'}</span>
      <span class="mini-stat-value" style="color: var(--danger)">{pendingCount}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'محلولة' : 'Resolved'}</span>
      <span class="mini-stat-value" style="color: var(--success)">{resolvedCount}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'معدل الحل' : 'Resolution Rate'}</span>
      <span class="mini-stat-value" style="color: var(--info)">
        {data.inquiries.length > 0 ? Math.round((resolvedCount / data.inquiries.length) * 100) : 0}%
      </span>
    </div>
  </div>

  <div class="table-container">
    <div class="table-head-bar">
      <div class="toolbar" style="margin: 0; flex: 1;">
        <div class="search-box" style="flex: 1; max-width: 300px;">
          <Search size={15} />
          <input type="search" placeholder={$t('common.search')} bind:value={search} />
        </div>
        <select class="form-select" style="height: 36px; width: 160px;" bind:value={statusFilter}>
          <option value="all">{$t('common.all')}</option>
          <option value="pending">{$lang === 'ar' ? 'معلق' : 'Pending'}</option>
          <option value="resolved">{$lang === 'ar' ? 'محلول' : 'Resolved'}</option>
        </select>
      </div>
      <span class="table-title">{filtered.length} {$lang === 'ar' ? 'استفسار' : 'Inquiries'}</span>
    </div>
    <div class="table-scroll">
      <table>
        <thead>
          <tr>
            <th>ID</th>
            <th>{$lang === 'ar' ? 'المستخدم' : 'Client Email'}</th>
            <th>{$lang === 'ar' ? 'الرسالة' : 'Message'}</th>
            <th>{$lang === 'ar' ? 'الوقت' : 'Submitted At'}</th>
            <th>{$t('common.status')}</th>
            <th>{$t('common.actions')}</th>
          </tr>
        </thead>
        <tbody>
          {#each filtered as tk}
            <tr>
              <td><span class="mono text-gold" style="font-size: 11px; font-weight: 700">{tk.id.slice(0, 8)}...</span></td>
              <td class="text-muted mono" style="font-size: 12.5px">{tk.client_email}</td>
              <td style="max-width: 320px; white-space: normal; word-break: break-word; font-size: 13px;">
                {tk.message}
              </td>
              <td class="text-muted" style="font-size: 12px">{formatDate(tk.created_at)}</td>
              <td><span class={statusClass(tk.status)}>{statusLabel(tk.status)}</span></td>
              <td>
                {#if tk.status === 'pending'}
                  <form method="POST" action="?/resolve" use:enhance>
                    <input type="hidden" name="id" value={tk.id} />
                    <button type="submit" class="btn btn-gold btn-xs">
                      <CheckCircle2 size={12} />
                      {$lang === 'ar' ? 'حل التذكرة' : 'Resolve'}
                    </button>
                  </form>
                {:else}
                  <span class="text-muted" style="font-size: 12px;">{$lang === 'ar' ? 'محلول' : 'Resolved'}</span>
                {/if}
              </td>
            </tr>
          {/each}
          {#if filtered.length === 0}
            <tr>
              <td colspan="6">
                <div class="empty-state" style="padding: 40px 0; text-align: center; color: var(--text-ghost);">
                  <div class="empty-icon" style="margin-bottom: 12px;"><MessageSquare size={28} /></div>
                  <h3>{$lang === 'ar' ? 'لا توجد استفسارات مطابقة' : 'No inquiries found'}</h3>
                </div>
              </td>
            </tr>
          {/if}
        </tbody>
      </table>
    </div>
  </div>
</div>

<style>
  .supp-stats { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; margin-bottom: 20px; }
  .mini-stat { padding: 16px 18px; display: flex; flex-direction: column; gap: 6px; }
  .mini-stat-label { font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.4px; color: var(--text-ghost); }
  .mini-stat-value { font-size: 22px; font-weight: 800; letter-spacing: -0.5px; }
  @media (max-width: 900px) { .supp-stats { grid-template-columns: repeat(2, 1fr); } }
</style>
