<script lang="ts">
  import { getLocale } from "$lib/paraglide/runtime.js";
  import Button from "./Button.svelte";

  const isRtl = $derived(getLocale() === "ar");

  // Gallery images library
  const galleryImages = [
    { src: "/categories/wedding-palace.webp", title_en: "The Royal Ballroom Dome", title_ar: "قبة القاعة الملكية", width: 800, height: 600 },
    { src: "/categories/hotel-venue.webp", title_en: "Modern Luxury Grand Foyer", title_ar: "بهو الفندق الحديث الفاخر", width: 800, height: 600 },
    { src: "/categories/villa-resort.webp", title_en: "Intimate Garden Pool Villa", title_ar: "فيلا المسبح والحديقة الخاصة", width: 800, height: 600 },
    { src: "/categories/outdoor-garden.webp", title_en: "Evening Chandelier Lawn Setup", title_ar: "تجهيز الساحة الخارجية بالثريات", width: 800, height: 600 },
    { src: "/categories/rooftop-venue.webp", title_en: "Skyline Sunset Terrace Lounge", title_ar: "شرفة أفق الغروب الخارجية", width: 800, height: 600 },
    { src: "/categories/private-beach.webp", title_en: "Red Sea Coastal Wedding Pier", title_ar: "رصيف شاطئ البحر الأحمر الخاص", width: 800, height: 600 }
  ];

  // Lightbox Modal state
  let isLightboxOpen = $state(false);
  let activeImageIndex = $state(0);
  let triggerElement = $state<HTMLElement | null>(null);
  let modalEl = $state<HTMLDivElement | null>(null);
  let closeBtnEl = $state<HTMLButtonElement | null>(null);

  const activeImage = $derived(galleryImages[activeImageIndex]);

  function openLightbox(e: MouseEvent, index: number) {
    triggerElement = e.currentTarget as HTMLElement;
    activeImageIndex = index;
    isLightboxOpen = true;
    // Set focus on close button after render
    setTimeout(() => {
      closeBtnEl?.focus();
    }, 50);
  }

  function closeLightbox() {
    isLightboxOpen = false;
    // Return focus to triggering element
    setTimeout(() => {
      triggerElement?.focus();
    }, 50);
  }

  function nextImage() {
    activeImageIndex = (activeImageIndex + 1) % galleryImages.length;
  }

  function prevImage() {
    activeImageIndex = (activeImageIndex - 1 + galleryImages.length) % galleryImages.length;
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (!isLightboxOpen) return;

    if (e.key === "Escape") {
      e.preventDefault();
      closeLightbox();
    } else if (e.key === "ArrowRight") {
      e.preventDefault();
      isRtl ? prevImage() : nextImage();
    } else if (e.key === "ArrowLeft") {
      e.preventDefault();
      isRtl ? nextImage() : prevImage();
    } else if (e.key === "Tab") {
      // Focus Trap within lightbox modal
      if (!modalEl) return;
      const focusableSelectors = 'button, [tabindex="0"]';
      const focusables = Array.from(modalEl.querySelectorAll(focusableSelectors)) as HTMLElement[];
      if (focusables.length === 0) return;

      const first = focusables[0];
      const last = focusables[focusables.length - 1];

      if (e.shiftKey && document.activeElement === first) {
        e.preventDefault();
        last.focus();
      } else if (!e.shiftKey && document.activeElement === last) {
        e.preventDefault();
        first.focus();
      }
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

<section class="py-zw-16 md:py-zw-24 bg-zw-surface-alt relative overflow-hidden" aria-labelledby="venue-gallery-title">
  <div class="w-full max-w-[1400px] mx-auto px-zw-4 sm:px-zw-6 md:px-zw-8 lg:px-zw-12">
    <!-- Title Section -->
    <div class="flex flex-col md:flex-row md:items-end justify-between gap-zw-6 mb-zw-12">
      <div>
        <span class="text-zw-primary text-zw-xs font-bold tracking-widest uppercase mb-zw-2 block">
          {isRtl ? "معرض الصور الملهمة" : "Inspiration Gallery"}
        </span>
        <h2 id="venue-gallery-title" class="font-display text-zw-3xl sm:text-zw-4xl font-bold text-zw-secondary leading-tight">
          {isRtl ? "شاهد جمال قاعاتنا الشريكة" : "Photography Showcase"}
        </h2>
        <p class="text-zw-muted text-zw-sm max-w-xl mt-zw-2">
          {isRtl
            ? "تصفح لقطات تفصيلية حية من التجهيزات الفاخرة للصالات الكبرى والمسابح والحدائق والمنصات الخارجية."
            : "Browse high resolution details from real luxury ballroom layouts, private coastal decks, and wedding lawn setups."}
        </p>
      </div>
    </div>

    <!-- Responsive Grid Gallery -->
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-zw-6">
      {#each galleryImages as img, idx}
        <button
          onclick={(e) => openLightbox(e, idx)}
          class="group relative rounded-zw-2xl overflow-hidden aspect-[4/3] border border-zw-border bg-zw-secondary/10 cursor-pointer focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-zw-primary select-none w-full"
          aria-label={isRtl ? `افتح صورة: ${img.title_ar}` : `View image: ${img.title_en}`}
        >
          <img
            src={img.src}
            alt={isRtl ? img.title_ar : img.title_en}
            class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
            loading="lazy"
            decoding="async"
            width={img.width}
            height={img.height}
          />
          <div class="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity duration-300 flex items-end p-zw-5">
            <span class="text-white font-display text-zw-sm font-bold text-start">
              {isRtl ? img.title_ar : img.title_en}
            </span>
          </div>
        </button>
      {/each}
    </div>
  </div>

  <!-- Accessible Lightbox Modal Backdrop Overlay -->
  {#if isLightboxOpen}
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      bind:this={modalEl}
      class="fixed inset-0 bg-zw-secondary/95 backdrop-blur-md z-[200] flex flex-col justify-between items-center p-zw-6"
      role="dialog"
      aria-modal="true"
      aria-label={isRtl ? "معرض الصور المكبرة" : "Expanded Photo View"}
      onclick={closeLightbox}
      tabindex="-1"
    >
      <!-- Top Action Bar (Close button) -->
      <div class="w-full max-w-[1200px] flex justify-end py-zw-2 z-30">
        <button
          bind:this={closeBtnEl}
          onclick={closeLightbox}
          class="bg-white/10 hover:bg-white/20 text-white border border-white/20 rounded-zw-full w-zw-12 h-zw-12 flex items-center justify-center cursor-pointer select-none text-zw-xl transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-zw-primary"
          aria-label={isRtl ? "إغلاق المعرض" : "Close Gallery"}
        >
          ✕
        </button>
      </div>

      <!-- Center Image + Navigation Arrow Buttons -->
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
      <div
        class="relative flex-grow flex items-center justify-center max-w-[1200px] w-full z-20"
        onclick={(e) => e.stopPropagation()}
        role="presentation"
      >
        <!-- Previous Arrow Button -->
        <button
          onclick={prevImage}
          class="absolute start-0 bg-black/60 hover:bg-black/80 text-white w-zw-12 h-zw-12 rounded-zw-full flex items-center justify-center cursor-pointer select-none focus-visible:outline focus-visible:outline-2 focus-visible:outline-zw-primary z-30 transform hover:scale-105 transition-all rtl:rotate-180"
          aria-label={isRtl ? "الصورة السابقة" : "Previous Image"}
        >
          ←
        </button>

        <!-- Dynamic Main Image View -->
        <div class="flex flex-col items-center gap-zw-4 max-h-[70vh] max-w-[85%] select-none animate-fade-in">
          <img
            src={activeImage.src}
            alt={isRtl ? activeImage.title_ar : activeImage.title_en}
            class="max-h-[65vh] object-contain rounded-zw-xl shadow-zw-xl border border-white/10"
            width={activeImage.width}
            height={activeImage.height}
            decoding="async"
          />
          <h3 class="text-white font-display text-zw-base font-bold text-center tracking-wide">
            {isRtl ? activeImage.title_ar : activeImage.title_en}
          </h3>
        </div>

        <!-- Next Arrow Button -->
        <button
          onclick={nextImage}
          class="absolute end-0 bg-black/60 hover:bg-black/80 text-white w-zw-12 h-zw-12 rounded-zw-full flex items-center justify-center cursor-pointer select-none focus-visible:outline focus-visible:outline-2 focus-visible:outline-zw-primary z-30 transform hover:scale-105 transition-all rtl:rotate-180"
          aria-label={isRtl ? "الصورة التالية" : "Next Image"}
        >
          →
        </button>
      </div>

      <!-- Slide indicator info at the bottom -->
      <div class="pb-zw-4 text-white/60 text-zw-xs font-semibold select-none z-20">
        {activeImageIndex + 1} / {galleryImages.length}
      </div>
    </div>
  {/if}
</section>
