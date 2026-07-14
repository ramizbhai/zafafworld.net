import type { PageServerLoad } from './$types';
import { vendorService } from '$lib/services/api/vendor.service.js';

export const load: PageServerLoad = async ({ params, fetch }) => {
  const id = params.id;
  let venue = null;

  try {
    venue = await vendorService.getById(id, fetch);
  } catch (err) {
    console.error(`[Booking Loader] Failed to load id ${id}:`, err);
  }

  return { venue };
};
