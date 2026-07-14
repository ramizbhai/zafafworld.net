<script lang="ts">
  import { resolveMediaUrl, getOptimizedImage } from '$lib/shared/utils/media.js';
  import { ChevronLeft, ChevronRight, X, Play, Maximize2, Volume2 } from 'lucide-svelte';
  import { getLocale } from '$lib/paraglide/runtime.js';

  let { images = [], title = '' } = $props<{
    images: { url: string; thumbnailUrl?: string; mediaType?: string; durationSeconds?: number }[];
    title?: string;
  }>();

  let activeIndex = $state(0);
  let isFullscreen = $state(false);
  let isRtl = $derived(getLocale() === 'ar');
  let isVideoPlaying = $state(false);

  /** For each image, derive a fallback URL (original path without size suffix). */
  function getOriginalUrl(url: string): string {
    // Strip _thumb, _card, _medium, _large suffix if present
    return url.replace(/_(thumb|card|medium|large)(\.webp)$/i, '$2');
  }

  $effect(() => {
    // Reset video playing state when active slide changes
    const _ = activeIndex;
    isVideoPlaying = false;
  });

  function next() {
    if (images.length === 0) return;
    activeIndex = (activeIndex + 1) % images.length;
  }

  function prev() {
    if (images.length === 0) return;
    activeIndex = (activeIndex - 1 + images.length) % images.length;
  }

  // Swipe handling
  let touchStartX = 0;
  function handleTouchStart(e: TouchEvent) {
    touchStartX = e.changedTouches[0].screenX;
  }
  function handleTouchEnd(e: TouchEvent) {
    const touchEndX = e.changedTouches[0].screenX;
    const diff = touchStartX - touchEndX;
    if (Math.abs(diff) > 50) {
      if (diff > 0) {
        next();
      } else {
        prev();
      }
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (isFullscreen) {
      if (e.key === 'Escape') isFullscreen = false;
      if (e.key === 'ArrowRight') next();
      if (e.key === 'ArrowLeft') prev();
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if images.length > 0}
  <section class="relative w-full h-[60vh] md:h-[80vh] bg-slate-950 overflow-hidden group select-none" aria-label="Media Gallery">
    
    <!-- Active Slide Display with Smooth Fade & Absolute Stacking -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div 
      class="relative w-full h-full overflow-hidden"
      ontouchstart={handleTouchStart}
      ontouchend={handleTouchEnd}
    >
      {#each images as img, i (img.url + i)}
        <div 
          class="absolute inset-0 w-full h-full transition-all duration-700 ease-in-out flex items-center justify-center {i === activeIndex ? 'opacity-100 scale-100 z-10 pointer-events-auto' : 'opacity-0 scale-105 z-0 pointer-events-none'}"
        >
          {#if img.mediaType === 'video'}
            <div class="relative w-full h-full flex items-center justify-center bg-black">
              {#if i === activeIndex && isVideoPlaying}
                <!-- Active Playing Video: Mounted dynamically to prevent background download/play audio leakage -->
                <!-- svelte-ignore a11y_media_has_caption -->
                <video 
                  src={resolveMediaUrl(img.url)}
                  poster={img.thumbnailUrl ? resolveMediaUrl(img.thumbnailUrl) : ''}
                  controls
                  autoplay
                  playsinline
                  preload="auto"
                  class="w-full h-full object-contain max-h-full transition-opacity duration-300"
                ></video>
              {:else}
                <!-- Video Thumbnail Placeholder with pulsing Play button overlay -->
                <!-- svelte-ignore a11y_click_events_have_key_events -->
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div 
                  class="relative w-full h-full cursor-pointer overflow-hidden flex items-center justify-center"
                  onclick={() => {
                    if (i === activeIndex) {
                      isVideoPlaying = true;
                    } else {
                      activeIndex = i;
                    }
                  }}
                >
                  <img 
                    src={img.thumbnailUrl ? resolveMediaUrl(img.thumbnailUrl) : resolveMediaUrl(getOptimizedImage(img.url, 'large'))} 
                    alt={`${title} video preview`} 
                    loading={i === 0 ? "eager" : "lazy"}
                    width="1280"
                    height="720"
                    onerror={(e) => {
                      const el = e.currentTarget as HTMLImageElement;
                      const fallback = resolveMediaUrl(getOriginalUrl(img.url));
                      if (el.src !== fallback) el.src = fallback;
                    }}
                    class="w-full h-full object-cover brightness-75 hover:brightness-90 transition-all duration-700 ease-out"
                  />
                  <!-- Elegant Glassmorphic Pulsing Play Button Overlay -->
                  <div class="absolute inset-0 flex items-center justify-center">
                    <div class="w-20 h-20 rounded-full bg-slate-900/60 border border-white/20 backdrop-blur-md flex items-center justify-center shadow-2xl transition-all duration-300 transform hover:scale-110 active:scale-95 group/btn">
                      <!-- Pulsing Ring -->
                      <div class="absolute inset-0 rounded-full border-2 border-amber-500/50 animate-ping opacity-75"></div>
                      <Play size={36} class="fill-amber-400 text-amber-400 ml-1.5 transition-transform duration-300 group-hover/btn:scale-105" />
                    </div>
                  </div>
                  <!-- Duration/Status Overlay if available -->
                  {#if img.durationSeconds}
                    <div class="absolute bottom-6 end-6 bg-slate-950/80 backdrop-blur-md border border-white/10 text-white text-[10px] font-bold px-3 py-1 rounded-full uppercase tracking-wider flex items-center gap-1">
                      <span>{Math.floor(img.durationSeconds / 60)}:{String(img.durationSeconds % 60).padStart(2, '0')}</span>
                    </div>
                  {/if}
                </div>
              {/if}
            </div>
          {:else}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div 
              class="relative w-full h-full cursor-pointer overflow-hidden" 
              onclick={() => isFullscreen = true}
            >
              <img 
                src={resolveMediaUrl(getOptimizedImage(img.url, 'large'))} 
                alt={`${title} - ${i + 1}`} 
                loading={i === 0 ? "eager" : "lazy"}
                width="1280"
                height="800"
                onerror={(e) => {
                  const el = e.currentTarget as HTMLImageElement;
                  const fallback = resolveMediaUrl(getOriginalUrl(img.url));
                  if (el.src !== fallback) el.src = fallback;
                }}
                class="w-full h-full object-cover transform hover:scale-105 transition-transform duration-700 ease-out brightness-95 hover:brightness-100"
              />
              <div class="absolute inset-0 bg-gradient-to-t from-slate-950/80 via-slate-950/20 to-transparent pointer-events-none"></div>
            </div>
          {/if}
        </div>
      {/each}
    </div>

    <!-- Top Left / Right Overlay Controls -->
    <div class="absolute top-6 left-6 right-6 z-20 flex items-center justify-between pointer-events-none">
      <div class="bg-slate-900/60 backdrop-blur-md px-3 py-1.5 rounded-full text-white/90 text-xs font-medium border border-white/10 shadow-lg pointer-events-auto flex items-center gap-2">
        <span>{activeIndex + 1} / {images.length}</span>
        {#if images[activeIndex]?.mediaType === 'video'}
          <span class="inline-flex items-center gap-1 text-amber-400 bg-amber-400/10 px-2 py-0.5 rounded-full text-[10px]">
            <Play size={10} class="fill-amber-400" /> Video
          </span>
        {/if}
      </div>

      {#if images[activeIndex]?.mediaType !== 'video'}
        <button 
          onclick={() => isFullscreen = true}
          class="bg-slate-900/60 hover:bg-slate-900/90 backdrop-blur-md p-2.5 rounded-full text-white/90 hover:text-white border border-white/10 shadow-lg transition-all pointer-events-auto"
          aria-label="Fullscreen View"
        >
          <Maximize2 size={18} />
        </button>
      {/if}
    </div>

    <!-- Navigation Arrows: Logically pointing outwards on left & right sides -->
    {#if images.length > 1}
      <button 
        onclick={prev}
        class="absolute left-4 top-1/2 -translate-y-1/2 z-20 w-12 h-12 rounded-full bg-slate-900/60 hover:bg-slate-900/90 backdrop-blur-md flex items-center justify-center text-white border border-white/15 shadow-xl sm:opacity-0 sm:group-hover:opacity-100 opacity-90 transition-all duration-300 transform hover:scale-110 active:scale-95 cursor-pointer"
        aria-label="Previous Media"
      >
        <ChevronLeft size={26} />
      </button>

      <button 
        onclick={next}
        class="absolute right-4 top-1/2 -translate-y-1/2 z-20 w-12 h-12 rounded-full bg-slate-900/60 hover:bg-slate-900/90 backdrop-blur-md flex items-center justify-center text-white border border-white/15 shadow-xl sm:opacity-0 sm:group-hover:opacity-100 opacity-90 transition-all duration-300 transform hover:scale-110 active:scale-95 cursor-pointer"
        aria-label="Next Media"
      >
        <ChevronRight size={26} />
      </button>

      <!-- Bottom Thumbnail Dots / Previews -->
      <div class="absolute bottom-6 left-1/2 -translate-x-1/2 z-20 flex items-center gap-2 px-4 py-2 rounded-full bg-slate-900/60 backdrop-blur-md border border-white/10 max-w-[90vw] overflow-x-auto no-scrollbar">
        {#each images as img, i}
          <button 
            class="relative h-2 rounded-full transition-all duration-300 overflow-hidden cursor-pointer {i === activeIndex ? 'w-8 bg-amber-500 shadow-lg shadow-amber-500/50' : 'w-2 bg-white/40 hover:bg-white/80'}"
            onclick={() => activeIndex = i}
            aria-label={`Go to item ${i + 1}`}
          >
          </button>
        {/each}
      </div>
    {/if}
  </section>

  <!-- Fullscreen Lightbox Modal -->
  {#if isFullscreen}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="fixed inset-0 z-[99999] bg-slate-950/98 backdrop-blur-2xl flex items-center justify-center select-none animate-in fade-in duration-300">
      <button 
        class="absolute top-6 right-6 text-white/70 hover:text-white p-3 z-[100000] cursor-pointer bg-white/10 hover:bg-white/20 rounded-full backdrop-blur-md border border-white/10 transition-all shadow-xl" 
        onclick={() => isFullscreen = false}
      >
        <X size={24} />
      </button>
      
      <div class="relative max-w-7xl max-h-[90vh] w-full h-full flex items-center justify-center p-4">
        {#if images[activeIndex].mediaType === 'video'}
          <!-- svelte-ignore a11y_media_has_caption -->
          <video 
            src={resolveMediaUrl(images[activeIndex].url)}
            poster={images[activeIndex].thumbnailUrl ? resolveMediaUrl(images[activeIndex].thumbnailUrl) : ''}
            controls
            autoplay
            playsinline
            class="max-w-full max-h-full rounded-2xl shadow-2xl object-contain"
          ></video>
        {:else}
          <img 
            src={resolveMediaUrl(images[activeIndex].url)} 
            alt={title} 
            class="max-w-full max-h-full rounded-2xl shadow-2xl object-contain transform animate-in zoom-in-95 duration-300" 
          />
        {/if}
      </div>
      
      {#if images.length > 1}
        <button 
          onclick={prev} 
          class="absolute left-6 text-white/70 hover:text-white p-4 bg-white/10 hover:bg-white/20 rounded-full border border-white/15 backdrop-blur-md transition-all shadow-xl cursor-pointer"
        >
          <ChevronLeft size={32} />
        </button>
        <button 
          onclick={next} 
          class="absolute right-6 text-white/70 hover:text-white p-4 bg-white/10 hover:bg-white/20 rounded-full border border-white/15 backdrop-blur-md transition-all shadow-xl cursor-pointer"
        >
          <ChevronRight size={32} />
        </button>
      {/if}
    </div>
  {/if}
{:else}
  <!-- Fallback Hero -->
  <div class="w-full h-[50vh] bg-gradient-to-br from-slate-900 via-slate-800 to-amber-950 flex items-center justify-center">
    <span class="text-2xl font-medium text-amber-200/60 tracking-wider">No Gallery Media Available</span>
  </div>
{/if}
