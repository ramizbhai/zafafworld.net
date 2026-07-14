/**
 * listing.service.ts
 * Listing-centric API service — replaces vendor.service.ts as the primary catalog source.
 * All methods operate on vendor_products (listings), not vendor accounts.
 */

import { env } from '$env/dynamic/public';
import type {
  Listing,
  ListingDetail,
  ListingSearchParams,
  PaginatedListings,
  ListingBookingFormData,
} from '$lib/types/index.js';

import { apiClient } from '$lib/services/api/client.js';
import { browser } from '$app/environment';

// ── Internal helpers ──────────────────────────────────────────────────────────

function buildHeaders(): HeaderObj {
  const h: HeaderObj = { 'Content-Type': 'application/json' };
  // Country header passed if stored (set by Navbar/i18n)
  if (typeof window !== 'undefined') {
    const country = localStorage.getItem('zafaf_selected_country') ?? 'sa';
    h['X-Country-ID'] = country;
  }
  return h;
}

type HeaderObj = Record<string, string>;

function buildQuery(params: ListingSearchParams): string {
  const q = new URLSearchParams();

  if (params.category)      q.set('category',     params.category);
  if (params.city)          q.set('city',          params.city);
  if (params.cityId)        q.set('city_id',       params.cityId);
  if (params.countryId)     q.set('country_id',    params.countryId);
  if (params.gender)        q.set('gender',        params.gender);
  if (params.minCapacity)   q.set('min_capacity',  String(params.minCapacity));
  if (params.maxCapacity)   q.set('max_capacity',  String(params.maxCapacity));
  if (params.priceMin)      q.set('price_min',     String(params.priceMin));
  if (params.priceMax)      q.set('price_max',     String(params.priceMax));
  if (params.amenities?.length) q.set('amenities', params.amenities.join(','));
  if (params.featured)      q.set('featured',      'true');
  if (params.sort)          q.set('sort',          params.sort);
  if (params.page)          q.set('page',          String(params.page));
  if (params.limit)         q.set('limit',         String(params.limit));
  // V2: keyword full-text search across Arabic + English title
  if ((params as any).q)   q.set('q',             (params as any).q);
  // V2: women-only filter shorthand
  if ((params as any).womenOnly) q.set('women_only', 'true');
  // V2: min_women_capacity / min_men_capacity
  if ((params as any).minWomenCapacity) q.set('min_women_capacity', String((params as any).minWomenCapacity));
  if ((params as any).minMenCapacity)   q.set('min_men_capacity',   String((params as any).minMenCapacity));

  const qs = q.toString();
  return qs ? `?${qs}` : '';
}

