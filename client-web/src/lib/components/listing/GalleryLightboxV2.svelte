<script lang="ts">
    import { X, ChevronLeft, ChevronRight, Play } from "lucide-svelte";
    import { getLocale } from "$lib/paraglide/runtime.js";
    import { resolveMediaUrl } from "$lib/shared/utils/media.js";
    import { onMount, tick } from "svelte";

    let { 
        images = [], 
        activeIndex = 0, 
        isOpen = false, 
        title = "", 
        onClose, 
        onSelectIndex 
    } = $props<{
        images: any[];
        activeIndex: number;
        isOpen: boolean;
        title: string;
        onClose: () => void;
        onSelectIndex: (index: number) => void;
    }>();

    const isAr = $derived(getLocale() === 'ar');
    let videoEl = $state<HTMLVideoElement | null>(null);
    let triggerElement = $state<HTMLElement | null>(null);
    let modalEl = $state<HTMLElement | null>(null);

    // Dynamic HLS script loading helper
    function loadHlsPlayer(video: HTMLVideoElement, src: string) {
        if (!src) return;
        const resolved = resolveMediaUrl(src);

        if (video.canPlayType('application/vnd.apple.mpegurl')) {
            // Native Safari HLS support
            video.src = resolved;
        } else {
            // Load HLS.js dynamically from CDN
            const scriptId = "hls-js-script";
            let script = document.getElementById(scriptId) as HTMLScriptElement | null;
            
            const initHls = () => {
                const Hls = (window as any).Hls;
                if (Hls && Hls.isSupported()) {
                    const hls = new Hls();
                    hls.loadSource(resolved);
                    hls.attachMedia(video);
                } else {
                    video.src = resolved; // Fallback
                }
            };

            if (!script) {
                script = document.createElement("script");
                script.id = scriptId;
                script.src = "https://cdn.jsdelivr.net/npm/hls.js@1.5.0/dist/hls.min.js";
                script.onload = initHls;
                document.head.appendChild(script);
            } else if ((window as any).Hls) {
                initHls();
            } else {
                script.addEventListener("load", initHls);
            }
        }
    }

    // Capture the trigger element that opened the modal to restore focus later
    $effect(() => {
        if (isOpen && typeof document !== 'undefined') {
            triggerElement = document.activeElement as HTMLElement;
            // Lock body scroll
            document.body.style.overflow = 'hidden';
            
            // Focus trap: set focus to the modal container
            tick().then(() => {
                modalEl?.focus();
            });
        }

        return () => {
            if (typeof document !== 'undefined') {
                document.body.style.overflow = '';
            }
            if (triggerElement) {
                triggerElement.focus();
            }
        };
    });

    // Reactive video loader when active item changes to video
    $effect(() => {
        const activeMedia = images[activeIndex];
        if (activeMedia && activeMedia.mediaType === 'video' && videoEl) {
            loadHlsPlayer(videoEl, activeMedia.fileUrl || activeMedia.url);
        }
    });

    function handleKeyDown(e: KeyboardEvent) {
        if (!isOpen) return;

        if (e.key === 'Escape') {
            onClose();
        } else if (e.key === 'ArrowRight') {
            if (isAr) {
                prevSlide();
            } else {
                nextSlide();
            }
        } else if (e.key === 'ArrowLeft') {
            if (isAr) {
                nextSlide();
            } else {
                prevSlide();
            }
        } else if (e.key === 'Tab') {
            // Simple Focus Lock
            if (modalEl) {
                const focusables = modalEl.querySelectorAll('button, video, img');
                if (focusables.length > 0) {
                    const first = focusables[0] as HTMLElement;
                    const last = focusables[focusables.length - 1] as HTMLElement;
                    if (e.shiftKey && document.activeElement === first) {
                        last.focus();
                        e.preventDefault();
                    } else if (!e.shiftKey && document.activeElement === last) {
                        first.focus();
                        e.preventDefault();
                    }
                }
            }
        }
    }

    function nextSlide() {
        if (images.length === 0) return;
        onSelectIndex((activeIndex + 1) % images.length);
    }

    function prevSlide() {
        if (images.length === 0) return;
        onSelectIndex((activeIndex - 1 + images.length) % images.length);
    }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if isOpen && images.length > 0}
    <!-- Lightbox Overlay Container -->
    <!-- svelte-ignore a11y_no_noninteractive_element_to_interactive_role -->
    <div 
        bind:this={modalEl}
        tabindex="-1"
        class="fixed inset-0 z-[99999] bg-slate-950/98 backdrop-blur-2xl flex flex-col justify-between select-none outline-none"
        role="dialog"
        aria-modal="true"
        aria-label={isAr ? "معرض الصور الكاملة" : "Full Media Gallery"}
    >
        <!-- Top Toolbar Header -->
        <header class="w-full flex items-center justify-between px-6 py-4 border-b border-white/5 z-10 bg-slate-950/40 backdrop-blur-md">
            <div class="text-white/90 text-sm font-semibold">
                <span>{activeIndex + 1} / {images.length}</span>
                <span class="mx-2 text-white/20">|</span>
                <span class="truncate max-w-[200px] sm:max-w-md inline-block align-middle">{title}</span>
            </div>

            <button 
                onclick={onClose}
                class="text-white/70 hover:text-white p-2.5 rounded-full bg-white/10 hover:bg-white/20 border border-white/10 transition-all cursor-pointer flex items-center justify-center"
                aria-label={isAr ? "إغلاق المعرض" : "Close Gallery"}
            >
                <X size={20} />
            </button>
        </header>

        <!-- Main Media Stage -->
        <div class="relative flex-1 w-full flex items-center justify-center p-4">
            
            <!-- Prev Slide Arrow -->
            {#if images.length > 1}
                <button 
                    onclick={prevSlide}
                    class="absolute left-6 z-20 text-white/70 hover:text-white p-4 bg-white/5 hover:bg-white/15 rounded-full border border-white/10 backdrop-blur-md transition-all cursor-pointer hidden md:flex items-center justify-center"
                    aria-label={isAr ? "الصورة التالية" : "Previous Slide"}
                >
                    <ChevronLeft size={28} />
                </button>
            {/if}

            <!-- Center Viewport Container -->
            <div class="max-w-5xl max-h-[72vh] w-full h-full flex items-center justify-center relative">
                {#if images[activeIndex].mediaType === 'video'}
                    <!-- svelte-ignore a11y_media_has_caption -->
                    <video 
                        bind:this={videoEl}
                        controls
                        autoplay
                        playsinline
                        class="max-w-full max-h-full rounded-2xl shadow-2xl object-contain"
                    ></video>
                {:else}
                    <img 
                        src={resolveMediaUrl(images[activeIndex].url)} 
                        alt={`${title} - image ${activeIndex + 1}`}
                        class="max-w-full max-h-full rounded-2xl shadow-2xl object-contain animate-in zoom-in-95 duration-200" 
                    />
                {/if}
            </div>

            <!-- Next Slide Arrow -->
            {#if images.length > 1}
                <button 
                    onclick={nextSlide}
                    class="absolute right-6 z-20 text-white/70 hover:text-white p-4 bg-white/5 hover:bg-white/15 rounded-full border border-white/10 backdrop-blur-md transition-all cursor-pointer hidden md:flex items-center justify-center"
                    aria-label={isAr ? "الصورة السابقة" : "Next Slide"}
                >
                    <ChevronRight size={28} />
                </button>
            {/if}

        </div>

        <!-- Bottom Thumbnails Navigator Strip -->
        {#if images.length > 1}
            <footer class="w-full py-6 bg-slate-950/60 border-t border-white/5 backdrop-blur-md flex flex-col items-center gap-3">
                <div class="flex items-center gap-2 max-w-[90vw] overflow-x-auto no-scrollbar py-1 px-4">
                    {#each images as img, i}
                        <button 
                            onclick={() => onSelectIndex(i)}
                            class="relative shrink-0 w-16 h-12 rounded-lg overflow-hidden border-2 transition-all cursor-pointer {i === activeIndex ? 'border-amber-400 scale-105 shadow-md shadow-amber-500/20' : 'border-transparent opacity-50 hover:opacity-80'}"
                            aria-label={`Go to item ${i + 1}`}
                        >
                            {#if img.mediaType === 'video'}
                                <div class="absolute inset-0 bg-slate-950/40 flex items-center justify-center z-10 text-white">
                                    <Play size={14} fill="currentColor" />
                                </div>
                            {/if}
                            <img 
                                src={resolveMediaUrl(img.thumbnailUrl || img.url)} 
                                alt=""
                                class="w-full h-full object-cover" 
                                loading="lazy"
                            />
                        </button>
                    {/each}
                </div>
            </footer>
        {/if}

    </div>
{/if}
