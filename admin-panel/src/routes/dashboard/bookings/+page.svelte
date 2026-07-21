<script lang="ts">
  import { t, lang } from '$lib/i18n/index.js';
  import { goto, invalidateAll } from '$app/navigation';
  import { enhance } from '$app/forms';

  interface Props {
    data: {
      bookings: any[];
      total: number;
      page: number;
      totalPages: number;
      status: string;
      search: string;
      error?: string;
    };
  }

  let { data }: Props = $props();

  // Local state synced from page data
  let searchInput = $state('');
  let statusFilter = $state('all');

  // Sync state if url changes externally
  $effect(() => {
    searchInput = data.search || '';
    statusFilter = data.status || 'all';
  });

  function applyFilters(page: number = 1) {
    const params = new URLSearchParams();
    if (searchInput.trim()) {
      params.set('search', searchInput.trim());
    }
    if (statusFilter && statusFilter !== 'all') {
      params.set('status', statusFilter);
    }
    params.set('page', String(page));
    goto(`/dashboard/bookings?${params.toString()}`, { keepFocus: true, noScroll: true });
  }

  function handleFilterSubmit(e: Event) {
    e.preventDefault();
    applyFilters(1);
  }

  function handlePageChange(newPage: number) {
    applyFilters(newPage);
  }

  function statusClass(s: string) {
    if (s === 'confirmed') return 'badge badge-dot badge-success';
    if (s === 'completed') return 'badge badge-dot badge-info';
    if (s === 'cancelled') return 'badge badge-dot badge-danger';
    return 'badge badge-dot badge-warning';
  }

  function statusLabel(s: string) {
    const map: Record<string, string> = {
      confirmed: $lang === 'ar' ? 'مؤكد' : 'Confirmed',
      completed: $lang === 'ar' ? 'مكتمل' : 'Completed',
      cancelled: $lang === 'ar' ? 'ملغى' : 'Cancelled',
      pending:   $lang === 'ar' ? 'معلق' : 'Pending',
    };
    return map[s] ?? s;
  }

  function fmt(n: number) {
    if (n === undefined || n === null) return '0';
    return n.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  }

  function fmtDate(d: string) {
    if (!d) return '-';
    return new Date(d).toLocaleDateString($lang === 'ar' ? 'ar-SA' : 'en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric'
    });
  }
</script>

