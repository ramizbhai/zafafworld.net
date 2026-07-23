import { writable } from 'svelte/store';
import { getOrCreateTraceId, clearTraceId } from '$lib/utils/trace';


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
    status?: 'queued' | 'uploading' | 'processing' | 'completed' | 'failed';
    error?: string;
    xhr?: XMLHttpRequest;
    previewUrl?: string;
    fileId?: string;
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
    submitError: string;
    formData: ListingFormData;
    savedFormData: ListingFormData;
    schema?: any;
    schemaLoading?: boolean;
    schemaError?: string | null;
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
    ssRemove('zafaf_wiz_new_active');
    clearTraceId();
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

    const fData = JSON.parse(JSON.stringify(defaultFormData));
    return {
        productId:            storedProductId   ?? null,
        version:              storedVersion     ? parseInt(storedVersion, 10)     : 1,
        highestCompletedStep: storedHighestStep ? parseInt(storedHighestStep, 10) : 0,
        submitError:          '',
        formData:             fData,
        savedFormData:        JSON.parse(JSON.stringify(fData)),
    };
}

// ── Store factory ─────────────────────────────────────────────────────────────
function createListingStore() {
    const { subscribe, set, update } = writable<ListingState>(buildInitialState());

    return {
        subscribe,
        set,
        update,

        /** Returns the wizard-scoped trace ID (created lazily on first call). */
        getTraceId: (): string => getOrCreateTraceId(),

        setSchema: (schema: any) => update(s => {
            s.schema = schema;
            return s;
        }),

        setSchemaLoading: (loading: boolean) => update(s => {
            s.schemaLoading = loading;
            return s;
        }),

        setSchemaError: (error: string | null) => update(s => {
            s.schemaError = error;
            return s;
        }),

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

        setError: (error: string) => update(s => {
            s.submitError = error;
            return s;
        }),

        /**
         * Clears all per-category data from both formData and savedFormData.
         *
         * Call this after a successful Step 1 save when the category was changed
         * in edit mode. Attributes, cultural data, features, and gender section are
         * all category-specific: leaving stale values from the previous category
         * would cause later steps to submit invalid data to the backend.
         */
        clearCategoryDependentData: () => update(s => {
            const cleared = {
                categoryAttributes:  {} as Record<string, any>,
                featuresSelection:   {} as Record<string, boolean>,
                culturalAttributes:  {} as Record<string, any>,
                genderSection:       '',
            };
            s.formData      = { ...s.formData,      ...cleared };
            s.savedFormData = { ...s.savedFormData,  ...JSON.parse(JSON.stringify(cleared)) };
            return s;
        }),

        // Commit current formData fields to savedFormData to mark them as clean/saved
        commitStepSave: (stepId: number) => update(s => {
            if (stepId === 1) {
                s.savedFormData.selectedCategory = s.formData.selectedCategory;
            } else if (stepId === 2) {
                s.savedFormData.titleEn = s.formData.titleEn;
                s.savedFormData.titleAr = s.formData.titleAr;
                s.savedFormData.basePriceSar = s.formData.basePriceSar;
                s.savedFormData.priceOnInquiry = s.formData.priceOnInquiry;
                s.savedFormData.depositPercentage = s.formData.depositPercentage;
                s.savedFormData.selectedCityId = s.formData.selectedCityId;
                s.savedFormData.googleMapsUrl = s.formData.googleMapsUrl;
                s.savedFormData.latitude = s.formData.latitude;
                s.savedFormData.longitude = s.formData.longitude;
            } else if (stepId === 3) {
                s.savedFormData.descriptionAr = s.formData.descriptionAr;
                s.savedFormData.descriptionEn = s.formData.descriptionEn;
                s.savedFormData.metaTitleAr = s.formData.metaTitleAr;
                s.savedFormData.metaTitleEn = s.formData.metaTitleEn;
                s.savedFormData.metaDescriptionAr = s.formData.metaDescriptionAr;
                s.savedFormData.metaDescriptionEn = s.formData.metaDescriptionEn;
            } else if (stepId === 4) {
                s.savedFormData.genderSection = s.formData.genderSection;
                s.savedFormData.culturalAttributes = JSON.parse(JSON.stringify(s.formData.culturalAttributes));
            } else if (stepId === 5) {
                s.savedFormData.categoryAttributes = JSON.parse(JSON.stringify(s.formData.categoryAttributes));
                s.savedFormData.featuresSelection = JSON.parse(JSON.stringify(s.formData.featuresSelection));
            } else if (stepId === 6) {
                s.savedFormData.coordinatorNameAr = s.formData.coordinatorNameAr;
                s.savedFormData.coordinatorNameEn = s.formData.coordinatorNameEn;
                s.savedFormData.coordinatorPhone = s.formData.coordinatorPhone;
                s.savedFormData.coordinatorWhatsapp = s.formData.coordinatorWhatsapp;
                s.savedFormData.coordinatorEmail = s.formData.coordinatorEmail;
                s.savedFormData.coordinatorMobile = s.formData.coordinatorMobile;
            } else if (stepId === 7) {
                s.savedFormData.coverItem = s.formData.coverItem ? JSON.parse(JSON.stringify(s.formData.coverItem)) : null;
                s.savedFormData.galleryItems = JSON.parse(JSON.stringify(s.formData.galleryItems));
            }
            // Steps 8 (Preview) and 9 (Submit) are read-only review/action steps.
            // They own no form data, so commitStepSave is intentionally a no-op for them.
            return s;
        }),

        // Check if the current formData of a step has differences against savedFormData.
        // Steps 8 (Preview) and 9 (Submit) are read-only review/action steps that own no
        // form data — they always report as not dirty so no spurious PUT requests are made.
        // All other steps fall through to `return true` (dirty) as a safe default.
        isStepDirty: (stepId: number, s: ListingState) => {
            if (stepId === 1) {
                return s.formData.selectedCategory !== s.savedFormData.selectedCategory;
            }
            if (stepId === 2) {
                return s.formData.titleEn !== s.savedFormData.titleEn ||
                       s.formData.titleAr !== s.savedFormData.titleAr ||
                       s.formData.basePriceSar !== s.savedFormData.basePriceSar ||
                       s.formData.priceOnInquiry !== s.savedFormData.priceOnInquiry ||
                       s.formData.depositPercentage !== s.savedFormData.depositPercentage ||
                       s.formData.selectedCityId !== s.savedFormData.selectedCityId ||
                       s.formData.googleMapsUrl !== s.savedFormData.googleMapsUrl ||
                       s.formData.latitude !== s.savedFormData.latitude ||
                       s.formData.longitude !== s.savedFormData.longitude;
            }
            if (stepId === 3) {
                return s.formData.descriptionAr !== s.savedFormData.descriptionAr ||
                       s.formData.descriptionEn !== s.savedFormData.descriptionEn ||
                       s.formData.metaTitleAr !== s.savedFormData.metaTitleAr ||
                       s.formData.metaTitleEn !== s.savedFormData.metaTitleEn ||
                       s.formData.metaDescriptionAr !== s.savedFormData.metaDescriptionAr ||
                       s.formData.metaDescriptionEn !== s.savedFormData.metaDescriptionEn;
            }
            if (stepId === 4) {
                return s.formData.genderSection !== s.savedFormData.genderSection ||
                       JSON.stringify(s.formData.culturalAttributes) !== JSON.stringify(s.savedFormData.culturalAttributes);
            }
            if (stepId === 5) {
                return JSON.stringify(s.formData.categoryAttributes) !== JSON.stringify(s.savedFormData.categoryAttributes) ||
                       JSON.stringify(s.formData.featuresSelection) !== JSON.stringify(s.savedFormData.featuresSelection);
            }
            if (stepId === 6) {
                return s.formData.coordinatorNameAr !== s.savedFormData.coordinatorNameAr ||
                       s.formData.coordinatorNameEn !== s.savedFormData.coordinatorNameEn ||
                       s.formData.coordinatorPhone !== s.savedFormData.coordinatorPhone ||
                       s.formData.coordinatorWhatsapp !== s.savedFormData.coordinatorWhatsapp ||
                       s.formData.coordinatorEmail !== s.savedFormData.coordinatorEmail ||
                       s.formData.coordinatorMobile !== s.savedFormData.coordinatorMobile;
            }
            if (stepId === 7) {
                const coverChanged = (s.formData.coverItem?.url !== s.savedFormData.coverItem?.url) ||
                                     (s.formData.coverItem?.status !== s.savedFormData.coverItem?.status);
                if (coverChanged) return true;

                if (s.formData.galleryItems.length !== s.savedFormData.galleryItems.length) return true;
                for (let i = 0; i < s.formData.galleryItems.length; i++) {
                    const itemA = s.formData.galleryItems[i];
                    const itemB = s.savedFormData.galleryItems[i];
                    if (itemA.url !== itemB.url || itemA.status !== itemB.status || itemA.sortOrder !== itemB.sortOrder) {
                        return true;
                    }
                }
                return false;
            }
            // Steps 8 and 9 own no form data — always report clean (not dirty).
            if (stepId === 8 || stepId === 9) {
                return false;
            }
            // Unknown step: safe default is dirty to avoid silently skipping saves.
            return true;
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
                mediaType: img.mediaType || img.media_type || 'image',
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
                // cityId is always nested in metadata in API response — no snake_case fallback needed
                selectedCityId: product.metadata?.cityId ?? '',
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

            // Capture initial clean snapshot
            s.savedFormData = JSON.parse(JSON.stringify(s.formData));
            return s;
        }),

        reset: () => {
            // Wipe sessionStorage so a future fresh wizard starts clean
            ssClearAll();
            const fData = JSON.parse(JSON.stringify(defaultFormData));
            set({
                productId: null,
                version: 1,
                highestCompletedStep: 0,
                submitError: '',
                formData: fData,
                savedFormData: JSON.parse(JSON.stringify(fData)),
            });
        },
    };
}

export const listingStore = createListingStore();
