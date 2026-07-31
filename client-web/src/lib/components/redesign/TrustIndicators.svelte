<script lang="ts">
  import { getLocale } from "$lib/paraglide/runtime.js";

  const isRtl = $derived(getLocale() === "ar");

  // Define 5 generic partner brands
  const partners = $derived([
    { name: isRtl ? "فندق الرياض الفاخر" : "Riyadh Luxury Hotel", icon: "🏛️" },
    { name: isRtl ? "قصر الأفراح الملكي" : "Royal Wedding Palace", icon: "🏰" },
    { name: isRtl ? "مجموعة النخبة للمناسبات" : "Elite Events Group", icon: "✨" },
    { name: isRtl ? "شركة الضيافة الشرقية" : "Eastern Hospitality Co", icon: "🍽️" },
    { name: isRtl ? "تصاميم الذهب للديكور" : "Golden Glow Decor", icon: "🎨" }
  ]);

  // Double the list for seamless marquee scroll loop on mobile
  const marqueeItems = $derived([...partners, ...partners]);
</script>

<div class="py-zw-8 bg-zw-surface border-y border-zw-border/40 overflow-hidden relative select-none">
  <div class="w-full max-w-[1400px] mx-auto px-zw-4 sm:px-zw-6 md:px-zw-8 lg:px-zw-12">
    <!-- Desktop static horizontal grid -->
    <div class="hidden md:flex items-center justify-between gap-zw-6 flex-wrap opacity-60">
      {#each partners as partner}
        <div class="flex items-center gap-zw-2 filter grayscale hover:grayscale-0 transition-all duration-300">
          <span class="text-zw-2xl" aria-hidden="true">{partner.icon}</span>
          <span class="font-display text-zw-xs font-bold tracking-wide text-zw-secondary uppercase">
            {partner.name}
          </span>
        </div>
      {/each}
    </div>
  </div>

  <!-- Mobile auto-scrolling marquee ticker -->
  <div class="md:hidden relative w-full flex items-center overflow-hidden">
    <!-- Fade-out masks -->
    <div class="absolute inset-y-0 left-0 w-zw-12 bg-gradient-to-r from-zw-surface to-transparent z-10 pointer-events-none" aria-hidden="true"></div>
    <div class="absolute inset-y-0 right-0 w-zw-12 bg-gradient-to-l from-zw-surface to-transparent z-10 pointer-events-none" aria-hidden="true"></div>

    <!-- Scrolling Track -->
    <div
      class="flex flex-row flex-nowrap gap-zw-8 shrink-0 min-w-full animate-marquee motion-reduce:animate-none motion-reduce:flex-wrap motion-reduce:justify-center opacity-65"
      role="region"
      aria-label={isRtl ? "شركاء زفاف وورلد" : "ZafafWorld Partners"}
    >
      {#each marqueeItems as item}
        <div class="flex items-center gap-zw-2 shrink-0 select-none">
          <span class="text-zw-xl" aria-hidden="true">{item.icon}</span>
          <span class="font-display text-zw-xs font-bold tracking-wider text-zw-secondary uppercase">
            {item.name}
          </span>
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
  /* Infinite horizontal scroll animation */
  .animate-marquee {
    display: flex;
    animation: marquee-scroll 20s linear infinite;
  }

  @keyframes marquee-scroll {
    0% { transform: translateX(0%); }
    100% { transform: translateX(-50%); }
  }

  /* Direction-aware marquee alignment for RTL viewports */
  :global([dir="rtl"]) .animate-marquee {
    animation: marquee-scroll-rtl 20s linear infinite;
  }

  @keyframes marquee-scroll-rtl {
    0% { transform: translateX(0%); }
    100% { transform: translateX(50%); }
  }
</style>
