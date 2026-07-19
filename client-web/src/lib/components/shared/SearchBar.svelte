<script lang="ts">
  import { goto } from '$app/navigation';
  import * as m from '$lib/paraglide/messages.js';
import { getLocale } from '$lib/paraglide/runtime.js';
import { getLocalizedField } from '$lib/utils/localize.js';
  import { countryStore } from '$lib/stores/country.svelte.js';
  import { page } from '$app/stores';
  import { buildFilteredRoute } from '$lib/utils/navigation.js';
  import { untrack, onMount } from 'svelte';
  import { env } from '$env/dynamic/public';

  interface Props {
    compact?: boolean;
    class?: string;
  }

  let { compact = false, class: extraClass = '' }: Props = $props();

  let query       = $state('');
  let city        = $state('');
  let category    = $state('');
  let date        = $state('');
  let guestCount  = $state('');

  let localCategories = $state<any[]>([]);
  let localCities = $state<any[]>([]);

  onMount(async () => {
    try {
      const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';
      const [catRes, cityRes] = await Promise.all([
        fetch(`${API_BASE}/api/v1/public/categories`).then(r => r.json()),
        fetch(`${API_BASE}/api/v1/public/cities`).then(r => r.json())
      ]);

      if (catRes && catRes.status === 'success') {
        let flattened: any[] = [];
        if (Array.isArray(catRes.allCategories)) {
          flattened = catRes.allCategories;
        } else if (catRes.categories) {
          const venuesList = catRes.categories.venues || [];
          const servicesList = catRes.categories.services || [];
          flattened = [...venuesList, ...servicesList];
        }

        const iconMap: Record<string, string> = {
            'wedding-palace': '🏛️', 'hotel-venue': '🏨', 'villa-resort': '🏡',
            'restaurant-event': '🍽️', 'outdoor-garden': '🌿', 'rooftop-venue': '🌃',
            'private-beach': '🏖️', 'chalet': '🏕️',
            'wedding-gown': '👗', 'haute-couture': '✨', 'abaya-jalabiya': '🧕',
            'groom-attire': '👘',
            'hair-makeup': '💄', 'beauty-skincare': '🧴', 'henna-art': '🌿',
            'male-grooming': '🪒',
            'photography-video': '📷', 'photo-studio': '📸',
            'catering': '🍱', 'wedding-cake': '🎂', 'wedding-sweets': '🍬',
            'entertainment-dj': '🎵', 'zaffa': '🥁', 'nasheed-band': '🎶',
            'wedding-jewelry': '💍', 'wedding-gifts': '🎁',
            'wedding-planner': '📋', 'khosha-decor': '🌺', 'flowers-floral': '💐',
            'wedding-invitation': '✉️', 'lighting-av': '💡',
            'wedding-car': '🚗',
        };

        localCategories = flattened.map((c: any) => ({
          key: c.slug,
          icon: iconMap[c.slug] ?? '✨',
          labelAr: c.ar,
          labelEn: c.en,
          listingsCount: c.listingsCount ?? 0
        }));
      }

      if (cityRes && cityRes.status === 'success') {
        const rawCities = cityRes.cities || cityRes.data || [];
        localCities = rawCities.map((c: any) => ({
          id: c.id,
          slug: c.slug,
          name_ar: c.name_ar || c.ar || '',
          name_en: c.name_en || c.en || '',
          country_id: c.country_id || ''
        }));
      }
    } catch (err) {
      console.error('Failed to dynamically fetch categories/cities in SearchBar:', err);
    }
  });

  $effect(() => {
    const url = $page.url;
    untrack(() => {
      query       = url.searchParams.get('q') ?? '';
      city        = url.searchParams.get('city') ?? '';
      category    = url.searchParams.get('category') ?? '';
      date        = url.searchParams.get('date') ?? '';
      guestCount  = url.searchParams.get('guests') ?? '';
    });
  });

  const categories = $derived(localCategories.length > 0 ? localCategories : ($page.data?.metadata?.categories || []));
  const cities = $derived(localCities.length > 0 ? localCities : ($page.data?.metadata?.cities || []));

  const filteredCities = $derived(
    cities.filter((c: any) => !countryStore.activeCode || c.country_id?.toLowerCase() === countryStore.activeCode?.toLowerCase())
  );

  function handleSearch(e: SubmitEvent) {
    e.preventDefault();
    
    const langPrefix = $page.params.language ? `/${$page.params.language}` : '';
    const country = $page.params.country || 'sa';
    
    // For compact search, category is not used as a filter directly here.
    // Use the explicit component category or fallback to current page category
    const targetCategory = (!compact && category) ? category : ($page.params.category || 'all');
    
    const url = new URL(`${langPrefix}/listings/${country}/${targetCategory}`, window.location.origin);

    if (compact && query) url.searchParams.set('q', query);
    if (city) url.searchParams.set('city', city);
    
    if (compact) {
      if (date) url.searchParams.set('date', date);
      if (guestCount) url.searchParams.set('guests', guestCount);
    }

    goto(url.pathname + url.search);
  }
