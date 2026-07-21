<script lang="ts">
  import { t, lang } from '$lib/i18n/index.js';
  import { Search, MessageSquare, Clock, AlertCircle, CheckCircle2, Eye, Trash2, Filter, User, Mail, Phone, Tag } from 'lucide-svelte';
  import { enhance } from '$app/forms';

  interface SupportItem {
    id: string;
    name: string;
    email: string;
    phone: string | null;
    subject: string;
    message: string;
    status: 'new' | 'in_progress' | 'resolved' | 'closed';
    assigned_admin_id: string | null;
    assigned_admin_name: string | null;
    created_at: string;
    updated_at: string;
  }

  interface Props {
    data: {
      items: SupportItem[];
      pagination: {
        total: number;
        page: number;
        limit: number;
        pages: number;
      };
      statusFilter?: string;
      searchFilter?: string;
      error?: string;
    };
    form?: {
      success?: boolean;
      message?: string;
      error?: string;
    };
  }

  let { data, form }: Props = $props();

  let search = $state(data.searchFilter || '');
  let statusFilter = $state(data.statusFilter || 'all');
  let selectedMessage = $state<SupportItem | null>(null);

  let filteredItems = $derived((data.items || []).filter(item => {
    const q = search.toLowerCase().trim();
    const matchesSearch = !q || 
      item.name.toLowerCase().includes(q) || 
      item.email.toLowerCase().includes(q) || 
      item.subject.toLowerCase().includes(q) ||
      item.message.toLowerCase().includes(q);
    const matchesStatus = statusFilter === 'all' || item.status === statusFilter;
    return matchesSearch && matchesStatus;
  }));

  let newCount = $derived((data.items || []).filter(i => i.status === 'new').length);
  let inProgressCount = $derived((data.items || []).filter(i => i.status === 'in_progress').length);
  let resolvedCount = $derived((data.items || []).filter(i => i.status === 'resolved' || i.status === 'closed').length);

  function statusClass(s: string) {
    if (s === 'new') return 'badge badge-dot badge-danger';
    if (s === 'in_progress') return 'badge badge-dot badge-warning';
    if (s === 'resolved') return 'badge badge-dot badge-success';
    return 'badge badge-dot badge-muted';
  }

  function statusLabel(s: string) {
    const map: Record<string, string> = {
      new: $lang === 'ar' ? 'جديد' : 'New',
      in_progress: $lang === 'ar' ? 'قيد المعالجة' : 'In Progress',
      resolved: $lang === 'ar' ? 'تم الحل' : 'Resolved',
      closed: $lang === 'ar' ? 'مغلق' : 'Closed'
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
</script>

<div class="fade-in">
  <div class="page-header">
    <div class="page-header-left">
      <h1 class="page-title">{$t('nav.support')}</h1>
      <p class="page-subtitle">{$lang === 'ar' ? 'إدارة رسائل واستفسارات الدعم الفني الواردة من نموذج اتصل بنا' : 'Manage public Contact Us support inquiries & customer tickets'}</p>
    </div>
  </div>

  {#if data.error || form?.error}
    <div class="notice-banner error">
      <AlertCircle size={18} class="notice-icon" />
      <div class="notice-text">{data.error || form?.error}</div>
    </div>
  {/if}

  {#if form?.success}
    <div class="notice-banner success mb-4">
      <CheckCircle2 size={18} class="notice-icon" />
      <div class="notice-text">{form.message || ($lang === 'ar' ? 'تم تحديث حالة الرسالة بنجاح' : 'Message updated successfully')}</div>
    </div>
  {/if}

  <!-- Stats Grid -->
  <div class="supp-stats">
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'إجمالي الرسائل' : 'Total Submissions'}</span>
      <span class="mini-stat-value" style="color: var(--gold)">{data.pagination?.total || data.items.length}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'رسائل جديدة' : 'New Tickets'}</span>
      <span class="mini-stat-value" style="color: var(--danger)">{newCount}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'قيد المعالجة' : 'In Progress'}</span>
      <span class="mini-stat-value" style="color: var(--warning)">{inProgressCount}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'تم الحل / مغلقة' : 'Resolved / Closed'}</span>
      <span class="mini-stat-value" style="color: var(--success)">{resolvedCount}</span>
    </div>
  </div>

  <!-- Main Table Container -->
  <div class="table-container">
    <div class="table-head-bar">
      <div class="toolbar" style="margin: 0; flex: 1;">
        <div class="search-box" style="flex: 1; max-width: 320px;">
          <Search size={15} />
          <input type="search" placeholder={$lang === 'ar' ? 'البحث بالاسم، البريد أو الموضوع...' : 'Search name, email or subject...'} bind:value={search} />
        </div>
        <select class="form-select" style="height: 36px; width: 180px;" bind:value={statusFilter}>
          <option value="all">{$t('common.all')} ({$lang === 'ar' ? 'الكل' : 'All Statuses'})</option>
          <option value="new">{$lang === 'ar' ? 'جديد' : 'New'}</option>
          <option value="in_progress">{$lang === 'ar' ? 'قيد المعالجة' : 'In Progress'}</option>
          <option value="resolved">{$lang === 'ar' ? 'تم الحل' : 'Resolved'}</option>
          <option value="closed">{$lang === 'ar' ? 'مغلق' : 'Closed'}</option>
        </select>
      </div>
      <span class="table-title">{filteredItems.length} {$lang === 'ar' ? 'رسالة' : 'Messages'}</span>
    </div>

    <div class="table-scroll">
      <table>
        <thead>
          <tr>
            <th>{$lang === 'ar' ? 'الاسم والبيانات' : 'Sender Details'}</th>
            <th>{$lang === 'ar' ? 'الموضوع والرسالة' : 'Subject & Excerpt'}</th>
            <th>{$lang === 'ar' ? 'التاريخ والوقت' : 'Submitted Date'}</th>
            <th>{$t('common.status')}</th>
            <th class="text-end">{$t('common.actions')}</th>
          </tr>
        </thead>
        <tbody>
          {#each filteredItems as item}
            <tr>
              <td>
                <div class="flex flex-col gap-0.5">
                  <span class="font-semibold text-main" style="font-size: 13.5px;">{item.name}</span>
                  <span class="text-muted mono" style="font-size: 12px">{item.email}</span>
                  {#if item.phone}
                    <span class="text-ghost" style="font-size: 11px">{item.phone}</span>
                  {/if}
                </div>
              </td>

              <td style="max-width: 300px;">
                <div class="flex flex-col gap-1">
                  <span class="font-bold text-main" style="font-size: 13px;">{item.subject}</span>
                  <span class="text-muted line-clamp-2" style="font-size: 12px; line-height: 1.4;">
                    {item.message}
                  </span>
                </div>
              </td>

              <td>
                <span class="text-muted" style="font-size: 12px; whitespace: nowrap;">
                  {formatDate(item.created_at)}
                </span>
              </td>

              <td>
                <span class={statusClass(item.status)}>{statusLabel(item.status)}</span>
              </td>

              <td class="text-end">
                <div class="flex items-center justify-end gap-2">
                  <!-- View Details Modal Trigger -->
                  <button 
                    type="button" 
                    class="btn btn-ghost btn-sm"
                    title={$lang === 'ar' ? 'عرض التفاصيل' : 'View Message'}
                    onclick={() => selectedMessage = item}
                  >
                    <Eye size={15} />
                  </button>

                  <!-- Update Status Action Form -->
                  <form method="POST" action="?/updateStatus" use:enhance class="inline-flex">
                    <input type="hidden" name="id" value={item.id} />
                    <select 
                      name="status" 
                      class="form-select text-xs py-1 px-2"
                      style="height: 28px; width: 115px;"
                      onchange={(e) => e.currentTarget.form?.requestSubmit()}
                    >
                      <option value="new" selected={item.status === 'new'}>{$lang === 'ar' ? 'جديد' : 'New'}</option>
                      <option value="in_progress" selected={item.status === 'in_progress'}>{$lang === 'ar' ? 'قيد المعالجة' : 'In Progress'}</option>
                      <option value="resolved" selected={item.status === 'resolved'}>{$lang === 'ar' ? 'تم الحل' : 'Resolved'}</option>
                      <option value="closed" selected={item.status === 'closed'}>{$lang === 'ar' ? 'مغلق' : 'Closed'}</option>
                    </select>
                  </form>

                  <!-- Delete Action Form -->
                  <form method="POST" action="?/deleteMessage" use:enhance class="inline-flex" onsubmit={(e) => {
                    if (!confirm($lang === 'ar' ? 'هل أنت تأكد من حذف هذه الرسالة؟' : 'Are you sure you want to delete this message?')) {
                      e.preventDefault();
                    }
                  }}>
                    <input type="hidden" name="id" value={item.id} />
                    <button type="submit" class="btn btn-ghost btn-sm text-danger" title={$lang === 'ar' ? 'حذف' : 'Delete'}>
                      <Trash2 size={15} />
                    </button>
                  </form>
                </div>
              </td>
            </tr>
          {/each}

          {#if filteredItems.length === 0}
            <tr>
              <td colspan="5">
                <div class="empty-state" style="padding: 40px 0; text-align: center; color: var(--text-ghost);">
                  <div class="empty-icon" style="margin-bottom: 12px;"><MessageSquare size={32} /></div>
                  <h3>{$lang === 'ar' ? 'لا توجد رسائل دعم مطابقة' : 'No support messages found'}</h3>
                </div>
              </td>
            </tr>
          {/if}
        </tbody>
      </table>
    </div>
  </div>
</div>

<!-- View Message Detail Modal -->
{#if selectedMessage}
  <div class="modal-backdrop" onclick={() => selectedMessage = null} role="presentation">
    <div class="modal-card max-w-xl p-6 bg-white rounded-2xl shadow-xl" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true">
      <div class="flex items-center justify-between pb-4 border-b border-gray-100 mb-5">
        <h2 class="text-lg font-bold text-main flex items-center gap-2">
          <Tag size={18} class="text-gold" />
          {selectedMessage.subject}
        </h2>
        <button type="button" class="btn btn-ghost btn-sm" onclick={() => selectedMessage = null}>✕</button>
      </div>

      <div class="flex flex-col gap-4 text-sm mb-6">
        <div class="grid grid-cols-2 gap-4 p-4 rounded-xl bg-gray-50 border border-gray-100">
          <div>
            <span class="text-xs font-semibold text-muted block mb-1">{$lang === 'ar' ? 'اسم المرسل' : 'Sender Name'}</span>
            <span class="font-bold text-main">{selectedMessage.name}</span>
          </div>
          <div>
            <span class="text-xs font-semibold text-muted block mb-1">{$lang === 'ar' ? 'البريد الإلكتروني' : 'Email Address'}</span>
            <a href="mailto:{selectedMessage.email}" class="text-gold font-mono hover:underline">{selectedMessage.email}</a>
          </div>
          {#if selectedMessage.phone}
            <div>
              <span class="text-xs font-semibold text-muted block mb-1">{$lang === 'ar' ? 'رقم الهاتف' : 'Phone Number'}</span>
              <a href="tel:{selectedMessage.phone}" class="font-mono text-main hover:underline">{selectedMessage.phone}</a>
            </div>
          {/if}
          <div>
            <span class="text-xs font-semibold text-muted block mb-1">{$lang === 'ar' ? 'وقت الإرسال' : 'Submitted At'}</span>
            <span class="text-main">{formatDate(selectedMessage.created_at)}</span>
          </div>
        </div>

        <div>
          <span class="text-xs font-semibold text-muted block mb-2">{$lang === 'ar' ? 'مضمون الرسالة' : 'Full Message Body'}</span>
          <div class="p-4 rounded-xl bg-surface-alt border border-gray-200 text-main leading-relaxed whitespace-pre-wrap">
            {selectedMessage.message}
          </div>
        </div>
      </div>

      <div class="flex items-center justify-between pt-4 border-t border-gray-100">
        <!-- Change Status Form inside Modal -->
        <form method="POST" action="?/updateStatus" use:enhance class="flex items-center gap-3">
          <input type="hidden" name="id" value={selectedMessage.id} />
          <span class="text-xs font-semibold text-muted">{$t('common.status')}:</span>
          <select 
            name="status" 
            class="form-select text-xs py-1.5 px-3"
            value={selectedMessage.status}
            onchange={(e) => e.currentTarget.form?.requestSubmit()}
          >
            <option value="new">{$lang === 'ar' ? 'جديد' : 'New'}</option>
            <option value="in_progress">{$lang === 'ar' ? 'قيد المعالجة' : 'In Progress'}</option>
            <option value="resolved">{$lang === 'ar' ? 'تم الحل' : 'Resolved'}</option>
            <option value="closed">{$lang === 'ar' ? 'مغلق' : 'Closed'}</option>
          </select>
        </form>

        <button type="button" class="btn btn-outline btn-sm" onclick={() => selectedMessage = null}>
          {$lang === 'ar' ? 'إغلاق' : 'Close'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .supp-stats { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; margin-bottom: 20px; }
  .mini-stat { padding: 16px 18px; display: flex; flex-direction: column; gap: 6px; }
  .mini-stat-label { font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.4px; color: var(--text-ghost); }
  .mini-stat-value { font-size: 22px; font-weight: 800; letter-spacing: -0.5px; }
  @media (max-width: 900px) { .supp-stats { grid-template-columns: repeat(2, 1fr); } }

  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    backdrop-filter: blur(2px);
    z-index: 999;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 16px;
  }
  .modal-card {
    width: 100%;
  }
</style>
