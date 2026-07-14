import { invalidateAll } from '$app/navigation';
export class ProfileState {
    name_en = $state('');
    name_ar = $state('');
    description_en = $state('');
    description_ar = $state('');
    crm_venue_id = $state('');
    star_rating = $state('');
    website = $state('');
    maps_url = $state('');
    video_url_1 = $state('');
    
    has_partition = $state(false);
    selected_city_id = $state('');
    capacity_min = $state('');
    capacity_max = $state('');
    selectedAmenities = $state<string[]>([]);

    clientError = $state('');
    isSubmitting = $state(false);

    constructor() {}

    initialize(data: any) {
        if (!data?.vendor) return;
        this.name_en = data.vendor.name_en ?? '';
        this.name_ar = data.vendor.name_ar ?? '';
        this.description_en = data.vendor.description_en ?? '';
        this.description_ar = data.vendor.description_ar ?? '';
        this.crm_venue_id = data.vendor.crm_venue_id ?? '';
        this.star_rating = data.vendor.star_rating ?? '';
        this.website = data.vendor.website ?? '';
        this.maps_url = data.vendor.maps_url ?? '';
        this.video_url_1 = data.vendor.video_url_1 ?? '';
        this.has_partition = data.vendor.has_partition ?? false;
        this.selected_city_id = data.vendor.city_id ?? '';
        this.capacity_min = data.vendor.capacity_min ?? '';
        this.capacity_max = data.vendor.capacity_max ?? '';
        this.selectedAmenities = data.vendor.amenities || [];
    }

    handleEnhance = (i18n: any) => {
        return ({ cancel }: { cancel: () => void }) => {
            this.clientError = '';

            if (!this.name_en.trim() || !this.name_ar.trim()) {
                this.clientError = i18n.t.pagesConfig.tradeNameRequired;
                cancel();
                return;
            }

            this.isSubmitting = true;

            return async ({ result, update }: any) => {
                this.isSubmitting = false;
                
                if (result.type === 'success') {
                    // Dispatch targeted background update for layout dependencies instantly
                    await invalidateAll();
                }
                
                // Smoothly flush local form state transformations and errors blocks
                await update();
            };
        };
    };
}
