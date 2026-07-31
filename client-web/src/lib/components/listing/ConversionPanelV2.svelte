<script lang="ts">
    import { onMount } from "svelte";
    import type { ListingDetail, Listing } from "$lib/types/index.js";
    import { ShieldCheck, PhoneCall, MessageCircle, Heart, Share2, Calendar } from "lucide-svelte";
    import { getLocale } from "$lib/paraglide/runtime.js";
    import { listingService } from "$lib/services/api/listing.service.js";
    import InquiryForm from "$lib/components/shared/InquiryForm.svelte";
    import ListingCard from "$lib/components/shared/ListingCard.svelte";

    let { listing } = $props<{ listing: ListingDetail }>();

    const isAr = $derived(getLocale() === "ar");

    // Tier definitions
    const tierId = $derived(listing.subscriptionBadge?.tierId || "free");
    const isDiamond = $derived(tierId === "diamond");
    const isVip = $derived(tierId === "vip");
    const isGold = $derived(tierId === "gold");
    
    // Direct contact and competitors logic
    const showDirectContact = $derived(isDiamond || isVip);
    const hideCompetitors = $derived(isDiamond || isVip);

    // Save/Wishlist interactive state
    let isSaved = $state(false);

    // Share status feedback
    let shareFeedback = $state(false);

    // Similar listings state
    let relatedListings = $state<Listing[]>([]);
    let loading = $state(true);

    onMount(async () => {
        if (hideCompetitors) {
            loading = false;
            return;
        }

        try {
            const data = await listingService.getAll({
                category: listing.category,
                limit: 4,
            });
            relatedListings = (data.listings || [])
                .filter((l: Listing) => l.id !== listing.id)
                .slice(0, 3);
        } catch (e) {
            console.error("Failed to fetch related listings", e);
        } finally {
            loading = false;
        }
    });

    // Formatting currency and price
    const priceText = $derived.by(() => {
        if (listing.priceOnInquiry) {
            return isAr ? "السعر عند الطلب" : "Price on Request";
        }
        if (listing.startingPrice) {
            const formatted = new Intl.NumberFormat(isAr ? 'ar-SA' : 'en-US').format(listing.startingPrice);
            const currencyText = isAr ? "ر.س" : "SAR";
            return `${formatted} ${currencyText}`;
        }
        return isAr ? "السعر عند الطلب" : "Price on Request";
    });

    async function handleShare() {
        if (navigator.share) {
            try {
                await navigator.share({
                    title: isAr ? listing.titleAr : listing.titleEn,
                    url: window.location.href
                });
            } catch (err) {
                console.warn("Share failed:", err);
            }
        } else {
            // Copy to clipboard fallback
            try {
                await navigator.clipboard.writeText(window.location.href);
                shareFeedback = true;
                setTimeout(() => {
                    shareFeedback = false;
                }, 2000);
            } catch (err) {
                console.error("Clipboard copy failed:", err);
            }
        }
    }
</script>

