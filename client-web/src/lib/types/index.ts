// ── Venue Types ──────────────────────────────────────────────────────────

export type VenueCategory =
    | 'wedding'
    | 'engagement'
    | 'corporate'
    | 'birthday'
    | 'conference'
    | 'exhibition';

export type AmenityKey =
    | 'parking'
    | 'catering'
    | 'decoration'
    | 'music'
    | 'photography'
    | 'ac'
    | 'wifi'
    | 'stage'
    | 'valet'
    | 'bridal_suite'
    | 'outdoor'
    | 'smoking_area'
    | 'in_house_catering'
    | 'valet_parking'
    | 'zaffa_setup';

export interface Amenity {
    key: AmenityKey;
    labelAr: string;
    labelEn: string;
    icon: string;
}

export interface VenueImage {
    id: string;
    url: string;
    alt: string;
    isPrimary: boolean;
}

export interface VenuePricing {
    basePrice: number;
    weekendSurcharge?: number;
    depositPercentage: number;
    includedServices: string[];
    additionalServices: {
        name: string;
        price: number;
    }[];
}

export interface VenueLocation {
    city: string;
    district: string;
    address: string;
    lat: number;
    lng: number;
    mapsUrl?: string;
    country?: string;
}

export interface VenueReview {
    id: string;
    authorName: string;
    authorAvatar?: string;
    rating: number;
    date: string;
    comment: string;
    eventType: VenueCategory;
    helpful: number;
}

export interface Venue {
    id: string;
    slug: string;
    nameAr: string;
    nameEn: string;
    descriptionAr: string;
    descriptionEn: string;
    category: VenueCategory[];
    images: VenueImage[];
    pricing: VenuePricing;
    location: VenueLocation;
    capacity: {
        min: number;
        max: number;
    };
    areaSqm: number;
    amenities: AmenityKey[];
    rating: number;
    reviewCount: number;
    reviews: VenueReview[];
    isFeatured: boolean;
    isAvailable: boolean;
    vendor: VendorSummary;
    phone?: string;
    email?: string;
    website?: string;
    mapsUrl?: string;
    coordinator?: any;
    createdAt: string;
    updatedAt: string;
    halls?: VenueHall[];
    packages?: any[];
}

export interface VenueHall {
    id: string;
    title: string;
    slug: string;
    description: string | null;
    productCategory: string;
    attributes: Record<string, any>;
    basePriceSar: number | null;
    depositPercentage: number;
    status: string;
    isAvailable: boolean;
    images: { id: string; url: string; alt: string; isPrimary: boolean }[];
}

// ── Vendor Types ──────────────────────────────────────────────────────────

export interface VendorSummary {
    id: string;
    nameAr: string;
    nameEn: string;
    avatar?: string;
    rating: number;
    venueCount: number;
    verified: boolean;
}

export interface Vendor extends VendorSummary {
    descriptionAr: string;
    descriptionEn: string;
    phone: string;
    email: string;
    website?: string;
    venues: string[]; // venue IDs
    socialLinks: {
        instagram?: string;
        twitter?: string;
        facebook?: string;
        tiktok?: string;
    };
    createdAt: string;
}

// ── Booking Types ──────────────────────────────────────────────────────────

export type BookingStatus =
    | 'pending'
    | 'confirmed'
    | 'cancelled'
    | 'completed'
    | 'rejected';

export interface BookingFormData {
    venueId: string;
    eventDate: string;
    eventType: VenueCategory;
    guestCount: number;
    specialRequests?: string;
    contactInfo: {
        firstName: string;
        lastName: string;
        email: string;
        phone: string;
    };
}

export interface Booking {
    id: string;
    bookingNumber: string;
    venue: Pick<Venue, 'id' | 'nameAr' | 'nameEn' | 'images' | 'location'>;
    status: BookingStatus;
    eventDate: string;
    eventType: VenueCategory;
    guestCount: number;
    totalPrice: number;
    depositPaid: number;
    specialRequests?: string;
    contactInfo: BookingFormData['contactInfo'];
    createdAt: string;
    updatedAt: string;
}

