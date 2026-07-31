<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { getLocale } from "$lib/paraglide/runtime.js";
  import * as m from "$lib/paraglide/messages.js";
  import { getLocalizedField } from "$lib/utils/localize.js";
  import Card from "./Card.svelte";

  interface Props {
    testimonials?: any[];
  }

  let { testimonials = [] }: Props = $props();

  const isRtl = $derived(getLocale() === "ar");

  // Mobile slider state & keyboard controls
  let sliderEl = $state<HTMLDivElement | null>(null);
  let activeIndex = $state(0);
  let isHovered = $state(false);
  let autoplayInterval: any = null;

  function handleScroll() {
    if (!sliderEl) return;
    const scrollLeft = Math.abs(sliderEl.scrollLeft);
    const width = sliderEl.clientWidth;
    activeIndex = Math.min(
      Math.round(scrollLeft / (width || 1)),
      testimonials.length - 1
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

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "ArrowRight") {
      e.preventDefault();
      const nextIndex = isRtl
        ? Math.max(activeIndex - 1, 0)
        : Math.min(activeIndex + 1, testimonials.length - 1);
      scrollTo(nextIndex);
    } else if (e.key === "ArrowLeft") {
      e.preventDefault();
      const prevIndex = isRtl
        ? Math.min(activeIndex + 1, testimonials.length - 1)
        : Math.max(activeIndex - 1, 0);
      scrollTo(prevIndex);
    }
  }

  function startAutoplay() {
    if (typeof window !== "undefined" && window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      return;
    }
    stopAutoplay();
    autoplayInterval = setInterval(() => {
      if (!isHovered && testimonials.length > 0) {
        const nextIndex = (activeIndex + 1) % testimonials.length;
        scrollTo(nextIndex);
      }
    }, 5000);
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

<section
  class="py-zw-16 md:py-zw-24 bg-zw-surface-alt relative overflow-hidden"
  aria-labelledby="testimonials-title"
>
  <div class="w-full max-w-[1400px] mx-auto px-zw-4 sm:px-zw-6 md:px-zw-8 lg:px-zw-12">
    <!-- Header -->
    <div class="text-center max-w-2xl mx-auto mb-zw-12">
      <span class="text-zw-primary text-zw-xs font-bold tracking-widest uppercase mb-zw-2 block">
        {isRtl ? "قصص النجاح" : "Success Stories"}
      </span>
      <h2 id="testimonials-title" class="font-display text-zw-3xl sm:text-zw-4xl font-bold text-zw-secondary leading-tight">
        {isRtl ? "ماذا يقول شركاء السعادة؟" : "What Happy Couples Say"}
      </h2>
      <p class="text-zw-muted text-zw-sm mt-zw-2 leading-relaxed">
        {isRtl
          ? "تجارب حقيقية من أشخاص وثقوا بـ زفاف وورلد لحجز وتنسيق حفلات زفافهم الأسطورية."
          : "Genuine stories from couples who trusted ZafafWorld to manage, coordinate, and host their special day."}
      </p>
    </div>

    <!-- Desktop Testimonial Cards Grid -->
    <div class="hidden md:grid md:grid-cols-3 gap-zw-6">
      {#each testimonials.slice(0, 3) as t}
        <article
          class="bg-zw-surface rounded-zw-2xl p-zw-6 md:p-zw-8 border border-zw-border hover:border-zw-border-hover shadow-zw-sm hover:shadow-zw-md transition-all duration-300 flex flex-col justify-between"
          aria-label={getLocalizedField(t, "name", getLocale())}
        >
          <div class="flex flex-col gap-zw-4">
            <!-- Stars + Quote -->
            <div class="flex justify-between items-center select-none">
              <!-- Star rating SVG loop -->
              <div class="flex gap-zw-1" aria-label={`Rating: ${t.rating || 5} stars`}>
                {#each Array(5) as _, idx}
                  <svg
                    class="w-4 h-4 {idx < (t.rating || 5) ? 'text-zw-primary' : 'text-zw-border'}"
                    fill="currentColor"
                    viewBox="0 0 20 20"
                    aria-hidden="true"
                  >
                    <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
                  </svg>
                {/each}
              </div>
              <span class="text-zw-primary/20 font-display text-zw-3xl leading-none" aria-hidden="true">“</span>
            </div>

            <!-- Testimonial Text -->
            <p class="text-zw-secondary text-zw-xs leading-relaxed text-start">
              "{getLocalizedField(t, "text", getLocale())}"
            </p>
          </div>

          <!-- Bottom Customer Meta Card -->
          <div class="flex items-center gap-zw-4 pt-zw-4 border-t border-zw-border mt-zw-6">
            <!-- Avatar container with reserved space -->
            <div class="w-12 h-12 rounded-zw-full overflow-hidden shrink-0 border border-zw-primary/30 bg-zw-border/20">
              <img
                src={t.image || "/categories/hair-makeup.webp"}
                alt={getLocalizedField(t, "name", getLocale())}
                class="w-full h-full object-cover"
                loading="lazy"
                decoding="async"
                width="48"
                height="48"
              />
            </div>
            
            <div class="text-start">
              <p class="font-display text-zw-xs font-bold text-zw-secondary leading-tight">
                {getLocalizedField(t, "name", getLocale())}
              </p>
              <div class="flex flex-wrap gap-x-zw-2 text-[10px] text-zw-muted font-medium mt-zw-1">
                <span>📍 {getLocalizedField(t.city || t, "name", getLocale())}</span>
                <span class="text-zw-primary">•</span>
                <span>📅 {t.weddingDate || (isRtl ? "أكتوبر ٢٠٢٥" : "Oct 2025")}</span>
                <span class="text-zw-primary">•</span>
                <span class="text-zw-primary-contrast font-bold">💍 {t.vendorUsed || (isRtl ? "القصر الملكي" : "Royal Palace")}</span>
              </div>
            </div>
          </div>
        </article>
      {/each}
    </div>

    <!-- Mobile Swipe Carousel -->
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
      aria-label={isRtl ? "آراء العملاء" : "Couples reviews slider"}
      tabindex="0"
    >
      <div
        bind:this={sliderEl}
        onscroll={handleScroll}
        class="flex flex-row flex-nowrap items-stretch gap-zw-4 overflow-x-auto scroll-smooth snap-x snap-mandatory hide-scrollbar py-zw-2"
        aria-live="polite"
      >
        {#each testimonials as t}
          <div class="snap-start shrink-0 w-full snap-always">
            <article
              class="bg-zw-surface rounded-zw-2xl p-zw-6 border border-zw-border shadow-zw-sm flex flex-col justify-between h-full"
              aria-label={getLocalizedField(t, "name", getLocale())}
            >
              <div class="flex flex-col gap-zw-4">
                <div class="flex justify-between items-center select-none">
                  <div class="flex gap-zw-1" aria-label={`Rating: ${t.rating || 5} stars`}>
                    {#each Array(5) as _, idx}
                      <svg
                        class="w-4 h-4 {idx < (t.rating || 5) ? 'text-zw-primary' : 'text-zw-border'}"
                        fill="currentColor"
                        viewBox="0 0 20 20"
                        aria-hidden="true"
                      >
                        <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
                      </svg>
                    {/each}
                  </div>
                  <span class="text-zw-primary/20 font-display text-zw-2xl" aria-hidden="true">“</span>
                </div>
                <p class="text-zw-secondary text-zw-xs leading-relaxed text-start">
                  "{getLocalizedField(t, "text", getLocale())}"
                </p>
              </div>

              <div class="flex items-center gap-zw-4 pt-zw-4 border-t border-zw-border mt-zw-5">
                <div class="w-10 h-10 rounded-zw-full overflow-hidden shrink-0 border border-zw-primary/30 bg-zw-border/20">
                  <img
                    src={t.image || "/categories/hair-makeup.webp"}
                    alt={getLocalizedField(t, "name", getLocale())}
                    class="w-full h-full object-cover"
                    loading="lazy"
                    decoding="async"
                    width="40"
                    height="40"
                  />
                </div>
                <div class="text-start">
                  <p class="font-display text-zw-xs font-bold text-zw-secondary leading-tight">
                    {getLocalizedField(t, "name", getLocale())}
                  </p>
                  <div class="flex flex-wrap gap-x-zw-2 text-[10px] text-zw-muted mt-zw-1">
                    <span>📍 {getLocalizedField(t.city || t, "name", getLocale())}</span>
                    <span class="text-zw-primary">•</span>
                    <span>💍 {t.vendorUsed || (isRtl ? "القصر الملكي" : "Royal Palace")}</span>
                  </div>
                </div>
              </div>
            </article>
          </div>
        {/each}
      </div>

      <!-- Slide indicator dots -->
      <div class="flex justify-center gap-zw-2 mt-zw-4" aria-label="Slide indicators">
        {#each testimonials as _, idx}
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
  </div>
</section>

<style>
  .hide-scrollbar::-webkit-scrollbar {
    display: none;
  }
  .hide-scrollbar {
    -ms-overflow-style: none;
    scrollbar-width: none;
  }
</style>
