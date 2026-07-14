<script lang="ts">
  import { getLocale } from "$lib/paraglide/runtime.js";
  import * as m from "$lib/paraglide/messages.js";
  import { buildFilteredRoute } from "$lib/utils/navigation";
  import type { NavbarState } from "$lib/stores/navbarState.svelte.js";

  let { state, isGlass, isMobile = false } = $props<{ state: NavbarState, isGlass: boolean, isMobile?: boolean }>();
</script>

<div class="{isMobile ? 'relative w-full px-2' : 'hidden md:flex flex-1 max-w-[460px] mx-auto relative'} search-container">
  <form onsubmit={state.handleSearchSubmit} class={isMobile ? 'relative' : `flex items-center w-full rounded-full border overflow-hidden transition-all duration-200
    ${isGlass
      ? 'bg-[#1A1612]/40 hover:bg-[#1A1612]/60 focus-within:bg-[#1A1612]/75 border-[#C9A96E]/20 hover:border-[#C9A96E]/40 focus-within:border-[#C9A96E]/60 text-white'
      : 'bg-[var(--color-surface-alt)] hover:bg-[var(--color-border)] focus-within:bg-white border-[var(--color-border)] text-[var(--color-secondary)]'}`}>
    
    {#if isMobile}
      <input
        type="search"
        bind:value={state.searchQuery}
        oninput={state.handleSearchInput}
        onfocus={() => { if (state.searchQuery.trim().length >= 2) state.showSuggestions = true; }}
        onclick={(e) => e.stopPropagation()}
        placeholder="ما الذي تبحث عنه؟"
        class="w-full bg-white border border-[var(--color-border)] rounded-xl py-2.5 px-4 ps-10 text-xs text-[var(--color-text)] focus:outline-none"
      />
      <button
        type="submit"
        onclick={(e) => e.stopPropagation()}
        class="absolute start-6 top-1/2 -translate-y-1/2 text-[var(--color-primary-contrast)] cursor-pointer"
        aria-label="Submit search"
      >
        <svg viewBox="0 0 24 24" class="w-4 h-4" fill="none" stroke="currentColor" stroke-width="2.5">
          <path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z" />
        </svg>
      </button>
    {:else}
      <button
        type="submit"
        onclick={(e) => e.stopPropagation()}
        class="p-3 transition-colors cursor-pointer {isGlass ? 'text-[#C9A96E] hover:text-[#E8D5B0]' : 'text-[var(--color-muted)] hover:text-[var(--color-secondary)]'}"
        aria-label="Search"
      >
        <svg viewBox="0 0 24 24" class="w-4 h-4" fill="none" stroke="currentColor" stroke-width="2.5">
          <path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z" />
        </svg>
      </button>
      <input
        type="search"
        bind:value={state.searchQuery}
        oninput={state.handleSearchInput}
        onfocus={() => { if (state.searchQuery.trim().length >= 2) state.showSuggestions = true; }}
        onclick={(e) => e.stopPropagation()}
        placeholder={m.auto_what_are_you_looking()}
        class="w-full bg-transparent border-none py-2.5 pe-4 text-xs focus:outline-none {isGlass ? 'text-[#E8D5B0] placeholder:text-[#E8D5B0]/60' : 'text-[var(--color-secondary)] placeholder:text-[var(--color-muted)]'}"
      />
    {/if}
  </form>

  {#if state.showSuggestions && (state.suggestions.categories.length > 0 || state.suggestions.listings.length > 0 || state.isSearching)}
    <div class="absolute top-full mt-2 {isMobile ? 'left-2 right-2' : 'w-full'} bg-white rounded-2xl shadow-[var(--shadow-gold)] border border-[var(--color-border)] overflow-hidden z-50 {isMobile ? 'max-h-[300px]' : 'max-h-[400px]'} overflow-y-auto">
      {#if state.isSearching}
        <div class="p-4 text-center text-sm text-[var(--color-muted)]">Searching...</div>
      {:else}
        {#if state.suggestions.categories.length > 0}
          <div class="px-4 py-2 bg-gray-50 text-xs font-semibold text-[var(--color-muted)] uppercase tracking-wider">Categories</div>
          {#each state.suggestions.categories as cat}
            <a href={buildFilteredRoute("/listings", { category: cat.slug })} onclick={() => { state.showSuggestions = false; if (isMobile) state.isMenuOpen = false; }} class="flex items-center gap-3 px-4 py-3 hover:bg-gray-50 transition-colors border-b border-gray-100 last:border-0">
              <span class="text-xl">{cat.emoji || '📂'}</span>
              <span class="text-sm font-medium text-[var(--color-secondary)] truncate">{getLocale() === 'ar' ? cat.name_ar : cat.name_en}</span>
            </a>
          {/each}
        {/if}
        {#if state.suggestions.listings.length > 0}
          <div class="px-4 py-2 bg-gray-50 text-xs font-semibold text-[var(--color-muted)] uppercase tracking-wider">Listings</div>
          {#each state.suggestions.listings as listing}
            <a href={`/listings/${listing.slug}`} onclick={() => { state.showSuggestions = false; if (isMobile) state.isMenuOpen = false; }} class="flex items-center gap-3 px-4 py-3 hover:bg-gray-50 transition-colors border-b border-gray-100 last:border-0">
              <div class="w-8 h-8 shrink-0 rounded-full bg-[var(--color-primary)]/10 flex items-center justify-center text-[var(--color-primary)]">
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" class="w-4 h-4"><path stroke-linecap="round" stroke-linejoin="round" d="M13.5 21v-7.5a.75.75 0 01.75-.75h3a.75.75 0 01.75.75V21m-4.5 0H2.36m11.14 0H18m0 0h3.64m-1.39 0V9.349m-16.5 11.65V9.35m0 0a3.001 3.001 0 003.75-.615A2.993 2.993 0 009.75 9.75c.896 0 1.7-.393 2.25-1.016a2.993 2.993 0 002.25 1.016c.896 0 1.7-.393 2.25-1.015a3.001 3.001 0 003.75.614m-16.5 0a3.004 3.004 0 01-.621-4.72L4.318 3.44A1.5 1.5 0 015.378 3h13.243a1.5 1.5 0 011.06.44l1.19 1.189a3 3 0 01-.621 4.72m-13.5 8.65h3.75a.75.75 0 00.75-.75V13.5a.75.75 0 00-.75-.75H6.75a.75.75 0 00-.75.75v3.75c0 .415.336.75.75.75z" /></svg>
              </div>
              <span class="text-sm font-medium text-[var(--color-secondary)] truncate">{getLocale() === 'ar' ? listing.title_ar : listing.title_en}</span>
            </a>
          {/each}
        {/if}
      {/if}
    </div>
  {/if}
</div>
