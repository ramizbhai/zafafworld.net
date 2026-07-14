<script lang="ts">
  import { t, lang } from '$lib/i18n/index.js';
  import { TrendingUp, Users, Building2, BarChart3, Globe, Calendar } from 'lucide-svelte';

  interface Props {
    data: {
      summary?: {
        total_revenue: number;
        total_bookings_count: number;
        active_vendors_count: number;
        total_users_count: number;
        regional_distribution: Array<{
          country_id: string;
          country_name_en: string;
          country_name_ar: string;
          active_vendors_count: number;
        }>;
        category_distribution: Array<{
          category: string;
          category_count: number;
          percentage: number;
        }>;
        monthly_revenue: Array<{
          month_ar: string;
          month_en: string;
          revenue: number;
        }>;
        bookings_trend: number[];
        users_trend: number[];
        vendors_trend: number[];
      } | null;
    };
  }

  let { data }: Props = $props();
  let summary = $derived(data.summary);

  // Dynamic spark datasets based on actual summary telemetry data
  let revenueTrend = $derived(
    (summary?.monthly_revenue && summary.monthly_revenue.length > 0)
      ? summary.monthly_revenue.map(m => m.revenue)
      : [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
  );
  let maxRevenueTrend = $derived(Math.max(...revenueTrend, 1));

  let bookingsTrend = $derived(summary?.bookings_trend || [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
  let maxBookingsTrend = $derived(Math.max(...bookingsTrend, 1));

  let usersTrend = $derived(summary?.users_trend || [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
  let maxUsersTrend = $derived(Math.max(...usersTrend, 1));

  let vendorsTrend = $derived(summary?.vendors_trend || [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
  let maxVendorsTrend = $derived(Math.max(...vendorsTrend, 1));

  function sparkPath(data: number[], max: number, w: number, h: number): string {
    if (data.length <= 1) return `M 0,${h} L ${w},${h}`;
    const pts = data.map((v, i) => `${(i / (data.length - 1)) * w},${h - (v / max) * h}`);
    return `M ${pts.join(' L ')}`;
  }

  // Dynamic distribution table replacing the static mock topVendors list
  let categories = $derived(summary?.category_distribution || []);

  function fmt(n: number) { return n.toLocaleString(); }
</script>

<div class="fade-in">
  <div class="page-header">
    <div class="page-header-left">
      <h1 class="page-title">{$t('nav.analytics')}</h1>
      <p class="page-subtitle">{$lang === 'ar' ? 'تحليلات متعمقة لأداء المنصة' : 'In-depth platform performance analytics'}</p>
    </div>
    <div class="page-header-right">
      <form data-sveltekit-noscroll data-sveltekit-keepfocus>
        <select class="form-select" name="range" style="height:36px; width:160px;" onchange={(e) => e.currentTarget.form?.submit()}>
          <option value="last_12_months" selected>{$lang === 'ar' ? 'آخر ١٢ شهراً' : 'Last 12 months'}</option>
          <option value="this_month">{$lang === 'ar' ? 'هذا الشهر' : 'This month'}</option>
        </select>
      </form>
    </div>
  </div>

  <div class="spark-grid">
    {#each [
      { label_ar: 'إجمالي المستخدمين', label_en: 'Total Users', value: fmt(summary?.total_users_count ?? 0), trend: 'نشط', trend_en: 'Active', color: 'var(--info)', data: usersTrend, max: maxUsersTrend, id: 'sparkBlue' },
      { label_ar: 'إجمالي الحجوزات', label_en: 'Total Bookings', value: fmt(summary?.total_bookings_count ?? 0), trend: 'مكتمل', trend_en: 'Processed', color: 'var(--success)', data: bookingsTrend, max: maxBookingsTrend, id: 'sparkGreen' },
      { label_ar: 'إجمالي الإيرادات', label_en: 'Total Revenue', value: `${fmt(summary?.total_revenue ?? 0)} SAR`, trend: 'إجمالي', trend_en: 'Gross', color: 'var(--gold)', data: revenueTrend, max: maxRevenueTrend, id: 'sparkGold' },
      { label_ar: 'الموردون النشطون', label_en: 'Active Vendors', value: fmt(summary?.active_vendors_count ?? 0), trend: 'معتمد', trend_en: 'Approved', color: 'var(--purple)', data: vendorsTrend, max: maxVendorsTrend, id: 'sparkPurple' },
    ] as s, i}
      <div class="card spark-card">
        <div class="spark-info">
          <span class="spark-label">{$lang === 'ar' ? s.label_ar : s.label_en}</span>
          <span class="spark-value">{s.value}</span>
          <span class="stat-trend stat-trend-up">{$lang === 'ar' ? s.trend : (s.trend_en ?? s.trend)}</span>
        </div>
        <svg viewBox="0 0 160 50" class="sparkline-svg" role="img" aria-label={$lang === 'ar' ? s.label_ar : s.label_en}>
          <defs>
            <linearGradient id={s.id} x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color={s.color} />
              <stop offset="100%" stop-color="transparent" />
            </linearGradient>
          </defs>
          <path d={sparkPath(s.data, s.max, 160, 50)} fill="none" stroke={s.color} stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
          <path d={`${sparkPath(s.data, s.max, 160, 50)} L 160,50 L 0,50 Z`} fill={`url(#${s.id})`} opacity="0.25" />
        </svg>
      </div>
    {/each}
  </div>

  <div class="table-container">
    <div class="table-head-bar">
      <span class="table-title">{$lang === 'ar' ? 'تحليل فئات الموردين' : 'Supplier Category Share Analysis'}</span>
    </div>
    <div class="table-scroll">
      <table>
        <thead>
          <tr>
            <th>#</th>
            <th>{$lang === 'ar' ? 'الفئة' : 'Category'}</th>
            <th style="text-align:end">{$lang === 'ar' ? 'عدد الموردين' : 'Vendor Count'}</th>
            <th style="text-align:end">{$lang === 'ar' ? 'الحصة السوقية' : 'Market Share'}</th>
            <th>{$lang === 'ar' ? 'التوزيع النسبي' : 'Relative Distribution'}</th>
          </tr>
        </thead>
        <tbody>
          {#if categories.length === 0}
            <tr>
              <td colspan="5" style="text-align:center; color:var(--text-ghost)">
                {$lang === 'ar' ? 'لا توجد بيانات متاحة' : 'No data available'}
              </td>
            </tr>
          {:else}
            {#each categories as cat, i}
              <tr>
                <td style="font-size:13px; font-weight:800; color:var(--text-ghost)">{i + 1}</td>
                <td style="font-weight:600">{cat.category}</td>
                <td style="text-align:end; font-weight:700">{fmt(cat.category_count)}</td>
                <td style="text-align:end; color:var(--gold); font-weight:700">{cat.percentage}%</td>
                <td style="width:180px;">
                  <div class="progress-track">
                    <div class="progress-fill progress-gold" style="width:{cat.percentage}%"></div>
                  </div>
                </td>
              </tr>
            {/each}
          {/if}
        </tbody>
      </table>
    </div>
  </div>
</div>

<style>
  .spark-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 14px; margin-bottom: 20px; }
  .spark-card { padding: 16px 18px; display: flex; flex-direction: column; gap: 10px; }
  .spark-info { display: flex; flex-direction: column; gap: 5px; }
  .spark-label { font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-ghost); }
  .spark-value { font-size: 22px; font-weight: 800; letter-spacing: -0.4px; color: var(--text-primary); }
  .sparkline-svg { width: 100%; height: 50px; overflow: visible; }
  @media (max-width: 1100px) { .spark-grid { grid-template-columns: repeat(2, 1fr); } }
  @media (max-width: 600px) { .spark-grid { grid-template-columns: 1fr; } }
</style>
