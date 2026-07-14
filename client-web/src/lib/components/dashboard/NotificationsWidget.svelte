<script lang="ts">
  import * as m from "$lib/paraglide/messages.js";
  import { getLocale } from "$lib/paraglide/runtime.js";
  import { getLocalizedField } from "$lib/utils/localize.js";
  import { enhance } from "$app/forms";
  import { handleReadNotifications } from "$lib/services/api/dashboard.service.js";
  import type { DashboardState } from "$lib/stores/dashboardState.svelte.js";

  let { state, notifications = [] } = $props<{ state: DashboardState, notifications?: any[] }>();
</script>

<section
  class="bg-white rounded-2xl border border-[var(--color-border)] p-6 shadow-sm flex flex-col gap-4"
  aria-labelledby="notifications-title"
>
  <div class="pb-3 border-b border-[var(--color-border)] flex justify-between items-center">
    <div>
      <h2
        id="notifications-title"
        class="font-display text-sm font-bold text-[var(--color-secondary)] uppercase tracking-wider"
      >
        {m.auto_system_notifications()}
      </h2>
      <span class="text-[10px] text-[var(--color-muted)] block mt-0.5">{m.auto_unread_system_alerts()}</span>
    </div>
    {#if notifications.length > 0}
      <form
        method="POST"
        action="?/readNotifications"
        use:enhance={handleReadNotifications(state)}
      >
        <button
          type="submit"
          disabled={state.readingAll}
          class="text-[10px] text-[var(--color-primary)] hover:underline font-bold cursor-pointer disabled:opacity-50"
        >
          {m.auto_mark_all_read()}
        </button>
      </form>
    {/if}
  </div>

  <div class="flex flex-col gap-3 max-h-[300px] overflow-y-auto pe-1">
    {#if notifications.length === 0}
      <div class="py-6 text-center text-[var(--color-muted)] text-xs">
        <span>🔔</span>
        <p class="mt-1">
          {m.auto_no_unread_notificati()}
        </p>
      </div>
    {:else}
      {#each notifications as notif}
        <div class="p-3 rounded-xl bg-amber-50/50 border border-amber-100 flex gap-2">
          <span class="text-sm shrink-0">✨</span>
          <div class="flex-1 min-w-0">
            <p class="text-xs text-slate-800 leading-normal font-medium">
              {getLocalizedField(notif.message, "name", getLocale())}
            </p>
            <span class="text-[9px] text-[var(--color-muted)] block mt-1">
              {new Date(notif.createdAt).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
            </span>
          </div>
        </div>
      {/each}
    {/if}
  </div>
  <p class="text-[9px] text-[var(--color-muted)] italic border-t border-[var(--color-border)] pt-2 mt-1">
    * {m.auto_system_notifications_1()}
  </p>
</section>
