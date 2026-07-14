<script lang="ts">
  import { t, lang } from '$lib/i18n/index.js';
  import { ChevronRight, Activity } from 'lucide-svelte';
  import type { AdminDashboardState } from '../../../../core/stores/adminState.svelte.js';

  let { state }: { state: AdminDashboardState } = $props();
</script>

<div class="dash-row">
  <!-- Activity Feed -->
  <section class="glass-panel activity-panel fade-in" aria-label="{$t('dash.recent_activity')}">
    <div class="panel-header">
      <div>
        <h2 class="panel-title">{$t('dash.recent_activity')}</h2>
        <p class="panel-sub">{$lang === 'ar' ? 'آخر الأنشطة على المنصة' : 'Latest platform activity'}</p>
      </div>
      <div class="status-dot status-live" aria-hidden="true"></div>
    </div>

    <div class="activity-list">
      {#if state.activities.length === 0}
        <div class="activity-empty">
          <Activity size={20} aria-hidden="true" />
          <span>{$lang === 'ar' ? 'لا توجد أنشطة حديثة' : 'No recent activity'}</span>
        </div>
      {:else}
        {#each state.activities as act, i}
          <div class="activity-item fade-in" style="animation-delay: {i * 40}ms">
            <div class="act-icon {act.iconClass}">
              <!-- Svelte 5 dynamic component rendering -->
              <act.icon size={14} aria-hidden="true" />
            </div>
            <div class="act-content">
              <p class="act-msg">{$lang === 'ar' ? act.msg_ar : act.msg_en}</p>
              <span class="act-time">{act.time} {$lang === 'ar' ? 'منذ' : 'ago'}</span>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </section>

  <!-- Moderation Queue -->
  <section class="glass-panel queue-panel fade-in" aria-label="{$t('dash.moderation_queue')}">
    <div class="panel-header">
      <div>
        <h2 class="panel-title">{$t('dash.moderation_queue')}</h2>
        <p class="panel-sub">{$t('dash.moderation_queue_sub')}</p>
      </div>
    </div>

    <div class="queue-list">
      {#each state.queue as item, i}
        <div class="queue-item fade-in" style="animation-delay: {i * 50}ms">
          <div class="queue-priority" class:priority-critical={item.priority === 'critical'} class:priority-pending={item.priority === 'pending'}>
            {item.priority === 'critical'
              ? ($lang === 'ar' ? 'حرج' : 'CRITICAL')
              : ($lang === 'ar' ? 'معلق' : 'PENDING')}
          </div>
          <div class="queue-info">
            <h3 class="queue-name">{$lang === 'ar' ? item.name_ar : item.name_en}</h3>
            <span class="queue-meta">
              {$lang === 'ar' ? item.city_ar : item.city_en} — {item.time} {$lang === 'ar' ? 'مضت' : 'ago'}
            </span>
          </div>
          <a href="/dashboard/vendors" class="queue-action-btn">
            {$lang === 'ar' ? 'مراجعة' : 'Review'}
            <ChevronRight size={13} aria-hidden="true" />
          </a>
        </div>
      {/each}
    </div>
  </section>
</div>

<style>
  .dash-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 20px;
    align-items: start;
  }

  .panel-header {
    display: flex; align-items: flex-start; justify-content: space-between;
    padding: 18px 20px 14px;
    border-bottom: 1px solid var(--glass-border);
    gap: 12px;
  }
  .panel-title { font-size: 15px; font-weight: 700; color: var(--text-primary); margin-bottom: 3px; }
  .panel-sub { font-size: 12px; color: var(--text-tertiary); margin: 0; }
  .status-live { background: var(--success); box-shadow: 0 0 8px var(--success); }

  /* Activity Panel */
  .activity-list { padding: 12px 0; }
  .activity-item {
    display: flex; align-items: flex-start; gap: 10px;
    padding: 10px 20px;
    transition: background 150ms;
  }
  .activity-item:hover { background: var(--bg-hover-sm); }
  .act-icon {
    width: 30px; height: 30px; border-radius: 8px;
    display: flex; align-items: center; justify-content: center;
    flex-shrink: 0;
  }
  .act-success { background: var(--success-dim); color: var(--success); border: 1px solid var(--success-border); }
  .act-gold    { background: var(--gold-subtle); color: var(--gold); border: 1px solid var(--gold-border); }
  .act-danger  { background: var(--danger-dim);  color: var(--danger); border: 1px solid var(--danger-border); }
  .act-info    { background: var(--info-dim);    color: var(--info); border: 1px solid var(--info-border); }
  .act-purple  { background: var(--purple-dim);  color: var(--purple); border: 1px solid var(--purple-border); }
  .act-content { flex: 1; min-width: 0; }
  .act-msg { font-size: 12.5px; color: var(--text-secondary); line-height: 1.5; margin-bottom: 2px; }
  .act-time { font-size: 11px; color: var(--text-ghost); }
  .activity-empty {
    display: flex; align-items: center; justify-content: center; gap: 8px;
    padding: 32px 20px; color: var(--text-ghost); font-size: 13px; font-style: italic;
  }

  /* Moderation Queue */
  .queue-list { padding: 8px 0; }
  .queue-item {
    display: flex; align-items: center; gap: 10px;
    padding: 12px 20px;
    border-bottom: 1px solid var(--glass-border);
    transition: background 150ms;
  }
  .queue-item:last-child { border-bottom: none; }
  .queue-item:hover { background: var(--bg-hover-sm); }
  .queue-priority {
    font-size: 9.5px; font-weight: 800; letter-spacing: 0.5px;
    padding: 3px 7px; border-radius: 4px; flex-shrink: 0;
  }
  .priority-critical { background: var(--danger-dim); color: var(--danger); border: 1px solid var(--danger-border); }
  .priority-pending  { background: var(--warning-dim); color: var(--warning); border: 1px solid var(--warning-border); }
  .queue-info { flex: 1; min-width: 0; }
  .queue-name { font-size: 13px; font-weight: 600; color: var(--text-primary); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; margin-bottom: 2px; }
  .queue-meta { font-size: 11px; color: var(--text-ghost); }
  .queue-action-btn {
    display: inline-flex; align-items: center; gap: 4px;
    font-size: 12px; font-weight: 700; color: var(--gold);
    background: var(--gold-subtle); border: 1px solid var(--gold-border);
    padding: 5px 10px; border-radius: 6px;
    text-decoration: none; white-space: nowrap; flex-shrink: 0;
    transition: all 180ms var(--ease-smooth);
  }
  .queue-action-btn:hover { background: var(--gold-glow); }

  @media (max-width: 1100px) {
    .dash-row { grid-template-columns: 1fr; }
  }
</style>
