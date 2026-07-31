<script lang="ts">
    import type { ListingDetail } from "$lib/types/index.js";
    import { MessageCircle, PhoneCall, Calendar } from "lucide-svelte";
    import { getLocale } from "$lib/paraglide/runtime.js";

    let { listing } = $props<{ listing: ListingDetail }>();

    const isAr = $derived(getLocale() === "ar");

    // Tier definitions
    const tierId = $derived(listing.subscriptionBadge?.tierId || "free");
    const isDiamond = $derived(tierId === "diamond");
    const isVip = $derived(tierId === "vip");
    const showDirectContact = $derived(isDiamond || isVip);

    // Scroll to the main inquiry block
    function scrollToInquiry() {
        const formElement = document.getElementById("inquiry-form-anchor") || document.querySelector("aside");
        if (formElement) {
            formElement.scrollIntoView({ behavior: "smooth", block: "center" });
        }
    }

    // Formatting price
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
</script>

<!-- Mobile/Tablet Sticky Bottom Conversion Bar -->
<div 
    class="lg:hidden fixed bottom-0 left-0 right-0 z-40 bg-white/80 border-t border-slate-200/60 backdrop-blur-lg shadow-2xl pb-[env(safe-area-inset-bottom)] animate-in slide-in-from-bottom duration-300 select-none"
    role="region"
    aria-label={isAr ? "شريط إجراءات الحجز" : "Mobile Booking Bar"}
>
    <div class="px-5 py-3.5 flex items-center justify-between gap-4">
        
        <!-- Left Side: Pricing details -->
        <div class="space-y-0.5 min-w-0">
            <span class="text-[10px] text-slate-400 font-bold uppercase tracking-wider block">
                {isAr ? "تبدأ من" : "Starting from"}
            </span>
            <p class="text-base sm:text-lg font-extrabold text-slate-900 truncate">
                {priceText}
            </p>
        </div>

        <!-- Right Side: Primary and secondary actions -->
        <div class="flex items-center gap-2 shrink-0">
            <!-- Direct phone calling icon (VIP/Diamond tiers only) -->
            {#if showDirectContact && listing.vendor?.phone}
                <a 
                    href="tel:{listing.vendor.phone}"
                    class="w-10 h-10 rounded-xl border border-slate-200 hover:border-amber-400 hover:bg-amber-50/20 text-slate-700 hover:text-slate-900 flex items-center justify-center transition-all active:scale-95"
                    aria-label={isAr ? "اتصال بالهاتف" : "Call Phone"}
                >
                    <PhoneCall size={16} class="text-amber-500" />
                </a>
            {/if}

            <!-- Quick WhatsApp icon (VIP/Diamond tiers only) -->
            {#if showDirectContact && listing.coordinator?.whatsapp}
                <a 
                    href="https://wa.me/{listing.coordinator.whatsapp}"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="w-10 h-10 rounded-xl border border-slate-200 hover:border-emerald-400 hover:bg-emerald-50/20 text-slate-700 hover:text-slate-900 flex items-center justify-center transition-all active:scale-95"
                    aria-label={isAr ? "محادثة واتساب" : "Chat on WhatsApp"}
                >
                    <MessageCircle size={16} class="text-emerald-500" />
                </a>
            {/if}

            <!-- Primary Action CTA Button: Sends User to inquiry block -->
            <button 
                onclick={scrollToInquiry}
                class="px-5 py-2.5 bg-slate-900 hover:bg-slate-800 text-white font-bold text-xs sm:text-sm rounded-xl transition-all shadow-md active:scale-97 cursor-pointer flex items-center gap-1.5 focus:outline-hidden focus:ring-2 focus:ring-amber-500"
            >
                <Calendar size={14} class="text-amber-400" />
                <span>{isAr ? "طلب تسعير" : "Request Quote"}</span>
            </button>
        </div>

    </div>
</div>
