<script lang="ts">
  import * as m from "$lib/paraglide/messages.js";
  import Button from "$lib/components/ui/Button.svelte";
  import { formatDate } from "$lib/utils/localize.js";
  import { getStatusLabel, getStatusColor } from "$lib/services/api/dashboard.service.js";
  import type { DashboardState } from "$lib/stores/dashboardState.svelte.js";

  let { state, activeBookings = [] } = $props<{ state: DashboardState, activeBookings?: any[] }>();
</script>

<section
  class="bg-white rounded-2xl border border-[var(--color-border)] p-6 shadow-sm flex flex-col gap-4"
  aria-labelledby="active-bookings-title"
>
  <div class="pb-3 border-b border-[var(--color-border)] flex justify-between items-center">
    <div>
      <h2
        id="active-bookings-title"
        class="font-display text-lg font-bold text-[var(--color-secondary)]"
      >
        {m.auto_my_active_bookings()}
      </h2>
      <p class="text-xs text-[var(--color-muted)]">
        {m.auto_current_booking_inqu()}
      </p>
    </div>
    <span class="text-xs font-semibold px-2 py-0.5 rounded-full bg-slate-100 text-slate-700">
      {activeBookings.length}
    </span>
  </div>

  {#if activeBookings.length === 0}
    <div class="py-8 text-center text-[var(--color-muted)] border border-dashed border-[var(--color-border)] rounded-xl bg-[var(--color-surface-alt)]">
      <span class="text-3xl mb-2 block">🗓️</span>
      <p class="font-semibold text-sm">
        {m.auto_no_active_reservatio()}
      </p>
      <p class="text-xs mt-1 mb-4">
        {m.auto_browse_our_vip_v()}
      </p>
      <Button href="/venues" variant="primary" size="sm" class="btn-royal-purple">
        {m.auto_start_exploring_hall()}
      </Button>
    </div>
  {:else}
    <!-- Mobile Bookings List (visible only on mobile) -->
    <div class="md:hidden flex flex-col gap-3">
      {#each activeBookings as b (b.id)}
        <div class="p-4 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-alt)]/20 flex flex-col gap-3">
          <div class="flex justify-between items-center">
            <span class="font-mono font-bold text-xs text-[var(--color-secondary)]">#{b.bookingNumber}</span>
            <span class="px-2.5 py-0.5 rounded-full text-[10px] font-semibold border {getStatusColor(b.status)}">
              {getStatusLabel(b.status)}
            </span>
          </div>
          <div class="grid grid-cols-2 gap-2 text-xs border-y border-[var(--color-border)]/50 py-2 my-0.5">
            <div>
              <span class="text-[10px] text-[var(--color-muted)] block mb-0.5 uppercase font-medium">{m.auto_event_type()}</span>
              <span class="font-semibold text-[var(--color-secondary)] capitalize">{b.eventType || "wedding"}</span>
            </div>
            <div>
              <span class="text-[10px] text-[var(--color-muted)] block mb-0.5 uppercase font-medium">{m.auto_event_date()}</span>
              <span class="font-semibold text-[var(--color-secondary)]">{formatDate(b.weddingDate)}</span>
            </div>
          </div>
          <div class="flex justify-between items-center">
            <div>
              <span class="text-[10px] text-[var(--color-muted)] block mb-0.5 uppercase font-medium">{m.auto_total_price()}</span>
              <span class="font-bold text-sm text-[var(--color-secondary)]">
                {b.totalPrice ? b.totalPrice.toLocaleString() : "0"}
                <span class="text-xs font-normal text-[var(--color-muted)]">{m.common_currency()}</span>
              </span>
            </div>
            <Button variant="outline" size="sm" class="py-1 px-3 text-xs" onclick={() => (state.selectedBooking = b)}>
              {m.auto_view_details()}
            </Button>
          </div>
        </div>
      {/each}
    </div>

    <!-- Desktop Bookings Table (visible only on desktop) -->
    <div class="hidden md:block overflow-x-auto border border-[var(--color-border)] rounded-xl">
      <table class="w-full text-sm text-start collapse border-none">
        <thead>
          <tr class="bg-[var(--color-surface-alt)] border-b border-[var(--color-border)]">
            <th class="p-3 text-start font-semibold text-[var(--color-secondary)]">{m.auto_booking_()}</th>
            <th class="p-3 text-start font-semibold text-[var(--color-secondary)]">{m.auto_event_type()}</th>
            <th class="p-3 text-start font-semibold text-[var(--color-secondary)]">{m.auto_event_date()}</th>
            <th class="p-3 text-start font-semibold text-[var(--color-secondary)]">{m.auto_total_price()}</th>
            <th class="p-3 text-start font-semibold text-[var(--color-secondary)]">{m.auto_status()}</th>
            <th class="p-3 text-start font-semibold text-[var(--color-secondary)]"></th>
          </tr>
        </thead>
        <tbody>
          {#each activeBookings as b (b.id)}
            <tr class="border-b border-[var(--color-border)] last:border-0 hover:bg-[var(--color-surface-alt)]/50 transition-colors">
              <td class="p-3 font-mono font-bold text-[var(--color-secondary)]">{b.bookingNumber}</td>
              <td class="p-3 capitalize">{b.eventType || "wedding"}</td>
              <td class="p-3 text-xs text-[var(--color-muted)]">{formatDate(b.weddingDate)}</td>
              <td class="p-3 font-semibold">
                {b.totalPrice ? b.totalPrice.toLocaleString() : "0"}
                <span class="text-xs text-[var(--color-muted)]">{m.common_currency()}</span>
              </td>
              <td class="p-3">
                <span class="px-2.5 py-0.5 rounded-full text-xs font-semibold border {getStatusColor(b.status)}">
                  {getStatusLabel(b.status)}
                </span>
              </td>
              <td class="p-3 text-end">
                <Button variant="outline" size="sm" onclick={() => (state.selectedBooking = b)}>
                  {m.auto_view_details()}
                </Button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</section>
