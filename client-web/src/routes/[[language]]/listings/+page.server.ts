import { env } from '$env/dynamic/public';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ url, fetch, cookies }) => {
    const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';
    
    // Extract query parameters
    const category = url.searchParams.get('category') || undefined;
    const city = url.searchParams.get('city') || undefined;
    const country = url.searchParams.get('country') || cookies.get('zafaf_selected_country') || undefined;
    const gender = url.searchParams.get('gender') || undefined;
    const priceMin = url.searchParams.get('priceMin') || undefined;
    const priceMax = url.searchParams.get('priceMax') || undefined;
    const minCapacity = url.searchParams.get('minCapacity') || undefined;
    const maxCapacity = url.searchParams.get('maxCapacity') || undefined;
    const sort = url.searchParams.get('sort') || 'weighted';
    const page = url.searchParams.get('page') || '1';
    
    // Build query string
    const q = new URLSearchParams();
    if (category) q.set('category', category);
    if (city) q.set('city', city);
    if (country) q.set('country_id', country.toLowerCase());
    if (gender) q.set('gender', gender);
    if (priceMin) q.set('price_min', priceMin);
    if (priceMax) q.set('price_max', priceMax);
    if (minCapacity) q.set('min_capacity', minCapacity);
    if (maxCapacity) q.set('max_capacity', maxCapacity);
    if (sort) q.set('sort', sort);
    q.set('page', page);
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
            initialTotal = data.total || 0;
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
            activePromotions
        };
    } catch (err) {
        console.error('[Listings SSR] Failed to fetch listings:', err);
    }
    
    return {
        initialListings: [],
        initialTotal: 0,
        initialTotalPages: 1,
        activePromotions: []
    };
};
