<script lang="ts">
    import { Play, Grid, ChevronLeft, ChevronRight } from "lucide-svelte";
    import { getLocale } from "$lib/paraglide/runtime.js";
    import { resolveMediaUrl } from "$lib/shared/utils/media.js";
    import GalleryLightboxV2 from "./GalleryLightboxV2.svelte";
    import OptimizedImage from "$lib/components/shared/OptimizedImage.svelte";

    let { images = [], title = "" } = $props<{
        images: any[];
        title: string;
    }>();

    const isAr = $derived(getLocale() === 'ar');
    let activeIndex = $state(0);
    let isLightboxOpen = $state(false);

    // Carousel internal state (for tablet/mobile carousel layout)
    let carouselIndex = $state(0);
    let touchStartX = 0;

    function handleTouchStart(e: TouchEvent) {
        touchStartX = e.changedTouches[0].screenX;
    }

    function handleTouchEnd(e: TouchEvent) {
        const touchEndX = e.changedTouches[0].screenX;
        const diff = touchStartX - touchEndX;
        if (Math.abs(diff) > 50) {
            if (diff > 0) {
                nextCarousel();
            } else {
                prevCarousel();
            }
        }
    }

    function nextCarousel() {
        if (images.length === 0) return;
        carouselIndex = (carouselIndex + 1) % images.length;
    }

    function prevCarousel() {
        if (images.length === 0) return;
        carouselIndex = (carouselIndex - 1 + images.length) % images.length;
    }

    function openLightbox(index: number) {
        activeIndex = index;
        isLightboxOpen = true;
    }
</script>

