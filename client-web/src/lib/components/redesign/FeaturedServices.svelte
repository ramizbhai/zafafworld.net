<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { getLocale } from "$lib/paraglide/runtime.js";
  import * as m from "$lib/paraglide/messages.js";
  import { getLocalizedField } from "$lib/utils/localize.js";
  import Card from "./Card.svelte";
  import Button from "./Button.svelte";

  interface Props {
    listings?: any[];
  }

  let { listings = [] }: Props = $props();

  const isRtl = $derived(getLocale() === "ar");

  // Mobile slider controls
  let sliderEl = $state<HTMLDivElement | null>(null);
  let activeIndex = $state(0);
  let isHovered = $state(false);
  let autoplayInterval: any = null;

  function handleScroll() {
    if (!sliderEl) return;
    const scrollLeft = Math.abs(sliderEl.scrollLeft);
    const width = sliderEl.clientWidth;
    // Calculate current slide index based on scroll position
    activeIndex = Math.min(
      Math.round(scrollLeft / (width || 1)),
      listings.length - 1
    );
  }

  function scrollTo(index: number) {
    if (!sliderEl) return;
    const direction = isRtl ? -1 : 1;
    const cardWidth = sliderEl.clientWidth;
    sliderEl.scrollTo({
      left: cardWidth * index * direction,
      behavior: "smooth"
    });
    activeIndex = index;
  }

  // Keyboard navigation for carousel
  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "ArrowRight") {
      e.preventDefault();
      const nextIndex = isRtl
        ? Math.max(activeIndex - 1, 0)
        : Math.min(activeIndex + 1, listings.length - 1);
      scrollTo(nextIndex);
    } else if (e.key === "ArrowLeft") {
      e.preventDefault();
      const prevIndex = isRtl
        ? Math.min(activeIndex + 1, listings.length - 1)
        : Math.max(activeIndex - 1, 0);
      scrollTo(prevIndex);
    }
  }

  function startAutoplay() {
    // Disable if prefers-reduced-motion is enabled
    if (typeof window !== "undefined" && window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      return;
    }
    stopAutoplay();
    autoplayInterval = setInterval(() => {
      if (!isHovered && listings.length > 0) {
        const nextIndex = (activeIndex + 1) % listings.length;
        scrollTo(nextIndex);
      }
    }, 4500);
  }

  function stopAutoplay() {
    if (autoplayInterval) {
      clearInterval(autoplayInterval);
      autoplayInterval = null;
    }
  }

  onMount(() => {
    startAutoplay();
  });

  onDestroy(() => {
    stopAutoplay();
  });
</script>

