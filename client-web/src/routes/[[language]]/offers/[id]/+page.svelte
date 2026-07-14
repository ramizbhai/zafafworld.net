<script lang="ts">
    import { getLocale } from "$lib/paraglide/runtime.js";
    import * as m from "$lib/paraglide/messages.js";
    import { getLocalizedField, formatCurrency, formatDate } from "$lib/utils/localize.js";
    import { resolveMediaUrl, getOptimizedImage } from "$lib/shared/utils/media.js";
    import { Calendar, Tag, Hourglass, ArrowLeft, ArrowRight, ShieldCheck } from "lucide-svelte";
    import ListingCard from "$lib/components/shared/ListingCard.svelte";
    import { onMount } from "svelte";

    let { data } = $props<{ data: { promotion: any, listing: any } }>();

    // Client-side countdown updater
    let now = $state(new Date());

    onMount(() => {
        const interval = setInterval(() => {
            now = new Date();
        }, 1000);
        return () => clearInterval(interval);
    });

    const isExpired = $derived(
        !data.promotion.end_at || new Date(data.promotion.end_at).getTime() <= now.getTime()
    );

    const countdownParts = $derived.by(() => {
        if (isExpired) return null;
        const end = new Date(data.promotion.end_at);
        const diff = end.getTime() - now.getTime();
        
        const secs = Math.floor(diff / 1000);
        const mins = Math.floor(secs / 60);
        const hours = Math.floor(mins / 60);
        const days = Math.floor(hours / 24);

        return {
            days,
            hours: hours % 24,
            minutes: mins % 60,
            seconds: secs % 60
        };
    });

    const promoTitle = $derived(getLocale() === 'ar' ? data.promotion.title_ar : data.promotion.title_en);
    const promoDesc = $derived(getLocale() === 'ar' ? (data.promotion.description_ar || '') : (data.promotion.description_en || ''));
    const badgeText = $derived(getLocale() === 'ar' ? (data.promotion.badge_text_ar || data.promotion.badge_text_en || '') : (data.promotion.badge_text_en || data.promotion.badge_text_ar || ''));
</script>

<svelte:head>
    <title>{promoTitle} | ZafafWorld</title>
</svelte:head>