<div class="fade-in">
  <div class="page-header">
    <div class="page-header-left">
      <h1 class="page-title">{$t('nav.bookings')}</h1>
      <p class="page-subtitle">{$lang === 'ar' ? 'إدارة ومتابعة جميع حجوزات المدفوعات والضمان للمنصة' : 'Manage and track all platform escrow payment bookings'}</p>
    </div>
  </div>

  {#if data.error}
    <div class="notice-banner error">
      <AlertCircle size={18} class="notice-icon" />
      <div class="notice-text">{data.error}</div>
    </div>
  {/if}

  <div class="bk-stats">
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'إجمالي الحجوزات المطابقة' : 'Matching Bookings'}</span>
      <span class="mini-stat-value" style="color: var(--gold)">{data.total}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'الصحفة الحالية' : 'Current Page'}</span>
      <span class="mini-stat-value" style="color: var(--info)">{data.page} / {data.totalPages}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'فلتر الحالة' : 'Status Filter'}</span>
      <span class="mini-stat-value" style="color: var(--success)">
        {statusLabel(data.status)}
      </span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'بوابة المعالجة' : 'Processing Core'}</span>
      <span class="mini-stat-value" style="color: var(--text-main)">{$lang === 'ar' ? 'سحابي ضامن' : 'Escrow Core'}</span>
    </div>
  </div>

  <div class="table-container">
    <div class="table-head-bar">
      <form class="toolbar" onsubmit={handleFilterSubmit} style="margin:0; flex:1; display:flex; gap:12px; align-items:center;">
        <div class="search-box" style="flex:1; max-width:300px;">
          <Search size={15} />
          <input 
            type="search" 
            placeholder={$t('common.search')} 
            bind:value={searchInput} 
            aria-label="Search bookings"
          />
        </div>
        <select 
          class="form-select" 
          style="height:36px; width:140px;" 
          bind:value={statusFilter} 
          onchange={handleFilterSubmit}
          aria-label="Filter by booking status"
        >
          <option value="all">{$t('common.all')}</option>
          <option value="pending">{$lang === 'ar' ? 'معلق' : 'Pending'}</option>
          <option value="confirmed">{$lang === 'ar' ? 'مؤكد' : 'Confirmed'}</option>
          <option value="completed">{$lang === 'ar' ? 'مكتمل' : 'Completed'}</option>
          <option value="cancelled">{$lang === 'ar' ? 'ملغى' : 'Cancelled'}</option>
        </select>
        <button type="submit" class="btn btn-outline btn-sm">
          {$lang === 'ar' ? 'تطبيق' : 'Apply'}
        </button>
      </form>
      <span class="table-title">{data.total} {$lang === 'ar' ? 'حجز' : 'Bookings'}</span>
    </div>

    <div class="table-scroll">
      <table aria-label="Bookings ledger table">
        <thead>
          <tr>
            <th>{$lang === 'ar' ? 'رقم الحجز' : 'Booking ID'}</th>
            <th>{$lang === 'ar' ? 'العميل' : 'Customer'}</th>
            <th>{$lang === 'ar' ? 'المورد' : 'Vendor'}</th>
            <th>{$lang === 'ar' ? 'المنتج / الخدمة' : 'Product / Service'}</th>
            <th>{$lang === 'ar' ? 'تاريخ الحفل' : 'Event Date'}</th>
            <th style="text-align:end">{$lang === 'ar' ? 'إجمالي المبلغ' : 'Total Price'}</th>
            <th style="text-align:end">{$lang === 'ar' ? 'الدفعة المدفوعة' : 'Deposit Paid'}</th>
            <th>{$t('common.status')}</th>
            <th>{$t('common.actions')}</th>
          </tr>
        </thead>
        <tbody>
          {#each data.bookings as b (b.id)}
            <tr>
              <td>
                <span class="mono" style="font-size:12px; color:var(--gold); font-weight:700">
                  {b.bookingNumber}
                </span>
              </td>
              <td style="font-weight:600">
                {b.customerFirstName} {b.customerLastName}
                <div class="cell-sub" style="font-size:11px; color:var(--text-ghost)">{b.customerEmail}</div>
              </td>
              <td class="text-muted">
                {$lang === 'ar' && b.vendorNameAr ? b.vendorNameAr : b.vendorNameEn}
              </td>
              <td>
                <span class="badge badge-muted">
                  {$lang === 'ar' && b.productNameAr ? b.productNameAr : b.productNameEn}
                </span>
                <div class="cell-sub" style="font-size:11px; color:var(--text-ghost)">{b.eventType}</div>
              </td>
              <td class="text-muted" style="font-size:12.5px">{fmtDate(b.weddingDate)}</td>
              <td style="text-align:end; font-weight:700; font-size:13.5px; color:var(--gold)">
                {fmt(b.totalPrice)} SAR
              </td>
              <td style="text-align:end; font-weight:600; font-size:13px; color:var(--success)">
                {fmt(b.depositPaid)} SAR
              </td>
              <td>
                <div style="display:flex; gap:6px;">
                  {#if b.status === 'pending'}
                    <form method="POST" action="?/updateStatus" use:enhance={() => {
                      return async ({ update }) => {
                        await invalidateAll();
                        update();
                      };
                    }}>
                      <input type="hidden" name="id" value={b.id} />
                      <input type="hidden" name="to_status" value="confirmed" />
                      <button type="submit" class="btn btn-sm btn-success" style="padding: 4px 8px; font-size:11px;">
                        <CheckCircle2 size={12} /> {$lang === 'ar' ? 'تأكيد' : 'Confirm'}
                      </button>
                    </form>
                  {/if}
                  {#if b.status !== 'cancelled' && b.status !== 'completed'}
                    <form method="POST" action="?/updateStatus" use:enhance={() => {
                      return async ({ update }) => {
                        await invalidateAll();
                        update();
                      };
                    }}>
                      <input type="hidden" name="id" value={b.id} />
                      <input type="hidden" name="to_status" value="cancelled" />
                      <button type="submit" class="btn btn-sm btn-outline" style="padding: 4px 8px; font-size:11px; color:var(--danger); border-color:var(--danger-border);" onclick={(e) => {
                        if (!confirm($lang === 'ar' ? 'هل أنت تأكد من إلغاء هذا الحجز؟' : 'Are you sure you want to cancel this booking?')) {
                          e.preventDefault();
                        }
                      }}>
                        <XCircle size={12} /> {$lang === 'ar' ? 'إلغاء' : 'Cancel'}
                      </button>
                    </form>
                  {/if}
                </div>
              </td>

            </tr>
          {/each}
          {#if data.bookings.length === 0}
            <tr>
              <td colspan="9">
                <div class="empty-state">
                  <div class="empty-icon"><CalendarCheck size={28} /></div>
                  <h3>{$t('common.no_data')}</h3>
                  <p>{$lang === 'ar' ? 'لا يوجد حجوزات مطابقة في قاعدة البيانات' : 'No bookings matching the filters found in the database'}</p>
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
            ? `عرض الصفحة ${data.page} من ${data.totalPages} (إجمالي ${data.total} حجز)`
            : `Showing page ${data.page} of ${data.totalPages} (Total ${data.total} bookings)`}
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
          {#each Array.from({length: data.totalPages}, (_, i) => i + 1) as p}
            <button 
              class="page-btn" 
              class:active={p === data.page} 
              onclick={() => handlePageChange(p)} 
              aria-label="Page {p}" 
              aria-current={p === data.page ? 'page' : undefined}
            >
              {p}
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

<style>
  .bk-stats { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; margin-bottom: 20px; }
  .mini-stat { padding: 16px 18px; display: flex; flex-direction: column; gap: 6px; }
  .mini-stat-label { font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.4px; color: var(--text-ghost); }
  .mini-stat-value { font-size: 20px; font-weight: 800; letter-spacing: -0.5px; }
  @media (max-width: 900px) { .bk-stats { grid-template-columns: repeat(2, 1fr); } }
  @media (max-width: 560px) { .bk-stats { grid-template-columns: 1fr; } }
</style>
