<script lang="ts">
  import { t, lang } from '$lib/i18n/index.js';
  import { Percent, Search, AlertCircle } from 'lucide-svelte';

  interface Props {
    data: {
      commissions: any[];
      summary: { total_commissions: number, paid_commissions: number, pending_commissions: number };
      settings?: any;
      error?: string;
    };
  }

  let { data }: Props = $props();

  // Local state
  let searchInput = $state('');

  // Filter commissions based on search
  let filteredCommissions = $derived(
    data.commissions.filter(v => {
      const q = searchInput.toLowerCase().trim();
      if (!q) return true;
      return (v.vendorNameEn && v.vendorNameEn.toLowerCase().includes(q)) || 
             (v.vendorNameAr && v.vendorNameAr.includes(q));
    })
  );

  // Overall aggregates for KPIs
  let totalCommissionsSum = $derived(data.summary?.total_commissions ?? 0);
  let paidCommissionsSum = $derived(data.summary?.paid_commissions ?? 0);
  let pendingCommissionsSum = $derived(data.summary?.pending_commissions ?? 0);

  function fmt(n: number) {
    if (n === undefined || n === null) return '0';
    return n.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
  }

  function statusCls(s: string) {
    if (s === 'paid') return 'badge badge-dot badge-success';
    if (s === 'pending') return 'badge badge-dot badge-warning';
    return 'badge badge-dot badge-info';
  }

  function statusLbl(s: string) {
    const m: Record<string, string> = {
      paid: $lang === 'ar' ? 'مدفوع' : 'Paid',
      pending: $lang === 'ar' ? 'معلق' : 'Pending',
      processing: $lang === 'ar' ? 'قيد التحويل' : 'Processing'
    };
    return m[s] ?? s;
  }
</script>

<div class="fade-in">
  <div class="page-header">
    <div class="page-header-left">
      <h1 class="page-title">{$t('nav.commissions')}</h1>
      <p class="page-subtitle">{$lang === 'ar' ? 'تتبع وإدارة عمولات الموردين الفعلي' : 'Track and manage dynamic vendor commission payouts'}</p>
    </div>
  </div>

  {#if data.error}
    <div class="notice-banner error">
      <AlertCircle size={18} class="notice-icon" />
      <div class="notice-text">{data.error}</div>
    </div>
  {/if}

  <div class="comm-stats">
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'إجمالي العمولات المقدرة' : 'Total Commissions'}</span>
      <span class="mini-stat-value text-gold">SAR {fmt(totalCommissionsSum)}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'المدفوعة (بدون حجوزات معلقة)' : 'Paid (Zero Pending Bookings)'}</span>
      <span class="mini-stat-value text-success">SAR {fmt(paidCommissionsSum)}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'مستحقات معلقة (بانتظار قبول المورد)' : 'Pending Payouts'}</span>
      <span class="mini-stat-value" style="color:var(--warning)">SAR {fmt(pendingCommissionsSum)}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'نسبة العمولة الثابتة' : 'Commissions Rate'}</span>
      <span class="mini-stat-value text-info">{(data as any).settings?.platform_commission_rate ?? '10.0'}%</span>
    </div>
  </div>

  <div class="table-container">
    <div class="table-head-bar" style="display:flex; justify-content:space-between; align-items:center;">
      <div class="search-box" style="width:280px; margin: 0;">
        <Search size={15} />
        <input 
          type="search" 
          placeholder={$lang === 'ar' ? 'بحث باسم المورد...' : 'Search by vendor name...'} 
          bind:value={searchInput} 
          aria-label="Search commissions"
        />
      </div>
      <span class="table-title">
        {$lang === 'ar' ? 'عمولات الموردين المباشرة' : 'Vendor Direct Commissions'} ({filteredCommissions.length})
      </span>
    </div>

    <div class="table-scroll">
      <table aria-label="Commissions table">
        <thead>
          <tr>
            <th>{$lang === 'ar' ? 'المورد' : 'Vendor'}</th>
            <th>{$lang === 'ar' ? 'آخر خدمة / منتج' : 'Last Product'}</th>
            <th style="text-align:end">{$lang === 'ar' ? 'النسبة الثابتة' : 'Rate'}</th>
            <th style="text-align:end">{$lang === 'ar' ? 'عدد الحجوزات' : 'Bookings'}</th>
            <th style="text-align:end">{$lang === 'ar' ? 'إجمالي المبيعات' : 'Gross Sales'}</th>
            <th style="text-align:end">{$lang === 'ar' ? 'تقدير العمولة' : 'Est. Commission'}</th>
            <th>{$t('common.status')}</th>
          </tr>
        </thead>
        <tbody>
          {#each filteredCommissions as c}
            <tr>
              <td style="font-weight:600">
                {$lang === 'ar' ? c.vendorNameAr : c.vendorNameEn}
              </td>
              <td>
                <span class="badge badge-muted">
                  {$lang === 'ar' ? c.productNameAr : c.productNameEn}
                </span>
              </td>
              <td style="text-align:end; font-weight:700; color:var(--gold)">
                {c.commission_rate}%
              </td>
              <td style="text-align:end">{c.bookings_count}</td>
              <td style="text-align:end; font-weight:600">
                SAR {fmt(c.total_revenue)}
              </td>
              <td style="text-align:end; font-weight:800; color:var(--success)">
                SAR {fmt(c.commission_earned)}
              </td>
              <td><span class={statusCls(c.payout_status)}>{statusLbl(c.payout_status)}</span></td>
            </tr>
          {/each}
          {#if filteredCommissions.length === 0}
            <tr>
              <td colspan="7">
                <div class="empty-state">
                  <div class="empty-icon"><Percent size={28} /></div>
                  <h3>{$t('common.no_data')}</h3>
                  <p>{$lang === 'ar' ? 'لا يوجد عمولات موردين مسجلة' : 'No vendor commission payouts logged'}</p>
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
  .comm-stats { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; margin-bottom: 20px; }
  .mini-stat { padding: 16px 18px; display: flex; flex-direction: column; gap: 6px; }
  .mini-stat-label { font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.4px; color: var(--text-ghost); }
  .mini-stat-value { font-size: 20px; font-weight: 800; letter-spacing: -0.4px; }
  @media (max-width: 900px) { .comm-stats { grid-template-columns: repeat(2, 1fr); } }
  @media (max-width: 560px) { .comm-stats { grid-template-columns: 1fr; } }
</style>