<section class="py-zw-16 md:py-zw-24 bg-zw-surface-alt relative overflow-hidden" aria-labelledby="featured-services-title">
  <div class="w-full max-w-[1400px] mx-auto px-zw-4 sm:px-zw-6 md:px-zw-8 lg:px-zw-12">
    <!-- Header Block -->
    <div class="flex flex-col md:flex-row md:items-end justify-between gap-zw-6 mb-zw-12">
      <div>
        <span class="text-zw-primary text-zw-xs font-bold tracking-widest uppercase mb-zw-2 block">
          {isRtl ? "مزودو خدمات الزفاف المميزون" : "Featured Wedding Suppliers"}
        </span>
        <h2 id="featured-services-title" class="font-display text-zw-3xl sm:text-zw-4xl font-bold text-zw-secondary leading-tight">
          {isRtl ? "اكتشف الخدمات الأكثر تميزاً" : "Premium Services Spotlight"}
        </h2>
        <p class="text-zw-muted text-zw-sm max-w-xl mt-zw-2 leading-relaxed">
          {isRtl
            ? "قائمة منتقاة بعناية من أفضل وأفخم القاعات ومصوري حفلات الزفاف ومقدمي الخدمة لتلبية رغباتك."
            : "Hand-picked luxury ballrooms, elite wedding designers, and verified catering suppliers for your dream night."}
        </p>
      </div>

      <!-- Desktop view more button -->
      <div class="hidden md:block">
        <Button href="/search?tier=diamond" variant="outline" size="md">
          <span>{isRtl ? "عرض جميع الخدمات" : "View All Premium"}</span>
          <span class="ms-zw-2 rtl:rotate-180" aria-hidden="true">→</span>
        </Button>
      </div>
    </div>

    <!-- Listings Grid (Desktop 3-card layout) -->
    <div class="hidden md:grid grid-cols-1 lg:grid-cols-3 gap-zw-6">
      {#each listings.slice(0, 3) as listing}
        <Card elevated class="flex flex-col h-full justify-between">
          <div class="flex flex-col gap-zw-4">
            <!-- Cover image -->
            <div class="relative w-full aspect-[4/3] rounded-zw-xl overflow-hidden bg-zw-border/20 shadow-inner group">
              <img
                src={listing.cover_image || "/categories/wedding-palace.webp"}
                alt={listing.title_en}
                class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
                loading="lazy"
                decoding="async"
                width="400"
                height="300"
              />
              <!-- Verified badge overlay -->
              <span class="absolute top-zw-3 start-zw-3 bg-zw-secondary/85 backdrop-blur-sm text-zw-primary border border-zw-primary/30 text-[9px] font-extrabold tracking-wider uppercase px-zw-2.5 py-zw-1.5 rounded-zw-md shadow-sm">
                ★ {isRtl ? "موثق" : "Verified"}
              </span>
            </div>

            <!-- Meta details -->
            <div class="text-start">
              <div class="flex justify-between items-center mb-zw-1">
                <span class="text-[10px] text-zw-primary font-bold uppercase tracking-wider">
                  {isRtl ? listing.category?.ar || "قاعات" : listing.category?.en || "Venues"}
                </span>
                <span class="text-zw-xs text-zw-muted">
                  📍 {isRtl ? listing.city?.name_ar || "الرياض" : listing.city?.name_en || "Riyadh"}
                </span>
              </div>
              <h3 class="font-display text-zw-lg font-bold text-zw-secondary line-clamp-1 mb-zw-2">
                {isRtl ? listing.title_ar || listing.title_en : listing.title_en}
              </h3>
              <p class="text-zw-muted text-zw-xs line-clamp-2 leading-relaxed">
                {isRtl ? listing.description_ar || listing.description_en : listing.description_en}
              </p>
            </div>
          </div>

          <div class="border-t border-zw-border/50 pt-zw-4 mt-zw-5 flex justify-between items-center">
            <div class="flex flex-col">
              <span class="text-[9px] text-zw-muted font-semibold uppercase">{isRtl ? "يبدأ من" : "Starting from"}</span>
              <span class="text-zw-sm font-bold text-zw-primary-contrast">
                {isRtl ? `ريال ${listing.price || "5,000"}` : `SAR ${listing.price || "5,000"}`}
              </span>
            </div>
            <Button href={`/listings/${listing.slug}`} variant="outline" size="sm">
              <span>{isRtl ? "احجز الآن" : "Details"}</span>
            </Button>
          </div>
        </Card>
      {/each}
    </div>

    <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="md:hidden relative w-full group/carousel"
      onmouseenter={() => { isHovered = true; stopAutoplay(); }}
      onmouseleave={() => { isHovered = false; startAutoplay(); }}
      onfocusin={() => { isHovered = true; stopAutoplay(); }}
      onfocusout={() => { isHovered = false; startAutoplay(); }}
      onkeydown={handleKeyDown}
      role="group"
      aria-label="Featured services carousel"
      tabindex="0"
    >
      <div
        bind:this={sliderEl}
        onscroll={handleScroll}
        class="flex flex-row flex-nowrap items-stretch gap-zw-4 overflow-x-auto scroll-smooth snap-x snap-mandatory hide-scrollbar py-zw-4"
      >
        {#each listings as listing}
          <div class="snap-start shrink-0 w-full snap-always">
            <Card elevated class="flex flex-col h-full justify-between">
              <div class="flex flex-col gap-zw-4">
                <!-- image -->
                <div class="relative w-full aspect-[4/3] rounded-zw-xl overflow-hidden bg-zw-border/20 shadow-inner">
                  <img
                    src={listing.cover_image || "/categories/wedding-palace.webp"}
                    alt={listing.title_en}
                    class="w-full h-full object-cover"
                    loading="lazy"
                    decoding="async"
                    width="400"
                    height="300"
                  />
                  <span class="absolute top-zw-3 start-zw-3 bg-zw-secondary/80 text-zw-primary text-[9px] font-extrabold uppercase px-zw-2 py-zw-1 rounded-zw-md">
                    ★ {isRtl ? "موثق" : "Verified"}
                  </span>
                </div>

                <div class="text-start">
                  <div class="flex justify-between items-center mb-zw-1">
                    <span class="text-[10px] text-zw-primary font-bold uppercase">
                      {isRtl ? listing.category?.ar || "خدمات" : listing.category?.en || "Services"}
                    </span>
                    <span class="text-zw-xs text-zw-muted">
                      📍 {isRtl ? listing.city?.name_ar || "الرياض" : listing.city?.name_en || "Riyadh"}
                    </span>
                  </div>
                  <h3 class="font-display text-zw-lg font-bold text-zw-secondary line-clamp-1 mb-zw-2">
                    {isRtl ? listing.title_ar || listing.title_en : listing.title_en}
                  </h3>
                  <p class="text-zw-muted text-zw-xs line-clamp-2 leading-relaxed">
                    {isRtl ? listing.description_ar || listing.description_en : listing.description_en}
                  </p>
                </div>
              </div>

              <div class="border-t border-zw-border/50 pt-zw-4 mt-zw-5 flex justify-between items-center">
                <div class="flex flex-col">
                  <span class="text-[9px] text-zw-muted font-semibold uppercase">{isRtl ? "يبدأ من" : "Starting from"}</span>
                  <span class="text-zw-sm font-bold text-zw-primary-contrast">
                    {isRtl ? `ريال ${listing.price || "5,000"}` : `SAR ${listing.price || "5,000"}`}
                  </span>
                </div>
                <Button href={`/listings/${listing.slug}`} variant="outline" size="sm">
                  <span>{isRtl ? "احجز الآن" : "Details"}</span>
                </Button>
              </div>
            </Card>
          </div>
        {/each}
      </div>

      <!-- Slide indicator dots -->
      <div class="flex justify-center gap-zw-2 mt-zw-4" aria-label="Slide indicators">
        {#each listings as _, idx}
          <button
            onclick={() => scrollTo(idx)}
            class="w-2 h-2 rounded-zw-full transition-all duration-300
            {idx === activeIndex ? 'bg-zw-primary w-6' : 'bg-zw-border'}"
            aria-label="Slide {idx + 1}"
            aria-current={idx === activeIndex}
          ></button>
        {/each}
      </div>
    </div>

    <!-- Mobile view more CTA -->
    <div class="md:hidden text-center mt-zw-8">
      <Button href="/search?tier=diamond" variant="outline" size="md" class="w-full">
        <span>{isRtl ? "عرض جميع الخدمات" : "View All Premium"}</span>
      </Button>
    </div>
  </div>
</section>

<style>
  /* Scrollbar hiding styles */
  .hide-scrollbar::-webkit-scrollbar {
    display: none;
  }
  .hide-scrollbar {
    -ms-overflow-style: none;
    scrollbar-width: none;
  }
</style>
