<script lang="ts">
  import * as m from "$lib/paraglide/messages.js";
  import { getLocale } from "$lib/paraglide/runtime.js";
  import Button from "$lib/components/ui/Button.svelte";
  import { formatDate } from "$lib/utils/localize.js";
  import { getInquiryStatusLabel, getInquiryStatusColor } from "$lib/services/api/dashboard.service.js";

  let { inquiries = [] } = $props<{ inquiries?: any[] }>();
</script>

<section
  class="bg-white rounded-2xl border border-[var(--color-border)] p-6 shadow-sm flex flex-col gap-4"
  aria-labelledby="recent-inquiries-title"
>
  <div class="pb-3 border-b border-[var(--color-border)] flex justify-between items-center">
    <div>
      <h2
        id="recent-inquiries-title"
        class="font-display text-lg font-bold text-[var(--color-secondary)]"
      >
        {m.auto_recent_inquiries()}
      </h2>
      <p class="text-xs text-[var(--color-muted)]">
        {m.auto_inquiry_requests_and()}
      </p>
    </div>
    <span class="text-xs font-semibold px-2 py-0.5 rounded-full bg-slate-100 text-slate-700">
      {inquiries.length}
    </span>
  </div>

  {#if inquiries.length === 0}
    <div class="py-8 text-center text-[var(--color-muted)] border border-dashed border-[var(--color-border)] rounded-xl bg-[var(--color-surface-alt)]">
      <span class="text-3xl mb-2 block">📩</span>
      <p class="font-semibold text-sm">
        {m.auto_no_inquiries_sent_ye()}
      </p>
      <p class="text-xs mt-1">
        {m.auto_when_you_contact_a_v()}
      </p>
    </div>
  {:else}
    <!-- Mobile Inquiries List (visible only on mobile) -->
    <div class="md:hidden flex flex-col gap-3">
      {#each inquiries as inq}
        <div class="p-4 rounded-xl border border-[var(--color-border)] bg-[var(--color-surface-alt)]/20 flex flex-col gap-2">
          <div class="flex justify-between items-start">
            <div class="min-w-0">
              <span class="font-bold text-sm text-[var(--color-secondary)] block truncate">
                {getLocale() === "ar" ? inq.vendorNameAr || "مورد" : inq.vendorNameEn || "Vendor"}
              </span>
              <span class="text-xs text-[var(--color-muted)] block truncate mt-0.5">
                {getLocale() === "ar" ? inq.productNameAr || "قاعة" : inq.productNameEn || "Venue Listing"}
              </span>
            </div>
            <span class="px-2.5 py-0.5 rounded-full text-[10px] font-semibold border shrink-0 {getInquiryStatusColor(inq.status)}">
              {getInquiryStatusLabel(inq.status)}
            </span>
          </div>

          <div class="flex justify-between items-center border-t border-[var(--color-border)]/50 pt-2.5 mt-1">
            <div>
              <span class="text-[10px] text-[var(--color-muted)] block mb-0.5 uppercase font-medium">{m.auto_target_date()}</span>
              <span class="text-xs font-semibold text-[var(--color-secondary)]">{formatDate(inq.eventDate)}</span>
            </div>
            {#if inq.conversationId}
              <Button href="/dashboard/messages?chatId={inq.conversationId}" variant="outline" size="sm" class="flex items-center justify-center gap-1.5 py-1 px-3 text-xs">
                <span>💬</span>
                {m.auto_chat()}
              </Button>
            {:else}
              <span class="text-xs text-[var(--color-muted)]">—</span>
            {/if}
          </div>
        </div>
      {/each}
    </div>

    <!-- Desktop Inquiries Table (visible only on desktop) -->
    <div class="hidden md:block overflow-x-auto border border-[var(--color-border)] rounded-xl">
      <table class="w-full text-sm text-start collapse border-none">
        <thead>
          <tr class="bg-[var(--color-surface-alt)] border-b border-[var(--color-border)]">
            <th class="p-3 text-start font-semibold text-[var(--color-secondary)]">{m.auto_vendor()}</th>
            <th class="p-3 text-start font-semibold text-[var(--color-secondary)]">{m.auto_product__hall()}</th>
            <th class="p-3 text-start font-semibold text-[var(--color-secondary)]">{m.auto_target_date()}</th>
            <th class="p-3 text-start font-semibold text-[var(--color-secondary)]">{m.auto_status()}</th>
            <th class="p-3 text-start font-semibold text-[var(--color-secondary)]"></th>
          </tr>
        </thead>
        <tbody>
          {#each inquiries as inq}
            <tr class="border-b border-[var(--color-border)] last:border-0 hover:bg-[var(--color-surface-alt)]/50 transition-colors">
              <td class="p-3 font-semibold text-[var(--color-secondary)]">
                {getLocale() === "ar" ? inq.vendorNameAr || "مورد" : inq.vendorNameEn || "Vendor"}
              </td>
              <td class="p-3">
                {getLocale() === "ar" ? inq.productNameAr || "قاعة" : inq.productNameEn || "Venue Listing"}
              </td>
              <td class="p-3 text-xs text-[var(--color-muted)]">{formatDate(inq.eventDate)}</td>
              <td class="p-3">
                <span class="px-2.5 py-0.5 rounded-full text-xs font-semibold border {getInquiryStatusColor(inq.status)}">
                  {getInquiryStatusLabel(inq.status)}
                </span>
              </td>
              <td class="p-3 text-end">
                {#if inq.conversationId}
                  <Button href="/dashboard/messages?chatId={inq.conversationId}" variant="outline" size="sm" class="flex items-center justify-center gap-1">
                    <span>💬</span>
                    {m.auto_chat()}
                  </Button>
                {:else}
                  <span class="text-xs text-[var(--color-muted)]">—</span>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</section>