</script>

{#if compact}
  <form
    onsubmit={handleSearch}
    novalidate
    class="flex flex-col sm:flex-row gap-1 bg-white rounded-2xl border border-[var(--color-border)] p-2 shadow-[var(--shadow-md)] {extraClass}"
    role="search"
    aria-label={m.auto_search_for_venues()}
  >
    <!-- Query input -->
    <div class="flex items-center gap-3 px-4 py-3 sm:px-5 sm:py-4 flex-1 min-w-0 w-full">
      <svg viewBox="0 0 24 24" class="w-5 h-5 flex-shrink-0 text-[var(--color-primary)]" fill="none" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z"/>
      </svg>
      <div class="flex-1 min-w-0">
        <input
          id="search-query"
          type="search"
          bind:value={query}
          placeholder={m.home_hero_searchPlaceholder()}
          class="w-full bg-transparent text-[var(--color-text)] placeholder:text-[var(--color-muted)] focus:outline-none text-sm"
        />
      </div>
    </div>

    <!-- City selector -->
    <div class="flex items-center gap-3 px-4 py-3 sm:px-5 sm:py-4 flex-1 min-w-0 w-full border-t sm:border-t-0 sm:border-s border-[var(--color-border)]">
      <svg viewBox="0 0 24 24" class="w-5 h-5 flex-shrink-0 text-[var(--color-primary)]" fill="none" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M15 10.5a3 3 0 11-6 0 3 3 0 016 0z"/>
        <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 10.5c0 7.142-7.5 11.25-7.5 11.25S4.5 17.642 4.5 10.5a7.5 7.5 0 1115 0z"/>
      </svg>
      <div class="flex-1 min-w-0">
        <select
          id="search-city"
          bind:value={city}
          class="w-full bg-transparent text-[var(--color-text)] focus:outline-none text-sm appearance-none cursor-pointer"
        >
          <option value="">{m.home_hero_searchByCity()}</option>
          {#each filteredCities as c}
            <option value={c.slug}>
              {getLocale() === 'ar' ? c.name_ar : c.name_en}
            </option>
          {/each}
        </select>
      </div>
    </div>

    <!-- Submit button -->
    <div class="flex-shrink-0 w-full sm:w-auto">
      <button
        type="submit"
        class="w-full sm:w-auto justify-center px-5 py-3 sm:py-2.5 rounded-xl bg-[var(--color-primary)] text-[var(--color-secondary)] font-bold text-sm hover:bg-[var(--color-primary-dark)] transition-colors flex items-center gap-2 cursor-pointer shadow-sm hover:shadow"
      >
        <svg viewBox="0 0 24 24" class="w-4 h-4" fill="none" stroke="currentColor" stroke-width="2.5">
          <path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z"/>
        </svg>
        {m.home_hero_searchButton()}
      </button>
    </div>
  </form>
{:else}
  <!-- The main home page search widget card -->
  <div class="bg-[var(--color-surface)] rounded-3xl shadow-[0_15px_40px_rgba(45,38,32,0.15)] border border-[var(--color-border)] py-6 px-8 md:py-8 md:px-10 w-full max-w-5xl mx-auto text-center md:text-start transition-all duration-300 {extraClass}">
    <h2 class="font-display text-2xl sm:text-3xl font-extrabold text-[var(--color-secondary)] mb-1 leading-tight">
      {m.auto_from_venue_to_weddin()}
    </h2>
    <p class="text-xs sm:text-sm text-[var(--color-muted)] mb-5 font-medium">
      {m.auto_two_out_of_three_cou()}
    </p>

    <form onsubmit={handleSearch} class="flex flex-col md:flex-row gap-3 items-stretch md:items-center justify-between w-full">
      <!-- Dropdown 1: Category / Search query -->
      <div class="flex-1 relative border border-[var(--color-border)] hover:border-[var(--color-primary)] rounded-full px-5 py-1.5 flex items-center justify-between bg-[var(--color-surface-alt)]/65 transition-all duration-200">
        <div class="flex flex-col text-start flex-1 min-w-0">
          <label for="search-category" class="text-[9px] text-gray-400 font-bold uppercase tracking-wider mb-0.5">
            {m.auto_what_are_you_looking()}
          </label>
          <select
            id="search-category"
            bind:value={category}
            class="w-full bg-transparent border-none p-0 text-xs font-extrabold text-[var(--color-secondary)] focus:outline-none appearance-none cursor-pointer"
          >
            <option value="">{m.auto_all_services()}</option>
            {#each categories as cat}
              <option value={cat.slug || cat.key}>{getLocale() === 'ar' ? (cat.labelAr || cat.ar || cat.key) : (cat.labelEn || cat.en || cat.key)}</option>
            {/each}
          </select>
        </div>
        <svg viewBox="0 0 20 20" class="w-4 h-4 text-gray-400 ms-2 pointer-events-none" fill="currentColor">
          <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clip-rule="evenodd"/>
        </svg>
      </div>

      <!-- Dropdown 2: City selector -->
      <div class="flex-1 relative border border-[var(--color-border)] hover:border-[var(--color-primary)] rounded-full px-5 py-1.5 flex items-center justify-between bg-[var(--color-surface-alt)]/65 transition-all duration-200">
        <div class="flex flex-col text-start flex-1 min-w-0">
          <label for="search-city" class="text-[9px] text-gray-400 font-bold uppercase tracking-wider mb-0.5">
            {m.auto_city()}
          </label>
          <select
            id="search-city"
            bind:value={city}
            class="w-full bg-transparent border-none p-0 text-xs font-extrabold text-[var(--color-secondary)] focus:outline-none appearance-none cursor-pointer"
          >
            <option value="">{m.auto_all_cities()}</option>
            {#each filteredCities as c}
              <option value={c.slug}>{getLocale() === 'ar' ? c.name_ar : c.name_en}</option>
            {/each}
          </select>
        </div>
        <svg viewBox="0 0 20 20" class="w-4 h-4 text-gray-400 ms-2 pointer-events-none" fill="currentColor">
          <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clip-rule="evenodd"/>
        </svg>
      </div>

      <!-- Search Submit Button (Pastel Pink #F6A7B5) -->
      <button
        type="submit"
        class="px-8 py-3.5 rounded-full font-bold text-sm bg-[#F6A7B5] hover:bg-[#e293a1] text-[#2D2620] transition-all duration-300 shadow-md select-none text-center flex items-center justify-center shrink-0 cursor-pointer"
      >
        {m.auto_service_providers_li()}
      </button>
    </form>
  </div>
{/if}
