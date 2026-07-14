<script lang="ts">
  import { t, lang } from '$lib/i18n/index.js';
  import { CreditCard, Search, Download, AlertCircle } from 'lucide-svelte';
  import { goto } from '$app/navigation';

  interface Props {
    data: {
      bookings: any[];
      total: number;
      page: number;
      totalPages: number;
      search: string;
      error?: string;
    };
  }

  let { data }: Props = $props();

  // Local state synced from page data
  let searchInput = $state('');

  // Sync state if url changes externally
  $effect(() => {
    searchInput = data.search || '';
  });

  function applyFilters(page: number = 1) {
    const params = new URLSearchParams();
    if (searchInput.trim()) {
      params.set('search', searchInput.trim());
    }
    params.set('page', String(page));
    goto(`/dashboard/payments?${params.toString()}`, { keepFocus: true, noScroll: true });
  }

  function handleSearchSubmit(e: Event) {
    e.preventDefault();
    applyFilters(1);
  }

  function handlePageChange(newPage: number) {
    applyFilters(newPage);
  }

  function statusClass(s: string) {
    if (s === 'confirmed' || s === 'completed' || s === 'Escrow_Verified' || s === 'Booking_Active') return 'badge badge-dot badge-success';
    if (s === 'pending' || s === 'Pending_Vendor_Acceptance' || s === 'Draft_Inquiry') return 'badge badge-dot badge-warning';
    if (s === 'cancelled') return 'badge badge-dot badge-danger';
    return 'badge badge-dot badge-info';
  }

  function statusLabel(s: string) {
    const map: Record<string, string> = {
      confirmed: $lang === 'ar' ? 'ناجح' : 'Success',
      completed: $lang === 'ar' ? 'مكتمل' : 'Completed',
      cancelled: $lang === 'ar' ? 'فاشل' : 'Failed',
      pending: $lang === 'ar' ? 'معلق' : 'Pending',
      Escrow_Verified: $lang === 'ar' ? 'ضمان مؤكد' : 'Escrow Verified',
      Booking_Active: $lang === 'ar' ? 'نشط' : 'Active',
      Draft_Inquiry: $lang === 'ar' ? 'استفسار' : 'Inquiry Draft',
      Pending_Vendor_Acceptance: $lang === 'ar' ? 'بانتظار القبول' : 'Pending Accept'
    };
    return map[s] ?? s;
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

  function fmt(n: number) {
    if (n === undefined || n === null) return '0';
    return n.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  }

  // Derived aggregates for KPIs
  let totalRevenueSum = $derived(
    data.bookings.reduce((sum, b) => sum + (b.totalPrice || 0), 0)
  );

  let totalStripeFees = $derived(
    data.bookings.reduce((sum, b) => sum + ((b.totalPrice || 0) * 0.025), 0)
  );

  let successCount = $derived(
    data.bookings.filter(b => b.status === 'confirmed' || b.status === 'completed' || b.status === 'Escrow_Verified').length
  );

  let successPercentage = $derived(
    data.bookings.length > 0 
      ? Math.round((successCount / data.bookings.length) * 100)
      : 100
  );
</script>

<div class="fade-in">
  <div class="page-header">
    <div class="page-header-left">
      <h1 class="page-title">{$t('nav.payments')}</h1>
      <p class="page-subtitle">{$lang === 'ar' ? 'سجل المعاملات المالية والمدفوعات الفعلي' : 'Live financial transactions and payment ledger'}</p>
    </div>
    <div class="page-header-right">
      <button class="btn btn-outline btn-sm" disabled>
        <Download size={14} /> 
        {$lang === 'ar' ? 'تصدير' : 'Export'}
      </button>
    </div>
  </div>

  {#if data.error}
    <div class="notice-banner error">
      <AlertCircle size={18} class="notice-icon" />
      <div class="notice-text">{data.error}</div>
    </div>
  {/if}

  <div class="pay-stats">
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'إجمالي المعاملات المطابقة' : 'Matching Transactions'}</span>
      <span class="mini-stat-value text-info">{data.total}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'مبيعات الصفحة الحالية' : 'Page Gross Value'}</span>
      <span class="mini-stat-value text-gold">SAR {fmt(totalRevenueSum)}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'نسبة النجاح للصفحة' : 'Page Success Rate'}</span>
      <span class="mini-stat-value text-success">{successPercentage}%</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'رسوم الدفع المقدرة (٢.٥٪)' : 'Est. Gateway Fees (2.5%)'}</span>
      <span class="mini-stat-value text-warning">SAR {fmt(totalStripeFees)}</span>
    </div>
  </div>

  <div class="table-container">
    <div class="table-head-bar">
      <form class="toolbar" onsubmit={handleSearchSubmit} style="margin:0; flex:1; display:flex; gap:12px; align-items:center;">
        <div class="search-box" style="width:280px;">
          <Search size={15} />
          <input 
            type="search" 
            placeholder={$lang === 'ar' ? 'بحث برقم الحجز...' : 'Search by booking number...'} 
            bind:value={searchInput} 
            aria-label="Search transactions"
          />
        </div>
        <button type="submit" class="btn btn-outline btn-sm">
          {$lang === 'ar' ? 'بحث' : 'Search'}
        </button>
      </form>
      <span class="table-title">
        {data.total} {$lang === 'ar' ? 'معاملة' : 'Transactions'}
      </span>
    </div>

    <div class="table-scroll">
      <table aria-label="Payments ledger table">
        <thead>
          <tr>
            <th>{$lang === 'ar' ? 'رقم المعاملة (معرف)' : 'Transaction ID'}</th>
            <th>{$lang === 'ar' ? 'رقم الحجز' : 'Booking'}</th>
            <th style="text-align:end">{$lang === 'ar' ? 'إجمالي المبلغ' : 'Amount'}</th>
            <th style="text-align:end">{$lang === 'ar' ? 'الرسوم المقدرة (٢.٥٪)' : 'Est. Fee'}</th>
            <th>{$lang === 'ar' ? 'طريقة الدفع' : 'Method'}</th>
            <th>{$lang === 'ar' ? 'البوابة' : 'Gateway'}</th>
            <th>{$lang === 'ar' ? 'التاريخ' : 'Date'}</th>
            <th>{$t('common.status')}</th>
          </tr>
        </thead>
        <tbody>
          {#each data.bookings as txn (txn.id)}
            <tr>
              <td>
                <span class="mono text-gold" style="font-size:12px; font-weight:700">
                  TXN-{txn.bookingNumber}
                </span>
              </td>
              <td>
                <span class="mono" style="font-size:12px; color:var(--text-tertiary)">
                  {txn.bookingNumber}
                </span>
              </td>
              <td style="text-align:end; font-weight:700">
                {fmt(txn.totalPrice)} SAR
              </td>
              <td style="text-align:end; font-size:12.5px; color:var(--text-tertiary)">
                {fmt(txn.totalPrice * 0.025)} SAR
              </td>
              <td class="text-muted">
                {$lang === 'ar' ? 'بطاقة ائتمان' : 'Credit Card'}
              </td>
              <td><span class="badge badge-muted">Stripe</span></td>
              <td class="text-muted" style="font-size:12px">
                {fmtDate(txn.createdAt)}
              </td>
              <td><span class={statusClass(txn.status)}>{statusLabel(txn.status)}</span></td>
            </tr>
          {/each}
          {#if data.bookings.length === 0}
            <tr>
              <td colspan="8">
                <div class="empty-state">
                  <div class="empty-icon"><CreditCard size={28} /></div>
                  <h3>{$t('common.no_data')}</h3>
                  <p>{$lang === 'ar' ? 'لا يوجد معاملات مدفوعات مسجلة في قاعدة البيانات' : 'No platform financial transactions logged'}</p>
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

<style>
  .pay-stats { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; margin-bottom: 20px; }
  .mini-stat { padding: 16px 18px; display: flex; flex-direction: column; gap: 6px; }
  .mini-stat-label { font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.4px; color: var(--text-ghost); }
  .mini-stat-value { font-size: 20px; font-weight: 800; letter-spacing: -0.5px; }
  @media (max-width: 900px) { .pay-stats { grid-template-columns: repeat(2, 1fr); } }
  @media (max-width: 560px) { .pay-stats { grid-template-columns: 1fr; } }
</style>
