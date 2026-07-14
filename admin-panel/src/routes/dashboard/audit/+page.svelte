<script lang="ts">
  import { t, lang } from '$lib/i18n/index.js';
  import { FileText, Search, Download, AlertCircle } from 'lucide-svelte';
  import { goto } from '$app/navigation';

  interface Props {
    data: {
      logs: any[];
      total: number;
      page: number;
      totalPages: number;
      operator: string;
      eventType: string;
      error?: string;
    };
  }

  let { data }: Props = $props();

  // Local state synced from page data
  let operatorInput = $state('');
  let typeFilter = $state('all');

  // Sync state if url changes externally
  $effect(() => {
    operatorInput = data.operator || '';
    typeFilter = data.eventType || 'all';
  });

  function applyFilters(page: number = 1) {
    const params = new URLSearchParams();
    if (operatorInput.trim()) {
      params.set('operator', operatorInput.trim());
    }
    if (typeFilter && typeFilter !== 'all') {
      params.set('eventType', typeFilter);
    }
    params.set('page', String(page));
    goto(`/dashboard/audit?${params.toString()}`, { keepFocus: true, noScroll: true });
  }

  function handleFilterSubmit(e: Event) {
    e.preventDefault();
    applyFilters(1);
  }

  function handlePageChange(newPage: number) {
    applyFilters(newPage);
  }

  function fmtDate(d: string) {
    if (!d) return '-';
    return new Date(d).toLocaleString($lang === 'ar' ? 'ar-SA' : 'en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  function formatEventType(et: string) {
    return et.replace(/_/g, ' ').toUpperCase();
  }
</script>

<div class="fade-in">
  <div class="page-header">
    <div class="page-header-left">
      <h1 class="page-title">{$t('nav.audit')}</h1>
      <p class="page-subtitle">{$lang === 'ar' ? 'سجل غير قابل للتغيير لجميع إجراءات المشرفين' : 'Immutable log of all administrator actions'}</p>
    </div>
    <button class="btn btn-outline btn-sm" disabled>
      <Download size={14} /> 
      {$lang === 'ar' ? 'تصدير' : 'Export CSV'}
    </button>
  </div>

  {#if data.error}
    <div class="notice-banner error">
      <AlertCircle size={18} class="notice-icon" />
      <div class="notice-text">{data.error}</div>
    </div>
  {/if}

  <div class="table-container">
    <div class="table-head-bar">
      <form class="toolbar" onsubmit={handleFilterSubmit} style="margin:0; flex:1; display:flex; gap:12px; align-items:center;">
        <div class="search-box" style="flex:1; max-width:300px;">
          <Search size={15} />
          <input 
            type="search" 
            placeholder={$lang === 'ar' ? 'بحث بالبريد الإلكتروني المشرف...' : 'Search operator email...'} 
            bind:value={operatorInput} 
            aria-label="Search operators"
          />
        </div>
        <select 
          class="form-select" 
          style="height:36px; width:200px;" 
          bind:value={typeFilter}
          onchange={handleFilterSubmit}
          aria-label="Filter by event type"
        >
          <option value="all">{$t('common.all')}</option>
          <option value="vendor_approved">Vendor Approved / تفعيل المورد</option>
          <option value="vendor_suspended">Vendor Suspended / إيقاف المورد</option>
          <option value="vendor_rejected">Vendor Rejected / رفض المورد</option>
          <option value="review_approved">Review Approved / موافقة التقييم</option>
          <option value="review_rejected">Review Rejected / رفض التقييم</option>
          <option value="booking_received">Booking Received / استلام الحجز</option>
          <option value="inquiry_received">Inquiry Received / استلام الاستفسار</option>
          <option value="listing_approved">Listing Approved / موافقة الإعلان</option>
          <option value="listing_rejected">Listing Rejected / رفض الإعلان</option>
          <option value="system_alert">System Alert / تنبيه النظام</option>
        </select>
        <button type="submit" class="btn btn-outline btn-sm">
          {$lang === 'ar' ? 'تطبيق' : 'Apply'}
        </button>
      </form>
      <span class="table-title">
        {data.total} {$lang === 'ar' ? 'سجل' : 'Entries'}
      </span>
    </div>

    <div class="table-scroll">
      <table aria-label="Administrator audit log table">
        <thead>
          <tr>
            <th>ID</th>
            <th>{$lang === 'ar' ? 'المشغل' : 'Operator'}</th>
            <th>{$lang === 'ar' ? 'نوع الحدث' : 'Event Type'}</th>
            <th>{$lang === 'ar' ? 'الوصف الإجراء' : 'Action Details'}</th>
            <th>IP</th>
            <th>{$lang === 'ar' ? 'التاريخ' : 'Timestamp'}</th>
            <th>{$lang === 'ar' ? 'الحالة' : 'Status'}</th>
          </tr>
        </thead>
        <tbody>
          {#each data.logs as log (log.id)}
            <tr>
              <td>
                <span class="mono" style="font-size:11.5px; color:var(--text-tertiary)">
                  {log.id.slice(0, 8)}
                </span>
              </td>
              <td>
                <span class="mono" style="font-size:12px; color:var(--info); font-weight: 600;">
                  {log.operatorEmail}
                </span>
              </td>
              <td>
                <span class="badge badge-purple" style="font-size: 11px;">
                  {formatEventType(log.eventType)}
                </span>
              </td>
              <td style="font-weight:600; font-size:13px">
                {$lang === 'ar' ? log.messageAr : log.messageEn}
              </td>
              <td>
                <span class="mono" style="font-size:11px; color:var(--text-ghost)">
                  {log.ip}
                </span>
              </td>
              <td class="text-muted" style="font-size:12px">
                {fmtDate(log.createdAt)}
              </td>
              <td>
                <span class="badge badge-dot badge-success">
                  {$lang === 'ar' ? 'ناجح' : 'Success'}
                </span>
              </td>
            </tr>
          {/each}
          {#if data.logs.length === 0}
            <tr>
              <td colspan="7">
                <div class="empty-state">
                  <div class="empty-icon"><FileText size={28} /></div>
                  <h3>{$t('common.no_data')}</h3>
                  <p>{$lang === 'ar' ? 'لا يوجد سجلات تدقيق للمشرفين في قاعدة البيانات' : 'No operator audit logs found in the database'}</p>
                </div>
              </td>
            </tr>
          {/if}
        </tbody>
      </table>
    </div>

    {#if data.totalPages > 1}
      <div class="pagination">
        <span class="pagination-info">
          {$lang === 'ar'
            ? `عرض الصفحة ${data.page} من ${data.totalPages}`
            : `Showing page ${data.page} of ${data.totalPages}`}
        </span>
        <div class="pagination-controls">
          <button 
            class="page-btn" 
            onclick={() => handlePageChange(Math.max(1, data.page - 1))} 
            disabled={data.page === 1} 
            aria-label="Previous page"
          >
            ‹
          </button>
          {#each Array.from({length: data.totalPages}, (_, i) => i + 1) as pageNum}
            <button 
              class="page-btn" 
              class:active={pageNum === data.page} 
              onclick={() => handlePageChange(pageNum)} 
              aria-label="Page {pageNum}" 
              aria-current={pageNum === data.page ? 'page' : undefined}
            >
              {pageNum}
            </button>
          {/each}
          <button 
            class="page-btn" 
            onclick={() => handlePageChange(Math.min(data.totalPages, data.page + 1))} 
            disabled={data.page === data.totalPages} 
            aria-label="Next page"
          >
            ›
          </button>
        </div>
      </div>
    {/if}
  </div>
</div>
