<script lang="ts">
  import { env } from "$env/dynamic/public";
  import { onMount } from "svelte";
  import * as m from "$lib/paraglide/messages.js";
  import { getLocale } from "$lib/paraglide/runtime.js";
  import { fade } from "svelte/transition";
  import ListingCard from "$lib/components/shared/ListingCard.svelte";

  let listings = $state<any[]>([]);
  let isLoading = $state(true);
  let error = $state(false);

  const isAr = $derived(getLocale() === "ar");

  onMount(async () => {
    try {
      const seed = Math.floor(Math.random() * 1000000);
      const apiBase = env.PUBLIC_API_URL || "http://localhost:8080";
      const res = await fetch(
        `${apiBase}/api/v1/public/listings?tier=diamond&limit=5&seed=${seed}`,
      );

      if (!res.ok) throw new Error("Failed to fetch");
      const data = await res.json();

      if (data.status === "success" && data.listings) {
        listings = data.listings;
      } else {
        error = true;
      }
    } catch (e) {
      console.error("Diamond Showcase Fetch Error:", e);
      error = true;
    } finally {
      isLoading = false;
    }
  });

  // Carousel slider state & logic
  let sliderEl = $state<HTMLDivElement | null>(null);

  function scrollNext() {
    if (!sliderEl) return;
    const direction = isAr ? -1 : 1;
    sliderEl.scrollBy({ left: 320 * direction, behavior: "smooth" });
  }

  function scrollPrev() {
    if (!sliderEl) return;
    const direction = isAr ? -1 : 1;
    sliderEl.scrollBy({ left: -320 * direction, behavior: "smooth" });
  }
</script>

{#if isLoading}
  <section
    class="py-12 bg-[#F6F6F6] min-h-[400px] flex items-center justify-center"
  >
    <div class="animate-pulse flex flex-col items-center">
      <div
        class="w-12 h-12 rounded-full border-4 border-[#CC4869] border-t-transparent animate-spin mb-4"
      ></div>
      <p class="text-[#CC4869] font-medium text-sm">
        {isAr ? "جاري التحميل..." : "Loading..."}
      </p>
    </div>
  </section>
{:else if !error && listings.length > 0}
  <section class="relative py-16 bg-[#F6F6F6] overflow-hidden" transition:fade>
    <div class="container-page relative z-10">
      <!-- Section Header -->
      <div class="text-center mb-10">
        <h2 class="text-3xl sm:text-4xl font-bold text-[#111111] mb-3">
          {isAr
            ? "أفضل مزودي خدمات الزفاف المميزين"
            : "The best premium wedding providers"}
        </h2>
        <p class="text-[#555555] text-base max-w-2xl mx-auto">
          {isAr
            ? "تصفح خيارات عالم الزفاف المميزة من قائمة مزودي خدمات الزفاف"
            : "Browse the premium wedding world options from the list of wedding service providers"}
        </p>
      </div>

      <!-- Carousel Wrapper with outside arrows -->
      <div class="relative flex items-center group">
        <!-- Left Arrow -->
        <button
          onclick={scrollPrev}
          class="absolute -start-4 md:-start-10 z-20 w-10 h-10 flex items-center justify-center text-[#CC4869] hover:scale-110 transition-transform"
          aria-label="Previous"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke-width="3"
            stroke="currentColor"
            class="w-6 h-6 rtl:rotate-180"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M15.75 19.5L8.25 12l7.5-7.5"
            />
          </svg>
        </button>

        <!-- Horizontal Scrolling Container -->
        <div
          bind:this={sliderEl}
          class="flex flex-row flex-nowrap items-stretch gap-5 overflow-x-auto py-4 scroll-smooth snap-x snap-mandatory select-none hide-scrollbar w-full"
        >
          {#each listings as listing (listing.id)}
            <div class="snap-start shrink-0 w-[260px] sm:w-[280px] md:w-[300px]">
              <ListingCard {listing} layout="grid" size="md" class="w-full h-full" />
            </div>
          {/each}
        </div>

        <!-- Right Arrow -->
        <button
          onclick={scrollNext}
          class="absolute -end-4 md:-end-10 z-20 w-10 h-10 flex items-center justify-center text-[#CC4869] hover:scale-110 transition-transform"
          aria-label="Next"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 24 24"
            stroke-width="3"
            stroke="currentColor"
            class="w-6 h-6 rtl:rotate-180"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M8.25 4.5l7.5 7.5-7.5 7.5"
            />
          </svg>
        </button>
      </div>

      <!-- View More Button -->
      <div class="mt-12 text-center">
        <a
          href="/"
          class="inline-block px-10 py-3.5 rounded-full font-bold text-sm bg-[#F6A7B5] hover:bg-[#e293a1] text-[#2D2620] transition-all duration-300 shadow-md text-center"
        >
          {isAr ? "عرض المزيد" : "View More"}
        </a>
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
{/if}
