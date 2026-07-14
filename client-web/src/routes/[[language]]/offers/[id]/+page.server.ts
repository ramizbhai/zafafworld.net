import { env } from '$env/dynamic/public';
import { error } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { listingService } from '$lib/services/api/listing.service.js';

export const load: PageServerLoad = async ({ params, fetch }) => {
    const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';
    const promotionId = params.id;
    let promotion = null;
    let listing = null;

    try {
        // Fetch all active promotions to locate this one
        const response = await fetch(`${API_BASE}/api/v1/public/promotions`);
        if (response.ok) {
            const data = await response.json();
            if (data.status === 'success') {
                const promos = data.promotions || [];
                promotion = promos.find((p: any) => p.promotion_id === promotionId);
            }
        }
    } catch (err) {
        console.error(`[Promotion Loader] Failed to fetch promotions queue:`, err);
    }

    if (!promotion) {
        throw error(404, { message: 'Promotion not found or has expired.' });
    }

    try {
        // Load details for the targeted listing
        listing = await listingService.getBySlug(promotion.listing_slug, fetch);
    } catch (err) {
        console.error(`[Promotion Loader] Failed to load listing detail for slug ${promotion.listing_slug}:`, err);
    }

    return {
        promotion,
        listing
    };
};
