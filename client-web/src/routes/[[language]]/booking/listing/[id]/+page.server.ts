import { error } from '@sveltejs/kit';
import { listingService } from '$lib/services/api/listing.service.js';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params, fetch }) => {
  try {
    const listing = await listingService.getById(params.id, fetch);
    
    if (!listing) {
      error(404, 'Listing not found');
    }

    return {
      listing
    };
  } catch (err) {
    console.error(`[Booking SSR] Failed to fetch listing ${params.id}:`, err);
    error(404, 'Failed to fetch listing');
  }
};
