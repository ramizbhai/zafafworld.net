<script lang="ts">
  import { t, lang } from '$lib/i18n/index.js';
  import type { AdminDashboardState } from '../../../../core/stores/adminState.svelte.js';
  import { RBACService } from '../../../../core/auth/rbac.service.js';
  import { page } from '$app/stores';

  let { state }: { state: AdminDashboardState } = $props();

  let canViewFinancials = $derived(RBACService.canViewFinancials($page.data.user));
  const chartH = 120;

  function barHeight(v: number): number {
    return Math.round((v / state.maxRevenue) * chartH);
  }
</script>

{#if canViewFinancials}
<section class="glass-panel chart-panel fade-in" aria-label="{$t('dash.revenue_trend')}">
  <div class="panel-header">
    <div>
      <h2 class="panel-title">{$t('dash.revenue_trend')}</h2>
      <p class="panel-sub">{$lang === 'ar' ? 'إيرادات المنصة خلال ١٢ شهراً (ريال سعودي)' : 'Platform revenue over 12 months (SAR)'}</p>
    </div>
    <div class="chart-legend">
      <span class="legend-dot" style="background: var(--gold)"></span>
      <span class="legend-label">{$lang === 'ar' ? 'الإيرادات' : 'Revenue'}</span>
    </div>
  </div>

  <!-- SVG Bar Chart -->
  <div class="chart-wrap">
    <svg
      viewBox="0 0 780 160"
      preserveAspectRatio="none"
      role="img"
      aria-label="Revenue bar chart"
      class="revenue-svg"
    >
      <!-- Grid lines -->
      {#each [0.25, 0.5, 0.75, 1] as frac}
        <line
          x1="0" y1={chartH - (frac * chartH) + 20}
          x2="780" y2={chartH - (frac * chartH) + 20}
          stroke="rgba(0,0,0,0.05)" stroke-width="1"
        />
      {/each}

      <!-- Bars -->
      {#each state.revenueData as val, i}
        {@const bw = 44}
        {@const gap = 21}
        {@const x = i * (bw + gap) + 4}
        {@const h = barHeight(val)}
        {@const y = chartH - h + 20}
        {@const isHov = state.chartHovered === i}
        <g
          role="presentation"
          onmouseenter={() => state.chartHovered = i}
          onmouseleave={() => state.chartHovered = null}
          class="chart-bar-group"
        >
          <!-- Background bar -->
          <rect x={x} y={20} width={bw} height={chartH} rx="6" fill="rgba(91,33,182,0.015)" />
          <!-- Value bar -->
          <rect
            x={x} y={y} width={bw} height={h} rx="6"
            fill={isHov ? 'var(--gold-bright)' : 'var(--gold)'}
            opacity={isHov ? 1 : 0.85}
            style="transition: all 200ms ease;"
          />
          <!-- Glow on hover -->
          {#if isHov}
            <rect x={x} y={y} width={bw} height={h} rx="6"
              fill="url(#barGlow)" opacity="0.4" />
            <!-- Tooltip box -->
            <rect x={x - 10} y={y - 28} width={68} height={22} rx="5"
              fill="var(--bg-base)" stroke="var(--gold-border)" stroke-width="1" />
            <text x={x + bw/2} y={y - 13} fill="var(--gold-bright)"
              font-size="9" font-weight="700" text-anchor="middle">
              {val >= 1000 ? (val/1000).toFixed(0) + 'K' : val} SAR
            </text>
          {/if}
        </g>
      {/each}

      <defs>
        <radialGradient id="barGlow">
          <stop offset="0%" stop-color="var(--gold)" />
          <stop offset="100%" stop-color="transparent" />
        </radialGradient>
      </defs>
    </svg>

    <!-- Month labels -->
    <div class="chart-labels">
      {#each (($lang === 'ar') ? state.revenueMonths : state.revenueMonthsEn) as month}
        <span class="chart-label">{month}</span>
      {/each}
    </div>
  </div>

  <!-- Summary row -->
  <div class="chart-summary">
    <div class="chart-sum-item">
      <span class="chart-sum-label">{$lang === 'ar' ? 'أعلى شهر' : 'Peak Month'}</span>
      <span class="chart-sum-val text-gold">
        {#if state.revenueData.length > 0 && state.peakMonthIndex !== -1}
          {($lang === 'ar' ? state.revenueMonths[state.peakMonthIndex] : state.revenueMonthsEn[state.peakMonthIndex]) || 'N/A'} — {Math.max(...state.revenueData).toLocaleString()}
        {:else}
          N/A
        {/if}
      </span>
    </div>
    <div class="chart-sum-item">
      <span class="chart-sum-label">{$lang === 'ar' ? 'المتوسط الشهري' : 'Monthly Avg'}</span>
      <span class="chart-sum-val">
        {state.revenueAverage >= 1000 ? Math.round(state.revenueAverage/1000) + 'K' : Math.round(state.revenueAverage)} SAR
      </span>
    </div>
    <div class="chart-sum-item">
      <span class="chart-sum-label">{$lang === 'ar' ? 'النمو السنوي' : 'YoY Growth'}</span>
      <span class="chart-sum-val text-success">{state.yoyGrowthText}</span>
    </div>
  </div>
</section>
{/if}

<style>
  .panel-header {
    display: flex; align-items: flex-start; justify-content: space-between;
    padding: 18px 20px 14px;
    border-bottom: 1px solid var(--glass-border);
    gap: 12px;
  }
  .panel-title { font-size: 15px; font-weight: 700; color: var(--text-primary); margin-bottom: 3px; }
  .panel-sub { font-size: 12px; color: var(--text-tertiary); margin: 0; }
  .chart-legend { display: flex; align-items: center; gap: 6px; flex-shrink: 0; }
  .legend-dot { width: 10px; height: 10px; border-radius: 3px; }
  .legend-label { font-size: 12px; color: var(--text-tertiary); font-weight: 600; }

  .chart-wrap { padding: 16px 20px 8px; }
  .revenue-svg { width: 100%; height: 160px; overflow: visible; }
  :global(.chart-bar-group) { cursor: pointer; }

  .chart-labels {
    display: flex;
    justify-content: space-between;
    padding: 0 4px;
    margin-top: 6px;
  }
  .chart-label { font-size: 10px; color: var(--text-ghost); font-weight: 600; }

  .chart-summary {
    display: grid; grid-template-columns: repeat(3, 1fr);
    gap: 12px; padding: 14px 20px;
    border-top: 1px solid var(--glass-border);
  }
  .chart-sum-item { display: flex; flex-direction: column; gap: 3px; }
  .chart-sum-label { font-size: 10.5px; color: var(--text-ghost); font-weight: 600; text-transform: uppercase; letter-spacing: 0.4px; }
  .chart-sum-val { font-size: 14px; font-weight: 700; color: var(--text-primary); }

  @media (max-width: 768px) {
    .chart-summary { grid-template-columns: 1fr; }
  }
</style>
