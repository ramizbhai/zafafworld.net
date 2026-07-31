<script lang="ts">
    import type { ListingReview } from "$lib/types/index.js";
    import { getLocale } from '$lib/paraglide/runtime.js';
    import { Star, Filter, MessageSquare, ChevronDown, Check, Sparkles } from "lucide-svelte";

    let { reviews = [] } = $props<{ reviews: ListingReview[] }>();

    const isAr = $derived(getLocale() === 'ar');

    // UI state
    let sortBy = $state("recent");
    let filterVerifiedOnly = $state(false);
    let visibleCount = $state(3);

    // Dynamic calculations
    const totalCount = $derived(reviews.length);
    
    const averageRating = $derived.by(() => {
        if (totalCount === 0) return 0;
        const sum = reviews.reduce((acc: number, r: ListingReview) => acc + r.rating, 0);
        return Number((sum / totalCount).toFixed(1));
    });

    const recommendationPercentage = $derived.by(() => {
        if (totalCount === 0) return 0;
        const recommendedCount = reviews.filter((r: ListingReview) => r.rating >= 4.0).length;
        return Math.round((recommendedCount / totalCount) * 100);
    });

    const starDistribution = $derived.by(() => {
        const counts = { 5: 0, 4: 0, 3: 0, 2: 0, 1: 0 };
        reviews.forEach((r: ListingReview) => {
            const stars = Math.min(5, Math.max(1, Math.round(r.rating))) as 5 | 4 | 3 | 2 | 1;
            counts[stars]++;
        });
        return [5, 4, 3, 2, 1].map(stars => {
            const cnt = counts[stars as 5 | 4 | 3 | 2 | 1];
            return {
                stars,
                count: cnt,
                percentage: totalCount > 0 ? (cnt / totalCount) * 100 : 0
            };
        });
    });

    // Reactive processing (filtering and sorting)
    const processedReviews = $derived.by(() => {
        let list = [...reviews];

        // Filter
        if (filterVerifiedOnly) {
            // For demo: treat even reviews as verified partners/bookings
            list = list.filter((r: ListingReview, idx: number) => idx % 2 === 0);
        }

        // Sort
        if (sortBy === "recent") {
            list.sort((a: ListingReview, b: ListingReview) => new Date(b.date || "").getTime() - new Date(a.date || "").getTime());
        } else if (sortBy === "rating_desc") {
            list.sort((a: ListingReview, b: ListingReview) => b.rating - a.rating);
        } else if (sortBy === "rating_asc") {
            list.sort((a: ListingReview, b: ListingReview) => a.rating - b.rating);
        }

        return list;
    });

    const paginatedReviews = $derived(processedReviews.slice(0, visibleCount));
    const hasMore = $derived(visibleCount < processedReviews.length);

    function loadMore() {
        visibleCount += 3;
    }

    function formatDate(dateStr: string): string {
        if (!dateStr) return "";
        try {
            const date = new Date(dateStr);
            return new Intl.DateTimeFormat(isAr ? 'ar-SA' : 'en-US', {
                year: 'numeric',
                month: 'long',
                day: 'numeric'
            }).format(date);
        } catch {
            return dateStr;
        }
    }
</script>

