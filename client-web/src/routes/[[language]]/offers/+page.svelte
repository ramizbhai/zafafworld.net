<script lang="ts">
    import { getLocale } from "$lib/paraglide/runtime.js";
    import * as m from "$lib/paraglide/messages.js";
    import { getLocalizedField, formatCurrency } from "$lib/utils/localize.js";
    import { resolveMediaUrl, getOptimizedImage } from "$lib/shared/utils/media.js";
    import { Calendar, Tag, Hourglass, ArrowRight } from "lucide-svelte";
    import { onMount } from "svelte";

    let { data } = $props<{ data: { promotions: any[] } }>();

    function stripHtml(html: string): string {
        if (!html) return "";
        return html.replace(/<[^>]*>/g, ' ').replace(/\s+/g, ' ').trim();
    }

    // Client-side countdown updater
    let now = $state(new Date());

    onMount(() => {
        const interval = setInterval(() => {
            now = new Date();
        }, 1000);
        return () => clearInterval(interval);
    });

    function getCountdownText(endAtStr: string) {
        if (!endAtStr) return '';
        const end = new Date(endAtStr);
        const diff = end.getTime() - now.getTime();
        
        if (diff <= 0) {
            return getLocale() === 'ar' ? 'منتهي الصلاحية' : 'Expired';
        }

        const secs = Math.floor(diff / 1000);
        const mins = Math.floor(secs / 60);
        const hours = Math.floor(mins / 60);
        const days = Math.floor(hours / 24);

        if (days > 0) {
            return getLocale() === 'ar' 
                ? `ينتهي خلال ${days} يوم و ${hours % 24} ساعة` 
                : `Ends in ${days}d ${hours % 24}h`;
        } else if (hours > 0) {
            return getLocale() === 'ar'
                ? `ينتهي خلال ${hours} ساعة و ${mins % 60} دقيقة`
                : `Ends in ${hours}h ${mins % 60}m`;
        } else {
            return getLocale() === 'ar'
                ? `ينتهي خلال ${mins} دقيقة`
                : `Ends in ${mins}m`;
        }
    }
</script>

<svelte:head>
    <title>{getLocale() === 'ar' ? 'العروض الترويجية الحصرية' : 'Exclusive Promotions'} | ZafafWorld</title>
</svelte:head>