// ── User Types ──────────────────────────────────────────────────────────

export type UserRole = 'customer' | 'vendor' | 'admin';

export interface User {
    id: string;
    firstName: string;
    lastName: string;
    email: string;
    phone?: string;
    avatar?: string;
    role: UserRole;
    wishlist: string[]; // venue IDs
    createdAt: string;
}

// ── Blog Types ──────────────────────────────────────────────────────────

export type BlogCategory =
    | 'planning'
    | 'decoration'
    | 'photography'
    | 'catering'
    | 'fashion'
    | 'budgeting';

export interface BlogPost {
    id: string;
    slug: string;
    titleAr: string;
    titleEn: string;
    excerptAr: string;
    excerptEn: string;
    contentAr: string;
    contentEn: string;
    category: BlogCategory;
    tags: string[];
    coverImage: string;
    authorId: string;
    readTimeMinutes: number;
    publishedAt: string;
    updatedAt: string;
}

// ── Search Types ──────────────────────────────────────────────────────────

export interface VenueSearchParams {
    query?: string;
    city?: string;
    category?: VenueCategory;
    dateFrom?: string;
    dateTo?: string;
    guestCount?: number;
    priceMin?: number;
    priceMax?: number;
    amenities?: AmenityKey[];
    rating?: number;
    sortBy?: 'recommended' | 'price_asc' | 'price_desc' | 'rating' | 'newest';
    page?: number;
    limit?: number;

    // Phase 3 Saudi Filters
    partition?: boolean;
    minCapacity?: number;
    maxCapacity?: number;
}

export interface PaginatedResult<T> {
    data: T[];
    total: number;
    page: number;
    limit: number;
    totalPages: number;
}

// ── API Types ──────────────────────────────────────────────────────────

export interface ApiResponse<T> {
    success: boolean;
    data?: T;
    error?: string;
    message?: string;
}

export type AsyncState<T> =
    | { status: 'idle' }
    | { status: 'loading' }
    | { status: 'success'; data: T }
    | { status: 'error'; error: string };

// ═══════════════════════════════════════════════════════════════════════════
// LISTING-CENTRIC TYPE SYSTEM (Replaces VenueHall as the primary entity)
// ═══════════════════════════════════════════════════════════════════════════

export type GenderSection =
    | 'men_only'
    | 'women_only'
    | 'mixed'
    | 'dual_parallel'
    | 'family';

/** A vendor_products row — the canonical bookable listing */
export interface Listing {
    id: string;
    slug: string;

    // V2 bilingual titles — use locale-aware rendering
    titleAr: string;          // Arabic title (primary for SA market)
    titleEn: string;          // English title
    title: string;            // Legacy: fallback (= titleAr || titleEn)

    // V2 bilingual descriptions
    descriptionAr: string | null;
    descriptionEn: string | null;
    description: string | null;  // Legacy fallback

    // SEO properties
    metaTitleAr: string | null;
    metaTitleEn: string | null;
    metaDescriptionAr: string | null;
    metaDescriptionEn: string | null;

    category: string;
    attributes: Record<string, any>;
    featuresSelection?: Record<string, any>;
    coordinator?: {
        nameAr: string | null;
        nameEn: string | null;
        phone: string | null;
        whatsapp: string | null;
        email: string | null;
        mobile: string | null;
        avatar: string | null;
        gender?: string | null;
    };

    // V2 GCC cultural fields
    genderSection: string | null;   // 'dual_parallel' | 'women_only' | 'men_only' | etc.
    priceOnInquiry: boolean;        // If true, show CTA instead of price

    // V2 quality & trust signals
    qualityScore: number;           // 0–100 listing completeness
    verificationLevel: string;      // 'basic' | 'verified' | 'premium_verified' | 'official_partner'

    subscriptionBadge?: {
        tierId: string;
        ar: string;
        en: string;
    };

    // Per-listing pricing
    basePriceSar: string | null;  // Decimal string from backend
    startingPrice: number | null;
    depositPercentage: number;
    currency: 'SAR' | 'AED' | 'EGP';

