<script lang="ts">
  import * as m from "$lib/paraglide/messages.js";
  import { getLocale } from "$lib/paraglide/runtime.js";
  import Button from "$lib/components/ui/Button.svelte";
  import { enhance } from "$app/forms";
  import { formatDate } from "$lib/utils/localize.js";
  import { getStatusLabel, getStatusColor, handleCancelBooking } from "$lib/services/api/dashboard.service.js";
  import type { DashboardState } from "$lib/stores/dashboardState.svelte.js";

  let { state } = $props<{ state: DashboardState }>();
</script>

{#if state.selectedBooking}
  <!-- Modal Backdrop -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 z-[var(--z-modal)] bg-black/60 backdrop-blur-sm flex items-center justify-center p-4"
    onclick={() => (state.selectedBooking = null)}
  >
    <!-- Modal Card -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="bg-white rounded-2xl border border-[var(--color-border)] w-full max-w-lg shadow-[var(--shadow-lg)] overflow-hidden flex flex-col"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="p-6 border-b border-[var(--color-border)] flex justify-between items-center bg-[var(--color-surface-alt)]">
        <div>
          <h3 class="font-display text-lg font-bold text-[var(--color-secondary)]">
            {m.auto_booking_receipt_deta()}
          </h3>
          <p class="text-xs text-[var(--color-muted)] font-mono mt-0.5">
            #{state.selectedBooking.bookingNumber}
          </p>
        </div>
        <button
          class="text-[var(--color-muted)] hover:text-[var(--color-secondary)] cursor-pointer text-lg p-1"
          onclick={() => (state.selectedBooking = null)}
        >
          ✕
        </button>
      </div>

      <!-- Details List -->
      <div class="p-6 flex flex-col gap-4 overflow-y-auto max-h-[70vh]">
        <div class="grid grid-cols-2 gap-4">
          <div>
            <span class="text-xs text-[var(--color-muted)] block">{m.auto_hall_name()}</span>
            <span class="font-semibold text-sm text-[var(--color-secondary)]">
              {getLocale() === "ar"
                ? state.selectedBooking.productNameAr || "غير محدد"
                : state.selectedBooking.productNameEn || "Not specified"}
            </span>
          </div>
          <div>
            <span class="text-xs text-[var(--color-muted)] block">{m.auto_event_type()}</span>
            <span class="font-semibold text-sm capitalize">{state.selectedBooking.eventType || "wedding"}</span>
          </div>
          <div>
            <span class="text-xs text-[var(--color-muted)] block">{m.auto_event_date()}</span>
            <span class="font-semibold text-sm">{formatDate(state.selectedBooking.weddingDate)}</span>
          </div>
          <div>
            <span class="text-xs text-[var(--color-muted)] block">{m.auto_gender_section()}</span>
            <span class="font-semibold text-sm">
              {#if state.selectedBooking.bookedGenderSection === "women_only"}
                {m.auto_women_only()}
              {:else if state.selectedBooking.bookedGenderSection === "men_only"}
                {m.auto_men_only()}
              {:else if state.selectedBooking.bookedGenderSection === "dual_parallel"}
                {m.auto_dual_parallel()}
              {:else}
                {state.selectedBooking.bookedGenderSection || "Not specified"}
              {/if}
            </span>
          </div>
          <div>
            <span class="text-xs text-[var(--color-muted)] block">{m.auto_total_price()}</span>
            <span class="font-bold text-sm text-[var(--color-secondary)]">
              {state.selectedBooking.totalPrice ? state.selectedBooking.totalPrice.toLocaleString() : "0"}
              <span class="text-xs font-normal text-[var(--color-muted)]">{m.common_currency()}</span>
            </span>
          </div>
          <div>
            <span class="text-xs text-[var(--color-muted)] block">{m.auto_deposit_paid()}</span>
            <span class="font-semibold text-sm text-emerald-700">
              {state.selectedBooking.depositPaid ? state.selectedBooking.depositPaid.toLocaleString() : "0"}
              <span class="text-xs font-normal text-[var(--color-muted)]">{m.common_currency()}</span>
            </span>
          </div>
          <div>
            <span class="text-xs text-[var(--color-muted)] block">{m.auto_status()}</span>
            <span class="inline-block mt-1 px-2.5 py-0.5 rounded-full text-xs font-semibold border {getStatusColor(state.selectedBooking.status)}">
              {getStatusLabel(state.selectedBooking.status)}
            </span>
          </div>
          <div>
            <span class="text-xs text-[var(--color-muted)] block">{m.auto_created_at()}</span>
            <span class="font-semibold text-sm text-slate-500">{formatDate(state.selectedBooking.createdAt)}</span>
          </div>
        </div>

        <!-- Cancellation Section -->
        {#if state.selectedBooking.status !== "cancelled" && state.selectedBooking.status !== "booking_active" && state.selectedBooking.status !== "confirmed"}
          <div class="mt-6 border-t border-[var(--color-border)] pt-6 flex flex-col gap-3">
            <p class="text-xs text-[var(--color-muted)]">
              {m.auto_do_you_want_to_cance()}
            </p>
            <form
              method="POST"
              action="?/cancelBooking"
              use:enhance={handleCancelBooking(state)}
            >
              <input type="hidden" name="bookingId" value={state.selectedBooking.id} />
              <Button
                type="submit"
                variant="danger"
                size="sm"
                loading={state.cancelling}
                class="w-full"
              >
                {m.auto_request_cancellation()}
              </Button>
            </form>
          </div>
        {:else if state.selectedBooking.status === "booking_active" || state.selectedBooking.status === "confirmed"}
          <div class="mt-6 border-t border-[var(--color-border)] pt-6">
            <div class="p-3 bg-amber-50 border border-amber-200 text-amber-900 rounded-xl text-xs flex items-start gap-2">
              <span>⚠️</span>
              <p>
                {m.auto_this_booking_is_conf()}
              </p>
            </div>
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="p-4 border-t border-[var(--color-border)] bg-[var(--color-surface-alt)] flex justify-end">
        <Button variant="ghost" size="sm" onclick={() => (state.selectedBooking = null)}>
          {m.auto_close()}
        </Button>
      </div>
    </div>
  </div>
{/if}