/** Robust mapping from raw backend payload to canonical Listing interface */
function mapListing(l: any): Listing {
  if (!l) return l;

  const menCapacity = l.menCapacity !== undefined ? l.menCapacity : (l.men_capacity ?? null);
  const womenCapacity = l.womenCapacity !== undefined ? l.womenCapacity : (l.women_capacity ?? null);
  const totalCapacity = l.totalCapacity ?? l.total_capacity ?? ((menCapacity ?? 0) + (womenCapacity ?? 0));

  const rating = l.rating || {
    overall: l.rating_overall ?? l.overall_rating ?? 5.0,
    quality: l.rating_quality ?? l.quality_avg ?? 5.0,
    staff: l.rating_staff ?? l.staff_avg ?? 5.0,
    communication: l.rating_communication ?? l.comm_avg ?? l.communication_avg ?? 5.0,
    count: l.rating_count ?? l.review_count ?? 0,
  };

  const overall = typeof rating.overall === 'number' ? rating.overall : parseFloat(rating.overall) || 5.0;
  const quality = typeof rating.quality === 'number' ? rating.quality : parseFloat(rating.quality) || 5.0;
  const staff = typeof rating.staff === 'number' ? rating.staff : parseFloat(rating.staff) || 5.0;
  const communication = typeof rating.communication === 'number' ? rating.communication : parseFloat(rating.communication) || 5.0;
  const count = typeof rating.count === 'number' ? rating.count : parseInt(rating.count) || 0;

  const ratingObj = { overall, quality, staff, communication, count };

  const coordinatorRaw = l.coordinator ?? {};
  const coordinator = {
    nameAr: coordinatorRaw.nameAr ?? coordinatorRaw.name_ar ?? null,
    nameEn: coordinatorRaw.nameEn ?? coordinatorRaw.name_en ?? null,
    phone: coordinatorRaw.phone ?? coordinatorRaw.coordinator_phone ?? null,
    whatsapp: coordinatorRaw.whatsapp ?? coordinatorRaw.coordinator_whatsapp ?? null,
    avatar: coordinatorRaw.avatar ?? coordinatorRaw.coordinator_avatar ?? null,
    gender: coordinatorRaw.gender ?? coordinatorRaw.coordinator_gender ?? 'any',
  };

  const vendorRaw = l.vendor ?? {};
  const vendor = {
    id: vendorRaw.id ?? vendorRaw.vendor_id ?? '',
    slug: vendorRaw.slug ?? vendorRaw.vendor_slug ?? '',
    nameAr: vendorRaw.nameAr ?? vendorRaw.name_ar ?? '',
    nameEn: vendorRaw.nameEn ?? vendorRaw.name_en ?? '',
    phone: vendorRaw.phone ?? vendorRaw.vendor_phone ?? null,
    email: vendorRaw.email ?? vendorRaw.vendor_email ?? null,
    mapsUrl: vendorRaw.mapsUrl ?? vendorRaw.vendor_maps_url ?? vendorRaw.maps_url ?? null,
    website: vendorRaw.website ?? vendorRaw.vendor_website ?? null,
    subscriptionTierId: vendorRaw.subscriptionTierId ?? vendorRaw.subscription_tier_id ?? undefined,
    descriptionAr: vendorRaw.descriptionAr ?? vendorRaw.vendor_desc_ar ?? null,
    descriptionEn: vendorRaw.descriptionEn ?? vendorRaw.vendor_desc_en ?? null,
    videoUrl: vendorRaw.videoUrl ?? vendorRaw.vendor_video_url ?? null,
    starRating: vendorRaw.starRating ?? vendorRaw.vendor_star_rating ?? null,
  };

  const imagesRaw = l.images ?? [];
  const images = Array.isArray(imagesRaw) ? imagesRaw.map((img: any) => ({
    id: img.id ?? '',
    url: img.url ?? img.image_url ?? '',
    alt: img.alt ?? img.caption ?? '',
    isPrimary: img.isPrimary ?? img.is_primary ?? img.is_cover ?? false,
    mediaType: img.mediaType ?? img.media_type ?? 'image',
    fileUrl: img.fileUrl ?? img.file_url ?? img.url ?? img.image_url ?? '',
    thumbnailUrl: img.thumbnailUrl ?? img.thumbnail_url ?? null,
    fileSize: img.fileSize ?? img.file_size ?? null,
    durationSeconds: img.durationSeconds ?? img.duration_seconds ?? null,
  })) : [];

  // ── V2 bilingual field resolution ────────────────────────────────────────
  // Backend now returns titleAr, titleEn, descriptionAr, descriptionEn.
  // Fall back to legacy title/description for pre-V2 listings.
  const titleAr       = l.titleAr       ?? l.title_ar       ?? l.title ?? l.nameAr ?? l.name_ar ?? '';
  const titleEn       = l.titleEn       ?? l.title_en       ?? l.title ?? l.nameEn ?? l.name_en ?? '';
  const descriptionAr = l.descriptionAr ?? l.description_ar ?? l.description ?? null;
  const descriptionEn = l.descriptionEn ?? l.description_en ?? l.description ?? null;

  const metaTitleAr       = l.metaTitleAr       ?? l.meta_title_ar       ?? null;
  const metaTitleEn       = l.metaTitleEn       ?? l.meta_title_en       ?? null;
  const metaDescriptionAr = l.metaDescriptionAr ?? l.meta_description_ar ?? null;
  const metaDescriptionEn = l.metaDescriptionEn ?? l.meta_description_en ?? null;

  // Gender section — V2 dedicated column (used for filter badges)
  const genderSection = l.genderSection ?? l.gender_section ?? null;

  const googleMapsUrl = l.googleMapsUrl ?? l.google_maps_url ?? null;
  const latitude = l.latitude !== undefined ? l.latitude : (l.lat ?? null);
  const longitude = l.longitude !== undefined ? l.longitude : (l.lng ?? null);

  return {
    id: l.id,
    slug: l.slug,

    // V2 bilingual titles (used by locale-aware UI)
    titleAr,
    titleEn,
    // Legacy compat: title = Arabic if available, else English
    title: titleAr || titleEn,

    // V2 bilingual descriptions
    descriptionAr,
    descriptionEn,
    description: (descriptionAr || descriptionEn) ?? null,

    attributes: l.attributes ?? {},
    featuresSelection: l.featuresSelection ?? l.features_selection ?? {},
    coordinator: {
      nameAr: coordinator.nameAr,
      nameEn: coordinator.nameEn,
      phone: coordinator.phone,
      whatsapp: coordinator.whatsapp,
      email: coordinatorRaw.email ?? coordinatorRaw.coordinator_email ?? null,
      mobile: coordinatorRaw.mobile ?? coordinatorRaw.coordinator_mobile ?? null,
      avatar: coordinator.avatar,
      gender: coordinator.gender
    },

    category: l.category ?? l.product_category ?? '',

    // V2 gender section (critical GCC filter field)
    genderSection,

    // V2 quality & trust signals
    qualityScore:      l.qualityScore      ?? l.quality_score      ?? 0,
    verificationLevel: l.verificationLevel ?? l.verification_level ?? 'basic',
    priceOnInquiry:    l.priceOnInquiry    ?? l.price_on_inquiry    ?? false,

    basePriceSar: l.basePriceSar !== undefined
      ? (l.basePriceSar ? String(l.basePriceSar) : null)
      : (l.base_price_sar ? String(l.base_price_sar) : null),
    startingPrice:     l.startingPrice     ?? l.starting_price      ?? null,
    depositPercentage: l.depositPercentage ?? l.deposit_percentage   ?? 25,
    currency: l.currency ?? 'SAR',
    coverImage: l.coverImage ?? l.cover_image ?? '/images/fallbacks/default-cover.svg',
    imageCount: l.imageCount ?? l.image_count ?? l.listing_image_count ?? images.length ?? 0,
    images,
    vendor,
    citySlug:    l.citySlug    ?? l.city_slug     ?? null,
    cityAr:      l.cityAr      ?? l.city_ar       ?? l.city_name_ar ?? null,
    cityEn:      l.cityEn      ?? l.city_en       ?? l.city_name_en ?? null,
    countryCode: l.countryCode ?? l.country_code  ?? null,
    googleMapsUrl,
    latitude,
    longitude,
    rating: ratingObj,
    metaTitleAr,
    metaTitleEn,
    metaDescriptionAr,
    metaDescriptionEn,
    isFeatured:  l.isFeatured  ?? l.is_featured   ?? false,
    isAvailable: l.isAvailable ?? l.is_available  ?? true,
    subscriptionBadge: l.subscriptionBadge ?? l.subscription_badge ?? null,
    createdAt:   l.createdAt   ?? l.created_at    ?? '',
    detailUrl:   l.detailUrl   ?? l.detail_url    ?? `/listings/${l.slug}`,
    bookingUrl:  l.bookingUrl  ?? l.booking_url   ?? `/booking/listing/${l.id}`,
  };
}

