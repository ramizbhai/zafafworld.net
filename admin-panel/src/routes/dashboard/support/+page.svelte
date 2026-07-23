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

  function getActiveStatusBtnClass(s: string) {
    if (s === 'new') return 'bg-rose-500 text-white shadow-xs';
    if (s === 'in_progress') return 'bg-amber-500 text-white shadow-xs';
    if (s === 'resolved') return 'bg-emerald-500 text-white shadow-xs';
    return 'bg-zinc-500 text-white shadow-xs';
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

<svelte:window onkeydown={(e) => { if (e.key === 'Escape' && selectedMessage) selectedMessage = null; }} />

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
  <div class="modal-backdrop select-none" onclick={() => selectedMessage = null} role="presentation">
    <div 
      class="modal-card max-w-2xl w-full bg-white dark:bg-zinc-900 rounded-3xl shadow-2xl border border-zinc-100 dark:border-zinc-800/80 overflow-hidden flex flex-col max-h-[85vh] animate-in fade-in zoom-in-95 duration-200 select-text" 
      onclick={(e) => e.stopPropagation()} 
      role="dialog" 
      aria-modal="true"
    >
      <!-- Modal Header -->
      <div class="flex items-center justify-between px-6 py-5 bg-gradient-to-r from-zinc-50/50 to-white dark:from-zinc-950/20 dark:to-zinc-900 border-b border-zinc-100 dark:border-zinc-800">
        <div class="flex flex-col gap-1">
          <div class="flex items-center gap-2">
            <span class="p-2 bg-amber-50 dark:bg-amber-950/30 text-amber-600 dark:text-amber-400 rounded-xl">
              <Tag size={18} />
            </span>
            <h2 class="text-lg font-bold text-zinc-900 dark:text-white">
              {$lang === 'ar' ? 'تفاصيل طلب الدعم' : 'Support Request Details'}
            </h2>
          </div>
          <span class="text-[11px] font-mono text-zinc-400 dark:text-zinc-500">
            ID: {selectedMessage.id}
          </span>
        </div>
        <button 
          type="button" 
          class="p-2 hover:bg-zinc-100 dark:hover:bg-zinc-800 text-zinc-400 hover:text-zinc-650 dark:hover:text-zinc-350 rounded-full transition-colors duration-150 cursor-pointer" 
          onclick={() => selectedMessage = null}
          aria-label={$lang === 'ar' ? 'إغلاق' : 'Close'}
        >
          ✕
        </button>
      </div>

      <!-- Modal Body (Scrollable) -->
      <div class="p-6 overflow-y-auto flex-1 flex flex-col gap-5">
        <!-- Subject -->
        <div>
          <span class="text-[10px] font-bold text-zinc-400 dark:text-zinc-500 uppercase tracking-wider block mb-1">
            {$lang === 'ar' ? 'موضوع الرسالة' : 'Message Subject'}
          </span>
          <h3 class="text-base font-bold text-zinc-900 dark:text-white leading-snug">
            {selectedMessage.subject}
          </h3>
        </div>

        <!-- Sender Meta Grid -->
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 p-5 rounded-2xl bg-zinc-50 dark:bg-zinc-900/50 border border-zinc-100 dark:border-zinc-800/80">
          <div class="flex items-center gap-3">
            <span class="p-2 bg-white dark:bg-zinc-800 text-zinc-500 dark:text-zinc-400 rounded-xl shadow-xs border border-zinc-100 dark:border-zinc-700/50">
              <User size={16} />
            </span>
            <div>
              <span class="text-[10px] text-zinc-400 dark:text-zinc-500 block">{$lang === 'ar' ? 'اسم المرسل' : 'Sender Name'}</span>
              <span class="font-bold text-zinc-800 dark:text-zinc-200">{selectedMessage.name}</span>
            </div>
          </div>

          <div class="flex items-center gap-3">
            <span class="p-2 bg-white dark:bg-zinc-800 text-zinc-500 dark:text-zinc-400 rounded-xl shadow-xs border border-zinc-100 dark:border-zinc-700/50">
              <Mail size={16} />
            </span>
            <div class="min-w-0 flex-1">
              <span class="text-[10px] text-zinc-400 dark:text-zinc-500 block">{$lang === 'ar' ? 'البريد الإلكتروني' : 'Email Address'}</span>
              <a href="mailto:{selectedMessage.email}" class="font-mono text-amber-600 dark:text-amber-400 hover:underline block truncate" title={selectedMessage.email}>
                {selectedMessage.email}
              </a>
            </div>
          </div>

          {#if selectedMessage.phone}
            <div class="flex items-center gap-3">
              <span class="p-2 bg-white dark:bg-zinc-800 text-zinc-500 dark:text-zinc-400 rounded-xl shadow-xs border border-zinc-100 dark:border-zinc-700/50">
                <Phone size={16} />
              </span>
              <div>
                <span class="text-[10px] text-zinc-400 dark:text-zinc-500 block">{$lang === 'ar' ? 'رقم الهاتف' : 'Phone Number'}</span>
                <a href="tel:{selectedMessage.phone}" class="font-mono text-zinc-800 dark:text-zinc-200 hover:underline block">
                  {selectedMessage.phone}
                </a>
              </div>
            </div>
          {/if}

          <div class="flex items-center gap-3">
            <span class="p-2 bg-white dark:bg-zinc-800 text-zinc-500 dark:text-zinc-400 rounded-xl shadow-xs border border-zinc-100 dark:border-zinc-700/50">
              <Clock size={16} />
            </span>
            <div>
              <span class="text-[10px] text-zinc-400 dark:text-zinc-500 block">{$lang === 'ar' ? 'تاريخ الإرسال' : 'Submitted At'}</span>
              <span class="text-zinc-800 dark:text-zinc-200">{formatDate(selectedMessage.created_at)}</span>
            </div>
          </div>
        </div>

        <!-- Message Box -->
        <div class="flex flex-col gap-1.5">
          <span class="text-[10px] font-bold text-zinc-400 dark:text-zinc-500 uppercase tracking-wider">
            {$lang === 'ar' ? 'نص الرسالة الدعم' : 'Full Support Message'}
          </span>
          <div class="p-5 rounded-2xl bg-zinc-50 dark:bg-zinc-950 border border-zinc-100 dark:border-zinc-800/80 text-zinc-800 dark:text-zinc-200 leading-relaxed text-sm whitespace-pre-wrap break-words max-h-64 overflow-y-auto border-l-4 border-amber-500 dark:border-amber-600">
            {selectedMessage.message}
          </div>
        </div>
      </div>

      <!-- Modal Footer & Actions -->
      <div class="px-6 py-4 bg-zinc-50 dark:bg-zinc-950 border-t border-zinc-100 dark:border-zinc-850 flex flex-col md:flex-row md:items-center md:justify-between gap-4">
        <!-- Status Changer Form -->
        <div class="flex flex-col sm:flex-row sm:items-center gap-2">
          <span class="text-[10px] font-bold text-zinc-400 dark:text-zinc-500 uppercase tracking-wider whitespace-nowrap">
            {$lang === 'ar' ? 'حالة الطلب الحالية:' : 'Current Status:'}
          </span>
          <form method="POST" action="?/updateStatus" use:enhance class="inline-flex">
            <input type="hidden" name="id" value={selectedMessage.id} />
            <div class="flex flex-wrap items-center gap-1 p-1 bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-800 rounded-xl shadow-xs">
              {#each ['new', 'in_progress', 'resolved', 'closed'] as st}
                <button 
                  type="submit" 
                  name="status" 
                  value={st}
                  class="px-2.5 py-1 text-[11px] font-bold rounded-lg cursor-pointer transition-all duration-150 {selectedMessage.status === st ? getActiveStatusBtnClass(st) : 'text-zinc-500 dark:text-zinc-400 hover:bg-zinc-100 dark:hover:bg-zinc-800'}"
                >
                  {statusLabel(st)}
                </button>
              {/each}
            </div>
          </form>
        </div>

        <button 
          type="button" 
          class="px-5 py-2.5 rounded-xl border border-zinc-250 dark:border-zinc-700 hover:bg-zinc-100 dark:hover:bg-zinc-900 text-sm font-semibold text-zinc-700 dark:text-zinc-300 transition-colors duration-150 cursor-pointer self-end md:self-auto" 
          onclick={() => selectedMessage = null}
        >
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
    backdrop-filter: blur(4px);
    z-index: 999;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 16px;
  }
  .modal-card {
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
  }
</style>
