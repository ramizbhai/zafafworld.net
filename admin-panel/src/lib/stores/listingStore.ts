import { writable } from 'svelte/store';

export interface GalleryItem {
    id: string;
    url: string;
    file_path?: string;
    isCover: boolean;
    sortOrder: number;
    caption?: string | null;
    mediaType: 'image' | 'video';
    thumbnailUrl?: string | null;
    fileSize: number;
    durationSeconds?: number | null;
    file?: File;
    progress?: number;
    speed?: string;
    status?: 'uploading' | 'processing' | 'completed' | 'failed';
    error?: string;
    xhr?: XMLHttpRequest;
    previewUrl?: string;
}

export interface ListingFormData {
    // Step 1: Category
    selectedCategory: string;

    // Step 2: Basic Info
    titleAr: string;
    titleEn: string;
    basePriceSar: string;
    priceOnInquiry: boolean;
    depositPercentage: number;
    selectedCityId: string;
    googleMapsUrl: string;
    latitude: string;
    longitude: string;

    // Step 3: Descriptions & SEO
    descriptionAr: string;
    descriptionEn: string;
    metaTitleAr: string;
    metaTitleEn: string;
    metaDescriptionAr: string;
    metaDescriptionEn: string;

    // Step 4: Cultural / GCC Settings
    genderSection: string;
    culturalAttributes: Record<string, any>;

    // Step 5: Category Details
    categoryAttributes: Record<string, any>;
    featuresSelection: Record<string, any>;

    // Step 6: Coordinator
    coordinatorNameAr: string;
    coordinatorNameEn: string;
    coordinatorPhone: string;
    coordinatorWhatsapp: string;
    coordinatorEmail: string;
    coordinatorMobile: string;

    // Step 7: Gallery
    coverItem: GalleryItem | null;
    galleryItems: GalleryItem[];
}

export interface ListingState {
    productId: string | null;
    version: number;
    highestCompletedStep: number;
    isSubmitting: boolean;
    submitError: string;
    formData: ListingFormData;
    submitHandler: (() => Promise<void>) | null;
    canContinue: boolean;
}

// ── sessionStorage keys ───────────────────────────────────────────────────────
const SS_KEY_PRODUCT_ID            = 'zafaf_wiz_productId';
const SS_KEY_VERSION               = 'zafaf_wiz_version';
const SS_KEY_HIGHEST_STEP          = 'zafaf_wiz_highestStep';

/** Read a value from sessionStorage safely (SSR-safe). */
function ssGet(key: string): string | null {
    if (typeof window === 'undefined') return null;
    try { return sessionStorage.getItem(key); } catch { return null; }
}

/** Write a value to sessionStorage safely (SSR-safe). */
function ssSet(key: string, value: string): void {
    if (typeof window === 'undefined') return;
    try { sessionStorage.setItem(key, value); } catch { /* quota exceeded – ignore */ }
}

/** Remove a key from sessionStorage safely. */
function ssRemove(key: string): void {
    if (typeof window === 'undefined') return;
    try { sessionStorage.removeItem(key); } catch { /* ignore */ }
}

/** Flush all wizard-related sessionStorage keys. */
function ssClearAll(): void {
    ssRemove(SS_KEY_PRODUCT_ID);
    ssRemove(SS_KEY_VERSION);
    ssRemove(SS_KEY_HIGHEST_STEP);
}

// ── Default form state ────────────────────────────────────────────────────────
const defaultFormData: ListingFormData = {
    selectedCategory: '',
    titleAr: '',
    titleEn: '',
    basePriceSar: '',
    priceOnInquiry: false,
    depositPercentage: 25,
    selectedCityId: '',
    googleMapsUrl: '',
    latitude: '',
    longitude: '',
    descriptionAr: '',
    descriptionEn: '',
    metaTitleAr: '',
    metaTitleEn: '',
    metaDescriptionAr: '',
    metaDescriptionEn: '',
    genderSection: '',
    culturalAttributes: {},
    categoryAttributes: {},
    featuresSelection: {},
    coordinatorNameAr: '',
    coordinatorNameEn: '',
    coordinatorPhone: '',
    coordinatorWhatsapp: '',
    coordinatorEmail: '',
    coordinatorMobile: '',
    coverItem: null,
    galleryItems: [],
};

// ── Hydrate initial state from sessionStorage (runs once at module load time) ─
function buildInitialState(): ListingState {
    const storedProductId   = ssGet(SS_KEY_PRODUCT_ID);
    const storedVersion     = ssGet(SS_KEY_VERSION);
    const storedHighestStep = ssGet(SS_KEY_HIGHEST_STEP);

    return {
        productId:            storedProductId   ?? null,
        version:              storedVersion     ? parseInt(storedVersion, 10)     : 1,
        highestCompletedStep: storedHighestStep ? parseInt(storedHighestStep, 10) : 0,
        isSubmitting:         false,
        submitError:          '',
        formData:             JSON.parse(JSON.stringify(defaultFormData)),
        submitHandler:        null,
        canContinue:          false,
    };
}

