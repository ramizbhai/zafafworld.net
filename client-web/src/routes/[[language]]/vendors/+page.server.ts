import { vendorService } from '$lib/services/api/vendor.service.js';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch }) => {
  try {
    const res = await vendorService.getAll({}, fetch);
    
    const uniqueVendors = res.data.reduce((acc, venue) => {
      if (!acc.find((v: any) => v.id === venue.vendor.id)) {
        acc.push({
          ...venue.vendor,
          city: venue.location.city,
          preview: venue.images[0]?.url,
        });
      }
      return acc;
    }, [] as any[]);

    return {
      vendors: uniqueVendors
    };
  } catch (err) {
    console.error('[Vendors SSR] Failed to fetch vendors:', err);
    return {
      vendors: []
    };
  }
};
