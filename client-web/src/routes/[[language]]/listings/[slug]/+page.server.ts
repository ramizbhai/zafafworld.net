import type { PageServerLoad } from './$types';
import { listingService } from '$lib/services/api/listing.service.js';

export const load: PageServerLoad = async ({ params, fetch }) => {
    const slug = params.slug;
    let listing = null;

    try {
        listing = await listingService.getBySlug(slug, fetch);
    } catch (err) {
        console.error(`[Listings Loader] Failed to load slug ${slug}:`, err);
    }

    return { listing };
};
