import { env } from '$env/dynamic/public';
import { listingService } from '$lib/services/api/listing.service.js';
import type { PageServerLoad } from './$types';
import { getCached } from '$lib/services/api/cache';
import { apiClient } from '$lib/services/api/client.js';
import { buildListingsUrl } from '$lib/utils/navigation.js';

export const load: PageServerLoad = async ({ fetch: svelteFetch, setHeaders, cookies }) => {
  setHeaders({
    'Cache-Control': 'public, max-age=60, s-maxage=300',
  });
  const API_BASE = `${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/public`;
  const country = cookies.get('zafaf_selected_country') || 'sa';

  return getCached(`homepage:data:${country.toLowerCase()}`, async () => {
    let allCategories: any[] = [];
    try {
      const categoriesJson = await apiClient.get<any>('/api/v1/public/categories', {
        fetch: svelteFetch,
        isServer: true,
        headers: { 'X-Country-ID': country }
      });
      allCategories = categoriesJson?.allCategories || [];
    } catch (err) {
      console.error(`[Homepage Server Loader] Categories fetch exception:`, err);
    }

    // Filter categories with active listings (listingsCount > 0)
    const activeCategories = allCategories.filter((c: any) => (c.listingsCount || 0) > 0);

    // Limit server side load to first 2 active categories (for fast initial paint and SEO)
    const initialCategories = activeCategories.slice(0, 2);
    // Prepare remaining active categories (up to total of 5 categories across homepage)
    const pendingCategories = activeCategories.slice(2, 5);

    let categoryGroups: any[] = [];
    try {
      // Fetch listings for initial categories in parallel
      const groupPromises = initialCategories.map(async (c: any) => {
        try {
          const result = await listingService.getAll({
            category: c.slug,
            limit: 5,
            countryId: country,
            sort: 'weighted'  // B2B SaaS: tier-weighted rotation for homepage prominence
          });

          const listings = result.listings || [];

          // Determine layout strategy dynamically based on category traits
          let layoutType: 'A' | 'B' | 'C' | 'D' = 'C';
          if (
            c.slug === 'wedding-palace' || c.slug === 'hotel-venue' ||
            c.slug === 'villa-resort' || c.slug === 'restaurant-event' ||
            c.slug === 'outdoor-garden' || c.slug === 'rooftop-venue' ||
            c.slug === 'private-beach' || c.slug === 'chalet'
          ) {
            layoutType = 'A'; // Group 1: Mega Venues -> Hero Grid
          } else if (
            c.slug === 'photography-video' || c.slug === 'photo-studio' ||
            c.slug === 'wedding-gown' || c.slug === 'haute-couture' ||
            c.slug === 'abaya-jalabiya' || c.slug === 'groom-attire' ||
            c.slug === 'hair-makeup' || c.slug === 'beauty-skincare' ||
            c.slug === 'henna-art' ||
            c.slug === 'male-grooming'
          ) {
            layoutType = 'B'; // Group 2: Visual & Fashion -> Portrait Masonry
          } else if (
            c.slug === 'catering' || c.slug === 'wedding-planner' ||
            c.slug === 'wedding-car' || c.slug === 'entertainment-dj' ||
            c.slug === 'zaffa' || c.slug === 'nasheed-band' ||
            c.slug === 'khosha-decor' || c.slug === 'lighting-av'
          ) {
            layoutType = 'C'; // Group 3: Essential Services -> Horizontal Scroll
          } else if (
            c.slug === 'wedding-invitation' || c.slug === 'flowers-floral' ||
            c.slug === 'wedding-cake' || c.slug === 'wedding-sweets' ||
            c.slug === 'wedding-gifts' || c.slug === 'wedding-jewelry'
          ) {
            layoutType = 'D'; // Group 4: Small Items/Add-ons -> Compact Grid
          }

          return {
            key: c.slug,
            titleAr: c.ar,
            titleEn: c.en,
            subtitleAr: c.ar_subtitle || `أفضل مزودي ${c.ar} لمناسبتك السعيدة`,
            subtitleEn: c.en_subtitle || `Best ${c.en} selected for your premium event`,
            href: buildListingsUrl({ category: c.slug }),
            listings,
            listingsCount: c.listingsCount || 0,
            layoutType
          };
        } catch (err) {
          console.error(`[Homepage Server Loader] Error loading listings for category "${c.slug}":`, err);
          return null;
        }
      });

      const resolved = await Promise.all(groupPromises);
      categoryGroups = resolved.filter((g): g is NonNullable<typeof g> => g !== null);
    } catch (err) {
      console.error('[Homepage Server Loader] Error loading listings:', err);
    }

    const mappedPending = pendingCategories.map((c: any) => {
      let layoutType: 'A' | 'B' | 'C' | 'D' = 'C';
      if (
        c.slug === 'wedding-palace' || c.slug === 'hotel-venue' ||
        c.slug === 'villa-resort' || c.slug === 'restaurant-event' ||
        c.slug === 'outdoor-garden' || c.slug === 'rooftop-venue' ||
        c.slug === 'private-beach' || c.slug === 'chalet'
      ) {
        layoutType = 'A';
      } else if (
        c.slug === 'photography-video' || c.slug === 'photo-studio' ||
        c.slug === 'wedding-gown' || c.slug === 'haute-couture' ||
        c.slug === 'abaya-jalabiya' || c.slug === 'groom-attire' ||
        c.slug === 'hair-makeup' || c.slug === 'beauty-skincare' ||
        c.slug === 'henna-art' ||
        c.slug === 'male-grooming'
      ) {
        layoutType = 'B';
      } else if (
        c.slug === 'catering' || c.slug === 'wedding-planner' ||
        c.slug === 'wedding-car' || c.slug === 'entertainment-dj' ||
        c.slug === 'zaffa' || c.slug === 'nasheed-band' ||
        c.slug === 'khosha-decor' || c.slug === 'lighting-av'
      ) {
        layoutType = 'C';
      } else if (
        c.slug === 'wedding-invitation' || c.slug === 'flowers-floral' ||
        c.slug === 'wedding-cake' || c.slug === 'wedding-sweets' ||
        c.slug === 'wedding-gifts' || c.slug === 'wedding-jewelry'
      ) {
        layoutType = 'D';
      }

      return {
        key: c.slug,
        titleAr: c.ar,
        titleEn: c.en,
        subtitleAr: c.ar_subtitle || `أفضل مزودي ${c.ar} لمناسبتك السعيدة`,
        subtitleEn: c.en_subtitle || `Best ${c.en} selected for your premium event`,
        href: buildListingsUrl({ category: c.slug }),
        listings: [],
        listingsCount: c.listingsCount || 0,
        layoutType
      };
    });

    let stats = null;
    let testimonials = [];
    try {
      const [statsData, testimonialsData] = await Promise.all([
        apiClient.get<any>('/api/v1/public/platform/stats', { fetch: svelteFetch, isServer: true }).catch(() => null),
        apiClient.get<any>('/api/v1/public/testimonials', { fetch: svelteFetch, isServer: true }).catch(() => null)
      ]);

      if (statsData?.status === 'success') {
        stats = statsData.data;
      }
      if (testimonialsData?.status === 'success') {
        testimonials = testimonialsData.data || [];
      }
    } catch (err) {
      console.error('[Homepage Server Loader] Error loading stats/testimonials:', err);
    }

    return {
      categoryGroups,
      pendingCategories: mappedPending,
      selectedCountry: country,
      stats,
      testimonials
    };
  }, 5 * 60 * 1000); // Cache for 5 minutes
};