/** Robust mapping from raw backend payload to ListingDetail interface */
function mapListingDetail(l: any): ListingDetail {
  const base = mapListing(l);


  const reviewsRaw = l.reviews ?? [];
  const reviews = Array.isArray(reviewsRaw) ? reviewsRaw.map((r: any) => ({
    id: r.id,
    authorName: r.authorName ?? r.author_name ?? '',
    rating: r.rating ?? 5.0,
    ratingQuality: r.ratingQuality ?? r.rating_quality ?? 5.0,
    ratingStaff: r.ratingStaff ?? r.rating_staff ?? 5.0,
    ratingCommunication: r.ratingCommunication ?? r.rating_communication ?? 5.0,
    weddingDate: r.weddingDate ?? r.wedding_date ?? '',
    date: r.date ?? r.created_at ?? '',
    comment: r.comment ?? '',
  })) : [];

  const packagesRaw = l.packages ?? [];
  const packages = Array.isArray(packagesRaw) ? packagesRaw.map((p: any) => ({
    id: p.id,
    nameAr: p.nameAr ?? p.name_ar ?? '',
    nameEn: p.nameEn ?? p.name_en ?? '',
    originalPrice: p.originalPrice ?? p.original_price ?? 0,
    discountedPrice: p.discountedPrice ?? p.discounted_price ?? 0,
    isZafafExclusive: p.isZafafExclusive ?? p.is_zafaf_exclusive ?? false,
    expiryDate: p.expiryDate ?? p.expiry_date ?? '',
  })) : [];

  const siblingListingsRaw = l.siblingListings ?? l.sibling_listings ?? [];
  const siblingListings = Array.isArray(siblingListingsRaw) ? siblingListingsRaw.map((s: any) => ({
    id: s.id,
    slug: s.slug,
    nameAr: s.nameAr ?? s.name_ar ?? '',
    nameEn: s.nameEn ?? s.name_en ?? '',
    category: s.category ?? s.product_category ?? '',
    coverImage: s.coverImage ?? s.cover_image ?? null,
  })) : [];

  return {
    ...base,
    floorNumber: l.floorNumber ?? l.floor_number ?? null,
    reviews,
    packages,
    siblingListings,
    updatedAt: l.updatedAt ?? l.updated_at ?? '',
  };
}