{#if images.length === 0}
    <!-- Empty State Component -->
    <div class="w-full max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-2">
        <div class="w-full h-[300px] md:h-[450px] rounded-2xl bg-slate-100 border border-slate-200 flex flex-col items-center justify-center text-slate-400 gap-3">
            <Grid size={48} strokeWidth={1} />
            <span class="text-sm font-medium">{isAr ? "لا تتوفر صور في المعرض حالياً" : "No Gallery Media Available"}</span>
        </div>
    </div>
{:else}
    <section class="w-full max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-2" aria-label={isAr ? "معرض الصور" : "Media Gallery"}>
        
        <!-- 1. DESKTOP EXPERIENCE: 3-Photo Grid (>= 1024px) -->
        <div class="hidden lg:grid grid-cols-3 gap-3 h-[450px] relative">
            <button 
                onclick={() => openLightbox(0)}
                class="col-span-2 relative overflow-hidden rounded-2xl cursor-pointer group bg-slate-900 border border-slate-100 focus:outline-hidden focus:ring-2 focus:ring-amber-500 focus:ring-offset-2"
                aria-label={isAr ? "فتح صورة الغلاف في المعرض المكبر" : "Open cover image in full lightbox"}
            >
                <OptimizedImage 
                    src={images[0].url} 
                    alt={images[0].alt || `${title} - image 1`}
                    fetchpriority="high"
                    loading="eager"
                    className="w-full h-full object-cover group-hover:scale-[1.01] transition-transform duration-500 brightness-[0.98] group-hover:brightness-100"
                    sizes="(max-width: 1024px) 100vw, 66vw"
                    aspectRatio="16/9"
                />
                <div class="absolute inset-0 bg-gradient-to-t from-slate-950/20 to-transparent pointer-events-none"></div>
            </button>

            <!-- Right Column Stack (1 or 2 smaller previews) -->
            <div class="col-span-1 grid gap-3 {images.length === 2 ? 'grid-rows-1' : 'grid-rows-2'}">
                <!-- Slot 2: Secondary Photo -->
                {#if images.length >= 2}
                    <button 
                        onclick={() => openLightbox(1)}
                        class="relative overflow-hidden rounded-2xl cursor-pointer group bg-slate-900 border border-slate-100 focus:outline-hidden focus:ring-2 focus:ring-amber-500 focus:ring-offset-2"
                        aria-label={isAr ? "فتح الصورة الثانية في المعرض المكبر" : "Open second image in full lightbox"}
                    >
                        <OptimizedImage 
                            src={images[1].url} 
                            alt={images[1].alt || `${title} - image 2`}
                            loading="lazy"
                            className="w-full h-full object-cover group-hover:scale-[1.02] transition-transform duration-500 brightness-[0.98] group-hover:brightness-100"
                            sizes="(max-width: 1024px) 100vw, 33vw"
                            aspectRatio="16/9"
                        />
                        <div class="absolute inset-0 bg-gradient-to-t from-slate-950/20 to-transparent pointer-events-none"></div>
                    </button>
                {/if}

                <!-- Slot 3: Third Photo or Video Preview -->
                {#if images.length >= 3}
                    {@const isVideo = images[2].mediaType === "video"}
                    <button 
                        onclick={() => openLightbox(2)}
                        class="relative overflow-hidden rounded-2xl cursor-pointer group bg-slate-900 border border-slate-100 focus:outline-hidden focus:ring-2 focus:ring-amber-500 focus:ring-offset-2"
                        aria-label={isVideo 
                            ? (isAr ? "شغيل الفيديو الترويجي" : "Play promo video") 
                            : (isAr ? "فتح الصورة الثالثة في المعرض المكبر" : "Open third image in full lightbox")
                        }
                    >
                        <OptimizedImage 
                            src={images[2].thumbnailUrl || images[2].url} 
                            alt={images[2].alt || `${title} - image 3`}
                            loading="lazy"
                            className="w-full h-full object-cover group-hover:scale-[1.02] transition-transform duration-500 brightness-[0.98] group-hover:brightness-100"
                            sizes="(max-width: 1024px) 100vw, 33vw"
                            aspectRatio="16/9"
                        />
                        
                        {#if isVideo}
                            <!-- Glassmorphic Play Icon Overlay -->
                            <div class="absolute inset-0 bg-slate-950/40 group-hover:bg-slate-950/30 flex items-center justify-center transition-all">
                                <div class="w-14 h-14 rounded-full bg-slate-900/70 border border-white/20 backdrop-blur-md flex items-center justify-center shadow-xl group-hover:scale-110 active:scale-95 transition-transform duration-300">
                                    <Play size={20} class="fill-amber-400 text-amber-400 ml-0.5" />
                                </div>
                            </div>
                            <!-- Video badge label -->
                            <div class="absolute bottom-4 end-4 px-2.5 py-1 text-[10px] font-bold tracking-wide uppercase bg-slate-950/80 border border-white/10 text-white rounded-md flex items-center gap-1 backdrop-blur-xs">
                                <Play size={8} class="fill-amber-400 text-amber-400" />
                                <span>{isAr ? "فيديو" : "Video"}</span>
                            </div>
                        {:else}
                            <div class="absolute inset-0 bg-gradient-to-t from-slate-950/20 to-transparent pointer-events-none"></div>
                        {/if}
                    </button>
                {/if}
            </div>

            <!-- "View All Photos" Button Overlay (bottom right corner) -->
            {#if images.length > 3}
                <button 
                    onclick={() => openLightbox(0)}
                    class="absolute bottom-5 right-5 z-10 px-4 py-2.5 rounded-xl bg-slate-950/80 text-white font-semibold text-xs border border-white/10 shadow-lg hover:bg-slate-950 backdrop-blur-md flex items-center gap-2 cursor-pointer transition-all active:scale-97 focus:outline-hidden focus:ring-2 focus:ring-amber-500"
                >
                    <Grid size={14} class="text-amber-400" />
                    <span>{isAr ? `عرض جميع الصور (${images.length})` : `View all photos (${images.length})`}</span>
                </button>
            {/if}

        </div>

        <!-- 2. TABLET & MOBILE EXPERIENCE: Dynamic Touch Slider (< 1024px) -->
        <div class="block lg:hidden relative w-full h-[260px] sm:h-[380px] bg-slate-950 rounded-2xl overflow-hidden group select-none shadow-sm">
            <!-- Slider Track -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div 
                class="relative w-full h-full overflow-hidden cursor-pointer"
                onclick={() => openLightbox(carouselIndex)}
                ontouchstart={handleTouchStart}
                ontouchend={handleTouchEnd}
            >
                {#each images as img, i (img.url + i)}
                    <!-- Only mount current slide, next, and previous to maintain sliding window constraints -->
                    {#if Math.abs(i - carouselIndex) <= 1 || (carouselIndex === 0 && i === images.length - 1) || (carouselIndex === images.length - 1 && i === 0)}
                        <div 
                            class="absolute inset-0 w-full h-full transition-all duration-500 ease-in-out flex items-center justify-center {i === carouselIndex ? 'opacity-100 scale-100 z-10' : 'opacity-0 scale-105 pointer-events-none'}"
                        >
                            {#if img.mediaType === "video"}
                                <div class="relative w-full h-full flex items-center justify-center bg-black">
                                    <OptimizedImage 
                                        src={img.thumbnailUrl || img.url} 
                                        alt={title}
                                        className="w-full h-full object-cover brightness-75"
                                        loading={i === 0 ? "eager" : "lazy"}
                                        fetchpriority={i === 0 ? "high" : "auto"}
                                        sizes="100vw"
                                        aspectRatio="16/9"
                                    />
                                    <div class="absolute inset-0 flex items-center justify-center">
                                        <div class="w-14 h-14 rounded-full bg-slate-900/60 border border-white/20 backdrop-blur-md flex items-center justify-center">
                                            <Play size={20} class="fill-amber-400 text-amber-400 ml-0.5" />
                                        </div>
                                    </div>
                                </div>
                            {:else}
                                <OptimizedImage 
                                    src={img.url} 
                                    alt={img.alt || `${title} - image ${i + 1}`}
                                    loading={i === 0 ? "eager" : "lazy"}
                                    fetchpriority={i === 0 ? "high" : "auto"}
                                    className="w-full h-full object-cover"
                                    sizes="100vw"
                                    aspectRatio="16/9"
                                />
                            {/if}
                        </div>
                    {/if}
                {/each}
            </div>

            <!-- Header status badge overlay -->
            <div class="absolute top-4 left-4 z-20 bg-slate-950/70 border border-white/10 px-3 py-1.5 rounded-full text-white text-xs font-semibold backdrop-blur-xs select-none">
                <span>{carouselIndex + 1} / {images.length}</span>
                {#if images[carouselIndex]?.mediaType === 'video'}
                    <span class="inline-flex items-center gap-0.5 text-amber-400 ml-1.5 font-bold">
                        <Play size={8} class="fill-amber-400" /> {isAr ? "فيديو" : "Video"}
                    </span>
                {/if}
            </div>

            <!-- Responsive tablet arrow navigation (hidden on touch-only mobile screen sizes) -->
            {#if images.length > 1}
                <button 
                    onclick={(e) => { e.stopPropagation(); prevCarousel(); }}
                    class="absolute left-4 top-1/2 -translate-y-1/2 z-20 w-10 h-10 rounded-full bg-slate-950/60 flex items-center justify-center text-white border border-white/10 shadow-lg sm:opacity-90 opacity-0 transition-opacity cursor-pointer"
                    aria-label={isAr ? "الرمز التالي" : "Previous slide"}
                >
                    <ChevronLeft size={20} />
                </button>

                <button 
                    onclick={(e) => { e.stopPropagation(); nextCarousel(); }}
                    class="absolute right-4 top-1/2 -translate-y-1/2 z-20 w-10 h-10 rounded-full bg-slate-950/60 flex items-center justify-center text-white border border-white/10 shadow-lg sm:opacity-90 opacity-0 transition-opacity cursor-pointer"
                    aria-label={isAr ? "الرمز السابق" : "Next slide"}
                >
                    <ChevronRight size={20} />
                </button>
            {/if}
        </div>

    </section>
{/if}

<!-- 3. FULLSCREEN MODAL LIGHTBOX OVERLAY -->
<GalleryLightboxV2 
    {images} 
    {title} 
    activeIndex={activeIndex}
    isOpen={isLightboxOpen} 
    onClose={() => isLightboxOpen = false} 
    onSelectIndex={(index) => activeIndex = index} 
/>
