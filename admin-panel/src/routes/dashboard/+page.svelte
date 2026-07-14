<script lang="ts">
  import { t, lang } from '$lib/i18n/index.js';
  import { Sparkles } from 'lucide-svelte';
  import { AdminDashboardState } from '../../core/stores/adminState.svelte.js';
  import SystemHealthStats from '$lib/components/admin/dashboard/SystemHealthStats.svelte';
  import FinanceOverview from '$lib/components/admin/dashboard/FinanceOverview.svelte';
  import PendingAdminQueue from '$lib/components/admin/dashboard/PendingAdminQueue.svelte';
  import UserGrowthChart from '$lib/components/admin/dashboard/UserGrowthChart.svelte';

  let { data } = $props();

  const state = new AdminDashboardState(data);
</script>

<div class="dashboard">
  <!-- ─── Page Header ──────────────────────────────────────────────────────── -->
  <header class="dash-header fade-in">
    <div class="dash-header-glow" aria-hidden="true"></div>
    <div class="dash-header-content">
      <div class="dash-badge">
        <Sparkles size={12} aria-hidden="true" />
        <span>{$lang === 'ar' ? 'مركز العمليات — مباشر' : 'Operations Center — Live'}</span>
      </div>
      <h1>{$t('dash.title')}</h1>
      <p>{$t('dash.subtitle')}</p>
    </div>
    <div class="dash-header-meta">
      <div class="live-counter">
        <div class="status-dot status-live" aria-hidden="true"></div>
        <span>{$lang === 'ar' ? 'مباشر الآن' : 'Live Now'}</span>
      </div>
    </div>
  </header>

  <SystemHealthStats {state} />

  <FinanceOverview {state} />
  
  <UserGrowthChart {state} />

  <PendingAdminQueue {state} />
</div>

<style>
  .dashboard {
    display: flex;
    flex-direction: column;
    gap: 24px;
    width: 100%;
  }

  /* Header */
  .dash-header {
    position: relative;
    background: var(--glass-sm);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-xl);
    padding: 28px 32px;
    overflow: hidden;
    display: flex;
    justify-content: space-between;
    align-items: flex-end;
    gap: 16px;
  }
  .dash-header-glow {
    position: absolute; inset: 0;
    background: radial-gradient(ellipse 60% 80% at 0% 0%, var(--gold-subtle) 0%, transparent 60%);
    pointer-events: none;
  }
  .dash-header-content { position: relative; z-index: 2; }
  .dash-badge {
    display: inline-flex; align-items: center; gap: 5px;
    font-size: 10.5px; font-weight: 700; letter-spacing: 0.6px; text-transform: uppercase;
    color: var(--gold); background: var(--gold-subtle); border: 1px solid var(--gold-border);
    padding: 3px 9px; border-radius: 6px; margin-bottom: 10px;
  }
  .dash-header h1 {
    font-size: 26px; font-weight: 900; letter-spacing: -0.5px;
    background: linear-gradient(135deg, var(--text-primary) 0%, var(--text-secondary) 100%);
    -webkit-background-clip: text; background-clip: text;
    -webkit-text-fill-color: transparent;
    margin-bottom: 6px;
  }
  .dash-header p { font-size: 14px; color: var(--text-tertiary); margin: 0; }
  .dash-header-meta { position: relative; z-index: 2; }
  .live-counter {
    display: flex; align-items: center; gap: 7px;
    font-size: 12px; font-weight: 700; color: var(--success);
    background: var(--success-dim); border: 1px solid var(--success-border);
    padding: 6px 12px; border-radius: 8px;
  }
  .status-live { background: var(--success); box-shadow: 0 0 8px var(--success); }

  @media (max-width: 768px) {
    .dash-header { flex-direction: column; align-items: flex-start; }
  }
</style>