// ── Public API ────────────────────────────────────────────────────────────────

export const listingService = {
  /**
   * GET /api/v1/public/listings
   * Paginated listing catalog. Each result is an individual vendor_product.
   */
  async getAll(params: ListingSearchParams = {}): Promise<PaginatedListings> {
    const json = await apiClient.get<any>(`/api/v1/public/listings${buildQuery(params)}`, {
      headers: buildHeaders()
    });

    return {
      listings:   (json.listings || []).map(mapListing),
      total:      json.total      ?? 0,
      page:       json.page       ?? 1,
      limit:      json.limit      ?? 12,
      totalPages: json.totalPages ?? 1,
    };
  },

  /**
   * GET /api/v1/public/listings/:slug
   * Returns full listing detail including features, reviews, packages, siblings.
   */
  async getBySlug(slug: string, customFetch?: typeof fetch): Promise<ListingDetail> {
    const json = await apiClient.get<any>(`/api/v1/public/listings/${encodeURIComponent(slug)}`, {
      fetch: customFetch,
      headers: buildHeaders()
    });

    if (!json.listing) throw new Error('Invalid response from server');
    return mapListingDetail(json.listing);
  },

  /**
   * GET /api/v1/public/listings/:id (by UUID)
   * Same as getBySlug — the backend accepts both slug and UUID.
   */
  async getById(id: string, customFetch?: typeof fetch): Promise<ListingDetail> {
    return this.getBySlug(id, customFetch);
  },

  /**
   * GET /api/v1/public/listings?featured=true
   * Returns featured listings for homepage hero sections.
   */
  async getFeatured(limit = 6): Promise<Listing[]> {
    const result = await this.getAll({ featured: true, limit });
    return result.listings;
  },

  /**
   * Returns sibling listings from the same vendor, excluding the current listing.
   * Uses the siblingListings already embedded in the ListingDetail response.
   */
  getSiblings(detail: ListingDetail) {
    return detail.siblingListings ?? [];
  },

  /**
   * POST /api/v1/public/bookings
   * Creates a listing-centric booking. Sends listingId (not venueId).
   */
  async createBooking(
    data: ListingBookingFormData,
    token: string,
    idempotencyKey: string,
  ): Promise<{
    bookingNumber: string;
    totalPrice: number;
    depositPaid: number;
    weddingDate: string;
    listingId: string;
  }> {
    const payload = {
      listingId:       data.listingId,
      eventDate:       data.eventDate,
      eventType:       data.eventType,
      guestCount:      data.guestCount,
      specialRequests: data.specialRequests,
      firstName:       data.contactInfo.firstName,
      lastName:        data.contactInfo.lastName,
      email:           data.contactInfo.email,
      phone:           data.contactInfo.phone,
    };

    return apiClient.post<any>('/api/v1/public/bookings', payload, {
      token,
      headers: {
        ...buildHeaders(),
        'Idempotency-Key': idempotencyKey,
      }
    });
  },

  /**
   * POST /api/v1/public/inquiries (via same-origin BFF)
   * Submits a public inquiry — no auth required.
   */
  async submitInquiry(
    payload: {
      listing_id?: string;
      vendorId?: string;
      name: string;
      mobile: string;
      is_whatsapp: boolean;
      email?: string;
      message?: string;
      event_date?: string;
      guest_count?: number;
      type?: 'listing' | 'concierge';
    }
  ): Promise<{ status: string; tracking_id?: string; message?: string; errors?: Record<string, string> }> {
    // Send via BFF, so we use raw fetch or apiClient to /bff/v1/public/inquiries
    return apiClient.post<any>('/bff/v1/public/inquiries', payload, { isServer: false });
  },
};
