<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import * as m from '$lib/paraglide/messages.js';
import { getLocale } from '$lib/paraglide/runtime.js';
import { getLocalizedField, formatCurrency, formatNumber, formatDate } from '$lib/utils/localize.js';
  import { vendorService } from '$lib/services/api/vendor.service.js';
  const SAUDI_CITIES = $derived($page.data.metadata?.cities || []);
  import VenueCard from '$lib/components/shared/VenueCard.svelte';
  import VenueCardSkeleton from '$lib/components/shared/VenueCardSkeleton.svelte';
  import SearchBar from '$lib/components/shared/SearchBar.svelte';
  import type { Venue, VenueSearchParams, VenueCategory, AmenityKey } from '$lib/types/index.js';

  import { untrack, onMount, onDestroy } from 'svelte';
  
  // ── Permanent redirect: /venues → /listings ───────────────────────────────
  // The catalog has been migrated to a listing-centric architecture.
  // /venues is kept alive only for backward-compatibility with existing bookmarks
  // and external links. All new traffic should hit /listings directly.
  // NOTE: Server-side redirect is now handled in +page.server.ts.


  // ── State ────────────────────────────────────────────────────────────────
  let venues        = $state<Venue[]>([]);
  let total         = $state(0);
  let totalPages    = $state(1);
  let isFilterOpen  = $state(false);

  // Filters
  let city        = $state($page.url.searchParams.get('city') ?? '');
  let category    = $state<VenueCategory | ''>($page.url.searchParams.get('category') as VenueCategory ?? '');
  let priceMin    = $state($page.url.searchParams.get('priceMin') ?? '');
  let priceMax    = $state($page.url.searchParams.get('priceMax') ?? '');
  let guestCount  = $state($page.url.searchParams.get('guests') ?? '');
  let ratingMin   = $state($page.url.searchParams.get('rating') ?? '');
  let sortBy      = $state<VenueSearchParams['sortBy']>('recommended');
  let currentPage = $state(1);

  // Phase 3 Saudi Filters state
  let isAdvancedFiltersOpen = $state(true); // Default open to showcase features immediately
  let partition = $state<boolean | undefined>(
    $page.url.searchParams.get('partition') === 'true' ? true :
    $page.url.searchParams.get('partition') === 'false' ? false : undefined
  );
  let minCapacity = $state($page.url.searchParams.get('minCapacity') || $page.url.searchParams.get('min_capacity') || '');
  let maxCapacity = $state($page.url.searchParams.get('maxCapacity') || $page.url.searchParams.get('max_capacity') || '');
  
  let initialAmenities = $page.url.searchParams.get('amenities')?.split(',').map(a => a.trim() as AmenityKey).filter(Boolean) || [];
  let selectedAmenities = $state<AmenityKey[]>(initialAmenities);

  const saudiAmenitiesList = $derived($page.data.metadata?.amenities || []);

  function toggleSaudiAmenity(key: AmenityKey) {
    if (selectedAmenities.includes(key)) {
      selectedAmenities = selectedAmenities.filter(a => a !== key);
    } else {
      selectedAmenities = [...selectedAmenities, key];
    }
    currentPage = 1;
    loadVenues();
  }

  const limit = 9;

  // ── Local loading state (never touches the global overlay) ───────────────
  let isLoading = $state(true);

  // AbortController for the current in-flight fetch. Replaced on every new
  // call so stale responses are silently dropped.
  let currentAbortController: AbortController | null = null;

  // ── Load venues ──────────────────────────────────────────────────────────
  async function loadVenues() {
    // Cancel any in-flight request before starting a new one.
    currentAbortController?.abort();
    currentAbortController = new AbortController();
    const signal = currentAbortController.signal;

    isLoading = true;
    const params: VenueSearchParams = {
      city:       city || undefined,
      category:   category || undefined,
      priceMin:   priceMin    ? Number(priceMin)    : undefined,
      priceMax:   priceMax    ? Number(priceMax)    : undefined,
      guestCount: guestCount  ? Number(guestCount)  : undefined,
      rating:     ratingMin   ? Number(ratingMin)   : undefined,
      sortBy,
      page:  currentPage,
      limit,
      partition:  partition !== undefined ? partition : undefined,
      minCapacity: minCapacity ? Number(minCapacity) : undefined,
      maxCapacity: maxCapacity ? Number(maxCapacity) : undefined,
      amenities:  selectedAmenities.length > 0 ? selectedAmenities : undefined,
    };
    vendorService.getAll(params, undefined, signal).then((result) => {
      if (signal.aborted) return;
      venues     = result.data;
      total      = result.total;
      totalPages = result.totalPages;
    }).catch((err) => {
      if (signal.aborted) return; // Silently ignore cancellations
      venues = [];
      total = 0;
      totalPages = 1;
    }).finally(() => {
      if (!signal.aborted) isLoading = false;
    });
  }

  // Abort any pending request when the component is destroyed (i.e., user navigates away)
  onDestroy(() => {
    currentAbortController?.abort();
  });

  // Sync URL changes to local state, which is needed when navigating via Navbar links
  // because the component doesn't remount when only query params change.
  $effect(() => {
    const url = $page.url;
    let urlCity        = url.searchParams.get('city') ?? '';
    let urlCategory    = (url.searchParams.get('category') as VenueCategory) ?? '';
    let urlPriceMin    = url.searchParams.get('priceMin') ?? '';
    let urlPriceMax    = url.searchParams.get('priceMax') ?? '';
    let urlGuestCount  = url.searchParams.get('guests') ?? '';
    let urlRatingMin   = url.searchParams.get('rating') ?? '';
    let urlSortBy      = (url.searchParams.get('sortBy') as VenueSearchParams['sortBy']) ?? 'recommended';
    let urlMinCapacity = url.searchParams.get('minCapacity') || url.searchParams.get('min_capacity') || '';
    let urlMaxCapacity = url.searchParams.get('maxCapacity') || url.searchParams.get('max_capacity') || '';
    
    let urlPartition = url.searchParams.get('partition') === 'true' ? true :
                       url.searchParams.get('partition') === 'false' ? false : undefined;

    let changed = false;
    
    untrack(() => {
      if (city !== urlCity) { city = urlCity; changed = true; }
      if (category !== urlCategory) { category = urlCategory; changed = true; }
      if (priceMin !== urlPriceMin) { priceMin = urlPriceMin; changed = true; }
      if (priceMax !== urlPriceMax) { priceMax = urlPriceMax; changed = true; }
      if (guestCount !== urlGuestCount) { guestCount = urlGuestCount; changed = true; }
      if (ratingMin !== urlRatingMin) { ratingMin = urlRatingMin; changed = true; }
      if (sortBy !== urlSortBy) { sortBy = urlSortBy; changed = true; }
      if (minCapacity !== urlMinCapacity) { minCapacity = urlMinCapacity; changed = true; }
      if (maxCapacity !== urlMaxCapacity) { maxCapacity = urlMaxCapacity; changed = true; }
      if (partition !== urlPartition) { partition = urlPartition; changed = true; }
    });

    if (changed) {
      untrack(() => {
        currentPage = 1;
        loadVenues();
      });
    }
  });

  onMount(() => {
    loadVenues();
  });

  function applyFilters() {
    currentPage = 1;
    isFilterOpen = false;
    loadVenues();
  }

  function resetFilters() {
    city = ''; category = ''; priceMin = ''; priceMax = '';
    guestCount = ''; ratingMin = ''; sortBy = 'recommended';
    partition = undefined;
    minCapacity = '';
    maxCapacity = '';
    selectedAmenities = [];
    currentPage = 1;
    loadVenues();
  }

  const categoryOptions = $derived([
    { value: '',           label: m.common_all() },
    { value: 'wedding',    label: m.home_categories_list_weddings() },
    { value: 'engagement', label: m.home_categories_list_engagement() },
    { value: 'corporate',  label: m.home_categories_list_corporate() },
    { value: 'birthday',   label: m.home_categories_list_birthday() },
    { value: 'conference', label: m.home_categories_list_conference() },
  ]);

  const sortOptions = $derived([
    { value: 'recommended', label: m.venues_filter_sortOptions_recommended() },
    { value: 'price_asc',   label: m.venues_filter_sortOptions_priceAsc() },
    { value: 'price_desc',  label: m.venues_filter_sortOptions_priceDesc() },
    { value: 'rating',      label: m.venues_filter_sortOptions_rating() },
    { value: 'newest',      label: m.venues_filter_sortOptions_newest() },
  ]);
