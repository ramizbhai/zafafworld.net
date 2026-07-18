import { env } from '$env/dynamic/public';
import { countryStore } from '$lib/stores/country.svelte.js';
import * as m from '$lib/paraglide/messages.js';
import { getLocale } from '$lib/paraglide/runtime.js';
import { getLocalizedField } from '$lib/utils/localize.js';
import type { Venue, VenueSearchParams, PaginatedResult } from '$lib/types/index.js';

import { apiClient } from '$lib/services/api/client.js';

function mapPublicVendorToVenue(v: any): Venue {
  if (!v) return v;

  const categories = Array.isArray(v.categories) 
    ? v.categories 
    : (v.category ? [v.category] : []);

  const basePrice = v.startingPrice || v.price || 0;

  // Localized reactive getters for city and district
  const location = {
    get city() {
      return getLocale() === 'ar' ? (v.cityAr || '') : (v.cityEn || '');
    },
    get district() {
      return getLocale() === 'ar' ? (v.districtAr || '') : (v.districtEn || '');
    },
    cityAr: v.cityAr || '',
    cityEn: v.cityEn || '',
    districtAr: v.districtAr || '',
    districtEn: v.districtEn || '',
    address: v.address || '',
    lat: v.latitude || 0,
    lng: v.longitude || 0
  };

  const capacity = v.capacity || {
    min: v.capacityMin || v.capacity_min || 0,
    max: v.capacityMax || v.capacity_max || 1000
  };

  const images = v.images || (v.coverImage ? [{ id: 'cover', url: v.coverImage, alt: v.nameEn || '', isPrimary: true }] : []);

  const pricing = v.pricing || {
    basePrice: basePrice,
    depositPercentage: v.depositPercentage || 25,
    includedServices: [],
    additionalServices: []
  };

  return {
    id: v.id,
    slug: v.slug || '',
    nameAr: v.nameAr || v.name_ar || '',
    nameEn: v.nameEn || v.name_en || '',
    descriptionAr: v.descriptionAr || v.description_ar || '',
    descriptionEn: v.descriptionEn || v.description_en || '',
    category: categories,
    images: images,
    pricing: pricing,
    location: location,
    capacity: capacity,
    areaSqm: v.areaSqm || v.area_sqm || 0,
    amenities: v.amenities || [],
    rating: typeof v.rating === 'number' ? v.rating : (v.rating?.overall || 5.0),
    reviewCount: typeof v.reviewCount === 'number' ? v.reviewCount : (v.rating?.count || 0),
    reviews: v.reviews || [],
    isFeatured: v.isFeatured ?? false,
    isAvailable: v.isAvailable ?? true,
    vendor: v.vendor || {
      id: v.id,
      nameAr: v.nameAr || '',
      nameEn: v.nameEn || '',
      slug: v.slug || '',
    },
    halls: Array.isArray(v.halls) ? v.halls : [],
    packages: Array.isArray(v.packages) ? v.packages : [],
    phone: v.phone || '',
    email: v.email || '',
    website: v.website || '',
    mapsUrl: v.mapsUrl || v.maps_url || '',
    coordinator: v.coordinator || null,
    createdAt: v.createdAt || '',
    updatedAt: v.updatedAt || ''
  };
}

