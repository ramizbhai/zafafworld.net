<script lang="ts">
    import type { ListingDetail } from "$lib/types/index.js";
    import { MapPin, ShieldCheck, Star, ChevronRight, ChevronLeft, Calendar } from "lucide-svelte";
    import { getLocale } from "$lib/paraglide/runtime.js";
    import * as m from "$lib/paraglide/messages.js";

    let { listing, isLoading = false } = $props<{
        listing: ListingDetail | null;
        isLoading?: boolean;
    }>();

    const isAr = $derived(getLocale() === 'ar');

    // Extract basic fields
    const title = $derived(
        isAr
            ? (listing?.titleAr || listing?.title || "")
            : (listing?.titleEn || listing?.title || "")
    );

    const vendorName = $derived(
        isAr
            ? (listing?.vendor?.nameAr || "")
            : (listing?.vendor?.nameEn || "")
    );

    const city = $derived(
        isAr
            ? (listing?.cityAr || listing?.cityEn || "")
            : (listing?.cityEn || listing?.cityAr || "")
    );

    // Dynamic rating derivations
    const hasRating = $derived(!!(listing?.rating && listing.rating.count > 0));
    const overallRating = $derived(listing?.rating?.overall ?? 0);
    const reviewCount = $derived(listing?.rating?.count ?? 0);

    // Badge details
    const hasBadge = $derived(!!listing?.subscriptionBadge);
    const tierId = $derived(listing?.subscriptionBadge?.tierId || "free");
    const badgeLabel = $derived(
        isAr
            ? (listing?.subscriptionBadge?.ar || "مميز")
            : (listing?.subscriptionBadge?.en || "Premium")
    );

    // Verification check
    const isVerified = $derived(
        listing?.verificationLevel === 'verified' || 
        listing?.verificationLevel === 'premium_verified' || 
        listing?.verificationLevel === 'official_partner'
    );

    // Scroll helper to jump directly to reviews section
    function scrollToReviews(e: MouseEvent) {
        e.preventDefault();
        const reviewsSection = document.getElementById("reviews-section");
        if (reviewsSection) {
            reviewsSection.scrollIntoView({ behavior: "smooth", block: "start" });
        }
    }
</script>

