<script lang="ts">
  import { page } from "$app/stores";
  import * as m from "$lib/paraglide/messages.js";
  import { getLocale } from "$lib/paraglide/runtime.js";
  import {
    getLocalizedField,
    formatCurrency,
    formatNumber,
    formatDate,
  } from "$lib/utils/localize.js";
  import { listingService } from "$lib/services/api/listing.service.js";
  import ListingCard from "$lib/components/shared/ListingCard.svelte";
  import ListingCardSkeleton from "$lib/components/shared/ListingCardSkeleton.svelte";
  import type {
    Listing,
    ListingSearchParams,
    GenderSection,
  } from "$lib/types/index.js";

  import { goto } from "$app/navigation";
  import { untrack, onMount } from "svelte";
  import { countryStore } from "$lib/stores/country.svelte.js";
  import { uiStore } from '$lib/stores/ui.svelte';

  const activeCountryCode = $derived(
    countryStore.activeCode?.toLowerCase() || "sa",
  );
  const SAUDI_CITIES = $derived(
    ($page.data.metadata?.cities || []).filter(
      (c: any) => c.country_id?.toLowerCase() === activeCountryCode,
    ),
  );

  // ── State ─────────────────────────────────────────────────────────────────
  let { data } = $props();
  let listings = $derived(data.initialListings || []);
  let groupedListings = $derived({
    diamond: listings.filter(
      (l: Listing) => (l.subscriptionBadge?.tierId || "free") === "diamond",
    ),
    vip: listings.filter(
      (l: Listing) => (l.subscriptionBadge?.tierId || "free") === "vip",
    ),
    gold: listings.filter(
      (l: Listing) => (l.subscriptionBadge?.tierId || "free") === "gold",
    ),
    free: listings.filter(
      (l: Listing) => (l.subscriptionBadge?.tierId || "free") === "free",
    ),
  });

  const findPromotionForListing = (listingId: string) => {
    return data.activePromotions?.find((p: any) => p.listing_id === listingId || p.product_id === listingId);
  };

  let total = $derived(data.initialTotal || 0);
  let totalPages = $derived(data.initialTotalPages || 1);
  let isFilterOpen = $state(false);
  let isAdvancedOpen = $state(true);

  // Filter state
  let city = $state($page.url.searchParams.get("city") ?? "");
  let category = $state($page.url.searchParams.get("category") ?? "");
  let gender = $state<GenderSection | "">(
    ($page.url.searchParams.get("gender") as GenderSection) ?? "",
  );
  let priceMin = $state($page.url.searchParams.get("priceMin") ?? "");
  let priceMax = $state($page.url.searchParams.get("priceMax") ?? "");
  let minCapacity = $state($page.url.searchParams.get("minCapacity") ?? "");
  let maxCapacity = $state($page.url.searchParams.get("maxCapacity") ?? "");
  let sortBy = $state<ListingSearchParams["sort"]>(
    ($page.url.searchParams.get("sort") as ListingSearchParams["sort"]) ??
      "weighted",
  );
  let currentPage = $state(
    parseInt($page.url.searchParams.get("page") ?? "1", 10) || 1,
  );
  const limit = 48;

  // ── Load listings ─────────────────────────────────────────────────────────
  // Data fetching is now fully managed by SvelteKit's server load function.
  // When filters change via goto(), SvelteKit automatically runs the load function
  // and injects the new data into the page props.

  // Sync URL changes to local state
  $effect(() => {
    const url = $page.url;
    let urlCity = url.searchParams.get("city") ?? "";
    let urlCategory = url.searchParams.get("category") ?? "";
    let urlGender = (url.searchParams.get("gender") as GenderSection) ?? "";
    let urlPriceMin = url.searchParams.get("priceMin") ?? "";
    let urlPriceMax = url.searchParams.get("priceMax") ?? "";
    let urlMinCapacity = url.searchParams.get("minCapacity") ?? "";
    let urlMaxCapacity = url.searchParams.get("maxCapacity") ?? "";
    let urlSortBy =
      (url.searchParams.get("sort") as ListingSearchParams["sort"]) ??
      "weighted";
    let urlPage = parseInt(url.searchParams.get("page") ?? "1", 10) || 1;

    untrack(() => {
      if (city !== urlCity) {
        city = urlCity;
      }
      if (category !== urlCategory) {
        category = urlCategory;
      }
      if (gender !== urlGender) {
        gender = urlGender;
      }
      if (priceMin !== urlPriceMin) {
        priceMin = urlPriceMin;
      }
      if (priceMax !== urlPriceMax) {
        priceMax = urlPriceMax;
      }
      if (minCapacity !== urlMinCapacity) {
        minCapacity = urlMinCapacity;
      }
      if (maxCapacity !== urlMaxCapacity) {
        maxCapacity = urlMaxCapacity;
      }
      if (sortBy !== urlSortBy) {
        sortBy = urlSortBy;
      }
      if (currentPage !== urlPage) {
        currentPage = urlPage;
      }
    });
  });

  function applyFilters(keepPage: any = false) {
    if (keepPage !== true) {
      currentPage = 1;
    }
    isFilterOpen = false;
    uiStore.setLoading(true);

    // Update URL query params
    const url = new URL($page.url);
    if (city) url.searchParams.set("city", city);
    else url.searchParams.delete("city");
    if (category) url.searchParams.set("category", category);
    else url.searchParams.delete("category");
    if (gender) url.searchParams.set("gender", gender);
    else url.searchParams.delete("gender");
    if (priceMin) url.searchParams.set("priceMin", priceMin);
    else url.searchParams.delete("priceMin");
    if (priceMax) url.searchParams.set("priceMax", priceMax);
    else url.searchParams.delete("priceMax");
    if (minCapacity) url.searchParams.set("minCapacity", minCapacity);
    else url.searchParams.delete("minCapacity");
    if (maxCapacity) url.searchParams.set("maxCapacity", maxCapacity);
    else url.searchParams.delete("maxCapacity");
    if (sortBy) url.searchParams.set("sort", sortBy);
    else url.searchParams.delete("sort");
    url.searchParams.set("page", currentPage.toString());

    goto(url.toString(), {
      replaceState: false,
      keepFocus: true,
      noScroll: true,
    }).finally(() => {
      uiStore.setLoading(false);
    });
  }
  function resetFilters() {
    city = "";
    category = "";
    gender = "";
    priceMin = "";
    priceMax = "";
    minCapacity = "";
    maxCapacity = "";
    sortBy = "featured";
    currentPage = 1;

    // Clear URL parameters
    const url = new URL($page.url);
    url.search = "";
    goto(url.toString(), {
      replaceState: false,
      keepFocus: true,
      noScroll: true,
    });
  }

  // ── Dynamic Category options removed, using .data.metadata.categories ──

  const genderOptions = [
    { value: "", arLabel: "الكل", enLabel: "All" },
    { value: "women_only", arLabel: "نساء فقط", enLabel: "Ladies Only" },
    { value: "men_only", arLabel: "رجال فقط", enLabel: "Gents Only" },
    { value: "mixed", arLabel: "مختلط", enLabel: "Mixed" },
    {
      value: "dual_parallel",
      arLabel: "قاعتان منفصلتان",
      enLabel: "Dual Halls",
    },
    { value: "family", arLabel: "عائلي", enLabel: "Family" },
  ];

  const sortOptions = [
    { value: "weighted", arLabel: "موصى به", enLabel: "Recommended" },
    { value: "featured", arLabel: "المميزة", enLabel: "Featured" },
    { value: "price_asc", arLabel: "السعر (الأقل)", enLabel: "Price (Low)" },
    { value: "price_desc", arLabel: "السعر (الأعلى)", enLabel: "Price (High)" },
    { value: "rating", arLabel: "التقييم", enLabel: "Top Rated" },
    { value: "newest", arLabel: "الأحدث", enLabel: "Newest" },
  ];
