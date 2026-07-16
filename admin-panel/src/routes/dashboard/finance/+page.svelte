<script lang="ts">
  import { t, lang } from '$lib/i18n/index.js';
  import { BarChart3, TrendingUp, TrendingDown, DollarSign, Download, AlertCircle } from 'lucide-svelte';

  interface Props {
    data: {
      summary: {
        total_revenue: number;
        commission_estimate: number;
        payout_totals: number;
        total_bookings_count: number;
        monthly_revenue: any[];
        category_distribution: any[];
      } | null;
      error?: string;
    };
  }

  let { data }: Props = $props();

  const colors = ['var(--gold)', 'var(--info)', 'var(--success)', 'var(--purple)', 'var(--warning)', 'var(--text-ghost)'];

  function fmt(n: number) {
    if (n === undefined || n === null) return '0';
    return n.toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 2 });
  }

  function getCategoryLabel(category: string | null | undefined): string {
    if (!category) return $lang === 'ar' ? 'غير مصنف' : 'Uncategorized';
    const mapping: Record<string, string> = {
      // V2 Canonical slugs
      'wedding-palace':    $lang === 'ar' ? 'قصور الأفراح'   : 'Wedding Palace',
      'hotel-venue':       $lang === 'ar' ? 'فنادق وقاعات'   : 'Hotel Ballroom',
      'villa-resort':      $lang === 'ar' ? 'استراحات وفلل'  : 'Villa & Resort',
      'restaurant-event':  $lang === 'ar' ? 'مطاعم وقاعات'   : 'Restaurant & Dining',
      'outdoor-garden':    $lang === 'ar' ? 'حدائق مفتوحة'   : 'Outdoor Garden',
      'rooftop-venue':     $lang === 'ar' ? 'أسطح خارجية'    : 'Rooftop Venue',
      'private-beach':     $lang === 'ar' ? 'شواطئ خاصة'     : 'Private Beach',
      'chalet':            $lang === 'ar' ? 'شاليهات'        : 'Chalet',
      'wedding-gown':      $lang === 'ar' ? 'فساتين زفاف'    : 'Wedding Gown',
      'haute-couture':     $lang === 'ar' ? 'أزياء راقية'    : 'Haute Couture',
      'abaya-jalabiya':    $lang === 'ar' ? 'عباءات وجلابيات' : 'Abaya & Jalabiya',
      'groom-attire':      $lang === 'ar' ? 'ملابس العريس'   : 'Groom Attire',
      'hair-makeup':       $lang === 'ar' ? 'شعر ومكياج'     : 'Hair & Makeup',
      'beauty-skincare':   $lang === 'ar' ? 'تجميل وعناية'   : 'Beauty & Skincare',
      'henna-art':         $lang === 'ar' ? 'فن الحناء'      : 'Henna Art',
      'male-grooming':     $lang === 'ar' ? 'حلاقة رجالية'   : 'Male Grooming',
      'photography-video': $lang === 'ar' ? 'تصوير وفيديو'   : 'Photography & Video',
      'photo-studio':      $lang === 'ar' ? 'استوديو تصوير'  : 'Photo Studio',
      'catering':          $lang === 'ar' ? 'ضيافة وطعام'    : 'Catering',
      'wedding-cake':      $lang === 'ar' ? 'كيك الزفاف'     : 'Wedding Cake',
      'wedding-sweets':    $lang === 'ar' ? 'حلويات عربية'   : 'Arabic Sweets',
      'entertainment-dj':  $lang === 'ar' ? 'دي جي وترفيه'   : 'DJ & Entertainment',
      'zaffa':             $lang === 'ar' ? 'زفة وموكب'      : 'Zaffa & Procession',
      'nasheed-band':      $lang === 'ar' ? 'أناشيد وفرقة'   : 'Nasheed & Live Band',
      'wedding-jewelry':   $lang === 'ar' ? 'مجوهرات عروس'   : 'Bridal Jewelry',
      'wedding-gifts':     $lang === 'ar' ? 'هدايا وتوزيعات' : 'Wedding Gifts',
      'wedding-planner':   $lang === 'ar' ? 'منظم حفلات'     : 'Wedding Planner',
      'khosha-decor':      $lang === 'ar' ? 'كوشة وديكور'    : 'Khosha & Decor',
      'flowers-floral':    $lang === 'ar' ? 'ورد وزهور'      : 'Flowers & Floral',
      'wedding-invitation':$lang === 'ar' ? 'دعوات زفاف'     : 'Wedding Invitations',
      'lighting-av':       $lang === 'ar' ? 'إضاءة وصوتيات'  : 'Lighting & AV',
      'wedding-car':       $lang === 'ar' ? 'سيارة زفاف'     : 'Wedding Car',
      // Legacy aliases
      'photographers-and-videographers': $lang === 'ar' ? 'تصوير' : 'Photography',
      'wedding-planning':                $lang === 'ar' ? 'تنظيم حفلات' : 'Wedding Planning',
      'uncategorized':                   $lang === 'ar' ? 'غير مصنف' : 'Uncategorized',
    };
    return mapping[category] || category.split('-').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ');
  }