{#if isLoading}
    <!-- Premium Shimmer Loading Skeleton -->
    <div class="w-full max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6 space-y-4 animate-pulse" aria-hidden="true">
        <!-- Breadcrumb skeleton -->
        <div class="h-4 w-48 bg-slate-200 rounded-sm"></div>
        <!-- Badges row skeleton -->
        <div class="flex gap-2">
            <div class="h-6 w-16 bg-slate-200 rounded-full"></div>
            <div class="h-6 w-24 bg-slate-200 rounded-full"></div>
        </div>
        <!-- Title skeleton -->
        <div class="h-10 w-2/3 bg-slate-200 rounded-md"></div>
        <!-- Vendor brand skeleton -->
        <div class="h-6 w-1/3 bg-slate-200 rounded-md"></div>
        <!-- Trust row skeleton -->
        <div class="flex gap-4">
            <div class="h-5 w-20 bg-slate-200 rounded-sm"></div>
            <div class="h-5 w-32 bg-slate-200 rounded-sm"></div>
        </div>
    </div>
{:else if listing}
    <div class="w-full max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6 space-y-4">
        
        <!-- Module A: Breadcrumb Navigation -->
        <nav 
            class="flex items-center gap-1.5 text-xs sm:text-sm text-slate-500 font-medium" 
            aria-label={isAr ? "مسار التنقل" : "Breadcrumb"}
        >
            <a href="/" class="hover:text-slate-900 transition-colors">
                {isAr ? "الرئيسية" : "Home"}
            </a>
            
            {#if isAr}
                <ChevronLeft size={14} class="shrink-0 text-slate-300" aria-hidden="true" />
            {:else}
                <ChevronRight size={14} class="shrink-0 text-slate-300" aria-hidden="true" />
            {/if}
            
            <a href="/search" class="hover:text-slate-900 transition-colors">
                {isAr ? "البحث" : "Search"}
            </a>

            {#if listing.category}
                {#if isAr}
                    <ChevronLeft size={14} class="shrink-0 text-slate-300" aria-hidden="true" />
                {:else}
                    <ChevronRight size={14} class="shrink-0 text-slate-300" aria-hidden="true" />
                {/if}
                <span class="text-slate-700 capitalize">
                    {listing.category}
                </span>
            {/if}
        </nav>

        <!-- Main Header Info Wrapper -->
        <div class="space-y-3">
            
            <!-- Module B: Badge Row -->
            <div class="flex flex-wrap items-center gap-2" role="group" aria-label={isAr ? "أوسمة التميز" : "Listing Badges"}>
                <!-- Subscription Tier Badge -->
                {#if hasBadge && tierId !== "free"}
                    <span 
                        class="px-2.5 py-0.5 text-xs font-bold uppercase tracking-wider rounded-md text-amber-950 bg-amber-400 border border-amber-500 shadow-xs flex items-center gap-1"
                        role="status"
                    >
                        <Star size={11} fill="currentColor" aria-hidden="true" />
                        <span>{badgeLabel}</span>
                    </span>
                {/if}

                <!-- Verification badge -->
                {#if isVerified}
                    <span 
                        class="px-2.5 py-0.5 text-xs font-semibold rounded-md text-emerald-700 bg-emerald-50 border border-emerald-200 flex items-center gap-1"
                        role="status"
                    >
                        <ShieldCheck size={13} class="text-emerald-600" aria-hidden="true" />
                        <span>{isAr ? "شريك معتمد" : "Verified Partner"}</span>
                    </span>
                {/if}

                <!-- Category badge -->
                {#if listing.category}
                    <span class="px-2.5 py-0.5 text-xs font-medium rounded-md text-slate-600 bg-slate-100 border border-slate-200">
                        {listing.category}
                    </span>
                {/if}
            </div>

            <!-- Module C & D: Title & Vendor Stack -->
            <div class="space-y-1">
                <h1 class="text-2xl sm:text-3xl md:text-4xl font-extrabold text-slate-900 tracking-tight leading-tight">
                    {title}
                </h1>
                
                {#if vendorName}
                    <p class="text-sm sm:text-base md:text-lg text-slate-500 font-medium flex items-center gap-1">
                        <span>{isAr ? "بواسطة" : "by"}</span>
                        <span class="text-slate-800 font-semibold">{vendorName}</span>
                    </p>
                {/if}
            </div>

            <!-- Module E: Trust & Location Row -->
            <div class="flex flex-wrap items-center gap-x-4 gap-y-2 text-xs sm:text-sm text-slate-500 border-t border-slate-100 pt-3 mt-1">
                
                <!-- Ratings Summary -->
                {#if hasRating}
                    <div class="flex items-center gap-1" aria-label={isAr ? `التقييم ${overallRating} من 5` : `Rating ${overallRating} out of 5`}>
                        <div class="flex items-center text-amber-500">
                            <Star size={16} fill="currentColor" aria-hidden="true" />
                        </div>
                        <span class="font-bold text-slate-800">{overallRating}</span>
                        <a 
                            href="#reviews-section" 
                            onclick={scrollToReviews}
                            class="text-slate-400 hover:text-amber-600 hover:underline transition-all"
                            aria-label={isAr ? `شاهد جميع التقييمات البالغ عددها ${reviewCount}` : `View all ${reviewCount} reviews`}
                        >
                            ({isAr ? `${reviewCount} تقييم` : `${reviewCount} reviews`})
                        </a>
                    </div>
                {:else}
                    <span class="text-slate-400 font-medium">
                        {isAr ? "لا توجد تقييمات بعد" : "No reviews yet"}
                    </span>
                {/if}

                <!-- SeparatorDot (hidden on mobile if wrapped) -->
                <span class="hidden sm:inline text-slate-300" aria-hidden="true">•</span>

                <!-- Location details -->
                {#if city}
                    <div class="flex items-center gap-1 text-slate-600">
                        <MapPin size={16} class="text-slate-400 shrink-0" aria-hidden="true" />
                        <span class="font-medium">{city}</span>
                        
                        {#if listing.googleMapsUrl}
                            <a 
                                href={listing.googleMapsUrl}
                                target="_blank"
                                rel="noopener noreferrer"
                                class="text-amber-600 hover:text-amber-700 font-semibold hover:underline shrink-0 flex items-center gap-0.5 ml-1"
                                aria-label={isAr ? "عرض الموقع الجغرافي على خرائط جوجل" : "View venue location on Google Maps"}
                            >
                                <span>{isAr ? "عرض على الخريطة" : "View on Map"}</span>
                            </a>
                        {/if}
                    </div>
                {/if}
            </div>

        </div>

    </div>
{/if}