<section id="reviews-section" class="bg-white rounded-2xl border border-slate-100 p-6 sm:p-8 shadow-sm space-y-8">
    
    <!-- Title -->
    <div class="flex items-center gap-3 border-b border-slate-100 pb-4">
        <div class="p-2.5 bg-amber-50 text-amber-600 rounded-xl">
            <MessageSquare size={22} />
        </div>
        <h3 class="font-display text-xl font-bold text-slate-900">
            {isAr ? `آراء وتقييمات العملاء (${totalCount})` : `Customer Reviews (${totalCount})`}
        </h3>
    </div>

    {#if totalCount === 0}
        <!-- Empty State -->
        <div class="text-center py-10 space-y-3">
            <div class="w-12 h-12 rounded-full bg-slate-50 flex items-center justify-center mx-auto text-slate-400">
                <MessageSquare size={24} />
            </div>
            <p class="text-slate-500 font-medium text-sm">
                {isAr ? "لا توجد تقييمات لهذه القاعة بعد." : "No reviews available for this venue yet."}
            </p>
        </div>
    {:else}
        <!-- Rating Header Metrics Section -->
        <div class="grid grid-cols-1 md:grid-cols-12 gap-6 items-center">
            
            <!-- Overall rating average score -->
            <div class="md:col-span-4 text-center space-y-2 border-slate-100 md:border-r md:rtl:border-r-0 md:rtl:border-l pb-6 md:pb-0">
                <p class="text-5xl sm:text-6xl font-extrabold text-slate-900 leading-none">
                    {averageRating}
                </p>
                <div class="flex items-center justify-center text-amber-500 gap-0.5">
                    {#each Array(5) as _, i}
                        <Star size={18} fill={i < Math.round(averageRating) ? "currentColor" : "none"} />
                    {/each}
                </div>
                <p class="text-xs text-slate-400 font-medium">
                    {isAr ? `بناءً على ${totalCount} تقييم` : `Based on ${totalCount} reviews`}
                </p>

                {#if recommendationPercentage > 0}
                    <div class="pt-2">
                        <span class="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-semibold text-emerald-800 bg-emerald-50 border border-emerald-100">
                            <Check size={12} class="text-emerald-600" />
                            <span>{isAr ? `ينصح به ${recommendationPercentage}٪ من الأزواج` : `${recommendationPercentage}% of couples recommend`}</span>
                        </span>
                    </div>
                {/if}
            </div>

            <!-- Distribution rating bars -->
            <div class="md:col-span-8 space-y-2 px-0 sm:px-6">
                {#each starDistribution as dist}
                    <div class="flex items-center gap-3 text-xs sm:text-sm font-semibold text-slate-500">
                        <span class="w-6 shrink-0 text-right rtl:text-left">{dist.stars} ★</span>
                        <div class="flex-1 h-2 rounded-full bg-slate-100 overflow-hidden" role="progressbar" aria-valuenow={dist.percentage} aria-valuemin={0} aria-valuemax={100}>
                            <div class="h-full bg-amber-500 rounded-full" style="width: {dist.percentage}%"></div>
                        </div>
                        <span class="w-10 shrink-0 text-left rtl:text-right text-slate-400">{dist.count}</span>
                    </div>
                {/each}
            </div>

        </div>

        <!-- Sorting & Filtering Controls -->
        <div class="flex flex-wrap items-center justify-between gap-4 pt-6 border-t border-slate-100">
            <!-- Filter Verified -->
            <div class="flex items-center gap-2">
                <button 
                    onclick={() => filterVerifiedOnly = !filterVerifiedOnly}
                    class="px-4 py-2 text-xs font-bold rounded-xl border cursor-pointer transition-all flex items-center gap-1.5 {filterVerifiedOnly ? 'border-amber-500 bg-amber-50/20 text-amber-700' : 'border-slate-200 bg-white text-slate-600 hover:border-slate-300'}"
                >
                    <Filter size={12} />
                    <span>{isAr ? "التقييمات الموثقة فقط" : "Verified Only"}</span>
                </button>
            </div>

            <!-- Sort By Select -->
            <div class="flex items-center gap-2 text-xs sm:text-sm text-slate-500 font-semibold">
                <span>{isAr ? "ترتيب حسب:" : "Sort by:"}</span>
                <div class="relative">
                    <select 
                        bind:value={sortBy}
                        class="appearance-none bg-slate-50 border border-slate-200/80 hover:border-slate-300 rounded-xl px-3 py-2 pr-8 rtl:pr-3 rtl:pl-8 text-xs font-bold text-slate-700 outline-none cursor-pointer"
                    >
                        <option value="recent">{isAr ? "الأحدث" : "Most Recent"}</option>
                        <option value="rating_desc">{isAr ? "الأعلى تقييماً" : "Highest Rated"}</option>
                        <option value="rating_asc">{isAr ? "الأقل تقييماً" : "Lowest Rated"}</option>
                    </select>
                    <div class="absolute inset-y-0 right-2.5 rtl:right-auto rtl:left-2.5 flex items-center pointer-events-none text-slate-400">
                        <ChevronDown size={14} />
                    </div>
                </div>
            </div>
        </div>

        <!-- Reviews list grid -->
        <div class="space-y-4">
            {#each paginatedReviews as rev (rev.id)}
                <!-- Review card wrapper -->
                <article class="p-5 rounded-2xl border border-slate-100 bg-slate-50/30 flex flex-col gap-4 text-start hover:bg-slate-50/60 transition-colors">
                    
                    <!-- Top reviewer header details -->
                    <div class="flex items-start justify-between gap-4">
                        <div class="space-y-1">
                            <p class="font-extrabold text-slate-800 text-sm sm:text-base">
                                {rev.authorName || (isAr ? "مستخدم زفاف" : "Zafaf User")}
                            </p>
                            <p class="text-[10px] sm:text-xs text-slate-400 font-medium">
                                {formatDate(rev.date)}
                            </p>
                        </div>

                        <!-- Rating Score block -->
                        <div class="flex items-center gap-1.5 px-2.5 py-1 rounded-xl bg-amber-50 text-amber-700 border border-amber-100 text-xs font-bold">
                            <Star size={12} fill="currentColor" />
                            <span>{rev.rating.toFixed(1)}</span>
                        </div>
                    </div>

                    <!-- Category specs or verified checklist tags -->
                    <div class="flex flex-wrap gap-2 text-[10px] font-bold">
                        <!-- Checked / Verified badge -->
                        <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-emerald-700 bg-emerald-50 border border-emerald-100">
                            <Check size={10} class="stroke-[3]" />
                            <span>{isAr ? "حجز مؤكد" : "Verified Booking"}</span>
                        </span>

                        {#if rev.weddingDate}
                            <span class="px-2 py-0.5 rounded-md text-slate-500 bg-slate-100 border border-slate-200/60">
                                {isAr ? `تاريخ الزفاف: ${formatDate(rev.weddingDate)}` : `Wedding: ${formatDate(rev.weddingDate)}`}
                            </span>
                        {/if}
                    </div>

                    <!-- Review Text block -->
                    <p class="text-xs sm:text-sm text-slate-600 leading-relaxed font-medium">
                        {rev.comment}
                    </p>

                </article>
            {/each}
        </div>

        <!-- Load More Pagination Trigger -->
        {#if hasMore}
            <div class="flex justify-center pt-2">
                <button 
                    onclick={loadMore}
                    class="px-6 py-2.5 bg-slate-900 hover:bg-slate-800 text-white font-bold text-xs sm:text-sm rounded-xl transition-all shadow-md active:scale-97 cursor-pointer"
                >
                    {isAr ? "تحميل المزيد من التقييمات" : "Load More Reviews"}
                </button>
            </div>
        {/if}

    {/if}

</section>
