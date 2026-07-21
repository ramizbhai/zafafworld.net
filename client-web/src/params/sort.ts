import type { ParamMatcher } from '@sveltejs/kit';

/**
 * Bidirectional sort slug ↔ backend enum mapping.
 *
 * URL slug (hyphenated)  ←→  Backend API enum (underscored)
 *
 * Any value not in this map is rejected (404), not silently fallen back to
 * a default. This ensures clean routing and prevents unindexed sort pages.
 */
export const SORT_URL_TO_API: Record<string, string> = {
    weighted:   'weighted',
    featured:   'featured',
    'price-asc':  'price_asc',
    'price-desc': 'price_desc',
    rating:     'rating',
    newest:     'newest',
};

export const SORT_API_TO_URL: Record<string, string> = {
    weighted:   'weighted',
    featured:   'featured',
    price_asc:  'price-asc',
    price_desc: 'price-desc',
    rating:     'rating',
    newest:     'newest',
};

/** The default sort slug used in URLs when no sort is specified. */
export const DEFAULT_SORT_SLUG = 'weighted';

export const match: ParamMatcher = (param) => {
    return Object.prototype.hasOwnProperty.call(SORT_URL_TO_API, param.toLowerCase());
};
