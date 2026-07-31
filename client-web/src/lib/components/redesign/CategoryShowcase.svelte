<script lang="ts">
  import { onMount } from "svelte";
  import { getLocale } from "$lib/paraglide/runtime.js";
  import Button from "./Button.svelte";
  import Card from "./Card.svelte";

  interface Props {
    categories?: any[];
  }

  let { categories = [] }: Props = $props();

  const isRtl = $derived(getLocale() === "ar");

  // Lazy loading emulation for categories 3 to 6
  let lazyCategories = $state<any[]>([]);
  let isLoadingLazy = $state(true);

  // We load the first two categories instantly
  const instantCategories = $derived(categories.slice(0, 2));

  onMount(() => {
    // Emulates server-first-paint + client-hydration delay
    const timer = setTimeout(() => {
      lazyCategories = categories.slice(2, 6);
      isLoadingLazy = false;
    }, 1200);
    return () => clearTimeout(timer);
  });
</script>

<section class="py-zw-16 md:py-zw-24 bg-zw-surface relative overflow-hidden" aria-labelledby="categories-showcase-title">
  <div class="w-full max-w-[1400px] mx-auto px-zw-4 sm:px-zw-6 md:px-zw-8 lg:px-zw-12">
    <!-- Section Title -->
    <div class="text-center max-w-2xl mx-auto mb-zw-12">
      <span class="text-zw-primary text-zw-xs font-bold tracking-widest uppercase mb-zw-2 block">
        {isRtl ? "تصفح بحسب الخدمات" : "Browse by Category"}
      </span>
      <h2 id="categories-showcase-title" class="font-display text-zw-3xl sm:text-zw-4xl font-bold text-zw-secondary">
        {isRtl ? "كل ما يلزم لليلة العمر" : "Everything for Your Dream Wedding"}
      </h2>
      <p class="text-zw-muted text-zw-sm mt-zw-2">
        {isRtl ? "اختر الفئة المفضلة لديك وابدأ باستكشاف أرقى قاعات ومزودي خدمات الأفراح." : "Select your category to browse the finest venues and Suppliers across the Gulf region."}
      </p>
    </div>

    <!-- Asymmetrical Grid (palace spans larger, others standard) -->
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-zw-6 mb-zw-16">
      
      <!-- Palace Category (Spans 2 columns on desktop) -->
      <a
        href="/search?category=wedding-palace"
        class="sm:col-span-2 relative rounded-zw-3xl overflow-hidden aspect-[21/9] sm:aspect-[21/10] shadow-zw-md group flex flex-col justify-end p-zw-6 md:p-zw-8 border border-zw-border transition-all duration-300"
      >
        <img
          src="/categories/wedding-palace.webp"
          alt="Wedding Palace Category"
          class="absolute inset-0 w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
          loading="lazy"
          decoding="async"
          width="840"
          height="400"
        />
        <div class="absolute inset-0 bg-gradient-to-t from-black/80 via-black/30 to-transparent z-10"></div>
        <div class="relative z-20 text-start">
          <span class="bg-zw-primary text-zw-secondary text-[9px] font-extrabold uppercase px-zw-3 py-zw-1.5 rounded-zw-md tracking-wider mb-zw-2 inline-block">
            ★ {isRtl ? "الأكثر طلباً" : "Most Popular"}
          </span>
          <h3 class="font-display text-zw-2xl md:text-zw-3xl font-bold text-white mb-zw-1">
            {isRtl ? "قاعات الأفراح والقصور" : "Wedding Halls & Palaces"}
          </h3>
          <p class="text-white/80 text-zw-xs max-w-md">
            {isRtl ? "قصور كلاسيكية فخمة وصالات حفلات واسعة مجهزة بكافة سبل الراحة والجمال." : "Exquisite classical palaces and grand banquet halls equipped with state of the art layouts."}
          </p>
        </div>
      </a>

      <!-- Other standard category boxes -->
      {#each [
        { key: "hotel-venue", name_ar: "فنادق وقاعات", name_en: "Hotel Ballrooms", img: "/categories/hotel-venue.webp", desc_ar: "صالات حفلات فاخرة داخل أشهر الفنادق", desc_en: "Luxurious ballrooms inside world class hotels" },
        { key: "villa-resort", name_ar: "استراحات وفلل", name_en: "Villa & Resort", img: "/categories/villa-resort.webp", desc_ar: "استراحات ريفية وفلل خاصة للحفلات الحميمة", desc_en: "Rural getaways and private estates for boutique events" }
      ] as item}
        <a
          href={`/search?category=${item.key}`}
          class="relative rounded-zw-3xl overflow-hidden aspect-[4/3] sm:aspect-[4/3.5] shadow-zw-md group flex flex-col justify-end p-zw-6 border border-zw-border transition-all duration-300"
        >
          <img
            src={item.img}
            alt={item.name_en}
            class="absolute inset-0 w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
            loading="lazy"
            decoding="async"
            width="400"
            height="300"
          />
          <div class="absolute inset-0 bg-gradient-to-t from-black/80 via-black/20 to-transparent z-10"></div>
          <div class="relative z-20 text-start">
            <h3 class="font-display text-zw-xl font-bold text-white mb-zw-1">
              {isRtl ? item.name_ar : item.name_en}
            </h3>
            <p class="text-white/80 text-zw-xs">
              {isRtl ? item.desc_ar : item.desc_en}
            </p>
          </div>
        </a>
      {/each}
    </div>

    <!-- Dynamic Category Highlights block (Lazy-loaded cards with strict skeleton dimensions to prevent CLS) -->
    <div class="border-t border-zw-border pt-zw-16">
      <div class="text-start mb-zw-8">
        <h3 class="font-display text-zw-2xl font-bold text-zw-secondary">
          {isRtl ? "المزيد من فئات الخدمات" : "Discover Supplier Specialties"}
        </h3>
        <p class="text-zw-muted text-zw-xs mt-zw-1">
          {isRtl ? "فئات إضافية يتم تحميلها ديناميكياً لتسريع استجابة الصفحة الأولى." : "Explore specialized vendor categories that load dynamically to accelerate page load times."}
        </p>
      </div>

      <div class="grid grid-cols-2 md:grid-cols-4 gap-zw-6 text-start">
        <!-- Render Instant categories (Category 1 & 2) -->
        {#each instantCategories as cat}
          <a
            href={`/search?category=${cat.key}`}
            class="bg-zw-surface border border-zw-border hover:border-zw-primary-contrast rounded-zw-2xl p-zw-5 flex flex-col gap-zw-3 transition-all duration-200 hover:shadow-zw-md h-[116px] justify-between"
          >
            <div class="text-zw-2xl select-none" aria-hidden="true">✨</div>
            <div>
              <h4 class="font-display text-zw-sm font-bold text-zw-secondary">
                {isRtl ? cat.labelAr : cat.labelEn}
              </h4>
              <span class="text-[10px] text-zw-muted font-semibold">
                {cat.listingsCount} {isRtl ? "مزود خدمة" : "suppliers"}
              </span>
            </div>
          </a>
        {/each}

        <!-- Render Lazy loaded or Skeleton placeholders (Category 3 to 6) -->
        {#if isLoadingLazy}
          {#each Array(4) as _}
            <!-- Skeleton items EXACTLY matching the final dimensions (fixes CLS) -->
            <div
              class="bg-zw-surface-alt border border-zw-border/50 rounded-zw-2xl p-zw-5 flex flex-col gap-zw-3 animate-pulse h-[116px] justify-between"
              aria-hidden="true"
            >
              <div class="w-8 h-8 rounded-zw-full bg-zw-border/30"></div>
              <div class="flex flex-col gap-zw-1.5">
                <div class="w-3/4 h-3 bg-zw-border/30 rounded animate-pulse"></div>
                <div class="w-1/2 h-2.5 bg-zw-border/30 rounded animate-pulse"></div>
              </div>
            </div>
          {/each}
        {:else}
          {#each lazyCategories as cat}
            <a
              href={`/search?category=${cat.key}`}
              class="bg-zw-surface border border-zw-border hover:border-zw-primary-contrast rounded-zw-2xl p-zw-5 flex flex-col gap-zw-3 transition-all duration-200 hover:shadow-zw-md h-[116px] justify-between animate-fade-in"
            >
              <div class="text-zw-2xl select-none" aria-hidden="true">✨</div>
              <div>
                <h4 class="font-display text-zw-sm font-bold text-zw-secondary">
                  {isRtl ? cat.labelAr : cat.labelEn}
                </h4>
                <span class="text-[10px] text-zw-muted font-semibold">
                  {cat.listingsCount} {isRtl ? "مزود خدمة" : "suppliers"}
                </span>
              </div>
            </a>
          {/each}
        {/if}
      </div>
    </div>
  </div>
</section>
