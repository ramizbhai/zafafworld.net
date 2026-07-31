<script lang="ts">
    import type { ListingDetail } from "$lib/types/index.js";
    import { getLocale } from "$lib/paraglide/runtime.js";
    import { MapPin, Copy, ExternalLink, Car, Milestone, HelpCircle } from "lucide-svelte";
    import { onMount } from "svelte";

    let { listing } = $props<{ listing: ListingDetail }>();

    const isAr = $derived(getLocale() === 'ar');

    // UI state
    let copyFeedback = $state(false);
    let shouldLoadMap = $state(false);
    let containerEl = $state<HTMLElement | null>(null);

    // Coordinate extractors
    const lat = $derived(listing?.latitude);
    const lng = $derived(listing?.longitude);
    const hasCoordinates = $derived(!!(lat && lng));

    // Fallback embed URL using coordinate values if googleMapsUrl is not an embed URL
    const embedUrl = $derived.by(() => {
        if (hasCoordinates) {
            return `https://maps.google.com/maps?q=${lat},${lng}&t=&z=15&ie=UTF8&iwloc=&output=embed`;
        }
        return "";
    });

    const fullAddress = $derived.by(() => {
        const address = isAr ? (listing.addressAr || listing.addressEn) : (listing.addressEn || listing.addressAr);
        const city = isAr ? (listing.cityAr || listing.cityEn) : (listing.cityEn || listing.cityAr);
        const district = isAr ? (listing.districtAr || listing.districtEn) : (listing.districtEn || listing.districtAr);

        const parts = [address, district, city].filter(Boolean);
        return parts.join(", ") || (isAr ? "الموقع غير متوفر" : "Location details not available");
    });

    // Lazy load map observer
    onMount(() => {
        if (!containerEl) return;
        
        if (typeof IntersectionObserver !== 'undefined') {
            const observer = new IntersectionObserver((entries) => {
                if (entries[0].isIntersecting) {
                    shouldLoadMap = true;
                    observer.disconnect();
                }
            }, { rootMargin: "300px" });
            observer.observe(containerEl);
            return () => observer.disconnect();
        } else {
            shouldLoadMap = true;
        }
    });

    async function copyToClipboard() {
        try {
            await navigator.clipboard.writeText(fullAddress);
            copyFeedback = true;
            setTimeout(() => {
                copyFeedback = false;
            }, 2000);
        } catch (err) {
            console.error("Failed to copy address", err);
        }
    }
</script>

<section 
    bind:this={containerEl}
    id="location-section" 
    class="bg-white rounded-2xl border border-slate-100 p-6 sm:p-8 shadow-sm space-y-6"