export class VendorService {
  async getAll(params: VenueSearchParams = {}, customFetch?: typeof fetch, signal?: AbortSignal): Promise<PaginatedResult<Venue>> {
    const fetcher = customFetch || (typeof window !== 'undefined' ? window.fetch.bind(window) : fetch);
    const url = new URL(`http://localhost/api/v1/public/vendors`); // Base URL ignored by apiClient, used for path/search parsing
    const activeCountry = typeof window !== 'undefined' ? countryStore.activeCode : 'sa';
    url.searchParams.append('country', activeCountry);

    if (params.city) url.searchParams.append('city', params.city);
    if (params.category) url.searchParams.append('category', params.category);
    if (params.guestCount) url.searchParams.append('guestCount', params.guestCount.toString());
    if (params.priceMin !== undefined) url.searchParams.append('priceMin', params.priceMin.toString());
    if (params.priceMax !== undefined) url.searchParams.append('priceMax', params.priceMax.toString());
    if (params.rating) url.searchParams.append('rating', params.rating.toString());
    if (params.amenities?.length) url.searchParams.append('amenities', params.amenities.join(','));
    if (params.query) url.searchParams.append('query', params.query);
    if (params.sortBy) url.searchParams.append('sortBy', params.sortBy);
    if (params.page) url.searchParams.append('page', params.page.toString());
    if (params.limit) url.searchParams.append('limit', params.limit.toString());

    // Phase 3 Saudi Filters
    if (params.partition !== undefined) url.searchParams.append('partition', params.partition.toString());
    if (params.minCapacity !== undefined) url.searchParams.append('min_capacity', params.minCapacity.toString());
    if (params.maxCapacity !== undefined) url.searchParams.append('max_capacity', params.maxCapacity.toString());

    const res = await apiClient.get<any>(url.pathname + url.search, { fetch: customFetch, isServer: typeof window === 'undefined', signal });
    const vendors = res?.vendors || [];
    return {
      data: vendors.map(mapPublicVendorToVenue),
      total: res?.total || 0,
      page: res?.page || 1,
      limit: params.limit || 9,
      totalPages: res?.totalPages || 1
    };
  }

  async getById(id: string, customFetch?: typeof fetch): Promise<Venue | null> {
    try {
      const json = await apiClient.get<any>(`/api/v1/public/vendors/${id}`, { fetch: customFetch, isServer: typeof window === 'undefined' });
      return json?.vendor ? mapPublicVendorToVenue(json.vendor) : null;
    } catch (err: any) {
      if (err.status === 404) return null;
      throw new Error('Failed to fetch vendor by id');
    }
  }

  async getBySlug(slug: string, customFetch?: typeof fetch): Promise<Venue | null> {
    try {
      const json = await apiClient.get<any>(`/api/v1/public/vendors/${slug}`, { fetch: customFetch, isServer: typeof window === 'undefined' });
      return json?.vendor ? mapPublicVendorToVenue(json.vendor) : null;
    } catch (err: any) {
      if (err.status === 404) return null;
      throw new Error('Failed to fetch vendor by slug');
    }
  }

  async getFeatured(limit = 6, customFetch?: typeof fetch): Promise<Venue[]> {
    const fetcher = customFetch || (typeof window !== 'undefined' ? window.fetch.bind(window) : fetch);
    const activeCountry = typeof window !== 'undefined' ? countryStore.activeCode : 'sa';
    const url = new URL(`http://localhost/api/v1/public/vendors`);
    url.searchParams.append('country', activeCountry);
    url.searchParams.append('limit', limit.toString());

    const json = await apiClient.get<any>(url.pathname + url.search, { fetch: customFetch, isServer: typeof window === 'undefined' }).catch(() => ({ vendors: [] }));
    const vendors = json?.vendors || [];
    return vendors
      .map(mapPublicVendorToVenue)
      .sort((a: Venue, b: Venue) => (b.isFeatured ? 1 : 0) - (a.isFeatured ? 1 : 0))
      .slice(0, limit);
  }

  async getSimilar(venueId: string, limit = 3, customFetch?: typeof fetch): Promise<Venue[]> {
    const fetcher = customFetch || (typeof window !== 'undefined' ? window.fetch.bind(window) : fetch);
    const activeCountry = typeof window !== 'undefined' ? countryStore.activeCode : 'sa';
    const url = new URL(`http://localhost/api/v1/public/vendors`);
    url.searchParams.append('country', activeCountry);
    url.searchParams.append('limit', (limit + 1).toString()); // Fetch one extra in case we filter out the current venue
    
    const json = await apiClient.get<any>(url.pathname + url.search, { fetch: customFetch, isServer: typeof window === 'undefined' }).catch(() => ({ vendors: [] }));
    const vendors = json?.vendors || [];
    return vendors.map(mapPublicVendorToVenue).filter((v: Venue) => v.id !== venueId).slice(0, limit);
  }
}

export const vendorService = new VendorService();
