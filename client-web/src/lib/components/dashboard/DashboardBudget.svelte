<script lang="ts">
  import * as m from "$lib/paraglide/messages.js";
  import type { DashboardState } from "$lib/stores/dashboardState.svelte.js";

  let { state } = $props<{ state: DashboardState }>();
</script>

<section
  class="bg-white rounded-2xl border border-[var(--color-border)] p-6 shadow-sm flex flex-col justify-between"
  aria-labelledby="budget-tracking-title"
>
  <div class="flex justify-between items-start pb-3 border-b border-[var(--color-border)]">
    <div>
      <h2
        id="budget-tracking-title"
        class="font-display text-lg font-bold text-[var(--color-secondary)]"
      >
        {m.auto_budget_planner()}
      </h2>
      <p class="text-xs text-[var(--color-muted)]">
        {m.auto_monitor_spent_and_re()}
      </p>
    </div>
    {#if state.totalBudget > 0}
      <span class="text-xs font-semibold px-2.5 py-1 rounded-full bg-amber-50 text-[var(--color-primary)] border border-amber-200">
        {state.budgetPercent}% {m.auto_spent()}
      </span>
    {/if}
  </div>

  {#if state.totalBudget === 0}
    <div class="py-6 text-center text-[var(--color-muted)] bg-[var(--color-surface-alt)] rounded-xl border border-dashed border-[var(--color-border)]">
      <span class="text-2xl block mb-1">💸</span>
      <p class="text-sm font-semibold">
        {m.auto_no_budget_set_yet()}
      </p>
      <p class="text-xs mt-1">
        {m.auto_contact_support_to_c()}
      </p>
    </div>
  {:else}
    <!-- Metrics Row -->
    <div class="grid grid-cols-1 sm:grid-cols-3 gap-3 my-4">
      <div class="bg-[var(--color-surface-alt)] p-3.5 rounded-xl border border-[var(--color-border)] flex flex-col gap-0.5">
        <span class="text-[10px] text-[var(--color-muted)] font-semibold uppercase">{m.auto_total_budget()}</span>
        <span class="text-lg font-bold font-display text-[var(--color-secondary)]">
          {state.totalBudget.toLocaleString()}
          <span class="text-xs font-normal text-[var(--color-muted)]">{m.common_currency()}</span>
        </span>
      </div>
      <div class="bg-rose-50/50 p-3.5 rounded-xl border border-rose-100 flex flex-col gap-0.5">
        <span class="text-[10px] text-rose-700 font-semibold uppercase">{m.auto_spent_amount()}</span>
        <span class="text-lg font-bold font-display text-rose-800">
          {state.spentAmount.toLocaleString()}
          <span class="text-xs font-normal text-rose-600">{m.common_currency()}</span>
        </span>
      </div>
      <div class="bg-emerald-50/50 p-3.5 rounded-xl border border-emerald-100 flex flex-col gap-0.5">
        <span class="text-[10px] text-emerald-700 font-semibold uppercase">{m.auto_remaining_budget()}</span>
        <span class="text-lg font-bold font-display text-emerald-800">
          {state.remainingBudget.toLocaleString()}
          <span class="text-xs font-normal text-emerald-600">{m.common_currency()}</span>
        </span>
      </div>
    </div>

    <!-- Progress Bar -->
    <div class="w-full flex flex-col gap-1.5">
      <div class="w-full bg-[var(--color-surface-alt)] h-2.5 rounded-full overflow-hidden border border-[var(--color-border)]">
        <div
          class="bg-[var(--color-primary)] h-full transition-all duration-500 rounded-full"
          style="width: {state.budgetPercent}%"
        ></div>
      </div>
      <div class="flex justify-between text-[10px] text-[var(--color-muted)] font-semibold">
        <span>0%</span>
        <span>{m.auto_expenditure_index_d()}</span>
        <span>100%</span>
      </div>
    </div>
  {/if}
</section>