    // Images — listing-specific cover image, NOT vendor brand image
    coverImage: string;
    imageCount: number;
    images: ListingImage[];

    // Vendor brand summary (the umbrella)
    vendor: ListingVendorSummary;

    // Location (resolved from listing.city_id or vendor.city_id)
    citySlug: string | null;
    cityAr: string | null;
    cityEn: string | null;
    countryCode: string | null;
    googleMapsUrl: string | null;
    latitude: number | null;
    longitude: number | null;

    // Rating (vendor-level reviews, shown per listing)
    rating: {
        overall: number;
        quality: number;
        staff: number;
        communication: number;
        count: number;
    };

    isFeatured: boolean;
    isAvailable: boolean;

    // Timestamps
    createdAt: string;

    // Routing helpers (pre-computed by backend)
    detailUrl: string;    // /listings/:slug
    bookingUrl: string;   // /booking/listing/:id
}

/** Full listing detail (returned by GET /listings/:slug) */
export interface ListingDetail extends Listing {
    floorNumber: number | null;

    reviews: ListingReview[];
    packages: ListingPackage[];
    siblingListings: SiblingListing[];  // other listings by the same vendor
    updatedAt: string;
}

export interface ListingImage {
    id: string;
    url: string;
    alt: string;
    isPrimary: boolean;
    mediaType?: 'image' | 'video' | string;
    fileUrl?: string;
    thumbnailUrl?: string | null;
    fileSize?: number | null;
    durationSeconds?: number | null;
}

export interface ListingReview {
    id: string;
    authorName: string;
    rating: number;
    ratingQuality: number;
    ratingStaff: number;
    ratingCommunication: number;
    weddingDate: string;
    date: string;
    comment: string;
}

export interface ListingPackage {
    id: string;
    nameAr: string;
    nameEn: string;
    originalPrice: number;
    discountedPrice: number;
    isZafafExclusive: boolean;
    expiryDate: string;
}

export interface SiblingListing {
    id: string;
    slug: string;
    nameAr: string;
    nameEn: string;
    category: string;
    coverImage: string | null;
}

/** Vendor brand summary embedded inside a Listing */
export interface ListingVendorSummary {
    id: string;
    slug: string;
    nameAr: string;
    nameEn: string;
    phone: string | null;
    email: string | null;
    mapsUrl: string | null;
    website: string | null;
    // Only in ListingDetail:
    descriptionAr?: string | null;
    descriptionEn?: string | null;
    videoUrl?: string | null;
    starRating?: string | null;
    location?: {
        latitude: number | null;
        longitude: number | null;
        addressAr: string | null;
        addressEn: string | null;
        districtAr: string | null;
        districtEn: string | null;
    };
}

/** Search/filter params for the listing catalog */
export interface ListingSearchParams {
    category?: string;    // product_category value
    city?: string;        // city slug
    cityId?: string;
    countryId?: string;
    gender?: GenderSection;
    minCapacity?: number;
    maxCapacity?: number;
    priceMin?: number;
    priceMax?: number;
    amenities?: string[];
    featured?: boolean;
    sort?: 'price_asc' | 'price_desc' | 'rating' | 'newest' | 'featured' | 'weighted';
    page?: number;
    limit?: number;
    // V2 search params
    q?: string;                 // Full-text keyword search (Arabic + English)
    womenOnly?: boolean;        // Filter: gender_section = 'women_only'
    minWomenCapacity?: number;  // Filter: women_capacity >= N
    minMenCapacity?: number;    // Filter: men_capacity >= N
}

/** Booking form data — listing-centric (listing_id, NOT venue_id) */
export interface ListingBookingFormData {
    listingId: string;       // vendor_products.id
    eventDate: string;
    eventType: string;
    guestCount: number;
    specialRequests?: string;
    contactInfo: {
        firstName: string;
        lastName: string;
        email: string;
        phone: string;
    };
}

export interface PaginatedListings {
    listings: Listing[];
    total: number;
    page: number;
    limit: number;
    totalPages: number;
}