</script>

<div class="fade-in">
  <div class="page-header">
    <div class="page-header-left">
      <h1 class="page-title">{$t('nav.finance')}</h1>
      <p class="page-subtitle">{$lang === 'ar' ? 'نظرة شاملة على الأداء المالي للمنصة' : 'Comprehensive platform financial performance overview'}</p>
    </div>
    <button class="btn btn-outline btn-sm" disabled>
      <Download size={14} /> 
      {$lang === 'ar' ? 'تقرير مالي' : 'Financial Report'}
    </button>
  </div>

  {#if data.error}
    <div class="notice-banner error">
      <AlertCircle size={18} class="notice-icon" />
      <div class="notice-text">{data.error}</div>
    </div>
  {/if}

  <!-- KPI Row -->
  <div class="fin-kpis">
    <div class="fin-kpi-card card">
      <span class="fin-kpi-label">{$lang === 'ar' ? 'إجمالي المبيعات الإجمالية' : 'Gross Booking Value'}</span>
      <span class="fin-kpi-value" style="color: var(--gold)">
        SAR {fmt(data.summary?.total_revenue || 0)}
      </span>
    </div>
    <div class="fin-kpi-card card">
      <span class="fin-kpi-label">{$lang === 'ar' ? 'تقدير العمولات (' + ((data.summary as any)?.commission_rate || 10.0) + '٪)' : 'Commissions Estimate (' + ((data.summary as any)?.commission_rate || 10.0) + '%)'}</span>
      <span class="fin-kpi-value" style="color: var(--success)">
        SAR {fmt(data.summary?.commission_estimate || 0)}
      </span>
    </div>
    <div class="fin-kpi-card card">
      <span class="fin-kpi-label">{$lang === 'ar' ? 'تقدير مستحقات الموردين' : 'Estimated Vendor Payouts'}</span>
      <span class="fin-kpi-value" style="color: var(--info)">
        SAR {fmt(data.summary?.payout_totals || 0)}
      </span>
    </div>
    <div class="fin-kpi-card card">
      <span class="fin-kpi-label">{$lang === 'ar' ? 'إجمالي الحجوزات المعالجة' : 'Processed Bookings'}</span>
      <span class="fin-kpi-value" style="color: var(--purple)">
        {data.summary?.total_bookings_count || 0}
      </span>
    </div>
  </div>

  <div class="fin-grid">
    <!-- Monthly table -->
    <div class="table-container fin-table">
      <div class="table-head-bar">
        <span class="table-title">{$lang === 'ar' ? 'الإيرادات الشهرية (آخر ١٢ شهر)' : 'Monthly Revenue (12 Months)'}</span>
      </div>
      <div class="table-scroll">
        <table>
          <thead>
            <tr>
              <th>{$lang === 'ar' ? 'الشهر' : 'Month'}</th>
              <th style="text-align:end">{$lang === 'ar' ? 'الإيرادات الإجمالية' : 'Gross Revenue'}</th>
              <th style="text-align:end">{$lang === 'ar' ? 'تقدير العمولة' : 'Estimated Commission'}</th>
              <th style="text-align:end">{$lang === 'ar' ? 'عدد الحجوزات' : 'Bookings'}</th>
              <th style="text-align:end">{$lang === 'ar' ? 'النمو' : 'Growth'}</th>
            </tr>
          </thead>
          <tbody>
            {#each data.summary?.monthly_revenue || [] as row}
              <tr>
                <td style="font-weight:600">
                  {$lang === 'ar' ? row.month_ar : row.month_en}
                </td>
                <td style="text-align:end; font-weight:700; color:var(--gold)">
                  SAR {fmt(row.revenue)}
                </td>
                <td style="text-align:end; color:var(--info); font-weight:600">
                  SAR {fmt(row.commission)}
                </td>
                <td style="text-align:end">{row.bookings || 0}</td>
                <td style="text-align:end">
                  <span class="stat-trend" class:stat-trend-up={row.growth >= 0} class:stat-trend-down={row.growth < 0}>
                    {row.growth >= 0 ? '+' : ''}{row.growth}%
                  </span>
                </td>
              </tr>
            {/each}
            {#if !data.summary || data.summary.monthly_revenue.length === 0}
              <tr>
                <td colspan="5">
                  <div class="empty-state">
                    <div class="empty-icon"><BarChart3 size={24} /></div>
                    <h3>{$t('common.no_data')}</h3>
                    <p>{$lang === 'ar' ? 'لا يوجد حركات إيرادات مسجلة' : 'No platform monthly revenue recorded yet'}</p>
                  </div>
                </td>
              </tr>
            {/if}
          </tbody>
        </table>
      </div>
    </div>

    <!-- Category breakdown -->
    <div class="card fin-cat-card">
      <div class="card-header">
        <span class="table-title">{$lang === 'ar' ? 'الموردون النشطون حسب الفئة' : 'Active Vendors by Category'}</span>
      </div>
      <div class="card-body">
        {#each data.summary?.category_distribution || [] as cat, index}
          <div class="cat-bar-item">
            <div class="cat-bar-info">
              <span style="font-size:13px; font-weight:600; color:var(--text-primary)">
                {getCategoryLabel(cat.category)}
              </span>
              <span style="font-weight:700; color:{colors[index % colors.length]}">
                {cat.percentage}% ({cat.category_count})
              </span>
            </div>
            <div class="progress-track" style="margin-top:5px;">
              <div class="progress-fill" style="width:{cat.percentage}%; background:{colors[index % colors.length]};"></div>
            </div>
          </div>
        {/each}
        {#if !data.summary || data.summary.category_distribution.length === 0}
          <div class="empty-state" style="padding: 24px 0;">
            <p>{$lang === 'ar' ? 'لا يوجد فئات مسجلة' : 'No platform active categories mapped yet'}</p>
          </div>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  .fin-kpis { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; margin-bottom: 20px; }
  .fin-kpi-card { padding: 18px 20px; display: flex; flex-direction: column; gap: 8px; }
  .fin-kpi-label { font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-ghost); }
  .fin-kpi-value { font-size: 20px; font-weight: 800; letter-spacing: -0.4px; }
  .fin-grid { display: grid; grid-template-columns: 3fr 2fr; gap: 16px; }
  .cat-bar-item { margin-bottom: 14px; }
  .cat-bar-item:last-child { margin-bottom: 0; }
  .cat-bar-info { display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px; }
  @media (max-width: 1100px) { .fin-kpis { grid-template-columns: repeat(2, 1fr); } .fin-grid { grid-template-columns: 1fr; } }
</style>
