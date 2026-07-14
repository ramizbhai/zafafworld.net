<script lang="ts">
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import * as m from "$lib/paraglide/messages.js";

  let { venue, user, isVenue, t, onShowAuthPopup, onShowInquiryModal } = $props<{
    venue: any;
    user: any;
    isVenue: boolean;
    t: (ar: string, en: string) => string;
    onShowAuthPopup: () => void;
    onShowInquiryModal: () => void;
  }>();
</script>

<div class="flex flex-col gap-6 text-start">
  {#if !venue.halls || venue.halls.length === 0}
    <div
      class="p-12 text-center text-[var(--color-muted)] bg-white border border-[var(--color-border)] rounded-2xl"
    >
      <span class="text-5xl block mb-3">📦</span>
      <p class="font-medium text-sm">
        {t(
          "لم يتم العثور على خدمات أو منتجات مدرجة لهذا الحساب.",
          "No services or packages listed for this vendor yet.",
        )}
      </p>
    </div>
  {:else}
    {#each venue.halls as hall}
      <div
        class="bg-white p-6 rounded-2xl border border-[var(--color-border)] flex flex-col md:flex-row gap-6 hover:shadow-md transition-all duration-300"
      >
        <!-- Product Media Container -->
        <div class="w-full md:w-[240px] shrink-0">
          {#if hall.images && hall.images.length > 0}
            {@const cover =
              hall.images.find((img: any) => img.isPrimary) ||
              hall.images[0]}
            <div
              class="aspect-[4/3] rounded-xl overflow-hidden bg-black/5 relative group"
            >
              <img
                src={cover.url}
                alt={cover.alt || hall.title}
                class="w-full h-full object-cover transition duration-300 group-hover:scale-105"
              />
              {#if hall.images.length > 1}
                <div
                  class="absolute bottom-2 end-2 bg-black/60 px-2 py-0.5 rounded text-[10px] text-white font-medium"
                >
                  📷 {hall.images.length}
                </div>
              {/if}
            </div>
          {:else}
            <div
              class="aspect-[4/3] rounded-xl bg-[var(--color-surface-alt)] border border-[var(--color-border)] flex flex-col items-center justify-center text-[var(--color-muted)]"
            >
              <span class="text-3xl">🖼️</span>
              <span class="text-[10px] mt-1"
                >{t("لا توجد صور", "No images")}</span
              >
            </div>
          {/if}
        </div>

        <!-- Product Details -->
        <div class="flex-1 flex flex-col justify-between">
          <div>
            <!-- Header Title & Attributes -->
            <div
              class="flex items-start justify-between flex-wrap gap-2 mb-2"
            >
              <h3
                class="text-lg font-bold text-[var(--color-secondary)]"
              >
                {hall.title}
              </h3>
              {#if isVenue && hall.attributes?.genderSection}
                <Badge variant="primary">
                  {hall.attributes.genderSection === "women_only"
                    ? m.auto_women_only_()
                    : hall.attributes.genderSection === "men_only"
                      ? m.auto_men_only_()
                      : hall.attributes.genderSection ===
                          "dual_parallel"
                        ? m.auto_dual_parallel_()
                        : hall.attributes.genderSection === "mixed"
                          ? m.auto_mixed_()
                          : m.auto_family_()}
                </Badge>
              {/if}
            </div>

            <p
              class="text-xs md:text-sm text-[var(--color-muted)] leading-relaxed mb-4"
            >
              {hall.description || ""}
            </p>

            <!-- Specification Table if Venue -->
            {#if isVenue}
              <div
                class="grid grid-cols-3 gap-2 p-3 bg-[var(--color-surface)] border border-[var(--color-border)] rounded-xl text-xs mb-4"
              >
                <div>
                  <span
                    class="block text-[10px] text-[var(--color-muted)]"
                    >{m.auto_capacity()}</span
                  >
                  <span
                    class="font-bold text-[var(--color-secondary)]"
                  >
                    {#if hall.attributes?.genderSection === "dual_parallel"}
                      👥 {hall.attributes?.menCapacity || 0} M / {hall
                        .attributes?.womenCapacity || 0} W
                    {:else if hall.attributes?.genderSection === "men_only"}
                      ♂ {hall.attributes?.menCapacity || 0}
                    {:else}
                      ♀ {hall.attributes?.womenCapacity || 0}
                    {/if}
                  </span>
                </div>
                <div>
                  <span
                    class="block text-[10px] text-[var(--color-muted)]"
                    >{m.auto_area()}</span
                  >
                  <span
                    class="font-bold text-[var(--color-secondary)]"
                    >📐 {hall.attributes?.areaSqm
                      ? `${hall.attributes.areaSqm} m²`
                      : "N/A"}</span
                  >
                </div>
                <div>
                  <span
                    class="block text-[10px] text-[var(--color-muted)]"
                    >{m.auto_floor()}</span
                  >
                  <span
                    class="font-bold text-[var(--color-secondary)]"
                    >🏢 {hall.attributes?.floorNumber !== null && hall.attributes?.floorNumber !== undefined
                      ? `${hall.attributes.floorNumber}`
                      : "G"}</span
                  >
                </div>
              </div>
            {:else if hall.attributes && Object.keys(hall.attributes).length > 0}
              <!-- Highlight Specific service details if Service provider -->
              <div class="flex flex-wrap gap-1.5 mb-4">
                {#each Object.entries(hall.attributes) as [key, val]}
                  {#if val && typeof val === "string" && val.length < 50}
                    <span
                      class="text-[10px] px-2 py-0.5 rounded bg-[var(--color-surface-alt)] text-[var(--color-secondary)] border border-[var(--color-border)] font-medium"
                    >
                      🏷️ {val}
                    </span>
                  {/if}
                {/each}
              </div>
            {/if}
          </div>

          <!-- Booking actions and pricing for this specific service item -->
          <div
            class="pt-4 border-t border-[var(--color-border)] flex flex-wrap items-center justify-between gap-4"
          >
            <div>
              <span
                class="block text-[10px] text-[var(--color-muted)]"
                >{t(
                  "سعر الخدمة والمنتج",
                  "Starting price for item",
                )}</span
              >
              <span
                class="text-base font-extrabold text-[var(--color-primary-contrast)]"
              >
                {hall.basePriceSar
                  ? `${hall.basePriceSar.toLocaleString()} SAR`
                  : m.auto_inquire_for_price()}
              </span>
            </div>

            <div class="flex items-center gap-2">
              <Button
                href={user
                  ? `/booking/listing/${hall.id}`
                  : undefined}
                onclick={(e) => {
                  if (!user) {
                    e.preventDefault();
                    onShowAuthPopup();
                  }
                }}
                variant="primary"
                size="sm"
                class="font-bold text-xs"
              >
                {t("حجز الآن", "Book Now")}
              </Button>
              <Button
                onclick={() => {
                  if (!user) onShowAuthPopup();
                  else onShowInquiryModal();
                }}
                variant="outline"
                size="sm"
                class="text-xs"
              >
                {t("استعلام عن توفر", "Inquire")}
              </Button>
            </div>
          </div>
        </div>
      </div>
    {/each}
  {/if}
</div>