<div class="container mx-auto px-4 py-8 max-w-7xl" dir={getLocale() === 'ar' ? 'rtl' : 'ltr'}>
    <!-- Gorgeous header section -->
    <div class="text-center mb-12 py-10 bg-gradient-to-br from-amber-500/10 via-rose-500/5 to-transparent rounded-3xl border border-amber-200/20">
        <span class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full bg-amber-500/10 text-amber-700 text-xs font-bold uppercase tracking-wider mb-4">
            <Tag size={12} />
            {getLocale() === 'ar' ? 'خصومات وعروض ممتازة' : 'Premium Zafaf Deals'}
        </span>
        <h1 class="font-display text-3xl md:text-5xl font-black text-[var(--color-secondary)] mb-4">
            {getLocale() === 'ar' ? 'العروض والخصومات الحصرية' : 'Special Offers & Promotions'}
        </h1>
        <p class="text-sm md:text-base text-[var(--color-muted)] max-w-2xl mx-auto px-4">
            {getLocale() === 'ar' 
                ? 'اكتشف أفضل عروض الصالات وخدمات الأفراح في المملكة بأسعار مخفضة ولفترة محدودة فقط.' 
                : 'Explore curated wedding ballrooms and planning packages with exclusive seasonal discounts.'}
        </p>
    </div>

    <!-- Active Promotions Grid -->
    {#if !data.promotions || data.promotions.length === 0}
        <div class="text-center py-20 bg-[var(--color-surface-alt)] rounded-2xl border border-[var(--color-border)]">
            <div class="text-5xl mb-4">🎁</div>
            <h3 class="text-lg font-bold text-[var(--color-secondary)] mb-2">
                {getLocale() === 'ar' ? 'لا توجد عروض ترويجية جارية حالياً' : 'No Promotions Active Today'}
            </h3>
            <p class="text-xs text-[var(--color-muted)]">
                {getLocale() === 'ar' ? 'يرجى مراجعة هذه الصفحة لاحقاً للاطلاع على الصفقات والعروض الجديدة.' : 'Check back later for exclusive listings coupons and wedding ballrooms deals.'}
            </p>
        </div>
    {:else}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
            {#each data.promotions as promo}
                <article class="group bg-white rounded-2xl overflow-hidden border border-gray-100 shadow-sm hover:shadow-xl hover:-translate-y-1 transition-all duration-300 flex flex-col justify-between">
                    <!-- Banner or Cover Image -->
                    <div class="relative aspect-[16/9] w-full overflow-hidden bg-gray-50 shrink-0">
                        <!-- Discount Badge -->
                        <div class="absolute top-3 start-3 z-10 bg-gradient-to-r from-emerald-600 to-teal-500 text-white font-extrabold text-sm px-3.5 py-1.5 rounded-full shadow-md">
                            {#if promo.promo_type === 'discount'}
                                {#if promo.discount_type === 'percentage'}
                                    {promo.discount_percentage}% {getLocale() === 'ar' ? 'خصم' : 'OFF'}
                                {:else}
                                    {Number(promo.discount_fixed_amount).toLocaleString()} {getLocale() === 'ar' ? 'ر.س خصم' : 'SAR OFF'}
                                {/if}
                            {:else}
                                🎁 {getLocale() === 'ar' ? 'عرض قيمة مضافة' : 'Added Value'}
                            {/if}
                        </div>

                        <!-- Ribbon text if present -->
                        {#if promo.badge_text_ar || promo.badge_text_en}
                            <div class="absolute top-3 end-3 z-10 bg-black/60 backdrop-blur-sm text-white text-[10px] font-bold px-2.5 py-1 rounded-full uppercase tracking-wider">
                                {getLocale() === 'ar' ? (promo.badge_text_ar || promo.badge_text_en) : (promo.badge_text_en || promo.badge_text_ar)}
                            </div>
                        {/if}

                        <img 
                            src={(!promo.use_listing_cover_image && (promo.custom_banner_image_url || promo.banner_image_url)) ? resolveMediaUrl(getOptimizedImage(promo.custom_banner_image_url || promo.banner_image_url, 'card')) : (promo.cover_image ? resolveMediaUrl(getOptimizedImage(promo.cover_image, 'card')) : '/images/fallbacks/default-cover.svg')} 
                            alt={getLocale() === 'ar' ? promo.title_ar : promo.title_en}
                            class="w-full h-full object-cover transition-transform duration-700 group-hover:scale-105"
                        />
                    </div>

                    <!-- Promo Body -->
                    <div class="p-6 flex-1 flex flex-col justify-between">
                        <div>
                            <!-- Vendor Brand -->
                            <span class="text-xs font-semibold text-[var(--color-primary)] uppercase tracking-wider block mb-2">
                                🏢 {promo.vendor_name}
                            </span>
                            
                            <h2 class="font-display text-lg font-bold text-[var(--color-secondary)] mb-2 line-clamp-1 group-hover:text-[var(--color-primary)] transition-colors">
                                <a href="/offers/{promo.promotion_id}">
                                    {getLocale() === 'ar' ? promo.title_ar : promo.title_en}
                                </a>
                            </h2>

                            {#if promo.description_ar || promo.description_en}
                                <p class="text-xs text-[var(--color-muted)] mb-3 line-clamp-2 leading-relaxed">
                                    {stripHtml(getLocale() === 'ar' ? (promo.description_ar || promo.description_en) : (promo.description_en || promo.description_ar))}
                                </p>
                            {/if}

                            {#if promo.promo_type === 'benefit' && (promo.benefit_description_ar || promo.benefit_description_en)}
                                <div style="font-size: 11.5px; background: hsl(270, 80%, 97%); border: 1px solid hsl(270, 50%, 90%); padding: 10px; border-radius: 8px; margin-bottom: 12px; color: hsl(270, 70%, 25%);">
                                    🌟 <strong>{getLocale() === 'ar' ? 'الميزة المضافة:' : 'Added Value Benefit:'}</strong>
                                    <p style="margin: 4px 0 0; line-height: 1.4;">
                                        {getLocale() === 'ar' ? (promo.benefit_description_ar || promo.benefit_description_en) : (promo.benefit_description_en || promo.benefit_description_ar)}
                                    </p>
                                </div>
                            {/if}

                            <!-- Targeted listing snippet -->
                            <div class="bg-[var(--color-surface-alt)] border border-gray-100 rounded-xl p-3 mb-4">
                                <span class="text-[10px] text-[var(--color-muted)] font-bold uppercase tracking-wider block mb-1">
                                    {getLocale() === 'ar' ? 'العرض ساري على:' : 'Applies to Listing:'}
                                </span>
                                <a href="/listings/{promo.listing_slug}" class="text-xs font-bold text-[var(--color-secondary)] hover:underline flex items-center justify-between">
                                    <span>🎯 {getLocale() === 'ar' ? promo.listing_name_ar : promo.listing_name_en}</span>
                                    <span class="text-[10px] text-[var(--color-muted)] font-medium shrink-0">({promo.category})</span>
                                </a>
                            </div>
                        </div>

                        <!-- Time limit countdown -->
                        <div class="border-t border-gray-100 pt-4 mt-auto flex items-center justify-between gap-2">
                            <div class="flex items-center gap-1.5 text-xs font-bold text-amber-600 bg-amber-500/10 px-2.5 py-1 rounded-lg">
                                <Hourglass size={12} class="animate-pulse" />
                                <span>{getCountdownText(promo.end_at)}</span>
                            </div>
                            
                            <a href="/offers/{promo.promotion_id}" class="inline-flex items-center gap-1 text-xs font-bold text-[var(--color-secondary)] hover:text-[var(--color-primary)] transition-colors shrink-0">
                                <span>{getLocale() === 'ar' ? 'عرض التفاصيل' : 'View Deal'}</span>
                                <ArrowRight size={14} class="rtl:rotate-180" />
                            </a>
                        </div>
                    </div>
                </article>
            {/each}
        </div>
    {/if}
</div>