</script>

<svelte:head>
  <title>{m.auto_browse_listings()} — {m.meta_siteName()}</title>
  <meta name="description" content={m.auto_discover_the_perfect()} />
</svelte:head>

<!-- Page Header -->
<div
  class="bg-[var(--color-surface-alt)] border-b border-[var(--color-border)]"
>
  <div class="container-page py-12">
    <h1
      class="font-display text-3xl sm:text-4xl font-bold text-[var(--color-secondary)] mb-2"
    >
      {m.auto_browse_listings()}
    </h1>
    <p class="text-[var(--color-muted)]">
      {m.auto_discover_top_wedding()}
    </p>
  </div>
</div>

<div class="container-page py-10">
  <div class="flex flex-col lg:flex-row gap-8">
    <!-- ── Sidebar Filters ────────────────────────────────────────────────── -->
    <aside
      class="lg:w-72 flex-shrink-0 {isFilterOpen ? 'block' : 'hidden lg:block'}"
      aria-label={m.auto_filter_options()}
    >
      <div
        class="bg-white rounded-2xl border border-[var(--color-border)] p-6 sticky top-24"
      >
        <div class="flex items-center justify-between mb-6">
          <h2 class="font-semibold text-[var(--color-secondary)]">
            {m.auto_filter_results()}
          </h2>
          <button
            onclick={resetFilters}
            class="text-sm text-[var(--color-primary)] hover:underline"
          >
            {m.auto_reset()}
          </button>
        </div>

        <div class="flex flex-col gap-5">
          <!-- City -->
          <div>
            <label
              for="filter-city"
              class="block text-sm font-medium text-[var(--color-text)] mb-2"
            >
              {m.auto_city()}
            </label>
            <select
              id="filter-city"
              bind:value={city}
              class="w-full rounded-lg border border-[var(--color-border)] px-3 py-2.5 text-sm focus:outline-none focus:border-[var(--color-primary)] bg-white"
            >
              <option value="">{m.auto_all()}</option>
              {#each SAUDI_CITIES as c}
                <option value={c.slug}>
                  {getLocale() === "ar" ? c.name_ar : c.name_en}
                </option>
              {/each}
            </select>
          </div>

          <!-- Category -->
          <div>
            <label
              for="filter-category"
              class="block text-sm font-medium text-[var(--color-text)] mb-2"
            >
              {m.auto_category()}
            </label>
            <select
              id="filter-category"
              bind:value={category}
              class="w-full rounded-lg border border-[var(--color-border)] px-3 py-2.5 text-sm focus:outline-none focus:border-[var(--color-primary)] bg-white"
            >
              <option value="">{m.auto_all_categories()}</option>
              {#each $page.data.metadata?.categories || [] as cat}
                <option value={cat.key || cat.slug}
                  >{getLocale() === "ar"
                    ? cat.labelAr || cat.ar || cat.key
                    : cat.labelEn || cat.en || cat.key}</option
                >
              {/each}
            </select>
          </div>

          <!-- Gender Section -->
          <fieldset>
            <legend
              class="block text-sm font-medium text-[var(--color-text)] mb-2"
            >
              {m.auto_hall_section()}
            </legend>
            <div class="flex flex-wrap gap-2">
              {#each genderOptions as opt}
                <button
                  type="button"
                  onclick={() => {
                    gender = opt.value as GenderSection | "";
                  }}
                  aria-pressed={gender === opt.value}
                  class="px-3 py-1.5 rounded-lg text-xs font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-primary)] {gender === opt.value
                    ? 'bg-[var(--color-primary)] text-[var(--color-secondary)]'
                    : 'border border-[var(--color-border)] text-[var(--color-text)] hover:border-[var(--color-primary)]'}"
                >
                  {getLocale() === "ar" ? opt.arLabel : opt.enLabel}
                </button>
              {/each}
            </div>
          </fieldset>

          <!-- Price Range -->
          <div>
            <p class="text-sm font-medium text-[var(--color-text)] mb-2">
              {m.auto_price_range_sar()}
            </p>
            <div class="grid grid-cols-2 gap-2">
              <input
                type="number"
                bind:value={priceMin}
                placeholder={m.auto_min()}
                class="rounded-lg border border-[var(--color-border)] px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-primary)]"
              />
              <input
                type="number"
                bind:value={priceMax}
                placeholder={m.auto_max()}
                class="rounded-lg border border-[var(--color-border)] px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-primary)]"
              />
            </div>
          </div>

          <!-- Capacity -->
          <div>
            <p class="text-sm font-medium text-[var(--color-text)] mb-2">
              {m.auto_capacity_guests()}
            </p>
            <div class="grid grid-cols-2 gap-2">
              <input
                type="number"
                bind:value={minCapacity}
                placeholder={m.auto_min()}
                class="w-full rounded-lg border border-[var(--color-border)] px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-primary)] bg-white"
              />
              <input
                type="number"
                bind:value={maxCapacity}
                placeholder={m.auto_max()}
                class="w-full rounded-lg border border-[var(--color-border)] px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-primary)] bg-white"
              />
            </div>
          </div>

          <button
            onclick={applyFilters}
            class="w-full rounded-xl bg-[var(--color-primary)] text-[var(--color-secondary)] py-3 font-bold text-sm hover:bg-[var(--color-primary-dark)] transition-colors flex items-center justify-center gap-2 shadow-sm"
          >
            {m.auto_apply_filters()}
          </button>
        </div>
      </div>
    </aside>

    <!-- ── Main Content ───────────────────────────────────────────────────── -->
    <div class="flex-1 min-w-0">

      <!-- Active Filters Display -->
      {#if city || category || gender || priceMin || priceMax}
        <div class="flex flex-wrap items-center gap-2 mb-6 bg-white p-4 rounded-2xl border border-[var(--color-border)] shadow-sm">
          <span class="text-xs font-bold text-[var(--color-muted)] uppercase tracking-wider me-2">
            {getLocale() === 'ar' ? 'الفلاتر النشطة:' : 'Active Filters:'}
          </span>
          {#if city}
            {@const activeCity = SAUDI_CITIES.find((c: any) => c.slug === city)}
            {@const cityLabel = activeCity ? (getLocale() === 'ar' ? activeCity.name_ar : activeCity.name_en) : city}
            <span class="px-3 py-1.5 bg-[#F6A7B5]/20 text-[#CC4869] rounded-full text-xs font-semibold flex items-center gap-1 border border-[#F6A7B5]/40">
              📍 {cityLabel}
            </span>
          {/if}
          {#if category}
            {@const activeCat = ($page.data.metadata?.categories || []).find((c: any) => c.slug === category)}
            {@const catLabel = activeCat ? (getLocale() === 'ar' ? activeCat.ar : activeCat.en) : category}
            <span class="px-3 py-1.5 bg-[#F6A7B5]/20 text-[#CC4869] rounded-full text-xs font-semibold flex items-center gap-1 border border-[#F6A7B5]/40">
              ✨ {catLabel}
            </span>
          {/if}
          {#if gender}
            {@const activeGender = genderOptions.find(g => g.value === gender)}
            {#if activeGender}
              <span class="px-3 py-1.5 bg-[#F6A7B5]/20 text-[#CC4869] rounded-full text-xs font-semibold flex items-center gap-1 border border-[#F6A7B5]/40">
                👥 {getLocale() === 'ar' ? activeGender.arLabel : activeGender.enLabel}
              </span>
            {/if}
          {/if}
        </div>
      {/if}

      <!-- Toolbar -->
      <div class="flex items-center justify-between mb-6 gap-4 flex-wrap">
        <div class="flex items-center gap-3">
          <!-- Mobile filter toggle -->
          <button
            onclick={() => (isFilterOpen = !isFilterOpen)}
            class="lg:hidden flex items-center gap-2 px-4 py-2 rounded-lg border border-[var(--color-border)] text-sm font-medium hover:border-[var(--color-primary)] transition-colors"
          >
            <svg
              viewBox="0 0 24 24"
              class="w-4 h-4"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M10.5 6h9.75M10.5 6a1.5 1.5 0 11-3 0m3 0a1.5 1.5 0 10-3 0M3.75 6H7.5m3 12h9.75m-9.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-3.75 0H7.5m9-6h3.75m-3.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-9.75 0h9.75"
              />
            </svg>
            {m.auto_filters()}
          </button>

          <p class="text-sm text-[var(--color-muted)]">
            {formatNumber(total)}
            {m.auto_results()}
          </p>
        </div>

        <!-- Sort -->
        <div class="flex items-center gap-2">
          <label
            for="sort-by"
            class="text-sm text-[var(--color-muted)] whitespace-nowrap"
          >
            {m.auto_sort_by()}:
          </label>
          <select
            id="sort-by"
            bind:value={sortBy}
            onchange={applyFilters}
            class="rounded-lg border border-[var(--color-border)] px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-primary)] bg-white"
          >
            {#each sortOptions as opt}
              <option value={opt.value}
                >{getLocale() === "ar" ? opt.arLabel : opt.enLabel}</option
              >
            {/each}
          </select>
        </div>
      </div>

      <!-- Grid -->
      {#if listings.length === 0}
        <div
          class="flex flex-col items-center justify-center py-20 text-center bg-white rounded-3xl border border-gray-100 shadow-sm mt-4"
        >
          <div class="text-5xl mb-4" aria-hidden="true">✨</div>
          <h3
            class="font-display text-2xl sm:text-3xl font-extrabold text-[var(--color-secondary)] mb-3"
          >
            {getLocale() === 'ar' ? 'قريباً جداً!' : "It's Coming Soon!"}
          </h3>
          <p class="text-[var(--color-muted)] font-medium max-w-md mx-auto mb-8 leading-relaxed">
            {getLocale() === 'ar' 
              ? 'نحن نعمل حالياً على إضافة أفضل مزودي الخدمات في هذا القسم. يرجى تعديل البحث أو العودة لاحقاً لاكتشاف المزيد.' 
              : 'We are currently working on adding the best premium service providers in this section. Please adjust your search or check back later!'}
          </p>
          <button
            onclick={resetFilters}
            class="px-8 py-3.5 rounded-full bg-[var(--color-primary)] text-[var(--color-secondary)] font-bold shadow-md hover:bg-[var(--color-primary-dark)] hover:-translate-y-0.5 transition-all"
          >
            {m.auto_clear_filters()}
          </button>
        </div>
      {:else}
        <div class="flex flex-col gap-10">
          {#if groupedListings.diamond.length > 0}
            <div
              class="tier-section diamond-section relative rounded-2xl bg-gradient-to-br from-purple-50/60 via-white to-blue-50/40 border border-purple-200/50 p-6 -mx-2"
            >
              <h2
                class="font-display text-xl font-bold text-[var(--color-secondary)] mb-4 flex items-center gap-2"
              >
                💎 {getLocale() === "ar" ? "العروض الماسية" : "Diamond Offers"}
              </h2>
              <div class="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 gap-6">
                {#each groupedListings.diamond as listing, i (listing.id)}
                  <ListingCard
                    {listing}
                    promotion={findPromotionForListing(listing.id)}
                    size="lg"
                  />
                {/each}
              </div>
            </div>
          {/if}

          {#if groupedListings.vip.length > 0}
            <div
              class="tier-section vip-section relative rounded-2xl bg-gradient-to-br from-amber-50/50 via-white to-orange-50/30 border border-amber-200/40 p-6 -mx-2"
            >
              <h2
                class="font-display text-xl font-bold text-[var(--color-secondary)] mb-4 flex items-center gap-2"
              >
                👑 {getLocale() === "ar" ? "العروض المميزة" : "VIP Offers"}
              </h2>
              <div class="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 gap-6">
                {#each groupedListings.vip as listing, i (listing.id)}
                  <!-- If there are no diamond listings, the first 3 vip are eager -->
                  <ListingCard
                    {listing}
                    promotion={findPromotionForListing(listing.id)}
                    size="md"
                  />
                {/each}
              </div>
            </div>
          {/if}

          {#if groupedListings.gold.length > 0}
            <div class="tier-section gold-section">
              <h2
                class="font-display text-xl font-bold text-[var(--color-secondary)] mb-4 flex items-center gap-2"
              >
                🥇 {getLocale() === "ar" ? "العروض الذهبية" : "Gold Offers"}
              </h2>
              <div class="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 gap-6">
                {#each groupedListings.gold as listing (listing.id)}
                  <ListingCard {listing} promotion={findPromotionForListing(listing.id)} size="md" />
                {/each}
              </div>
            </div>
          {/if}

          {#if groupedListings.free.length > 0}
            <div class="tier-section free-section">
              <h2
                class="font-display text-lg font-bold text-[var(--color-secondary)] mb-4 flex items-center gap-2"
              >
                🌿 {getLocale() === "ar" ? "العروض العامة" : "Standard Offers"}
              </h2>
              <div class="grid grid-cols-2 sm:grid-cols-3 xl:grid-cols-4 gap-4">
                {#each groupedListings.free as listing (listing.id)}
                  <ListingCard {listing} promotion={findPromotionForListing(listing.id)} size="sm" />
                {/each}
              </div>
            </div>
          {/if}
        </div>

        <!-- Pagination -->
        {#if totalPages > 1}
          <nav
            class="flex items-center justify-center gap-2 mt-12"
            aria-label="Pagination"
          >
            <button
              onclick={() => {
                currentPage = Math.max(1, currentPage - 1);
                applyFilters(true);
              }}
              disabled={currentPage === 1}
              class="w-10 h-10 rounded-lg border border-[var(--color-border)] flex items-center justify-center disabled:opacity-40 hover:border-[var(--color-primary)] transition-colors"
              aria-label={m.auto_previous()}
            >
              <svg
                viewBox="0 0 20 20"
                class="w-4 h-4 rtl:rotate-180"
                fill="currentColor"
              >
                <path
                  fill-rule="evenodd"
                  d="M12.79 5.23a.75.75 0 01-.02 1.06L8.832 10l3.938 3.71a.75.75 0 11-1.04 1.08l-4.5-4.25a.75.75 0 010-1.08l4.5-4.25a.75.75 0 011.06.02z"
                  clip-rule="evenodd"
                />
              </svg>
            </button>

            {#each Array(Math.min(totalPages, 7)) as _, i}
              {@const p = i + 1}
              <button
                onclick={() => {
                  currentPage = p;
                  applyFilters(true);
                }}
                class="w-10 h-10 rounded-lg text-sm font-medium transition-colors {currentPage === p
                  ? 'bg-[var(--color-primary)] text-[var(--color-secondary)]'
                  : 'border border-[var(--color-border)] hover:border-[var(--color-primary)] text-[var(--color-text)]'}"
                aria-current={currentPage === p ? "page" : undefined}
                >{p}</button
              >
            {/each}

            <button
              onclick={() => {
                currentPage = Math.min(totalPages, currentPage + 1);
                applyFilters(true);
              }}
              disabled={currentPage === totalPages}
              class="w-10 h-10 rounded-lg border border-[var(--color-border)] flex items-center justify-center disabled:opacity-40 hover:border-[var(--color-primary)] transition-colors"
              aria-label={m.auto_next()}
            >
              <svg
                viewBox="0 0 20 20"
                class="w-4 h-4 rtl:rotate-180"
                fill="currentColor"
              >
                <path
                  fill-rule="evenodd"
                  d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z"
                  clip-rule="evenodd"
                />
              </svg>
            </button>
          </nav>
        {/if}
      {/if}
    </div>
  </div>
</div>