<aside class="sticky top-24 space-y-6">
    
    <!-- 1. Primary Conversion panel Card -->
    <div class="bg-white rounded-2xl border border-slate-100 shadow-xl overflow-hidden relative transition-all">
        
        <!-- Premium gradient header highlight based on subscription tier -->
        {#if isDiamond}
            <div class="absolute top-0 left-0 right-0 h-1.5 bg-gradient-to-r from-amber-400 via-amber-500 to-yellow-600"></div>
        {:else if isVip}
            <div class="absolute top-0 left-0 right-0 h-1.5 bg-gradient-to-r from-purple-500 to-indigo-600"></div>
        {:else if isGold}
            <div class="absolute top-0 left-0 right-0 h-1.5 bg-gradient-to-r from-slate-700 to-slate-900"></div>
        {/if}

        <div class="p-6 space-y-6">
            
            <!-- Price Summary Row -->
            <div class="flex items-baseline justify-between border-b border-slate-100 pb-4">
                <div class="space-y-1">
                    <span class="text-xs text-slate-400 font-bold uppercase tracking-wider">
                        {isAr ? "السعر المبدئي" : "Starting Price"}
                    </span>
                    <p class="text-2xl font-extrabold text-slate-900 tracking-tight">
                        {priceText}
                    </p>
                </div>

                <!-- Availability status indicator badge -->
                <div>
                    {#if listing.isAvailable}
                        <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-semibold text-emerald-800 bg-emerald-50 border border-emerald-200">
                            <span class="w-1.5 h-1.5 rounded-full bg-emerald-500"></span>
                            {isAr ? "متوفر" : "Available"}
                        </span>
                    {:else}
                        <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-semibold text-slate-800 bg-slate-50 border border-slate-200">
                            <span class="w-1.5 h-1.5 rounded-full bg-slate-400"></span>
                            {isAr ? "غير متوفر" : "Fully Booked"}
                        </span>
                    {/if}
                </div>
            </div>

            <!-- Inquiry Request Header -->
            <div class="space-y-2">
                <h4 class="text-lg font-extrabold text-slate-900 leading-tight">
                    {isAr ? "طلب تسعير ومعلومات" : "Request Quote & Info"}
                </h4>
                <p class="text-xs sm:text-sm text-slate-400">
                    {isAr ? "تواصل مباشرة للحصول على تسعيرة مخصصة ومواعيد التوفر." : "Send your event details to coordinate pricing."}
                </p>
            </div>

            <!-- Inquiry form inputs -->
            <InquiryForm listingId={listing.id} />

            <!-- Direct contacts block (VIP/Diamond tiers only) -->
            {#if showDirectContact && (listing.vendor?.phone || listing.coordinator?.whatsapp)}
                <div class="space-y-3 pt-4 border-t border-slate-100">
                    <h5 class="text-xs font-bold text-slate-400 uppercase tracking-wider">
                        {isAr ? "اتصال مباشر بالمنسق" : "Direct Contact Shortcuts"}
                    </h5>
                    
                    <div class="grid grid-cols-2 gap-2">
                        <!-- Direct Phone Call -->
                        {#if listing.vendor?.phone}
                            <a 
                                href="tel:{listing.vendor.phone}"
                                class="inline-flex items-center justify-center gap-2 px-3 py-2.5 rounded-xl border border-slate-200 hover:border-amber-500 hover:bg-amber-50/20 text-slate-700 hover:text-slate-900 font-semibold text-xs transition-all active:scale-97"
                            >
                                <PhoneCall size={14} class="text-amber-500 shrink-0" />
                                <span>{isAr ? "اتصال" : "Call Phone"}</span>
                            </a>
                        {/if}

                        <!-- WhatsApp chat link -->
                        {#if listing.coordinator?.whatsapp}
                            <a 
                                href="https://wa.me/{listing.coordinator.whatsapp}"
                                target="_blank"
                                rel="noopener noreferrer"
                                class="inline-flex items-center justify-center gap-2 px-3 py-2.5 rounded-xl border border-slate-200 hover:border-emerald-500 hover:bg-emerald-50/20 text-slate-700 hover:text-slate-900 font-semibold text-xs transition-all active:scale-97"
                            >
                                <MessageCircle size={14} class="text-emerald-500 shrink-0" />
                                <span>{isAr ? "واتساب" : "WhatsApp"}</span>
                            </a>
                        {/if}
                    </div>
                </div>
            {/if}

            <!-- Save / Share action links grid -->
            <div class="grid grid-cols-2 gap-2 border-t border-slate-100 pt-4">
                <!-- Save / Wishlist -->
                <button 
                    onclick={() => isSaved = !isSaved}
                    class="inline-flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl border border-slate-200 hover:border-rose-200 hover:bg-rose-50/20 text-slate-600 hover:text-rose-600 font-semibold text-xs cursor-pointer transition-all active:scale-97"
                >
                    <Heart size={14} fill={isSaved ? "currentColor" : "none"} class={isSaved ? "text-rose-500" : ""} />
                    <span>{isSaved ? (isAr ? "محفوظ" : "Saved") : (isAr ? "حفظ" : "Save")}</span>
                </button>

                <!-- Share -->
                <button 
                    onclick={handleShare}
                    class="inline-flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl border border-slate-200 hover:border-blue-200 hover:bg-blue-50/20 text-slate-600 hover:text-blue-600 font-semibold text-xs cursor-pointer transition-all active:scale-97"
                >
                    <Share2 size={14} />
                    <span>{shareFeedback ? (isAr ? "تم النسخ!" : "Copied!") : (isAr ? "مشاركة" : "Share")}</span>
                </button>
            </div>

        </div>

        <!-- Safe-inquiry Trust Badge -->
        <div class="bg-slate-50 px-6 py-4 flex items-start gap-3 border-t border-slate-100">
            <ShieldCheck class="text-emerald-500 shrink-0 mt-0.5" size={16} />
            <p class="text-[11px] text-slate-400 font-medium leading-relaxed">
                {isAr
                    ? "استفسار آمن ومباشر. لن نشارك بياناتك إلا مع هذا البائع لمناقشة طلبك."
                    : "Safe Inquiry. Your details are protectively shared only with this vendor to coordinate bookings."}
            </p>
        </div>

    </div>

    <!-- 2. GCC similar options catalog listings (Free / Gold tiers only) -->
    {#if !hideCompetitors}
        <div class="space-y-4 pt-2">
            <h4 class="text-base font-extrabold text-slate-900 tracking-tight">
                {isAr ? "خيارات مشابهة قد تعجبك" : "Similar Venues You May Like"}
            </h4>

            {#if loading}
                <div class="space-y-4" aria-hidden="true">
                    <div class="h-[240px] bg-slate-100 rounded-2xl animate-pulse"></div>
                    <div class="h-[240px] bg-slate-100 rounded-2xl animate-pulse"></div>
                </div>
            {:else if relatedListings.length > 0}
                <div class="flex flex-col gap-4">
                    {#each relatedListings as related (related.id)}
                        <ListingCard listing={related} layout="grid" />
                    {/each}
                </div>
            {:else}
                <p class="text-xs sm:text-sm text-slate-400 font-medium">
                    {isAr ? "لا توجد خيارات مشابهة متوفرة حالياً." : "No similar options available."}
                </p>
            {/if}
        </div>
    {/if}

</aside>
