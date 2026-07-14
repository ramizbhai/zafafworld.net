<script lang="ts">
  import { t, lang } from '$lib/i18n/index.js';
  import { ArrowUpRight, ArrowDownRight } from 'lucide-svelte';
  import type { AdminDashboardState } from '../../../../core/stores/adminState.svelte.js';
  import { RBACService } from '../../../../core/auth/rbac.service.js';
  import { page } from '$app/stores';

  let { state }: { state: AdminDashboardState } = $props();

  let canViewSystemHealth = $derived(RBACService.canViewSystemHealth($page.data.user));
</script>

{#if canViewSystemHealth}
<section class="kpi-grid" aria-label="Key performance indicators">
  {#each state.kpis as kpi, i}
    <div class="stat-card kpi-card fade-in" style="animation-delay: {i * 60}ms">
      <div class="stat-card-edge" style="background: {kpi.edgeColor}"></div>
      <div class="kpi-top">
        <span class="stat-label">{$t(kpi.key)}</span>
        <div class="stat-icon {kpi.iconClass}">
          <!-- Svelte 5 dynamic component rendering -->
          <kpi.icon size={19} aria-hidden="true" />
        </div>
      </div>
      <div class="kpi-value stat-value">
        {kpi.value_ar ? ($lang === 'ar' ? kpi.value_ar : kpi.value_en) : '0'}
      </div>
      <div class="kpi-footer">
        {#if kpi.trendDir === 'up'}
          <span class="stat-trend stat-trend-up">
            <ArrowUpRight size={12} aria-hidden="true" />
            {kpi.trend}
          </span>
        {:else if kpi.trendDir === 'down'}
          <span class="stat-trend stat-trend-down">
            <ArrowDownRight size={12} aria-hidden="true" />
            {kpi.trend}
          </span>
        {:else}
          <span class="stat-trend" style="background: var(--warning-dim); color: var(--warning);">
            ⚡ {$lang === 'ar' ? kpi.trend : (kpi.trend_en ?? kpi.trend)}
          </span>
        {/if}
        <span class="kpi-sub">
          {$lang === 'ar' ? kpi.sub_ar : kpi.sub_en}
        </span>
      </div>
    </div>
  {/each}
</section>
{/if}

<style>
  .kpi-grid {
    display: grid;
    grid-template-columns: repeat(6, 1fr);
    gap: 14px;
  }
  .kpi-card { cursor: default; }
  .kpi-top {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 8px;
  }
  .kpi-value { margin: 8px 0 6px; }
  .kpi-footer {
    display: flex; align-items: center; gap: 8px; flex-wrap: wrap;
  }
  .kpi-sub { font-size: 11px; color: var(--text-ghost); }

  @media (max-width: 1600px) {
    .kpi-grid { grid-template-columns: repeat(3, 1fr); }
  }
  @media (max-width: 768px) {
    .kpi-grid { grid-template-columns: repeat(2, 1fr); }
  }
  @media (max-width: 480px) {
    .kpi-grid { grid-template-columns: 1fr; }
  }
</style>
