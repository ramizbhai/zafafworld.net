<script lang="ts">
  import { t, lang } from '$lib/i18n/index.js';
  import { Globe } from 'lucide-svelte';
  import type { AdminDashboardState } from '../../../../core/stores/adminState.svelte.js';

  let { state }: { state: AdminDashboardState } = $props();
</script>

<div class="dash-row dash-row-bottom">
  <!-- Market Matrix -->
  <section class="glass-panel market-panel fade-in" aria-label="{$t('dash.market_matrix')}">
    <div class="panel-header">
      <div>
        <h2 class="panel-title">{$t('dash.market_matrix')}</h2>
        <p class="panel-sub">{$t('dash.market_matrix_sub')}</p>
      </div>
      <div class="market-live-pill">
        <Globe size={12} aria-hidden="true" />
        <span>{state.markets.length} {$lang === 'ar' ? 'سوق' : 'Markets'}</span>
      </div>
    </div>

    <div class="table-scroll">
      <table aria-label="Arab market performance data">
        <thead>
          <tr>
            <th>{$lang === 'ar' ? 'السوق' : 'Market'}</th>
            <th>{$lang === 'ar' ? 'الرمز' : 'Code'}</th>
            <th style="text-align:end">{$lang === 'ar' ? 'الموردون' : 'Vendors'}</th>
            <th style="text-align:end">{$lang === 'ar' ? 'الحصة' : 'Share'}</th>
          </tr>
        </thead>
        <tbody>
          {#each state.markets as mkt, i}
            <tr
              class="market-row"
              class:selected={state.selectedMarket === mkt.code}
              onclick={() => state.selectedMarket = state.selectedMarket === mkt.code ? null : mkt.code}
              onkeydown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.preventDefault();
                  state.selectedMarket = state.selectedMarket === mkt.code ? null : mkt.code;
                }
              }}
              role="button"
              tabindex="0"
              aria-pressed={state.selectedMarket === mkt.code}
            >
              <td>
                <div class="cell-avatar">
                  <div class="market-avatar cell-avatar-img" class:market-avatar-active={state.selectedMarket === mkt.code}>
                    {mkt.code}
                  </div>
                  <div>
                    <div class="cell-label">{$lang === 'ar' ? mkt.name_ar : mkt.name_en}</div>
                  </div>
                </div>
              </td>
              <td><span class="code-tag mono">{mkt.code}</span></td>
              <td style="text-align:end; font-weight:600;">{mkt.vendors.toLocaleString()}</td>
              <td style="text-align:end;">
                <div class="share-cell">
                  <span class="text-gold" style="font-weight:700">{mkt.share}%</span>
                  <div class="share-bar-wrap">
                    <div class="progress-track" style="width:60px">
                      <div class="progress-fill progress-gold" style="width:{mkt.share}%"></div>
                    </div>
                  </div>
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </section>

  <!-- Regional progress bars -->
  <section class="glass-panel queue-panel fade-in" style="display:flex; flex-direction:column; justify-content:center;">
    <div class="regional-stats">
      <h4 class="regional-title">{$lang === 'ar' ? 'الأداء الإقليمي' : 'Regional Performance'}</h4>
      {#each state.markets.slice(0, 3) as bar, i}
        <div class="reg-bar">
          <div class="reg-bar-info">
            <span>{$lang === 'ar' ? bar.name_ar : bar.name_en}</span>
            <span style="font-weight:700; color:var(--text-primary)">{bar.share}%</span>
          </div>
          <div class="progress-track">
            <div class="progress-fill {i === 0 ? 'progress-gold' : i === 1 ? 'progress-info' : 'progress-purple'}" style="width:{bar.share}%"></div>
          </div>
        </div>
      {/each}
    </div>
  </section>
</div>

<style>
  .dash-row-bottom { 
    display: grid;
    grid-template-columns: 3fr 2fr;
    gap: 20px;
  }
  .panel-header {
    display: flex; align-items: flex-start; justify-content: space-between;
    padding: 18px 20px 14px;
    border-bottom: 1px solid var(--glass-border);
    gap: 12px;
  }
  .panel-title { font-size: 15px; font-weight: 700; color: var(--text-primary); margin-bottom: 3px; }
  .panel-sub { font-size: 12px; color: var(--text-tertiary); margin: 0; }

  /* Market Panel */
  .market-live-pill {
    display: flex; align-items: center; gap: 5px;
    font-size: 11.5px; font-weight: 700; color: hsl(217,91%,72%);
    background: var(--info-dim); border: 1px solid var(--info-border);
    padding: 4px 10px; border-radius: 6px; flex-shrink: 0;
  }
  .market-row {
    cursor: pointer;
    transition: background 150ms;
  }
  .market-row:hover td { background: var(--bg-hover-sm) !important; }
  .market-row.selected td { background: var(--gold-subtle) !important; }
  .market-avatar {
    background: linear-gradient(135deg, var(--bg-raised), var(--bg-float));
    color: var(--text-secondary);
    font-size: 10px; font-weight: 800;
    font-family: var(--font-mono);
    border-radius: 7px;
  }
  .market-avatar-active { border-color: var(--gold-border) !important; color: var(--gold) !important; }
  .code-tag {
    font-size: 11px; font-weight: 700;
    background: var(--bg-float); border: 1px solid var(--glass-border);
    padding: 2px 6px; border-radius: 4px; color: var(--text-tertiary);
  }
  .share-cell { display: flex; flex-direction: column; align-items: flex-end; gap: 4px; }
  .share-bar-wrap { display: flex; justify-content: flex-end; }

  /* Regional stats */
  .regional-stats {
    padding: 16px 20px;
  }
  .regional-title { font-size: 10.5px; font-weight: 700; color: var(--text-ghost); text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 12px; }
  .reg-bar { margin-bottom: 10px; }
  .reg-bar-info { display: flex; justify-content: space-between; font-size: 12px; color: var(--text-tertiary); margin-bottom: 5px; }

  @media (max-width: 1100px) {
    .dash-row-bottom { grid-template-columns: 1fr; }
  }
</style>
