<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte";
  import { formatCurrency } from "$lib/utils/localize.js";
  import * as m from "$lib/paraglide/messages.js";
  import { getLocale } from "$lib/paraglide/runtime.js";

  let { venue, user, isVenue, t, onShowAuthPopup, onShowInquiryModal } = $props<{
    venue: any;
    user: any;
    isVenue: boolean;
    t: (ar: string, en: string) => string;
    onShowAuthPopup: () => void;
    onShowInquiryModal: () => void;
  }>();
</script>

<aside class="lg:col-span-1" aria-label="Quick Actions Sidebar">
  <div class="sticky top-28 flex flex-col gap-6">
    <!-- Sticky Booking Card -->
    <div
      class="bg-white rounded-2xl border border-[var(--color-border)] shadow-[var(--shadow-md)] overflow-hidden text-start"
    >
      <!-- Starting Price Header -->
      <div
        class="bg-[var(--color-secondary)] p-6 text-white text-start relative"
      >
        <div
          class="absolute top-0 start-0 w-[4px] h-full bg-[var(--color-primary)]"
        ></div>
        <p
          class="text-[10px] uppercase tracking-wider text-white/60 font-bold mb-1"
        >
          {isVenue
            ? m.venues_details_pricePerEvent()
            : t("سعر الخدمة الأولي", "Starting Price")}
        </p>
        <p
          class="font-display text-2xl md:text-3xl font-extrabold text-[var(--color-primary)]"
        >
          {venue.pricing.basePrice > 0
            ? formatCurrency(venue.pricing.basePrice)
            : t("الأسعار عند الطلب", "Price on Inquiry")}
        </p>
        {#if isVenue && venue.pricing.weekendSurcharge}
          <p class="text-[10px] text-white/50 mt-1 font-medium">
            + {formatCurrency(venue.pricing.weekendSurcharge)}
            {m.auto_weekend_surcharge()}
          </p>
        {/if}
      </div>

      <!-- Booking buttons & Action triggers -->
      <div class="p-6 flex flex-col gap-4">
        <!-- Quick Stats Table -->
        <div
          class="flex flex-col gap-3 text-xs md:text-sm border-b border-[var(--color-border)] pb-4"
        >
          {#if isVenue}
            <div class="flex items-center justify-between">
              <span class="text-[var(--color-muted)] font-medium"
                >{m.venues_details_capacity()}</span
              >
              <span class="font-bold text-[var(--color-secondary)]"
                >{venue.capacity?.min || 0} – {venue.capacity?.max || 0}
                {t("ضيف", "guests")}</span
              >
            </div>
            <div class="flex items-center justify-between">
              <span class="text-[var(--color-muted)] font-medium"
                >{m.venues_details_area()}</span
              >
              <span class="font-bold text-[var(--color-secondary)]"
                >{venue.areaSqm || 0} m²</span
              >
            </div>
          {/if}
          <div class="flex items-center justify-between">
            <span class="text-[var(--color-muted)] font-medium"
              >{m.auto_deposit()}</span
            >
            <span class="font-bold text-[var(--color-secondary)]"
              >{venue.pricing.depositPercentage}%</span
            >
          </div>
        </div>

        <!-- CTA buttons -->
        <div class="flex flex-col gap-2">
          <Button
            href={user ? `/booking/${venue.id}` : undefined}
            onclick={(e) => {
              if (!user) {
                e.preventDefault();
                onShowAuthPopup();
              }
            }}
            variant="primary"
            fullWidth
            size="lg"
            class="font-bold text-xs"
          >
            ⚡ {m.venues_details_bookNow()}
          </Button>

          <Button
            onclick={() => {
              if (!user) onShowAuthPopup();
              else onShowInquiryModal();
            }}
            fullWidth
            size="md"
            class="text-white bg-[#5b21b6] hover:bg-[#4c1d95] border-none shadow-md font-bold text-xs transition-colors duration-200"
          >
            ⚜️ {m.auto_check_availability_()}
          </Button>
        </div>

        <!-- Contact detail paywall -->
        <div class="border-t border-[var(--color-border)] pt-4">
          <h4
            class="font-bold text-xs text-[var(--color-secondary)] mb-3"
          >
            {t("معلومات الاتصال المباشر", "Direct Business Information")}
          </h4>

          {#if user}
            <div
              class="flex flex-col gap-2.5 text-xs text-[var(--color-text)]"
            >
              {#if venue.phone}
                <p class="flex items-center gap-2 font-medium">
                  <span>📞</span>
                  <a
                    href="tel:{venue.phone}"
                    class="hover:text-[var(--color-primary-contrast)] underline"
                    >{venue.phone}</a
                  >
                </p>
              {/if}
              {#if venue.email}
                <p class="flex items-center gap-2 font-medium">
                  <span>✉️</span>
                  <a
                    href="mailto:{venue.email}"
                    class="hover:text-[var(--color-primary-contrast)] underline"
                    >{venue.email}</a
                  >
                </p>
              {/if}
              {#if venue.website}
                <p class="flex items-center gap-2 font-medium">
                  <span>🌐</span>
                  <a
                    href={venue.website}
                    target="_blank"
                    rel="noopener noreferrer"
                    class="hover:text-[var(--color-primary-contrast)] underline truncate"
                    >{venue.website}</a
                  >
                </p>
              {/if}
            </div>
          {:else}
            <div
              class="flex flex-col gap-2.5 text-xs text-[var(--color-muted)]"
            >
              <p
                class="flex items-center gap-2 select-none filter blur-[3.5px]"
              >
                📞 <span>+966 50 000 0000</span>
              </p>
              <p
                class="flex items-center gap-2 select-none filter blur-[3.5px]"
              >
                ✉️ <span>vendor@zafafworld.net</span>
              </p>
              <p
                class="text-[11px] text-[var(--color-primary-contrast)] font-bold mt-1"
              >
                <a href="/auth/login" class="underline">
                  🔐 {m.auto_please_login_to_view()}
                </a>
              </p>
            </div>
          {/if}
        </div>
      </div>
    </div>

    <!-- Coordinator Info / Brand Owner Card -->
    {#if venue.coordinator?.nameEn || venue.coordinator?.nameAr}
      <div
        class="bg-white p-5 rounded-2xl border border-[var(--color-border)] shadow-sm text-start flex items-center gap-4"
      >
        <div
          class="w-12 h-12 rounded-full overflow-hidden border border-[var(--color-border)] bg-[var(--color-surface-alt)] shrink-0 flex items-center justify-center font-bold text-sm"
        >
          {#if venue.coordinator.avatar}
            <img
              src={venue.coordinator.avatar}
              alt="Coordinator Avatar"
              class="w-full h-full object-cover"
            />
          {:else}
            {(venue.coordinator.nameEn || venue.coordinator.nameAr || "C")
              .charAt(0)
              .toUpperCase()}
          {/if}
        </div>
        <div class="min-w-0">
          <span
            class="block text-[10px] text-[var(--color-muted)] font-bold uppercase tracking-wider"
            >{t("منسق الحساب المباشر", "Account Coordinator")}</span
          >
          <span
            class="font-bold text-sm text-[var(--color-secondary)] truncate block"
          >
            {getLocale() === "ar"
              ? venue.coordinator.nameAr
              : venue.coordinator.nameEn}
          </span>
          {#if venue.coordinator.phone || venue.coordinator.whatsapp}
            <span
              class="text-[10px] text-[var(--color-muted)] block mt-0.5 truncate"
              >{venue.coordinator.phone ||
                venue.coordinator.whatsapp}</span
            >
          {/if}
        </div>
      </div>
    {/if}
  </div>
</aside>
