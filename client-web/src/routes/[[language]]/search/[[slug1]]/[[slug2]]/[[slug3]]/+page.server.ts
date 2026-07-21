import { env } from '$env/dynamic/public';
import type { PageServerLoad } from './$types';
import { SORT_URL_TO_API, DEFAULT_SORT_SLUG } from '$params/sort.js';
import { CITY_COUNTRY_MAP, CITY_SLUGS } from '$params/city.js';
import { CATEGORY_SLUGS } from '$params/category.js';
import { error } from '@sveltejs/kit';

export const load: PageServerLoad = async ({ url, fetch, params }) => {
    const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';

    // ── Parse path slug params (slug1, slug2, slug3 are all optional) ──────────
    let citySlug: string | undefined = undefined;
    let categorySlug: string | undefined = undefined;
    let sortSlug: string | undefined = undefined;
    let hasAll = false;
    let isInvalid = false;

    const slugs = [params.slug1, params.slug2, params.slug3].filter(Boolean) as string[];

    for (const slug of slugs) {
        const lower = slug.toLowerCase();
        
        if (lower === 'all') {
            if (hasAll) {
                isInvalid = true;
                break;
            }
            hasAll = true;
            continue;
        }

        if (CITY_SLUGS.has(lower)) {
            if (citySlug) {
                isInvalid = true;
                break;
            }
            citySlug = lower;
        } else if (CATEGORY_SLUGS.has(lower)) {
            if (categorySlug) {
                isInvalid = true;
                break;
            }
            categorySlug = lower;
        } else if (Object.prototype.hasOwnProperty.call(SORT_URL_TO_API, lower)) {
            if (sortSlug) {
                isInvalid = true;
                break;
            }
            sortSlug = lower;
        } else {
            isInvalid = true;
            break;
        }
    }

    if (hasAll && categorySlug) {
        isInvalid = true;
    }

    if (isInvalid) {
        throw error(404, 'Not Found');
    }

    const resolvedCity = citySlug || null;
    const resolvedCategory = categorySlug || null;
    const resolvedSort = sortSlug || DEFAULT_SORT_SLUG;

    // Infer country from city via the static city→country map.
    // Falls back to undefined (backend will search all countries).
    const countryId = citySlug ? CITY_COUNTRY_MAP[citySlug] : undefined;

    // Convert hyphenated URL sort slug → underscored API enum.
    const sortApiValue = SORT_URL_TO_API[resolvedSort] ?? 'weighted';

    // ── Query params (secondary filters — never path segments) ───────────────
    const gender      = url.searchParams.get('gender')      || undefined;
    const priceMin    = url.searchParams.get('priceMin')    || undefined;
    const priceMax    = url.searchParams.get('priceMax')    || undefined;
    const minCapacity = url.searchParams.get('minCapacity') || undefined;
    const maxCapacity = url.searchParams.get('maxCapacity') || undefined;
    const page        = url.searchParams.get('page')        || '1';

    // ── Build backend query ──────────────────────────────────────────────────
    const q = new URLSearchParams();
    if (resolvedCategory)  q.set('category',     resolvedCategory);
    if (resolvedCity)      q.set('city',         resolvedCity);
    if (countryId)         q.set('country_id',   countryId);
    if (gender)            q.set('gender',       gender);
    if (priceMin)          q.set('price_min',    priceMin);
    if (priceMax)          q.set('price_max',    priceMax);
    if (minCapacity)       q.set('min_capacity', minCapacity);
    if (maxCapacity)       q.set('max_capacity', maxCapacity);
    q.set('sort',  sortApiValue);
    q.set('page',  page);
    q.set('limit', '48');

    try {
        const [response, promoResponse] = await Promise.all([
            fetch(`${API_BASE}/api/v1/public/listings?${q.toString()}`),
            fetch(`${API_BASE}/api/v1/public/promotions`).catch(() => null)
        ]);

        let initialListings = [];
        let initialTotal = 0;
        let initialTotalPages = 1;
        let activePromotions = [];

        if (response && response.ok) {
            const data = await response.json();
            initialListings = data.listings || [];
            initialTotal    = data.total     || 0;
            initialTotalPages = data.totalPages || 1;
        }

        if (promoResponse && promoResponse.ok) {
            const data = await promoResponse.json();
            if (data.status === 'success') {
                activePromotions = data.promotions || [];
            }
        }

        return {
            initialListings,
            initialTotal,
            initialTotalPages,
            activePromotions,
            // Pass resolved values back to the page for sidebar initialisation
            resolvedCity,
            resolvedCategory,
            resolvedSort,
        };
    } catch (err) {
        console.error('[Listings SSR] Failed to fetch listings:', err);
    }

    return {
        initialListings: [],
        initialTotal: 0,
        initialTotalPages: 1,
        activePromotions: [],
        resolvedCity,
        resolvedCategory,
        resolvedSort,
    };
};
