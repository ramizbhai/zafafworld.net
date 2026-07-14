<script lang="ts">
  import * as m from "$lib/paraglide/messages.js";
  import { getLocale } from "$lib/paraglide/runtime.js";
  import { getLocalizedField, formatDate } from "$lib/utils/localize.js";

  let { activities = [] } = $props<{ activities?: any[] }>();
</script>

<section
  class="bg-white rounded-2xl border border-[var(--color-border)] p-6 shadow-sm flex flex-col gap-4"
  aria-labelledby="activity-feed-title"
>
  <div>
    <h2
      id="activity-feed-title"
      class="font-display text-sm font-bold text-[var(--color-secondary)] uppercase tracking-wider"
    >
      {m.auto_activity_feed()}
    </h2>
    <p class="text-xs text-[var(--color-muted)] mt-0.5">
      {m.auto_timeline_of_recent_a()}
    </p>
  </div>

  <div class="relative ps-4 border-s border-[var(--color-border)] flex flex-col gap-6 max-h-[400px] overflow-y-auto py-2">
    {#if activities.length === 0}
      <div class="text-center text-[var(--color-muted)] text-xs py-4">
        {m.auto_no_recorded_activity()}
      </div>
    {:else}
      {#each activities as act}
        <div class="relative">
          <!-- Bullet marker -->
          <span class="absolute -start-[21px] top-1 w-2.5 h-2.5 rounded-full bg-purple-500 border-2 border-white ring-2 ring-purple-100"></span>
          <div class="min-w-0">
            <p class="text-xs text-slate-800 leading-normal">
              {getLocalizedField(act.message, "name", getLocale())}
            </p>
            <span class="text-[9px] text-[var(--color-muted)] font-mono block mt-1">
              {formatDate(act.createdAt)}
            </span>
          </div>
        </div>
      {/each}
    {/if}
  </div>
</section>
