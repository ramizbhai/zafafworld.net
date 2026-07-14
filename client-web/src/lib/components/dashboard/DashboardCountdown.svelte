<script lang="ts">
  import * as m from "$lib/paraglide/messages.js";
  import { formatDate } from "$lib/utils/localize.js";
  import type { DashboardState } from "$lib/stores/dashboardState.svelte.js";

  let { state } = $props<{ state: DashboardState }>();
</script>

<section
  class="bg-gradient-to-br from-[var(--color-secondary)] to-[#151D30] rounded-2xl p-6 text-white shadow-md relative overflow-hidden flex flex-col justify-between"
  aria-labelledby="countdown-title"
>
  <div class="absolute -right-16 -top-16 w-36 h-36 rounded-full bg-white/5 pointer-events-none"></div>
  <div>
    <h2
      id="countdown-title"
      class="font-display text-sm font-semibold uppercase tracking-wider text-[var(--color-primary)] mb-3"
    >
      {m.auto_wedding_countdown()}
    </h2>

    {#if state.hasWeddingDate}
      <div class="grid grid-cols-4 gap-2 text-center mt-2 max-w-sm">
        <div class="bg-white/10 rounded-lg p-2.5 backdrop-blur-sm">
          <span class="text-3xl font-black block font-display text-[var(--color-primary)]">{state.days}</span>
          <span class="text-[10px] text-slate-300 uppercase font-medium">{m.auto_days()}</span>
        </div>
        <div class="bg-white/10 rounded-lg p-2.5 backdrop-blur-sm">
          <span class="text-3xl font-black block font-display text-[var(--color-primary)]">{state.hours}</span>
          <span class="text-[10px] text-slate-300 uppercase font-medium">{m.auto_hours()}</span>
        </div>
        <div class="bg-white/10 rounded-lg p-2.5 backdrop-blur-sm">
          <span class="text-3xl font-black block font-display text-[var(--color-primary)]">{state.minutes}</span>
          <span class="text-[10px] text-slate-300 uppercase font-medium">{m.auto_mins()}</span>
        </div>
        <div class="bg-white/10 rounded-lg p-2.5 backdrop-blur-sm">
          <span class="text-3xl font-black block font-display text-[var(--color-primary)]">{state.seconds}</span>
          <span class="text-[10px] text-slate-300 uppercase font-medium">{m.auto_secs()}</span>
        </div>
      </div>
    {:else}
      <div class="py-2 flex flex-col gap-2">
        <span class="text-3xl">💍</span>
        <h3 class="font-semibold text-base">
          {m.auto_no_wedding_date_sele()}
        </h3>
        <p class="text-xs text-slate-300 max-w-md">
          {m.auto_complete_your_first_()}
        </p>
      </div>
    {/if}
  </div>
  {#if state.hasWeddingDate && state.weddingCountdown.weddingDate}
    <p class="text-xs text-slate-400 mt-4 border-t border-white/10 pt-3 text-start">
      {m.auto_target_wedding_date()}
      <span class="text-white font-semibold">{formatDate(state.weddingCountdown.weddingDate)}</span>
    </p>
  {/if}
</section>
