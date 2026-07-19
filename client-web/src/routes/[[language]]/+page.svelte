<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import * as m from '$lib/paraglide/messages.js';
  import { getLocale } from '$lib/paraglide/runtime.js';
  import { getLocalizedField } from '$lib/utils/localize.js';
  import HeroSection from '$lib/components/features/home/HeroSection.svelte';
  import DiamondShowcase from '$lib/components/features/home/DiamondShowcase.svelte';
  import HowItWorksSection from '$lib/components/features/home/HowItWorksSection.svelte';
  import StatsSection from '$lib/components/features/home/StatsSection.svelte';
  import TestimonialsSection from '$lib/components/features/home/TestimonialsSection.svelte';
  import CTASection from '$lib/components/features/home/CTASection.svelte';
  import ListingCard from '$lib/components/shared/ListingCard.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import { buildFilteredRoute } from '$lib/utils/navigation.js';
  import { getCategoryImageUrl } from '$lib/constants/categoryImages';
  import { listingService } from '$lib/services/api/listing.service.js';

  let { data }: { data: any } = $props();

  // svelte-ignore state_referenced_locally
  let categoryGroups = $state(data.categoryGroups || []);
  // svelte-ignore state_referenced_locally
  let pendingCategories = $state(data.pendingCategories || []);
  let isLoadingMore = $state(false);
  let sentinelEl = $state<HTMLDivElement | null>(null);

  $effect(() => {
    categoryGroups = data.categoryGroups || [];
    pendingCategories = data.pendingCategories || [];
  });

  async function loadNextCategory() {
    if (isLoadingMore || pendingCategories.length === 0) return;
    isLoadingMore = true;

    const nextCat = pendingCategories[0];
    pendingCategories = pendingCategories.slice(1);

    try {
      const result = await listingService.getAll({
        category: nextCat.key,
        limit: 5,
        countryId: data.selectedCountry || 'sa',
        sort: 'weighted'
      });

      const listings = result.listings || [];
      const loadedGroup = {
        ...nextCat,
        listings
      };

      categoryGroups = [...categoryGroups, loadedGroup];
    } catch (err) {
      console.error(`Failed to load listings for category ${nextCat.key}:`, err);
      pendingCategories = [nextCat, ...pendingCategories];
    } finally {
      isLoadingMore = false;
    }
  }

  $effect(() => {
    const el = sentinelEl;
    if (el) {
      const obs = new IntersectionObserver((entries) => {
        const first = entries[0];
        if (first.isIntersecting) {
          loadNextCategory();
        }
      }, {
        rootMargin: '300px'
      });
      obs.observe(el);
      return () => {
        obs.disconnect();
      };
    }
  });

  const categories = $derived(
    (data.metadata?.categories || []).map((c: any) => ({
      key: c.key,
      icon: c.icon,
      labelAr: c.labelAr || c.key,
      labelEn: c.labelEn || c.key,
      listingsCount: c.listingsCount || 0,
      href: buildFilteredRoute('/listings', { category: c.key })
    }))
  );

  // Top 3 category keys to be fixed at the top
  const top3Keys = ['wedding-invitation', 'photography-video', 'wedding-palace'];

  // Derived arrays for fixed and dynamic categories
  const fixedCategories = $derived(
    top3Keys
      .map(key => categories.find((c: any) => c.key === key))
      .filter((c): c is NonNullable<typeof c> => !!c)
  );

  const dynamicCategories = $derived(
    categories.filter((c: any) => !top3Keys.includes(c.key))
  );



  // Helper for localization pluralization rules
  function getProviderCountLabel(count: number, locale: string) {
    if (locale === 'ar') {
      if (count === 1) return 'مزود خدمة واحد';
      if (count === 2) return 'مزودا خدمة';
      if (count >= 3 && count <= 10) return `${count} مزودي خدمات`;
      return `${count} مزود خدمة`;
    }
    return `${count} ${count === 1 ? 'Service Provider' : 'Service Providers'}`;
  }

  // Carousel slider state & logic
  let sliderEl = $state<HTMLDivElement | null>(null);
  let isHovered = $state(false);
  let autoplayInterval: any = null;

  function scrollNext() {
    if (!sliderEl) return;
    const isRtl = getLocale() === 'ar';
    const currentScroll = sliderEl.scrollLeft;
    const scrollWidth = sliderEl.scrollWidth;
    const clientWidth = sliderEl.clientWidth;
    const maxScroll = scrollWidth - clientWidth;

    let isAtEnd = false;
    if (isRtl) {
      isAtEnd = Math.abs(currentScroll) >= maxScroll - 20;
    } else {
      isAtEnd = currentScroll >= maxScroll - 20;
    }

    if (isAtEnd) {
      sliderEl.scrollTo({ left: 0, behavior: 'smooth' });
    } else {
      const scrollAmount = 300; // standard width of card + gap
      const direction = isRtl ? -1 : 1;
      sliderEl.scrollBy({ left: scrollAmount * direction, behavior: 'smooth' });
    }
  }

  function scrollPrev() {
    if (!sliderEl) return;
    const isRtl = getLocale() === 'ar';
    const currentScroll = sliderEl.scrollLeft;
    const scrollWidth = sliderEl.scrollWidth;
    const clientWidth = sliderEl.clientWidth;
    const maxScroll = scrollWidth - clientWidth;

    let isAtStart = false;
    if (isRtl) {
      isAtStart = Math.abs(currentScroll) <= 20;
    } else {
      isAtStart = currentScroll <= 20;
    }

    if (isAtStart) {
      sliderEl.scrollTo({ left: isRtl ? -maxScroll : maxScroll, behavior: 'smooth' });
    } else {
      const scrollAmount = 300;
      const direction = isRtl ? -1 : 1;
      sliderEl.scrollBy({ left: -scrollAmount * direction, behavior: 'smooth' });
    }
  }

  function startAutoplay() {
    stopAutoplay();
    autoplayInterval = setInterval(() => {
      if (!isHovered) {
        scrollNext();
      }
    }, 4000);
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

<svelte:head>
  <title>{m.meta_siteName()} - {m.meta_tagline()}</title>
  <meta name="description" content={m.meta_description()} />
  <meta name="keywords" content="كاترينج, قاعات افراح, استراحات, زفاف, قصور افراح, عروس زفاف, موقع قاعات افراح, موقع حجز قاعات الافراح, قاعة افراح رجال, صور قاعه افراح, قاعه أفراح, صالات زفاف, صور قاعات الافراح, افراح قاعه, قاعات حفلات, افضل قاعات الافراح, قاعات الأفراح, قاعات افراح واسعارها, قاعة الأفراح, اسم قاعات افراح, القاعات الافراح, صالات أفراح, صور صالات اعراس, صور صالات افراح, صور صالات زفاف, صور قاعات أفراح, صور قاعات زفاف, صور قاعة الافراح, قاعات خطوبة, قاعات زواجات, قاعات مفتوحه, قاعة افراح مفتوحة, صور قصور افراح, قصور الافراح, استوديو تصوير, قاعات افراح داخل فنادق, قاعات افراح فى جدة, قاعات افراح فى السعودية, قاعات افراح فى الرياض, قاعات داخل فنادق, كيك زفاف, تصوير افراح, استوديوهات افراح, مجوهرات و خواتم, زفة عروسة, خصومات على قاعات الافراح, افضل قاعات الافراح, افضل قاعة فى الرياض, افضل قاعة افراح فى السعودية, احسن قاعة فى السعودية, احسن قاعة افراح فى جدة, استراحات و فلل, قاعات مفتوحة, فساتين زفاف, فستان زفاف, فستان عروس, قاعات الرياض, فساتين عروس, منظم حفلات, زفاف نت, حفل زفاف, موقع زفاف للقاعات, صالة زفاف, قاعة مذهلة بالرياض, قصر مذهله الرياض, زفاف نت جدة, قصر زفاف, مكان زفاف, قصر مذهلة, افراح زفاف, فرح زفاف, قاعات زفاف, منظم افراح, حفل زفاف العريس, زفاف نت الرياض, تصوير فستان العروس" />
  <script type="application/ld+json">
  {@html JSON.stringify({
    "@context": "https://schema.org",
    "@type": "Organization",
    "name": "ZafafWorld",
    "url": "https://zafafworld.net",
    "logo": "https://zafafworld.net/favicon.webp",
    "sameAs": [
      "https://instagram.com/zafafworld",
      "https://twitter.com/zafafworld",
      "https://facebook.com/zafafworld"
    ],
    "contactPoint": {
      "@type": "ContactPoint",
      "telephone": "+966-50-000-0000",
      "contactType": "customer service",
      "areaServed": "SA",
      "availableLanguage": ["Arabic", "English"]
    }
  }).replace(/</g, '\\x3c')}
  </script>
</svelte:head>

<!-- Hero Section with modern search widget -->
<HeroSection
  categories={data?.metadata?.categories || []}
  cities={data?.metadata?.cities || []}
/>

<!-- Diamond Vendors Showcase -->
<DiamondShowcase />

<!-- Categories Showcase Section -->
<section class="py-20 bg-[var(--color-surface-alt)] relative overflow-hidden" aria-labelledby="categories-title">
  <div class="container-page">
    <div class="text-center mb-12">
      <span class="divider-gold mx-auto"></span>
      <h2 id="categories-title" class="font-display text-3xl sm:text-4xl font-bold text-[var(--color-secondary)] mt-6 mb-4">
        {m.home_categories()}
      </h2>
      <p class="text-[var(--color-muted)] text-base max-w-2xl mx-auto leading-relaxed">{m.home_categoriesSubtitle()}</p>
    </div>

    <!-- TOP 3 FIXED GRID SHOWCASE -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-12">
      {#each fixedCategories as cat}
        <a
          href={cat.href}
          data-sveltekit-preload-data="hover"
          class="group flex flex-col bg-white border border-[var(--color-border)]/65 rounded-3xl p-4
            hover:border-[var(--color-primary)]/40 hover:shadow-[var(--shadow-gold)]
            transition-all duration-500
            {cat.key === 'wedding-palace' ? 'lg:col-span-2 md:col-span-2' : 'col-span-1'}"
        >
          <!-- Image Container with aspect ratios -->
          <div class="relative w-full overflow-hidden rounded-2xl bg-gray-100 shadow-inner
            {cat.key === 'wedding-palace' ? 'aspect-[2.1/1]' : 'aspect-[4/3]'}">
            <img
              src={getCategoryImageUrl(cat.key)}
              alt={getLocalizedField(cat, 'label', getLocale())}
              class="w-full h-full object-cover transform group-hover:scale-105 transition-transform duration-700 ease-out"
              loading="lazy"
              width={cat.key === 'wedding-palace' ? 540 : 260}
              height={cat.key === 'wedding-palace' ? 257 : 195}
            />
            <div class="absolute inset-0 bg-black/5 group-hover:bg-black/0 transition-colors duration-500"></div>
          </div>
          <!-- Text content centered below the image -->
          <div class="text-center mt-4 mb-2 flex-grow flex flex-col justify-center">
            <h3 class="font-display text-base sm:text-lg font-bold text-[var(--color-secondary)] group-hover:text-[var(--color-primary)] transition-colors duration-300">
              {getLocalizedField(cat, 'label', getLocale())}
            </h3>
            <p class="text-xs text-[var(--color-muted)] font-medium mt-1">
              {getProviderCountLabel(cat.listingsCount || 0, getLocale())}
            </p>
          </div>
        </a>
      {/each}
    </div>

    <!-- DYNAMIC CAROUSEL SECTION -->
    <div class="relative w-full group/carousel mt-12">
      <!-- Left Navigation Button (Floating, visible on hover, hidden on touch screens) -->
      <button
        onclick={scrollPrev}
        class="absolute top-1/2 -translate-y-1/2 start-0 -ms-4 z-20
          w-10 h-10 rounded-full bg-white/90 border border-[var(--color-border)] shadow-md
          flex items-center justify-center text-[var(--color-secondary)]
          hover:bg-[var(--color-primary)] hover:text-[var(--color-secondary)] hover:border-[var(--color-primary)]
          opacity-0 group-hover/carousel:opacity-100 transition-all duration-300
          hidden md:flex focus-visible:opacity-100"
        aria-label={getLocale() === 'ar' ? 'السابق' : 'Previous Category'}
      >
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2.5" stroke="currentColor" class="w-5 h-5 rtl:rotate-180">
          <path stroke-linecap="round" stroke-linejoin="round" d="M15.75 19.5L8.25 12l7.5-7.5" />
        </svg>
      </button>

      <!-- Right Navigation Button (Floating, visible on hover, hidden on touch screens) -->
      <button
        onclick={scrollNext}
        class="absolute top-1/2 -translate-y-1/2 end-0 -me-4 z-20
          w-10 h-10 rounded-full bg-white/90 border border-[var(--color-border)] shadow-md
          flex items-center justify-center text-[var(--color-secondary)]
          hover:bg-[var(--color-primary)] hover:text-[var(--color-secondary)] hover:border-[var(--color-primary)]
          opacity-0 group-hover/carousel:opacity-100 transition-all duration-300
          hidden md:flex focus-visible:opacity-100"
        aria-label={getLocale() === 'ar' ? 'التالي' : 'Next Category'}
      >
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2.5" stroke="currentColor" class="w-5 h-5 rtl:rotate-180">
          <path stroke-linecap="round" stroke-linejoin="round" d="M8.25 4.5l7.5 7.5-7.5 7.5" />
        </svg>
      </button>

      <!-- Gradient fades for elegant visual edges -->
      <div class="absolute inset-y-0 start-0 w-16 carousel-fade-start z-10 pointer-events-none" aria-hidden="true"></div>
      <div class="absolute inset-y-0 end-0 w-16 carousel-fade-end z-10 pointer-events-none" aria-hidden="true"></div>

      <!-- Horizontal scrolling track -->
      <div
        bind:this={sliderEl}
        onmouseenter={() => { isHovered = true; }}
        onmouseleave={() => { isHovered = false; }}
        class="categories-scroller
          flex flex-row flex-nowrap items-stretch gap-6 overflow-x-auto py-4 px-4 md:px-8
          scroll-smooth snap-x snap-mandatory select-none"
        role="region"
        aria-label={getLocale() === 'ar' ? 'تصنيفات إضافية' : 'More Categories'}
      >
        {#each dynamicCategories as cat (cat.key)}
          <a
            href={cat.href}
            data-sveltekit-preload-data="hover"
            class="group flex flex-col bg-white border border-[var(--color-border)]/65 rounded-3xl p-4
              hover:border-[var(--color-primary)]/40 hover:shadow-[var(--shadow-gold)]
              transition-all duration-500 snap-start flex-shrink-0
              w-[240px] sm:w-[260px] md:w-[280px]"
          >
            <!-- Image Container -->
            <div class="relative w-full aspect-[4/3] overflow-hidden rounded-2xl bg-gray-100 shadow-inner">
              <img
                src={getCategoryImageUrl(cat.key)}
                alt={getLocalizedField(cat, 'label', getLocale())}
                class="w-full h-full object-cover transform group-hover:scale-105 transition-transform duration-700 ease-out"
                loading="lazy"
                width="280"
                height="210"
              />
              <div class="absolute inset-0 bg-black/5 group-hover:bg-black/0 transition-colors duration-500"></div>
            </div>
            <!-- Text content centered below the image -->
            <div class="text-center mt-4 mb-2 flex-grow flex flex-col justify-center">
              <h3 class="font-display text-sm sm:text-base font-bold text-[var(--color-secondary)] group-hover:text-[var(--color-primary)] transition-colors duration-300">
                {getLocalizedField(cat, 'label', getLocale())}
              </h3>
              <p class="text-xs text-[var(--color-muted)] font-medium mt-1">
                {getProviderCountLabel(cat.listingsCount || 0, getLocale())}
              </p>
            </div>
          </a>
        {/each}
      </div>
    </div>

    <!-- Show All Categories Link -->
    <div class="text-center mt-12">
      <Button href="/listings" variant="outline" size="md" class="group hover:border-[var(--color-primary)] hover:text-[var(--color-primary)] transition-colors">
        {m.common_showAll()}
        <svg viewBox="0 0 20 20" class="w-4 h-4 rtl:rotate-180 ms-2 inline-block transition-transform duration-300 group-hover:translate-x-1 rtl:group-hover:-translate-x-1" fill="currentColor">
          <path fill-rule="evenodd" d="M3 10a.75.75 0 01.75-.75h10.638L10.23 5.29a.75.75 0 111.04-1.08l5.5 5.25a.75.75 0 010 1.08l-5.5 5.25a.75.75 0 11-1.04-1.08l4.158-3.96H3.75A.75.75 0 013 10z" clip-rule="evenodd"/>
        </svg>
      </Button>
    </div>
  </div>
</section>

<!-- Dynamic Multi-Layout Category Sections (Phase 1 Premium Redesign) -->
{#each categoryGroups as group (group.key)}
  <section class="py-24 bg-white border-b border-[var(--color-border)] last:border-b-0" aria-labelledby="section-{group.key}">
    <div class="container-page">
      <!-- Header Row -->
      <div class="flex items-end justify-between mb-10 gap-4 flex-wrap">
        <div>
          <span class="divider-gold"></span>
          <h2 id="section-{group.key}" class="font-display text-2xl sm:text-3xl font-bold text-[var(--color-secondary)] mt-4 mb-2">
            {getLocalizedField(group, 'title', getLocale())}
          </h2>
          <p class="text-[var(--color-muted)] text-sm">{getLocalizedField(group, 'subtitle', getLocale())}</p>
        </div>
        <Button href={group.href} variant="outline" size="sm">
          {m.common_showAll()}
          <svg viewBox="0 0 20 20" class="w-4 h-4 rtl:rotate-180" fill="currentColor">
            <path fill-rule="evenodd" d="M3 10a.75.75 0 01.75-.75h10.638L10.23 5.29a.75.75 0 111.04-1.08l5.5 5.25a.75.75 0 010 1.08l-5.5 5.25a.75.75 0 11-1.04-1.08l4.158-3.96H3.75A.75.75 0 013 10z" clip-rule="evenodd"/>
          </svg>
        </Button>
      </div>

      <!-- Layout Selection -->
      {#if group.listings.length === 0}
        <div class="flex flex-col items-center justify-center py-16 px-6 bg-[var(--color-surface-alt)]/40 border border-dashed border-[var(--color-border)]/80 rounded-3xl text-center backdrop-blur-sm transition-all duration-300 hover:border-[var(--color-primary)]/40">
          <div class="w-14 h-14 rounded-full bg-white flex items-center justify-center shadow-sm text-2xl mb-4 border border-[var(--color-border)]/50">
            ✨
          </div>
          <h3 class="font-display text-base sm:text-lg font-bold text-[var(--color-secondary)] mb-2">
            {getLocale() === 'ar' ? 'قريباً جداً' : 'Coming Soon'}
          </h3>
          <p class="text-xs sm:text-sm text-[var(--color-muted)] max-w-md">
            {getLocale() === 'ar' 
              ? 'نعمل حالياً على اختيار وتوثيق أفضل مزودي الخدمة في هذا القسم.' 
              : 'We are currently selecting and verifying the finest service providers for this section.'}
          </p>
        </div>
      {:else if group.layoutType === 'A'}
        <!-- ── LAYOUT A: THE HERO GRID (For Wedding Palaces & Halls) ── -->
        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <!-- Massive primary feature card -->
          {#if group.listings.length > 0}
            <div class="lg:col-span-2">
              <ListingCard listing={group.listings[0]} aspectRatio="landscape" class="h-full flex flex-col justify-between" />
            </div>
          {/if}

          <!-- Standard smaller cards next to it -->
          <div class="flex flex-col gap-6">
            {#each group.listings.slice(1, 3) as listing (listing.id)}
              <ListingCard {listing} aspectRatio="landscape" />
            {/each}
          </div>
        </div>

        {#if group.listings.length > 3}
          <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mt-6">
            {#each group.listings.slice(3, 5) as listing (listing.id)}
              <ListingCard {listing} aspectRatio="landscape" />
            {/each}
          </div>
        {/if}

      {:else if group.layoutType === 'B'}
        <!-- ── LAYOUT B: PORTRAIT MASONRY/CAROUSEL (For Gowns & Photographers) ── -->
        <div class="relative w-full">
          <div class="absolute inset-y-0 start-0 w-12 carousel-fade-start z-10 pointer-events-none" aria-hidden="true"></div>
          <div class="absolute inset-y-0 end-0 w-12 carousel-fade-end z-10 pointer-events-none" aria-hidden="true"></div>

          <div
            class="row-scroller
              flex flex-row flex-nowrap gap-6 overflow-x-auto py-4 px-2
              scroll-smooth snap-x snap-mandatory"
          >
            {#each group.listings as listing (listing.id)}
              <div class="snap-start flex-shrink-0 w-[230px] sm:w-[260px]">
                <ListingCard {listing} aspectRatio="portrait" />
              </div>
            {/each}
          </div>
        </div>

      {:else if group.layoutType === 'C'}
        <!-- ── LAYOUT C: CLASSIC PREMIUM HORIZONTAL SCROLL (For Services & Flowers) ── -->
        <div class="relative w-full">
          <div class="absolute inset-y-0 start-0 w-12 carousel-fade-start z-10 pointer-events-none" aria-hidden="true"></div>
          <div class="absolute inset-y-0 end-0 w-12 carousel-fade-end z-10 pointer-events-none" aria-hidden="true"></div>

          <div
            class="row-scroller
              flex flex-row flex-nowrap gap-6 overflow-x-auto py-4 px-2
              scroll-smooth snap-x snap-mandatory"
          >
            {#each group.listings as listing (listing.id)}
              <div class="snap-start flex-shrink-0 w-[290px] sm:w-[320px]">
                <ListingCard {listing} aspectRatio="landscape" />
              </div>
            {/each}
          </div>
        </div>

      {:else}
        <!-- ── LAYOUT D: COMPACT GRID (For Add-ons & Small Items) ── -->
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4 sm:gap-6">
          {#each group.listings as listing (listing.id)}
            <ListingCard {listing} aspectRatio="square" size="sm" />
          {/each}
        </div>
      {/if}
    </div>
  </section>
{/each}

{#if pendingCategories.length > 0}
  <div bind:this={sentinelEl} class="py-8 flex justify-center items-center" aria-hidden="true">
    {#if isLoadingMore}
      <!-- A beautiful gold spinner -->
      <div class="w-8 h-8 border-4 border-[var(--color-primary)]/20 border-t-[var(--color-primary)] rounded-full animate-spin"></div>
    {/if}
  </div>
{/if}

<!-- How It Works Section -->
<HowItWorksSection />

<!-- Statistics Section -->
<StatsSection serverStats={data.stats} />

<!-- Testimonials Section -->
<TestimonialsSection testimonials={data.testimonials} />

<!-- Call to Action Section -->
<CTASection />

<style>
  /* Categories & Rows Carousel styles */
  .categories-scroller, .row-scroller {
    scrollbar-width: none; /* Modern standard */
    -ms-overflow-style: none; /* Legacy IE/Edge */
  }
  .categories-scroller::-webkit-scrollbar, .row-scroller::-webkit-scrollbar {
    display: none; /* Legacy WebKit */
  }

  /* Direction-aware logical gradients to achieve a breathtaking edge fade */
  .carousel-fade-start {
    background: linear-gradient(to right, #ffffff, transparent);
  }
  :global([dir="rtl"]) .carousel-fade-start {
    background: linear-gradient(to left, #ffffff, transparent);
  }

  .carousel-fade-end {
    background: linear-gradient(to left, #ffffff, transparent);
  }
  :global([dir="rtl"]) .carousel-fade-end {
    background: linear-gradient(to right, #ffffff, transparent);
  }
</style>
