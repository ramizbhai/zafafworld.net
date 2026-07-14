import type { PageServerLoad } from './$types';
import { vendorService } from '$lib/services/api/vendor.service.js';

export const load: PageServerLoad = async ({ params, fetch }) => {
  const slug = params.slug;
  let venue = null;
  let similar: any[] = [];

  try {
    venue = await vendorService.getBySlug(slug, fetch);
    if (venue) {
      similar = await vendorService.getSimilar(venue.id, 3, fetch);
    }
  } catch (err) {
    console.error(`[Venues Loader] Failed to load slug ${slug}:`, err);
  }

  return { venue, similar };
};