// ── Store factory ─────────────────────────────────────────────────────────────
function createListingStore() {
    const { subscribe, set, update } = writable<ListingState>(buildInitialState());

    return {
        subscribe,

        setProductId: (id: string) => update(s => {
            s.productId = id;
            ssSet(SS_KEY_PRODUCT_ID, id);
            return s;
        }),

        setVersion: (version: number) => update(s => {
            s.version = version;
            ssSet(SS_KEY_VERSION, String(version));
            return s;
        }),

        setHighestStep: (step: number) => update(s => {
            if (step > s.highestCompletedStep) {
                s.highestCompletedStep = step;
                ssSet(SS_KEY_HIGHEST_STEP, String(step));
            }
            return s;
        }),

        updateFormData: (data: Partial<ListingFormData>) => update(s => {
            s.formData = { ...s.formData, ...data };
            return s;
        }),

        setSubmitting: (isSubmitting: boolean) => update(s => {
            s.isSubmitting = isSubmitting;
            return s;
        }),

        setError: (error: string) => update(s => {
            s.submitError = error;
            return s;
        }),

        setSubmitHandler: (handler: (() => Promise<void>) | null) => update(s => {
            s.submitHandler = handler;
            return s;
        }),

        setCanContinue: (val: boolean) => update(s => {
            s.canContinue = val;
            return s;
        }),

        submitCurrentStep: async () => {
            let handler: (() => Promise<void>) | null = null;
            let currentlySubmitting = false;

            update(s => {
                handler = s.submitHandler;
                currentlySubmitting = s.isSubmitting;
                return s;
            });

            // Prevent concurrent submissions (double clicks)
            if (currentlySubmitting) return;

            const h = handler as (() => Promise<void>) | null;
            if (h) {
                await h();
            }
        },

        initializeFromProduct: (product: any, images: any[] = []) => update(s => {
            s.productId = product.id;
            // The backend serializes version inside metadata.version (see row_to_product_json).
            // Fall back to top-level product.version for any future API changes.
            s.version = product.metadata?.version || product.version || 1;
            s.highestCompletedStep = 9;

            // Persist hydrated state to sessionStorage
            if (product.id) ssSet(SS_KEY_PRODUCT_ID, product.id);
            ssSet(SS_KEY_VERSION, String(s.version));
            ssSet(SS_KEY_HIGHEST_STEP, '9');

            const cover = images.find(img => img.isCover || img.is_cover);
            const coverItem = cover ? {
                id: cover.id,
                url: cover.imageUrl || cover.image_url || '',
                file_path: cover.filePath || cover.file_path,
                isCover: true,
                sortOrder: cover.sortOrder || cover.sort_order || 0,
                caption: cover.caption,
                mediaType: cover.mediaType || cover.media_type || 'image',
                thumbnailUrl: cover.thumbnailUrl || cover.thumbnail_url,
                fileSize: cover.fileSize || cover.file_size || 0,
                durationSeconds: cover.durationSeconds || cover.duration_seconds,
                status: 'completed' as any
            } : null;

            const galleryItems = images.filter(img => !img.isCover && !img.is_cover).map(img => ({
                id: img.id,
                url: img.imageUrl || img.image_url || '',
                file_path: img.filePath || img.file_path,
                isCover: false,
                sortOrder: img.sortOrder || img.sort_order || 0,
                caption: img.caption,
                mediaType: img.mediaType || cover?.media_type || 'image',
                thumbnailUrl: img.thumbnailUrl || img.thumbnail_url,
                fileSize: img.fileSize || img.file_size || 0,
                durationSeconds: img.durationSeconds || img.duration_seconds,
                status: 'completed' as any
            }));

            s.formData = {
                selectedCategory: product.productCategory || product.product_category || 'hotel-venue',
                titleAr: product.titleAr || product.title_ar || '',
                titleEn: product.titleEn || product.title_en || product.title || '',
                basePriceSar: product.pricing?.basePriceSar?.toString() || product.base_price_sar?.toString() || '',
                priceOnInquiry: product.pricing?.priceOnInquiry ?? product.price_on_inquiry ?? false,
                depositPercentage: product.pricing?.depositPercentage ?? product.deposit_percentage ?? 25,
                selectedCityId: product.metadata?.cityId || product.city_id || '',
                googleMapsUrl: product.googleMapsUrl || product.google_maps_url || '',
                latitude: product.latitude?.toString() || '',
                longitude: product.longitude?.toString() || '',
                descriptionAr: product.descriptionAr || product.description_ar || '',
                descriptionEn: product.descriptionEn || product.description_en || product.description || '',
                metaTitleAr: product.metaTitleAr || product.meta_title_ar || '',
                metaTitleEn: product.metaTitleEn || product.meta_title_en || '',
                metaDescriptionAr: product.metaDescriptionAr || product.meta_description_ar || '',
                metaDescriptionEn: product.metaDescriptionEn || product.meta_description_en || '',
                genderSection: product.genderSection || product.gender_section || '',
                culturalAttributes: product.culturalAttributes || {},
                categoryAttributes: product.attributes || {},
                featuresSelection: product.featuresSelection || product.features_selection || {},
                coordinatorNameAr: product.coordinator?.nameAr || product.coordinator_name_ar || '',
                coordinatorNameEn: product.coordinator?.nameEn || product.coordinator_name_en || product.coordinator_name || '',
                coordinatorPhone: product.coordinator?.phone || product.coordinator_phone || '',
                coordinatorWhatsapp: product.coordinator?.whatsapp || product.coordinator_whatsapp || '',
                coordinatorEmail: product.coordinator?.email || product.coordinator_email || '',
                coordinatorMobile: product.coordinator?.mobile || product.coordinator_mobile || '',
                coverItem,
                galleryItems
            };
            return s;
        }),

        reset: () => {
            // Wipe sessionStorage so a future fresh wizard starts clean
            ssClearAll();
            set({
                productId: null,
                version: 1,
                highestCompletedStep: 0,
                isSubmitting: false,
                submitError: '',
                formData: JSON.parse(JSON.stringify(defaultFormData)),
                submitHandler: null,
                canContinue: false,
            });
        },
    };
}

export const listingStore = createListingStore();
