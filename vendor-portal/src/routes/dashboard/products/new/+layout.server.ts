import { env } from "$env/dynamic/public";
import { fail, redirect } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types';

// ── V2 Venue categories (use gender/capacity section of form) ────────────────
const VENUE_CATEGORIES = new Set([
    'wedding-palace', 'hotel-venue', 'villa-resort', 'chalet',
    'restaurant-event', 'outdoor-garden', 'rooftop-venue', 'private-beach',
    // Legacy slugs during migration period
    'wedding_hall', 'reception_hall', 'private_villa', 'outdoor_garden'
]);

export const load: LayoutServerLoad = async ({ cookies, fetch, parent }) => {
    const sessionToken = cookies.get('zafaf_vendor_session');
    if (!sessionToken) throw redirect(303, '/login');


    const apiBase = env.PUBLIC_API_URL || 'http://localhost:8080';

    // Load cities for the form
    let cities: Array<{ id: string; slug: string; name_ar: string; name_en: string }> = [];
    try {
        const response = await fetch(`${apiBase}/api/v1/public/cities`);
        if (response.ok) {
            const data = await response.json();
            if (data.status === 'success' && Array.isArray(data.cities)) {
                cities = data.cities.map((city: any) => ({
                    id: city.id as string,
                    slug: city.slug as string,
                    name_ar: city.name_ar || city.ar || '',
                    name_en: city.name_en || city.en || ''
                }));
            }
        }
    } catch (err) {
        console.error('Failed to load cities for product form:', err);
    }

    // Load Phase 1 categories from DB (for wizard step 1)
    let categories: Array<{
        slug: string; name_ar: string; name_en: string;
        emoji: string; parent_group: string; launch_phase: number;
    }> = [];
    try {
        const response = await fetch(`${apiBase}/api/v1/public/categories`);
        if (response.ok) {
            const data = await response.json();
            if (data.status === 'success' && Array.isArray(data.categories)) {
                categories = data.categories;
            }
        }
    } catch (err) {
        console.error('Failed to load categories for product form:', err);
    }

    return { cities, categories, sessionToken };
};
