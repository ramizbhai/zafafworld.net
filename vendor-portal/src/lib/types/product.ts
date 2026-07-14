/**
 * product.ts — Canonical Data Contract for Product Listing Module
 *
 * This file is the SINGLE SOURCE OF TRUTH for the Frontend <-> Backend API contract.
 * All wizard steps and related components MUST import types from here.
 *
 * Backend equivalent: backend-rust/src/routes/vendor_management/products.rs
 *   - CreateProductRequest (serde rename_all = "camelCase")
 *   - UpdateProductRequest (serde rename_all = "camelCase")
 *   - GalleryItemInput     (serde rename_all = "camelCase")
 *
 * CONVENTION: All JSON keys are camelCase (enforced by serde rename_all on Rust side).
 */

// ─── ENUM TYPES ──────────────────────────────────────────────────────────────

/** All valid product categories — must match DB CHECK constraint chk_product_category */
export type ProductCategory =
    | 'wedding-palace'
    | 'hotel-venue'
    | 'villa-resort'
    | 'restaurant-event'
    | 'outdoor-garden'
    | 'rooftop-venue'
    | 'private-beach'
    | 'chalet'
    | 'wedding-gown'
    | 'haute-couture'
    | 'abaya-jalabiya'
    | 'groom-attire'
    | 'hair-makeup'
    | 'henna-art'
    | 'photography-video'
    | 'photo-studio'
    | 'catering'
    | 'wedding-cake'
    | 'wedding-sweets'
    | 'entertainment-dj'
    | 'wedding-jewelry'
    | 'wedding-gifts'
    | 'wedding-planner'
    | 'flowers-floral'
    | 'wedding-invitation'
    | 'lighting-av'
    | 'wedding-car';

/** All valid gender sections — must match DB CHECK constraint chk_gender_section */
export type GenderSection =
    | 'women_only'
    | 'men_only'
    | 'mixed'
    | 'dual_parallel'
    | 'family'
    | 'both_sections'
    | 'not_applicable';

// ─── REQUEST PAYLOADS (Frontend → API) ───────────────────────────────────────

/**
 * Represents a single gallery item in API requests.
 * Maps to GalleryItemInput Rust struct (serde rename_all = "camelCase").
 */
export interface GalleryItemPayload {
    imageUrl: string;
    filePath?: string | null;
    isCover: boolean;
    sortOrder: number;
    mediaType: 'image' | 'video';
    fileSize?: number | null;
    thumbnailUrl?: string | null;
    durationSeconds?: number | null;
    caption?: string | null;
}

/**
 * Represents a single item in a gallery reorder request.
 * Maps to ImageSortItem Rust struct (serde rename_all = "camelCase").
 */
export interface ReorderImagePayload {
    id: string; // UUID string
    sortOrder: number;
}

/**
 * Full product listing payload for create (POST) and update (PUT) requests.
 * All fields optional except where noted — backend uses COALESCE for partial updates.
 */
export interface ProductListingPayload {
    // Step 1: Category
    productCategory?: ProductCategory;
    // Required for PUT (optimistic concurrency control)
    version?: number;

    // Step 2: Basic Info
    titleEn?: string | null;
    titleAr?: string | null;
    /** null when priceOnInquiry is true */
    basePriceSar?: number | null;
    priceOnInquiry?: boolean;
    /** Integer: 10–100 */
    depositPercentage?: number;
    /** UUID string */
    cityId?: string | null;
    googleMapsUrl?: string | null;
    latitude?: number | null;
    longitude?: number | null;

    // Step 3: Descriptions & SEO
    descriptionAr?: string | null;
    descriptionEn?: string | null;
    metaTitleAr?: string | null;
    metaTitleEn?: string | null;
    metaDescriptionAr?: string | null;
    metaDescriptionEn?: string | null;

    // Step 4: Cultural / GCC Settings
    genderSection?: GenderSection | null;
    culturalAttributes?: Record<string, boolean | string | number>;

    // Step 5: Category Details
    /** Dynamic category-specific attributes stored as JSONB */
    attributes?: Record<string, unknown>;
    /** Dynamic features selected by vendor, keys are feature UUIDs */
    featuresSelection?: Record<string, boolean | string | number>;

    // Step 6: Coordinator
    coordinatorNameEn?: string | null;
    coordinatorNameAr?: string | null;
    coordinatorPhone?: string | null;
    coordinatorWhatsapp?: string | null;
    coordinatorEmail?: string | null;
    coordinatorMobile?: string | null;

    // Step 7: Gallery
    galleryItems?: GalleryItemPayload[];
}

// ─── API RESPONSE SHAPES (API → Frontend) ────────────────────────────────────

/**
 * Shape of a product returned by GET /vendor/products/:id
 * and GET /vendor/products/:id/edit-context
 *
 * Maps to row_to_product_json() in products.rs.
 */
export interface ProductApiResponse {
    id: string;
    vendorId: string;

    // Bilingual content
    titleAr: string;
    titleEn: string;
    title: string; // legacy — same as titleEn
    descriptionAr: string;
    descriptionEn: string;
    metaTitleAr: string;
    metaTitleEn: string;
    metaDescriptionAr: string;
    metaDescriptionEn: string;

    slug: string;
    productCategory: ProductCategory;
    genderSection: GenderSection | null;
    totalCapacity: number | null;
    qualityScore: number;
    googleMapsUrl: string | null;
    latitude: number | null;
    longitude: number | null;
    featuresSelection: Record<string, unknown>;
    culturalAttributes: Record<string, unknown>;
    attributes: Record<string, unknown>;

    pricing: {
        basePriceSar: number | null;
        depositPercentage: number;
        priceOnInquiry: boolean;
    };

    coordinator: {
        nameAr: string;
        nameEn: string;
        phone: string;
        whatsapp: string;
        avatar: string | null;
        gender: string;
        email: string;
        mobile: string;
    };

    /**
     * NOTE: cityId is nested here — use product.metadata.cityId
     * It is NOT available at the top level of the response.
     */
    metadata: {
        crmProductId: string | null;
        status: string;
        rejectionReason: string | null;
        isAvailable: boolean;
        isFeatured: boolean;
        featuredUntil: string | null;
        version: number;
        createdAt: string;
        updatedAt: string;
        cityId: string | null;
    };
}

/**
 * Shape of a gallery image returned by GET /vendor/products/:id/images
 * and embedded in GET /vendor/products/:id/edit-context
 */
export interface GalleryImageApiResponse {
    id: string;
    productId: string | null;
    imageUrl: string;
    filePath: string | null;
    isCover: boolean;
    sortOrder: number;
    caption: string | null;
    createdAt: string;
    mediaType: 'image' | 'video';
    fileUrl: string;
    thumbnailUrl: string | null;
    fileSize: number | null;
    durationSeconds: number | null;
}

/**
 * Response from POST /vendor/products and PUT /vendor/products/:id
 */
export interface ProductMutationResponse {
    status: 'success';
    message: string;
    productId?: string;
    id?: string;
    slug?: string;
    product?: {
        id?: string;
        version: number;
        titleEn?: string;
        titleAr?: string;
    };
}
