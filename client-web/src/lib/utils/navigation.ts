import { i18n } from '$lib/i18n.js';
import { getLocale } from '$lib/paraglide/runtime.js';
import { SORT_API_TO_URL, DEFAULT_SORT_SLUG } from '$params/sort.js';
import { CITY_SLUGS } from '$params/city.js';
import { CATEGORY_SLUGS } from '$params/category.js';

export interface RouteFilterParams {
  /** City slug (e.g. 'jeddah'). Must be in the CITY_SLUGS allowlist to become a path segment. */
  city?: string;
  /** Alias for city — accepted for backward compatibility. */
  location?: string;
  /** Category slug (e.g. 'hair-makeup'). Must be in CATEGORY_SLUGS to become a path segment. */
  category?: string;
  /**
   * Sort value — accepts EITHER the URL slug ('price-asc') OR the API enum ('price_asc').
   * Will be normalised to the URL slug form in the output path.
   */
  sort?: string;
  // ── Secondary filters (stay as query params) ─────────────────────────────
  gender?: string;
  minCapacity?: number;
  maxCapacity?: number;
  offer?: boolean;
  q?: string;
}

/**
 * Builds a clean path-based URL for the listings search page.
 *
 * Target format:  /{lang}/{city}/{category}/{sort}/?{secondary-filters}
 *
 * Rules:
 *  - city, category, sort are ONLY included as path segments when they
 *    are non-empty and in their respective allowlists.
 *  - sort defaults to 'weighted' and is OMITTED from the path when it is
 *    the default, to produce the shortest canonical URL.
 *    Exception: if category is absent, sort is also omitted regardless.
 *  - All secondary filters (gender, capacity, etc.) remain as query params.
 *  - The language prefix is added by i18n.resolveRoute().
 *
 * @param filters - Filter values to embed in the URL.
 * @returns  Localised clean path string ready to pass to goto() or href.
 */
export function buildListingsUrl(filters: RouteFilterParams): string {
  const cityVal     = (filters.city || filters.location || '').toLowerCase();
  const categoryVal = (filters.category || '').toLowerCase();

  // Normalise sort: accept both URL slug and API enum forms
  let sortInput = (filters.sort || '').toLowerCase();
  // If it looks like an API enum (has underscore), map to URL slug
  if (sortInput && SORT_API_TO_URL[sortInput]) {
    sortInput = SORT_API_TO_URL[sortInput];
  }
  const sortVal = sortInput || DEFAULT_SORT_SLUG;

  // Validate each segment against its allowlist before using as a path part
  const validCity     = cityVal     && CITY_SLUGS.has(cityVal)     ? cityVal     : '';
  const validCategory = categoryVal && CATEGORY_SLUGS.has(categoryVal) ? categoryVal : '';
  const validSort     = sortVal !== DEFAULT_SORT_SLUG ? sortVal : '';

  // Build the path segments according to the canonical architecture:
  // /search/[category]/[city]/[sort]
  const segments: string[] = ['search'];

  if (validCategory && validCity) {
    segments.push(validCategory);
    segments.push(validCity);
    if (validSort) segments.push(validSort);
  } else if (validCategory) {
    segments.push(validCategory);
    if (validSort) segments.push(validSort);
  } else if (validCity) {
    segments.push('all');
    segments.push(validCity);
    if (validSort) segments.push(validSort);
  } else {
    if (validSort) {
      segments.push('all');
      segments.push('all');
      segments.push(validSort);
    }
  }

  const rawPath = `/${segments.join('/')}/`;

  // ── Secondary filters → query params ────────────────────────────────────
  const qs = new URLSearchParams();
  if (filters.gender)        qs.set('gender',       filters.gender);
  if (filters.minCapacity)   qs.set('minCapacity',  String(filters.minCapacity));
  if (filters.maxCapacity)   qs.set('maxCapacity',  String(filters.maxCapacity));
  if (filters.offer)         qs.set('offer',        'true');
  if (filters.q)             qs.set('q',            filters.q);
  const queryString = qs.toString() ? `?${qs.toString()}` : '';

  const rawUrl = `${rawPath}${queryString}`;

  return i18n.resolveRoute(rawUrl, getLocale());
}

/**
 * @deprecated  Use buildListingsUrl() instead.
 *
 * Legacy function kept for any non-listings callers that pass '/vendors'.
 * For '/listings' and '/venues', delegates to buildListingsUrl() with the
 * same filter params.
 */
export function buildFilteredRoute(
  basePath: '/listings' | '/venues' | '/vendors',
  filters: RouteFilterParams
): string {
  if (basePath === '/vendors') {
    // Vendors still use the query-string approach (not refactored in this task)
    const params = new URLSearchParams();
    if (filters.category)    params.set('category',     filters.category);
    if (filters.gender)      params.set('gender',       filters.gender);
    if (filters.minCapacity) params.set('min_capacity', String(filters.minCapacity));
    if (filters.maxCapacity) params.set('max_capacity', String(filters.maxCapacity));
    if (filters.sort)        params.set('sort',         filters.sort);
    const cityVal = filters.location || filters.city;
    if (cityVal)             params.set('city',         cityVal);
    if (filters.offer)       params.set('offer', 'true');
    if (filters.q)           params.set('q',     filters.q);
    return i18n.resolveRoute(`/vendors?${params.toString()}`, getLocale());
  }

  // '/listings' and '/venues' (remapped) both use the new clean URL format
  return buildListingsUrl(filters);
}


