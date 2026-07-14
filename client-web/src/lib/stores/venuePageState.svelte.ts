import { toasts } from '$lib/stores/toast.svelte.js';
import * as m from '$lib/paraglide/messages.js';

export function createVenuePageState() {
    let activeTab = $state<"overview" | "halls" | "gallery" | "amenities" | "reviews" | "location">("overview");
    let activeImage = $state(0);
    let isLightboxOpen = $state(false);
    let lightboxIndex = $state(0);
    let showAuthPopup = $state(false);
    let showInquiryModal = $state(false);
    
    let eventDate = $state("");
    let guestCount = $state<number | null>(null);
    let inquiryMessage = $state("");
    let submittingInquiry = $state(false);
    let inquiryError = $state("");
    
    let liveReviews = $state<any[]>([]);
    let averageRating = $state(0);
    let totalReviews = $state(0);
    
    let newRating = $state(5);
    let reviewText = $state("");
    let reviewPhotos = $state<string[]>([]);
    let submittingReview = $state(false);
    let showReviewSuccessOverlay = $state(false);
    
    let isSaved = $state(false);

    return {
        get activeTab() { return activeTab; },
        set activeTab(v) { activeTab = v; },

        get activeImage() { return activeImage; },
        set activeImage(v) { activeImage = v; },

        get isLightboxOpen() { return isLightboxOpen; },
        set isLightboxOpen(v) { isLightboxOpen = v; },

        get lightboxIndex() { return lightboxIndex; },
        set lightboxIndex(v) { lightboxIndex = v; },

        get showAuthPopup() { return showAuthPopup; },
        set showAuthPopup(v) { showAuthPopup = v; },

        get showInquiryModal() { return showInquiryModal; },
        set showInquiryModal(v) { showInquiryModal = v; },

        get eventDate() { return eventDate; },
        set eventDate(v) { eventDate = v; },

        get guestCount() { return guestCount; },
        set guestCount(v) { guestCount = v; },

        get inquiryMessage() { return inquiryMessage; },
        set inquiryMessage(v) { inquiryMessage = v; },

        get submittingInquiry() { return submittingInquiry; },
        set submittingInquiry(v) { submittingInquiry = v; },

        get inquiryError() { return inquiryError; },
        set inquiryError(v) { inquiryError = v; },

        get liveReviews() { return liveReviews; },
        set liveReviews(v) { liveReviews = v; },

        get averageRating() { return averageRating; },
        set averageRating(v) { averageRating = v; },

        get totalReviews() { return totalReviews; },
        set totalReviews(v) { totalReviews = v; },

        get newRating() { return newRating; },
        set newRating(v) { newRating = v; },

        get reviewText() { return reviewText; },
        set reviewText(v) { reviewText = v; },

        get reviewPhotos() { return reviewPhotos; },
        set reviewPhotos(v) { reviewPhotos = v; },

        get submittingReview() { return submittingReview; },
        set submittingReview(v) { submittingReview = v; },

        get showReviewSuccessOverlay() { return showReviewSuccessOverlay; },
        set showReviewSuccessOverlay(v) { showReviewSuccessOverlay = v; },

        get isSaved() { return isSaved; },
        set isSaved(v) { isSaved = v; },
        
        handlePhotoUpload(e: Event) {
            const target = e.target as HTMLInputElement;
            if (target.files) {
                Array.from(target.files).forEach((file) => {
                    const reader = new FileReader();
                    reader.onload = (ev) => {
                        if (ev.target?.result) {
                            reviewPhotos.push(ev.target.result as string);
                        }
                    };
                    reader.readAsDataURL(file);
                });
            }
        },
        
        removePhoto(index: number) {
            reviewPhotos = reviewPhotos.filter((_, i) => i !== index);
        },

        toggleSave(venueId: string, t: (a: string, e: string) => string) {
            const saved = JSON.parse(localStorage.getItem("zafaf_saved_vendors") || "[]");
            if (saved.includes(venueId)) {
                const updated = saved.filter((id: string) => id !== venueId);
                localStorage.setItem("zafaf_saved_vendors", JSON.stringify(updated));
                isSaved = false;
                toasts.push("success", t("تمت الإزالة من المفضلة", "Removed from saved list"));
            } else {
                saved.push(venueId);
                localStorage.setItem("zafaf_saved_vendors", JSON.stringify(saved));
                isSaved = true;
                toasts.push("success", t("تم الإضافة إلى المفضلة", "Saved to wishlist"));
            }
        }
    };
}