>
    
    <!-- Title -->
    <div class="flex items-center gap-3 border-b border-slate-100 pb-4">
        <div class="p-2.5 bg-amber-50 text-amber-600 rounded-xl">
            <MapPin size={22} />
        </div>
        <h3 class="font-display text-xl font-bold text-slate-900">
            {isAr ? "الموقع الجغرافي والوصول" : "Location & Access"}
        </h3>
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-12 gap-6 items-start">
        
        <!-- Details Column -->
        <div class="lg:col-span-5 space-y-6">
            
            <!-- Address details card -->
            <div class="p-5 rounded-2xl bg-slate-50 border border-slate-100/80 space-y-4">
                <div class="space-y-1">
                    <span class="text-[10px] text-slate-400 font-bold uppercase tracking-wider block">
                        {isAr ? "العنوان بالتفصيل" : "Full Address"}
                    </span>
                    <p class="text-sm sm:text-base font-bold text-slate-800 leading-relaxed">
                        {fullAddress}
                    </p>
                </div>

                <div class="flex flex-wrap gap-2 pt-2">
                    <!-- Copy address button -->
                    <button 
                        onclick={copyToClipboard}
                        class="inline-flex items-center justify-center gap-1.5 px-3 py-2 rounded-xl border border-slate-200 hover:border-slate-300 bg-white text-slate-600 hover:text-slate-900 font-bold text-xs cursor-pointer transition-all active:scale-97"
                    >
                        <Copy size={12} />
                        <span>{copyFeedback ? (isAr ? "تم النسخ!" : "Copied!") : (isAr ? "نسخ العنوان" : "Copy Address")}</span>
                    </button>

                    <!-- Google Maps redirection action link -->
                    {#if listing.googleMapsUrl}
                        <a 
                            href={listing.googleMapsUrl}
                            target="_blank"
                            rel="noopener noreferrer"
                            class="inline-flex items-center justify-center gap-1.5 px-3 py-2 rounded-xl border border-slate-200 hover:border-amber-400 hover:bg-amber-50/20 text-slate-600 hover:text-amber-700 font-bold text-xs transition-all active:scale-97"
                        >
                            <ExternalLink size={12} />
                            <span>{isAr ? "فتح في الخرائط" : "Open in Maps"}</span>
                        </a>
                    {/if}
                </div>
            </div>

            <!-- Parking specs panel details -->
            <div class="p-5 rounded-2xl bg-slate-50 border border-slate-100/80 flex items-start gap-4">
                <div class="w-8 h-8 rounded-lg bg-amber-100 text-amber-700 flex items-center justify-center shrink-0">
                    <Car size={16} />
                </div>
                <div class="space-y-1">
                    <span class="text-[10px] text-slate-400 font-bold uppercase tracking-wider block">
                        {isAr ? "خدمات مواقف السيارات" : "Parking Details"}
                    </span>
                    <p class="text-sm font-bold text-slate-800">
                        {isAr ? "مواقف مجانية متوفرة للضيوف" : "Free parking slots available for guests"}
                    </p>
                    <p class="text-xs text-slate-500 font-medium">
                        {isAr ? "سعة المواقف كافية لأكثر من ١٥٠ سيارة" : "Accommodates up to 150+ vehicles comfortably"}
                    </p>
                </div>
            </div>

            <!-- Estimated access routing landmarks details -->
            <div class="p-5 rounded-2xl bg-slate-50 border border-slate-100/80 flex items-start gap-4">
                <div class="w-8 h-8 rounded-lg bg-amber-100 text-amber-700 flex items-center justify-center shrink-0">
                    <Milestone size={16} />
                </div>
                <div class="space-y-1">
                    <span class="text-[10px] text-slate-400 font-bold uppercase tracking-wider block">
                        {isAr ? "معالم قريبة" : "Nearby Landmarks"}
                    </span>
                    <p class="text-sm font-bold text-slate-800">
                        {isAr ? "سهل الوصول عبر الطرق الرئيسية" : "Easily accessible from major highways"}
                    </p>
                    <p class="text-xs text-slate-500 font-medium">
                        {isAr ? "تبعد ٢٠ دقيقة من مطار الملك خالد الدولي" : "Located 20 minutes from Airport district"}
                    </p>
                </div>
            </div>

        </div>

        <!-- Map Iframe Wrapper Column -->
        <div class="lg:col-span-7 w-full h-[320px] lg:h-[400px] rounded-2xl overflow-hidden border border-slate-100 bg-slate-50 relative">
            {#if shouldLoadMap && embedUrl}
                <iframe 
                    src={embedUrl}
                    title={isAr ? "موقع القاعة الجغرافي" : "Venue geographic location"}
                    class="w-full h-full border-0" 
                    allowfullscreen 
                    loading="lazy"
                ></iframe>
            {:else}
                <!-- Static Fallback / Skeleton view while not in viewport or coordinate missing -->
                <div class="absolute inset-0 flex flex-col items-center justify-center text-slate-400 bg-slate-100 gap-3">
                    <HelpCircle size={40} strokeWidth={1} class="animate-pulse" />
                    <span class="text-xs font-semibold">{isAr ? "جاري تحميل الخريطة التفاعلية..." : "Loading interactive map..."}</span>
                </div>
            {/if}
        </div>

    </div>

</section>
