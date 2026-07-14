<script lang="ts">
  import { lang } from "$lib/i18n/index.js";
  import type { MarketingState } from "$lib/features/admin/marketing/marketingState.svelte.js";

  let { state }: { state: MarketingState } = $props();
</script>

<div class="mkt-stats">
  <button
    onclick={() => (state.statusFilter = "all")}
    class="mini-stat card clickable-stat"
    class:active-stat={state.statusFilter === "all"}
  >
    <span class="mini-stat-label"
      >{$lang === "ar" ? "إجمالي الحملات" : "Total Campaigns"}</span
    >
    <span class="mini-stat-value text-muted">{state.totalCount}</span>
  </button>
  <button
    onclick={() => (state.statusFilter = "pending")}
    class="mini-stat card clickable-stat"
    class:active-stat={state.statusFilter === "pending"}
  >
    <span class="mini-stat-label"
      >{$lang === "ar" ? "قيد المراجعة" : "Pending Review"}</span
    >
    <span class="mini-stat-value text-gold">{state.pendingCount}</span>
  </button>
  <button
    onclick={() => (state.statusFilter = "approved")}
    class="mini-stat card clickable-stat"
    class:active-stat={state.statusFilter === "approved"}
  >
    <span class="mini-stat-label"
      >{$lang === "ar" ? "المعتمدة / النشطة" : "Approved / Active"}</span
    >
    <span class="mini-stat-value text-success">{state.approvedCount}</span>
  </button>
  <button
    onclick={() => (state.statusFilter = "rejected")}
    class="mini-stat card clickable-stat"
    class:active-stat={state.statusFilter === "rejected"}
  >
    <span class="mini-stat-label"
      >{$lang === "ar" ? "المرفوضة" : "Rejected"}</span
    >
    <span class="mini-stat-value text-danger">{state.rejectedCount}</span>
  </button>
</div>

<style>
  .mkt-stats {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 12px;
    margin-bottom: 10px;
  }
  .mini-stat {
    padding: 16px 18px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    text-align: start;
  }
  .clickable-stat {
    cursor: pointer;
    transition: all 0.2s;
    border: 1.5px solid var(--border);
    background: var(--card-bg);
  }
  .clickable-stat:hover {
    border-color: var(--teal);
    transform: translateY(-1px);
  }
  .active-stat {
    border-color: var(--teal);
    background-color: var(--bg-alt);
    box-shadow: var(--shadow-sm);
  }
  .mini-stat-label {
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.4px;
    color: var(--text-ghost);
  }
  .mini-stat-value {
    font-size: 22px;
    font-weight: 800;
    letter-spacing: -0.4px;
  }
  @media (max-width: 900px) {
    .mkt-stats {
      grid-template-columns: repeat(2, 1fr);
    }
  }
</style>