</script>

<svelte:head>
  <title>{m.venues_title()} - {m.meta_siteName()}</title>
  <meta name="description" content={m.venues_subtitle()} />
</svelte:head>

<!-- Page Header -->
<div class="bg-[var(--color-surface-alt)] border-b border-[var(--color-border)]">
  <div class="container-page py-12">
    <h1 class="font-display text-3xl sm:text-4xl font-bold text-[var(--color-secondary)] mb-2">
      {m.venues_title()}
    </h1>
    <p class="text-[var(--color-muted)]">{m.venues_subtitle()}</p>

    <!-- Compact search bar -->
    <div class="mt-6">
      <SearchBar compact />
    </div>
  </div>
</div>

<div class="container-page py-10">
  <div class="flex flex-col lg:flex-row gap-8">

    <!-- ── Sidebar Filters ── -->
    <aside
      class="lg:w-72 flex-shrink-0
        {isFilterOpen ? 'block' : 'hidden lg:block'}"
      aria-label={m.venues_filter_title()}
    >
      <div class="bg-white rounded-2xl border border-[var(--color-border)] p-6 sticky top-24">
        <div class="flex items-center justify-between mb-6">
          <h2 class="font-semibold text-[var(--color-secondary)]">{m.venues_filter_title()}</h2>
          <button
            onclick={resetFilters}
            class="text-sm text-[var(--color-primary)] hover:underline"
          >
            {m.venues_filter_reset()}
          </button>
        </div>

        <div class="flex flex-col gap-6">
          <!-- City -->
          <div>
            <label for="filter-city" class="block text-sm font-medium text-[var(--color-text)] mb-2">
              {m.venues_filter_city()}
            </label>
            <select
              id="filter-city"
              bind:value={city}
              class="w-full rounded-lg border border-[var(--color-border)] px-3 py-2.5 text-sm text-[var(--color-text)] focus:outline-none focus:border-[var(--color-primary)] bg-white"
            >
              <option value="">{m.common_all()}</option>
              {#each SAUDI_CITIES as c}
                <option value={getLocalizedField(c, '', getLocale())}>
                  {getLocalizedField(c, '', getLocale())}
                </option>
              {/each}
            </select>
          </div>

          <!-- Category -->
          <div>
            <label for="filter-category" class="block text-sm font-medium text-[var(--color-text)] mb-2">
              {m.venues_filter_category()}
            </label>
            <select
              id="filter-category"
              bind:value={category}
              class="w-full rounded-lg border border-[var(--color-border)] px-3 py-2.5 text-sm focus:outline-none focus:border-[var(--color-primary)] bg-white"
            >
              {#each categoryOptions as opt}
                <option value={opt.value}>{opt.label}</option>
              {/each}
            </select>
          </div>

          <!-- Price Range -->
          <div>
            <p class="text-sm font-medium text-[var(--color-text)] mb-2">{m.venues_filter_price()}</p>
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

          <!-- Guests -->
          <div>
            <label for="filter-guests" class="block text-sm font-medium text-[var(--color-text)] mb-2">
              {m.venues_filter_capacity()}
            </label>
            <input
              id="filter-guests"
              type="number"
              bind:value={guestCount}
              min="1"
              placeholder={m.auto_guest_count()}
              class="w-full rounded-lg border border-[var(--color-border)] px-3 py-2.5 text-sm focus:outline-none focus:border-[var(--color-primary)]"
            />
          </div>

          <!-- Rating -->
          <div>
            <p class="text-sm font-medium text-[var(--color-text)] mb-2">{m.venues_filter_rating()}</p>
            <div class="flex flex-col gap-2">
              {#each [4.5, 4, 3.5, 3] as r}
                <label class="flex items-center gap-2 cursor-pointer group">
                  <input
                    type="radio"
                    name="rating"
                    value={String(r)}
                    bind:group={ratingMin}
                    class="accent-[var(--color-primary)]"
                  />
                  <span class="text-sm text-[var(--color-text)] group-hover:text-[var(--color-primary)]">
                    {r}+ ★
                  </span>
                </label>
              {/each}
              <label class="flex items-center gap-2 cursor-pointer">
                <input type="radio" name="rating" value="" bind:group={ratingMin} class="accent-[var(--color-primary)]" />
                <span class="text-sm text-[var(--color-text)]">{m.common_all()}</span>
              </label>
            </div>
          </div>

          <!-- ─── Advanced Saudi Filters Section ─── -->
          <div class="border-t border-[var(--color-border)] pt-5">
            <button
              type="button"
              onclick={() => isAdvancedFiltersOpen = !isAdvancedFiltersOpen}
              class="w-full flex items-center justify-between font-semibold text-sm text-[var(--color-secondary)] hover:text-[var(--color-primary)] transition-colors mb-3"
            >
              <span class="flex items-center gap-1.5">
                <span>✨</span>
                <span>{m.auto_advanced_saudi_filte()}</span>
              </span>
              <svg 
                viewBox="0 0 20 20" 
                class="w-4 h-4 transform transition-transform duration-200 {isAdvancedFiltersOpen ? 'rotate-180' : ''}" 
                fill="currentColor"
              >
                <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clip-rule="evenodd"/>
              </svg>
            </button>

            {#if isAdvancedFiltersOpen}
              <div class="flex flex-col gap-5 pt-1">
                <!-- Segregation Toggle Switch -->
                <div class="flex items-center justify-between">
                  <span class="text-sm font-medium text-[var(--color-text)]">
                    {m.auto_ladiesgents_segrega()}
                  </span>
                  <button
                    type="button"
                    onclick={() => { partition = partition === true ? undefined : true; currentPage = 1; loadVenues(); }}
                    class="relative inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none
                      {partition === true ? 'bg-[var(--color-primary)]' : 'bg-gray-200'}"
                    role="switch"
                    aria-checked={partition === true}
                    aria-label={m.auto_ladiesgents_segrega()}
                  >
                    <span
                      class="pointer-events-none inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out
                        {partition === true ? 'rtl:-translate-x-5 ltr:translate-x-5' : 'translate-x-0'}"
                    ></span>
                  </button>
                </div>

                <!-- Capacity Ranges -->
                <div>
                  <span class="block text-sm font-medium text-[var(--color-text)] mb-2">
                    {m.auto_capacity_limits_gue()}
                  </span>
                  <div class="grid grid-cols-2 gap-2">
                    <input
                      type="number"
                      bind:value={minCapacity}
                      oninput={() => { currentPage = 1; }}
                      placeholder={m.auto_min()}
                      class="w-full rounded-lg border border-[var(--color-border)] px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-primary)] bg-white"
                    />
                    <input
                      type="number"
                      bind:value={maxCapacity}
                      oninput={() => { currentPage = 1; }}
                      placeholder={m.auto_max()}
                      class="w-full rounded-lg border border-[var(--color-border)] px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-primary)] bg-white"
                    />
                  </div>
                </div>

                <!-- Saudi Market Amenities Chips -->
                <div>
                  <span class="block text-sm font-medium text-[var(--color-text)] mb-2">
                    {m.auto_specialized_saudi_am()}
                  </span>
                  <div class="flex flex-wrap gap-2">
                    {#each saudiAmenitiesList as amen}
                      <button
                        type="button"
                        onclick={() => toggleSaudiAmenity(amen.key)}
                        class="px-3 py-1.5 rounded-full text-xs font-semibold border transition-all duration-200 flex items-center gap-1.5
                          {selectedAmenities.includes(amen.key)
                            ? 'bg-[var(--color-primary)] text-[var(--color-secondary)] border-[var(--color-primary)] shadow-sm'
                            : 'bg-white text-[var(--color-text)] border-[var(--color-border)] hover:border-[var(--color-primary)]'}"
                      >
                        <span>{amen.icon}</span>
                        <span>{getLocalizedField(amen, 'label', getLocale())}</span>
                      </button>
                    {/each}
                  </div>
                </div>
              </div>
            {/if}
          </div>

          <button
            onclick={applyFilters}
            class="w-full rounded-xl bg-[var(--color-primary)] text-[var(--color-secondary)] py-3 font-semibold text-sm hover:bg-[var(--color-primary-dark)] transition-colors"
          >
            {m.venues_filter_apply()}
          </button>
        </div>
      </div>
    </aside>

    <!-- ── Main Content ── -->
    <div class="flex-1 min-w-0">

      <!-- Toolbar -->
      <div class="flex items-center justify-between mb-6 gap-4 flex-wrap">
        <div class="flex items-center gap-3">
          <!-- Mobile filter toggle -->
          <button
            onclick={() => isFilterOpen = !isFilterOpen}
            class="lg:hidden flex items-center gap-2 px-4 py-2 rounded-lg border border-[var(--color-border)] text-sm font-medium hover:border-[var(--color-primary)] transition-colors"
          >
            <svg viewBox="0 0 24 24" class="w-4 h-4" fill="none" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M10.5 6h9.75M10.5 6a1.5 1.5 0 11-3 0m3 0a1.5 1.5 0 10-3 0M3.75 6H7.5m3 12h9.75m-9.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-3.75 0H7.5m9-6h3.75m-3.75 0a1.5 1.5 0 01-3 0m3 0a1.5 1.5 0 00-3 0m-9.75 0h9.75"/>
            </svg>
            {m.venues_filter_title()}
          </button>

          <p class="text-sm text-[var(--color-muted)]">
            {formatNumber(total)} {m.common_results()}
          </p>
        </div>

        <!-- Sort -->
        <div class="flex items-center gap-2">
          <label for="sort-by" class="text-sm text-[var(--color-muted)] whitespace-nowrap">
            {m.venues_filter_sortBy()}:
          </label>
          <select
            id="sort-by"
            bind:value={sortBy}
            onchange={loadVenues}
            class="rounded-lg border border-[var(--color-border)] px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-primary)] bg-white"
          >
            {#each sortOptions as opt}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </div>
      </div>

      <!-- Venues grid -->
      {#if isLoading}
        <div class="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 gap-6" aria-busy="true" aria-label="Loading venues">
          {#each Array(limit) as _}
            <VenueCardSkeleton />
          {/each}
        </div>
      {:else if venues.length === 0}
        <div class="flex flex-col items-center justify-center py-24 text-center">
          <div class="text-5xl mb-4" aria-hidden="true">🔍</div>
          <h3 class="font-display text-xl font-semibold text-[var(--color-secondary)] mb-2">
            {m.venues_noResults()}
          </h3>
          <p class="text-[var(--color-muted)] mb-6">{m.venues_noResultsSubtitle()}</p>
          <button
            onclick={resetFilters}
            class="px-6 py-3 rounded-xl bg-[var(--color-primary)] text-[var(--color-secondary)] font-semibold hover:bg-[var(--color-primary-dark)] transition-colors"
          >
            {m.venues_filter_reset()}
          </button>
        </div>
      {:else}
        <div class="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-3 gap-6">
          {#each venues as venue (venue.id)}
            <VenueCard {venue} />
          {/each}
        </div>

        <!-- Pagination -->
        {#if totalPages > 1}
          <nav class="flex items-center justify-center gap-2 mt-12" aria-label="Pagination">
            <button
              onclick={() => { currentPage = Math.max(1, currentPage - 1); loadVenues(); }}
              disabled={currentPage === 1}
              class="w-10 h-10 rounded-lg border border-[var(--color-border)] flex items-center justify-center disabled:opacity-40 hover:border-[var(--color-primary)] transition-colors"
              aria-label={m.common_previous()}
            >
              <svg viewBox="0 0 20 20" class="w-4 h-4 rtl:rotate-180" fill="currentColor">
                <path fill-rule="evenodd" d="M12.79 5.23a.75.75 0 01-.02 1.06L8.832 10l3.938 3.71a.75.75 0 11-1.04 1.08l-4.5-4.25a.75.75 0 010-1.08l4.5-4.25a.75.75 0 011.06.02z" clip-rule="evenodd"/>
              </svg>
            </button>

            {#each Array(totalPages) as _, i}
              <button
                onclick={() => { currentPage = i + 1; loadVenues(); }}
                class="w-10 h-10 rounded-lg text-sm font-medium transition-colors
                  {currentPage === i + 1
                    ? 'bg-[var(--color-primary)] text-[var(--color-secondary)]'
                    : 'border border-[var(--color-border)] hover:border-[var(--color-primary)] text-[var(--color-text)]'}"
                aria-label="Page {i + 1}"
                aria-current={currentPage === i + 1 ? 'page' : undefined}
              >
                {i + 1}
              </button>
            {/each}

            <button
              onclick={() => { currentPage = Math.min(totalPages, currentPage + 1); loadVenues(); }}
              disabled={currentPage === totalPages}
              class="w-10 h-10 rounded-lg border border-[var(--color-border)] flex items-center justify-center disabled:opacity-40 hover:border-[var(--color-primary)] transition-colors"
              aria-label={m.common_next()}
            >
              <svg viewBox="0 0 20 20" class="w-4 h-4 rtl:rotate-180" fill="currentColor">
                <path fill-rule="evenodd" d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z" clip-rule="evenodd"/>
              </svg>
            </button>
          </nav>
        {/if}
      {/if}
    </div>
  </div>
</div>