<div class="container mx-auto px-4 py-8 max-w-5xl" dir={getLocale() === 'ar' ? 'rtl' : 'ltr'}>
    <!-- Breadcrumbs -->
    <nav class="flex items-center gap-2 text-xs text-[var(--color-muted)] mb-6" aria-label="Breadcrumb">
        <a href="/" class="hover:text-[var(--color-primary)]">Home</a>
        <span>/</span>
        <a href="/offers" class="hover:text-[var(--color-primary)]">{getLocale() === 'ar' ? 'العروض الترويجية' : 'Promotions'}</a>
        <span>/</span>
        <span class="text-[var(--color-secondary)] font-bold truncate max-w-[240px]">{promoTitle}</span>
    </nav>

    <!-- 1. Hero Banner Section -->
    <div class="relative w-full h-[260px] md:h-[380px] rounded-3xl overflow-hidden shadow-lg border border-gray-100 mb-8 bg-gray-50">
        <!-- Floating Badges -->
        <div class="absolute top-4 start-4 z-10 flex flex-col gap-2">
            <span class="bg-gradient-to-r from-emerald-600 to-teal-500 text-white font-black text-sm md:text-base px-4 py-2 rounded-full shadow-lg">
                {#if data.promotion.promo_type === 'discount'}
                    {#if data.promotion.discount_type === 'percentage'}
                        {data.promotion.discount_percentage}% {getLocale() === 'ar' ? 'خصم' : 'OFF'}
                    {:else}
                        {Number(data.promotion.discount_fixed_amount).toLocaleString()} {getLocale() === 'ar' ? 'ر.س خصم' : 'SAR OFF'}
                    {/if}
                {:else}
                    🎁 {getLocale() === 'ar' ? 'عرض قيمة مضافة' : 'Added Value'}
                {/if}
            </span>
        </div>

        {#if badgeText}
            <div class="absolute top-4 end-4 z-10 bg-black/60 backdrop-blur-sm text-white text-[10px] font-bold px-3.5 py-1.5 rounded-full uppercase tracking-wider">
                {badgeText}
            </div>
        {/if}

        <!-- Bottom text overlay gradient -->
        <div class="absolute inset-0 bg-gradient-to-t from-black/80 via-black/35 to-transparent z-0 flex flex-col justify-end p-6 md:p-8">
            <div class="z-10">
                <span class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-emerald-500/25 text-emerald-300 text-[10px] font-bold uppercase tracking-wider mb-3 backdrop-blur-sm border border-emerald-500/20">
                    <ShieldCheck size={12} />
                    {getLocale() === 'ar' ? 'عرض معتمد وموثق' : 'Verified Zafaf Deal'}
                </span>
                <h1 class="font-display text-2xl md:text-4xl font-black text-white leading-tight drop-shadow-md">
                    {promoTitle}
                </h1>
            </div>
        </div>

        <!-- Banner Image -->
        <img 
            src={(!data.promotion.use_listing_cover_image && (data.promotion.custom_banner_image_url || data.promotion.banner_image_url)) ? resolveMediaUrl(getOptimizedImage(data.promotion.custom_banner_image_url || data.promotion.banner_image_url, 'large')) : (data.promotion.cover_image ? resolveMediaUrl(getOptimizedImage(data.promotion.cover_image, 'large')) : '/images/fallbacks/default-cover.svg')} 
            alt={promoTitle}
            class="absolute inset-0 w-full h-full object-cover"
        />
    </div>

    <!-- 2. Split Main Layout -->
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
        <!-- Main Content Column (2/3 Width) -->
        <div class="lg:col-span-2 space-y-8">
            <!-- Offer Description Card -->
            <div class="bg-white rounded-3xl border border-gray-100 p-6 md:p-8 shadow-sm">
                <h2 class="font-display text-lg font-black text-[var(--color-secondary)] mb-4 border-b border-gray-100 pb-3 flex items-center gap-2">
                    📄 {getLocale() === 'ar' ? 'تفاصيل العرض الترويجي' : 'Promotion Details'}
                </h2>
                
                {#if promoDesc}
                    <div class="text-sm text-[var(--color-text)] leading-relaxed prose max-w-none">
                        {@html promoDesc}
                    </div>
                {:else}
                    <p class="text-xs text-[var(--color-muted)] italic">{getLocale() === 'ar' ? 'لا يوجد وصف متوفر.' : 'No description provided.'}</p>
                {/if}
            </div>

            <!-- Promoted Listing Card Details -->
            <div class="bg-white rounded-3xl border border-gray-100 p-6 md:p-8 shadow-sm">
                <h2 class="font-display text-lg font-black text-[var(--color-secondary)] mb-6 flex items-center gap-2">
                    🎯 {getLocale() === 'ar' ? 'الخدمة / الصالون المشمول بالعرض' : 'Promoted Service & Listing'}
                </h2>

                {#if data.listing}
                    <div class="max-w-md">
                        <ListingCard 
                            listing={data.listing} 
                            promotion={data.promotion} 
                            size="lg" 
                        />
                    </div>
                {:else}
                    <p class="text-muted italic">{getLocale() === 'ar' ? 'تفاصيل الخدمة غير متوفرة حالياً.' : 'Target listing details could not be loaded.'}</p>
                {/if}
            </div>
        </div>

        <!-- Sticky Sidebar Column (1/3 Width) -->
        <div class="lg:col-span-1 space-y-6">
            <div class="bg-white rounded-3xl border border-gray-100 p-6 shadow-sm lg:sticky lg:top-6 space-y-6">
                <!-- Added Value Benefit Box -->
                {#if data.promotion.promo_type === 'benefit' && (data.promotion.benefit_description_ar || data.promotion.benefit_description_en)}
                    <div class="bg-violet-50/50 border border-violet-100/50 p-5 rounded-2xl text-violet-950">
                        <span class="inline-flex items-center gap-1.5 text-[10px] font-bold text-violet-700 uppercase tracking-wider mb-2">
                            ✨ {getLocale() === 'ar' ? 'الميزة المضافة الحصرية' : 'Exclusive Added Value'}
                        </span>
                        <p class="text-xs leading-relaxed font-semibold">
                            {getLocale() === 'ar' ? (data.promotion.benefit_description_ar || data.promotion.benefit_description_en) : (data.promotion.benefit_description_en || data.promotion.benefit_description_ar)}
                        </p>
                    </div>
                {/if}

                <!-- Campaign Window Schedule -->
                <div class="bg-[var(--color-surface-alt)] border border-gray-100/50 rounded-2xl p-5">
                    <h3 class="text-[10px] font-bold text-[var(--color-secondary)] uppercase tracking-wider mb-4 flex items-center gap-1.5">
                        📅 {getLocale() === 'ar' ? 'فترة صلاحية الحملة' : 'Campaign Schedule Window'}
                    </h3>
                    <div class="flex flex-col gap-3 text-xs text-[var(--color-text)]">
                        <div class="flex justify-between">
                            <span class="text-[var(--color-muted)]">{getLocale() === 'ar' ? 'تاريخ البدء:' : 'Start Date:'}</span>
                            <span class="font-bold text-[var(--color-secondary)]">{data.promotion.start_at ? new Date(data.promotion.start_at).toLocaleString() : ''}</span>
                        </div>
                        <div class="flex justify-between border-t border-gray-200/50 pt-3">
                            <span class="text-[var(--color-muted)]">{getLocale() === 'ar' ? 'تاريخ الانتهاء:' : 'End Date:'}</span>
                            <span class="font-bold text-[var(--color-secondary)]">{data.promotion.end_at ? new Date(data.promotion.end_at).toLocaleString() : ''}</span>
                        </div>
                    </div>
                </div>

                <!-- Expiration Countdown Block -->
                <div class="border-t border-gray-100 pt-6">
                    {#if isExpired}
                        <div class="bg-red-50 text-red-700 font-bold p-4 rounded-xl text-center border border-red-100 text-xs">
                            ⚠️ {getLocale() === 'ar' ? 'انتهت صلاحية هذا العرض' : 'This campaign has expired.'}
                        </div>
                    {:else if countdownParts}
                        <div>
                            <span class="text-[9px] text-[var(--color-muted)] font-black uppercase tracking-wider block mb-3 text-center lg:text-start">
                                ⏱️ {getLocale() === 'ar' ? 'ينتهي العرض الترويجي في:' : 'Promotion Countdown Expiry:'}
                            </span>
                            <div class="grid grid-cols-4 gap-2">
                                <div class="bg-amber-500/10 text-amber-800 rounded-xl py-2 text-center border border-amber-500/20">
                                    <span class="text-base font-black block leading-none">{countdownParts.days}</span>
                                    <span class="text-[8px] uppercase font-bold text-amber-700 mt-1 block">{getLocale() === 'ar' ? 'يوم' : 'Days'}</span>
                                </div>
                                <div class="bg-amber-500/10 text-amber-800 rounded-xl py-2 text-center border border-amber-500/20">
                                    <span class="text-base font-black block leading-none">{countdownParts.hours}</span>
                                    <span class="text-[8px] uppercase font-bold text-amber-700 mt-1 block">{getLocale() === 'ar' ? 'ساعة' : 'Hrs'}</span>
                                </div>
                                <div class="bg-amber-500/10 text-amber-800 rounded-xl py-2 text-center border border-amber-500/20">
                                    <span class="text-base font-black block leading-none">{countdownParts.minutes}</span>
                                    <span class="text-[8px] uppercase font-bold text-amber-700 mt-1 block">{getLocale() === 'ar' ? 'دقيقة' : 'Mins'}</span>
                                </div>
                                <div class="bg-amber-500/10 text-amber-800 rounded-xl py-2 text-center border border-amber-500/20">
                                    <span class="text-base font-black block leading-none text-red-600">{countdownParts.seconds}</span>
                                    <span class="text-[8px] uppercase font-bold text-amber-700 mt-1 block">{getLocale() === 'ar' ? 'ثانية' : 'Secs'}</span>
                                </div>
                            </div>
                        </div>
                    {/if}
                </div>

                <!-- Call to Action Link -->
                {#if data.listing}
                    <a 
                        href="/listings/{data.listing.slug}" 
                        class="w-full text-center bg-[var(--color-primary)] text-white hover:bg-[var(--color-primary-dark)] active:scale-[0.98] transition-all font-bold py-3.5 rounded-xl shadow-lg shadow-[var(--color-primary)]/15 block text-xs"
                    >
                        ⚡ {getLocale() === 'ar' ? 'احجز الخدمة الآن' : 'Book Promoted Service Now'}
                    </a>
                {/if}
            </div>
        </div>
    </div>
</div>
