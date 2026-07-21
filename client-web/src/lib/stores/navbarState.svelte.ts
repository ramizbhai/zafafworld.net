import { env } from "$env/dynamic/public";
import { goto } from "$app/navigation";
import { buildListingsUrl } from "$lib/utils/navigation";
import { countryStore } from "$lib/stores/country.svelte.js";

export function createNavbarState() {
  let isMenuOpen = $state(false);
  let isScrolled = $state(false);
  let isCountryOpen = $state(false);
  let activeDropdown = $state<string | null>(null);
  let searchQuery = $state("");
  let dropdownTimeout: any;

  // Search Suggestion States
  let suggestions = $state<{ categories: any[]; listings: any[] }>({
    categories: [],
    listings: [],
  });
  let isSearching = $state(false);
  let showSuggestions = $state(false);
  let searchDebounce: ReturnType<typeof setTimeout>;

  async function fetchSuggestions(q: string) {
    if (q.trim().length < 2) {
      suggestions = { categories: [], listings: [] };
      showSuggestions = false;
      return;
    }
    isSearching = true;
    showSuggestions = true;
    try {
      const res = await fetch(
        `${env.PUBLIC_API_URL || "http://localhost:8080"}/api/v1/public/search/suggestions?q=${encodeURIComponent(
          q.trim(),
        )}`,
      );
      if (res.ok) {
        const json = await res.json();
        if (json.status === "success") {
          suggestions = json.data;
        }
      }
    } catch (e) {
      console.error("Search error", e);
    } finally {
      isSearching = false;
    }
  }

  function handleSearchInput(e: Event) {
    const val = (e.target as HTMLInputElement).value;
    searchQuery = val;
    clearTimeout(searchDebounce);
    searchDebounce = setTimeout(() => {
      fetchSuggestions(val);
    }, 300);
  }

  function handleSearchSubmit(e: SubmitEvent) {
    e.preventDefault();
    if (searchQuery.trim()) {
      goto(buildListingsUrl({}));
    }
  }

  function handleGlobalClick(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (!target.closest(".search-container")) {
      showSuggestions = false;
    }
  }

  function toggleCountryDropdown(e: MouseEvent) {
    e.stopPropagation();
    isCountryOpen = !isCountryOpen;
    activeDropdown = null;
  }

  function selectCountry(code: string) {
    countryStore.setCountry(code);
    isCountryOpen = false;
  }

  function closeMenu() {
    isMenuOpen = false;
  }

  function toggleMenu() {
    isMenuOpen = !isMenuOpen;
  }

  function setDropdown(tab: string | null) {
    if (dropdownTimeout) clearTimeout(dropdownTimeout);
    activeDropdown = tab;
    isCountryOpen = false;
  }

  function handleMouseLeave() {
    dropdownTimeout = setTimeout(() => {
      activeDropdown = null;
    }, 250);
  }

  return {
    get isMenuOpen() { return isMenuOpen; },
    set isMenuOpen(v) { isMenuOpen = v; },
    get isScrolled() { return isScrolled; },
    set isScrolled(v) { isScrolled = v; },
    get isCountryOpen() { return isCountryOpen; },
    set isCountryOpen(v) { isCountryOpen = v; },
    get activeDropdown() { return activeDropdown; },
    set activeDropdown(v) { activeDropdown = v; },
    get searchQuery() { return searchQuery; },
    set searchQuery(v) { searchQuery = v; },
    get suggestions() { return suggestions; },
    get isSearching() { return isSearching; },
    get showSuggestions() { return showSuggestions; },
    set showSuggestions(v) { showSuggestions = v; },

    fetchSuggestions,
    handleSearchInput,
    handleSearchSubmit,
    handleGlobalClick,
    toggleCountryDropdown,
    selectCountry,
    closeMenu,
    toggleMenu,
    setDropdown,
    handleMouseLeave
  };
}

export type NavbarState = ReturnType<typeof createNavbarState>;
