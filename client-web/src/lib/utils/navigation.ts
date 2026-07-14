import { countryStore } from '$lib/stores/country.svelte.js';
import { i18n } from '$lib/i18n.js';
import { getLocale } from '$lib/paraglide/runtime.js';

export interface RouteFilterParams {
  category?: string;
  location?: string;
  city?: string;
  offer?: boolean;
  q?: string;
  gender?: string;
  minCapacity?: number;
  maxCapacity?: number;
  sort?: string;
}

/**
 * Builds a filtered query string routing contract for listings, venues, and vendors.
 *
 * ⚠️ Migration note: '/venues' is now remapped to '/listings' internally.
 * Any code still passing '/venues' will automatically land on the new catalog.
 */
export function buildFilteredRoute(
  basePath: '/listings' | '/venues' | '/vendors',
  filters: RouteFilterParams
): string {
  const params = new URLSearchParams();

  // Inject global country store state automatically
  if (countryStore.activeCode) {
    params.set('country', countryStore.activeCode.toLowerCase());
  }

  if (filters.category)    params.set('category',     filters.category);
  if (filters.gender)      params.set('gender',       filters.gender);
  if (filters.minCapacity) params.set('min_capacity', String(filters.minCapacity));
  if (filters.maxCapacity) params.set('max_capacity', String(filters.maxCapacity));
  if (filters.sort)        params.set('sort',         filters.sort);

  const cityVal = filters.location || filters.city;
  if (cityVal) params.set('city', cityVal);

  if (filters.offer) params.set('offer', 'true');
  if (filters.q)     params.set('q',     filters.q);

  // ── /venues → /listings migration remapping ─────────────────────────────
  const resolvedBase = basePath === '/venues' ? '/listings' : basePath;

  const rawUrl = `${resolvedBase}?${params.toString()}`;
  // Ensure the route maintains the current language state
  return i18n.resolveRoute(rawUrl, getLocale());
}
