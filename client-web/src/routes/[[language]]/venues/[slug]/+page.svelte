<script lang="ts">
  import { page } from "$app/stores";
  import { getLocale } from "$lib/paraglide/runtime.js";
  import { getLocalizedField } from "$lib/utils/localize.js";
  import * as m from "$lib/paraglide/messages.js";
  import VenueCard from "$lib/components/shared/VenueCard.svelte";
  import Skeleton from "$lib/components/ui/Skeleton.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import type { Venue } from "$lib/types/index.js";
  
  import { createVenuePageState } from "$lib/stores/venuePageState.svelte.js";
  import { submitVenueInquiry } from "$lib/services/api/inquiries.service.js";
  import { submitVenueReview, fetchLiveReviews } from "$lib/services/api/reviews.service.js";

  import VenueHeroGallery from "$lib/components/venues/VenueHeroGallery.svelte";
  import VenueOverview from "$lib/components/venues/VenueOverview.svelte";
  import VenueHalls from "$lib/components/venues/VenueHalls.svelte";
  import VenueReviews from "$lib/components/venues/VenueReviews.svelte";
  import VenueInquiryStickyBar from "$lib/components/venues/VenueInquiryStickyBar.svelte";

  let { data } = $props();
  const AMENITIES = $derived($page.data.metadata?.amenities || []);
  const user = $derived(data.user);
  let venue = $derived<Venue | null>(data.venue);
  let similar = $derived<Venue[]>(data.similar);
  let loading = $state(false);

  let pageState = createVenuePageState();

  // Localization helper
  const t = (ar: string, en: string) => (getLocale() === "ar" ? ar : en);

  const isVenue = $derived(
    venue?.category.some((cat: string) =>
      [
        "wedding-palace",
        "hotel-venue",
        "villa-resort",
        "restaurant-event",
        "outdoor-garden",
        "rooftop-venue",
        "private-beach",
        "chalet",
      ].includes(cat),
    ) ?? true,
  );

  const name = $derived(venue ? getLocalizedField(venue, "name", getLocale()) : "");
  const description = $derived(venue ? getLocalizedField(venue, "description", getLocale()) : "");
  const metaTitle = $derived(venue ? getLocalizedField(venue, "metaTitle", getLocale()) : "");
  const metaDescription = $derived(venue ? getLocalizedField(venue, "metaDescription", getLocale()) : "");

  $effect(() => {
    if (venue) {
      const saved = JSON.parse(localStorage.getItem("zafaf_saved_vendors") || "[]");
      pageState.isSaved = saved.includes(venue.id);
    }
  });

  $effect(() => {
    if (venue) {
      fetchLiveReviews(venue.id).then(res => {
        if (res) {
          pageState.liveReviews = res.liveReviews;
          pageState.averageRating = res.averageRating;
          pageState.totalReviews = res.totalReviews;
        }
      });
    }
  });

  async function handleInquiry(e: SubmitEvent) {
    e.preventDefault();
    const res = await submitVenueInquiry({
      venue,
      user,
      eventDate: pageState.eventDate,
      guestCount: pageState.guestCount,
      inquiryMessage: pageState.inquiryMessage,
      isVenue
    });
    
    if (res.success) {
      pageState.showInquiryModal = false;
      pageState.eventDate = "";
      pageState.guestCount = null;
      pageState.inquiryMessage = "";
    } else {
      pageState.inquiryError = res.error || m.auto_failed_to_submit_inq();
    }
    pageState.submittingInquiry = false;
  }

  async function handleReview(e: Event) {
    e.preventDefault();
    if (!venue) return;
    if (!user) {
      pageState.showAuthPopup = true;
      return;
    }
    pageState.submittingReview = true;
    const res = await submitVenueReview({
      venueId: venue.id,
      rating: pageState.newRating,
      reviewText: pageState.reviewText,
      reviewPhotos: pageState.reviewPhotos
    });

    if (res.success) {
      pageState.showReviewSuccessOverlay = true;
      pageState.reviewText = "";
      pageState.newRating = 5;
      pageState.reviewPhotos = [];
    }
    pageState.submittingReview = false;
  }
  
  // Expose review submit function to state so VenueReviews can call it directly
  $effect(() => {
      (pageState as any).submitReviewForm = handleReview;
  });

  function handleKeyDown(e: KeyboardEvent) {
    if (!pageState.isLightboxOpen || !venue) return;
    if (e.key === "Escape") {
      pageState.isLightboxOpen = false;
    } else if (e.key === "ArrowRight") {
      if (getLocale() === "ar") prevImage();
      else nextImage();
    } else if (e.key === "ArrowLeft") {
      if (getLocale() === "ar") nextImage();
      else prevImage();
    }
  }

  function nextImage() {
    if (!venue) return;
    pageState.lightboxIndex = (pageState.lightboxIndex + 1) % venue.images.length;
  }

  function prevImage() {
    if (!venue) return;
    pageState.lightboxIndex = (pageState.lightboxIndex - 1 + venue.images.length) % venue.images.length;
  }

  const tabs = $derived([
    { key: "overview", label: m.venues_details_overview() },
    ...(venue?.halls && venue.halls.length > 0
      ? [{ key: "halls", label: isVenue ? m.auto_event_halls() : t("الخدمات والمنتجات", "Services & Packages") }]
      : []),
    { key: "gallery", label: m.venues_details_gallery() },
    ...(isVenue ? [{ key: "amenities", label: m.venues_details_amenities() }] : []),
    { key: "reviews", label: m.venues_details_reviews() },
    { key: "location", label: m.venues_details_location() },
  ]);

  function getCategoryInfo(slug: string) {
    const backendCategories = $page.data?.metadata?.categories || [];
    const found = backendCategories.find((c: any) => c.key === slug || c.slug === slug);
    if (found) {
      return {
        slug: found.key || found.slug,
        icon: found.icon || "✨",
        label: getLocale() === "ar" ? found.labelAr || found.ar : found.labelEn || found.en,
      };
    }
    const fallbackMap: Record<string, { icon: string; ar: string; en: string }> = {
      "wedding-palace": { icon: "🏛️", ar: "قاعة أفراح", en: "Wedding Palace" },
      "hotel-venue": { icon: "🏨", ar: "قاعة فندق", en: "Hotel Ballroom" },
      "villa-resort": { icon: "🏡", ar: "استراحة وفيلا", en: "Villa & Resort" },
      "restaurant-event": { icon: "🍽️", ar: "مطعم وقاعة خاصة", en: "Restaurant & Dining" },
      "outdoor-garden": { icon: "🌿", ar: "حديقة مفتوحة", en: "Outdoor Garden" },
      chalet: { icon: "🏕️", ar: "شاليه", en: "Chalet" },
      "photography-video": { icon: "📷", ar: "تصوير وفيديو", en: "Photography & Video" },
      "photo-studio": { icon: "📸", ar: "استوديو تصوير", en: "Photo Studio" },
      "wedding-planner": { icon: "📋", ar: "منظم حفلات", en: "Wedding Planner" },
      "hair-makeup": { icon: "💄", ar: "شعر ومكياج", en: "Hair & Makeup" },
      "henna-art": { icon: "🌿", ar: "نقش حناء", en: "Henna Art" },
      "male-grooming": { icon: "🪒", ar: "صالون رجالي", en: "Male Grooming" },
      "wedding-gown": { icon: "👗", ar: "فساتين زفاف", en: "Wedding Gown" },
      "wedding-invitation": { icon: "✉️", ar: "بطاقات دعوة", en: "Wedding Invitation" },
      "entertainment-dj": { icon: "🎵", ar: "دي جي وحفلات", en: "DJ & Entertainment" },
      "wedding-jewelry": { icon: "💍", ar: "مجوهرات خواتم", en: "Bridal Jewelry" },
      "wedding-sweets": { icon: "🍬", ar: "حلويات وضيافة", en: "Arabic Sweets" },
      "wedding-gifts": { icon: "🎁", ar: "توزيعات هدايا", en: "Wedding Gifts" },
      catering: { icon: "🍱", ar: "ضيافة وبوفيه", en: "Wedding Catering" },
      "wedding-cake": { icon: "🎂", ar: "كيكة زفاف", en: "Wedding Cake" },
      "flowers-floral": { icon: "💐", ar: "ورد وزهور", en: "Flowers & Floral" },
    };
    const fb = fallbackMap[slug] || { icon: "✨", ar: slug, en: slug };
    return { slug, icon: fb.icon, label: getLocale() === "ar" ? fb.ar : fb.en };
  }
  
  async function shareProfile() {
    try {
      await navigator.clipboard.writeText(window.location.href);
    } catch (err) {
      console.error(err);
    }
  }
</script>

<svelte:window onkeydown={handleKeyDown} />

<svelte:head>
  {#if venue}
    <title>{metaTitle || `${name} - ${m.meta_siteName()}`}</title>
    <meta name="description" content={metaDescription || description.slice(0, 160)} />
  {:else}
    <title>{m.venues_title()} - {m.meta_siteName()}</title>
  {/if}
</svelte:head>

<!-- Breadcrumb -->
<div class="bg-[var(--color-surface)] border-b border-[var(--color-border)]">
  <div class="container-page py-4">
    <nav aria-label="Breadcrumb">
      <ol class="flex items-center gap-2 text-xs md:text-sm text-[var(--color-muted)]" role="list">
        <li>
          <a href="/" class="hover:text-[var(--color-primary)] transition-colors">{m.nav_home()}</a>
        </li>
        <li aria-hidden="true" class="rtl:rotate-180">/</li>
        <li>
          <a href="/" class="hover:text-[var(--color-primary)] transition-colors">{t("مزوّدي الخدمات", "Vendors")}</a>
        </li>
        <li aria-hidden="true" class="rtl:rotate-180">/</li>
        <li class="text-[var(--color-text)] font-semibold truncate max-w-[150px] md:max-w-xs" aria-current="page">
          {loading ? "..." : name}
        </li>
      </ol>
    </nav>
  </div>
</div>

{#if loading}
  <div class="container-page py-10">
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
      <div class="lg:col-span-2">
        <Skeleton height="h-96" rounded />
        <div class="mt-6 flex flex-col gap-3">
          <Skeleton height="h-8" width="w-1/2" />
          <Skeleton height="h-4" width="w-1/3" />
          <Skeleton height="h-4" />
          <Skeleton height="h-4" />
        </div>
      </div>
      <div>
        <Skeleton height="h-64" rounded />
      </div>
    </div>
  </div>
{:else if !venue}
  <div class="container-page py-24 text-center">
    <div class="text-6xl mb-6">✨</div>
    <h1 class="font-display text-3xl font-bold text-[var(--color-secondary)] mb-4">{m.errors_notFound()}</h1>
    <p class="text-[var(--color-muted)] mb-8 max-w-md mx-auto">{m.errors_notFoundDescription()}</p>
    <Button href="/" variant="primary">{m.errors_goHome()}</Button>
  </div>
{:else}
  <VenueHeroGallery 
    {venue}
    {name}
    totalReviews={pageState.totalReviews}
    isSaved={pageState.isSaved}
    {t}
    {getCategoryInfo}
    toggleSave={() => pageState.toggleSave(venue.id, t)}
    {shareProfile}
    onShowAuthPopup={() => pageState.showAuthPopup = true}
    onShowInquiryModal={() => pageState.showInquiryModal = true}
    bind:activeImage={pageState.activeImage}
    bind:activeTab={pageState.activeTab}
  />

  <main class="container-page py-10">
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
      <!-- LEFT / MAIN BLOCK: Tab Content (col-span-2) -->
      <section class="lg:col-span-2 min-w-0" aria-label="Main Details">
        <div id="tab-navigation-bar" class="border-b border-[var(--color-border)] mb-8 overflow-x-auto scrollbar-hide sticky top-[80px] bg-[var(--color-surface)] z-30 py-2">
          <div class="flex gap-1" role="tablist">
            {#each tabs as tab}
              <button
                role="tab"
                aria-selected={pageState.activeTab === tab.key}
                onclick={() => (pageState.activeTab = tab.key as typeof pageState.activeTab)}
                class="px-5 py-3 text-sm font-bold whitespace-nowrap border-b-2 transition-all duration-200 cursor-pointer {pageState.activeTab === tab.key ? 'border-[var(--color-primary-dark)] text-[var(--color-primary-contrast)]' : 'border-transparent text-[var(--color-muted)] hover:text-[var(--color-text)]'}"
              >
                {tab.label}
              </button>
            {/each}
          </div>
        </div>

        <div role="tabpanel" class="focus:outline-none">
          {#if pageState.activeTab === "overview"}
            <VenueOverview {venue} {description} {t} />
          {:else if pageState.activeTab === "halls"}
            <VenueHalls {venue} {user} {isVenue} {t} onShowAuthPopup={() => pageState.showAuthPopup = true} onShowInquiryModal={() => pageState.showInquiryModal = true} />
          {:else if pageState.activeTab === "gallery"}
            <div class="bg-white p-6 rounded-2xl border border-[var(--color-border)] shadow-sm text-start">
              <h2 class="font-display text-xl font-bold text-[var(--color-secondary)] mb-6 underline-gold pb-1">{t("معرض الصور والأعمال", "Photos & Works Gallery")}</h2>
              {#if !venue.images || venue.images.length === 0}
                <div class="p-8 text-center text-[var(--color-muted)] bg-[var(--color-surface-alt)] rounded-xl">{t("لا توجد صور متوفرة في المعرض حالياً.", "No images uploaded in the gallery yet.")}</div>
              {:else}
                <div class="grid grid-cols-2 sm:grid-cols-3 gap-3 md:gap-4">
                  {#each venue.images as img, i}
                    <button onclick={() => { pageState.lightboxIndex = i; pageState.isLightboxOpen = true; }} class="aspect-square rounded-xl overflow-hidden bg-[var(--color-surface-alt)] relative group shadow-sm transition hover:-translate-y-1 hover:shadow-md duration-300 cursor-zoom-in" aria-label="Expand Image {i + 1}">
                      <img src={img.url} alt={img.alt} class="w-full h-full object-cover transition-transform duration-500 group-hover:scale-105" loading="lazy" />
                      <div class="absolute inset-0 bg-black/25 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center"><span class="text-white text-xl">🔍</span></div>
                    </button>
                  {/each}
                </div>
              {/if}
            </div>
          {:else if pageState.activeTab === "amenities"}
            <div class="bg-white p-6 md:p-8 rounded-2xl border border-[var(--color-border)] shadow-sm text-start">
              <h2 class="font-display text-xl font-bold text-[var(--color-secondary)] mb-6 underline-gold pb-1">{m.venues_details_amenities()}</h2>
              {#if !venue.amenities || venue.amenities.length === 0}
                <div class="p-6 text-center text-[var(--color-muted)]">{t("لم يتم إدراج تجهيزات خاصة.", "No special facilities listed.")}</div>
              {:else}
                <div class="grid grid-cols-2 sm:grid-cols-3 gap-3 md:gap-4">
                  {#each AMENITIES.filter( (a: any) => venue?.amenities.includes(a.key), ) as amenity}
                    <div class="flex items-center gap-3 p-4 rounded-xl bg-[var(--color-surface-alt)] border border-[var(--color-border)] transition hover:bg-[var(--color-border)] duration-200">
                      <span class="text-2xl" aria-hidden="true">{amenity.icon}</span>
                      <span class="text-sm font-semibold text-[var(--color-text)]">{getLocalizedField(amenity, "label", getLocale())}</span>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>
          {:else if pageState.activeTab === "reviews"}
            <VenueReviews {venue} state={pageState} {t} />
          {:else if pageState.activeTab === "location"}
            <div class="bg-white p-6 md:p-8 rounded-2xl border border-[var(--color-border)] shadow-sm text-start">
              <h2 class="font-display text-xl font-bold text-[var(--color-secondary)] mb-6 underline-gold pb-1">{m.venues_details_location()}</h2>
              {#if data.user}
                <div class="flex items-start gap-2.5 mb-6 text-sm"><span class="text-xl">📍</span><p class="text-[var(--color-text)] font-semibold">{venue.location.address}</p></div>
                <div class="rounded-xl overflow-hidden h-72 md:h-96 bg-[var(--color-surface-alt)] border border-[var(--color-border)] relative flex items-center justify-center">
                  <div class="text-center p-6"><span class="text-4xl block mb-2">🗺️</span><p class="text-sm font-bold text-[var(--color-secondary)] mb-1">{t("خريطة موقع مزوّد الخدمة", "Vendor Location Map")}</p><p class="text-xs text-[var(--color-muted)] mb-4">{venue.location.lat}, {venue.location.lng}</p>{#if venue.mapsUrl}<a href={venue.mapsUrl} target="_blank" rel="noopener noreferrer" class="inline-flex items-center justify-center gap-2 font-medium rounded-lg transition-all duration-200 bg-transparent border border-[var(--color-primary-contrast)] text-[var(--color-primary-contrast)] hover:bg-[var(--color-primary-contrast)] hover:text-[var(--color-text-inverse)] px-4 py-2 text-sm font-bold">🗺️ {t("فتح في خرائط جوجل", "Open in Google Maps")}</a>{/if}</div>
                </div>
              {:else}
                <div class="flex flex-col items-center justify-center py-12 text-center bg-[var(--color-surface-alt)] rounded-xl border border-[var(--color-border)]">
                  <span class="text-5xl mb-4 select-none filter blur-[1px]">📍</span><p class="text-sm font-bold text-[var(--color-secondary)] mb-2">{m.auto_location_details_are()}</p><p class="text-xs text-[var(--color-muted)] max-w-sm mb-6 px-6">{m.auto_please_login_or_regi()}</p><Button href="/auth/login" variant="primary" size="sm" class="font-bold shadow-md">{m.auto_login_now()}</Button>
                </div>
              {/if}
            </div>
          {/if}
        </div>
      </section>

      <VenueInquiryStickyBar {venue} {user} {isVenue} {t} onShowAuthPopup={() => pageState.showAuthPopup = true} onShowInquiryModal={() => pageState.showInquiryModal = true} />
    </div>

    {#if similar && similar.length > 0}
      <section class="pt-10 border-t border-[var(--color-border)]" aria-labelledby="similar-vendors-title">
        <h2 id="similar-vendors-title" class="font-display text-2xl md:text-3xl font-extrabold text-[var(--color-secondary)] mb-8 text-start">
          ⚜️ {isVenue ? m.auto_similar_venues() : t("مزوّدو خدمات مشابهون", "Similar Vendors You May Like")}
        </h2>
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
          {#each similar as v (v.id)}
            <VenueCard venue={v} />
          {/each}
        </div>
      </section>
    {/if}
  </main>

  <div class="fixed bottom-0 inset-x-0 bg-white/90 backdrop-blur-md border-t border-[var(--color-border)] z-40 p-4 flex gap-3 lg:hidden shadow-[0_-4px_16px_rgba(45,38,32,0.08)]">
    <Button onclick={() => { if (!user) pageState.showAuthPopup = true; else pageState.showInquiryModal = true; }} size="md" class="flex-1 text-white bg-[#5b21b6] hover:bg-[#4c1d95] border-none font-bold text-xs">⚜️ {m.auto_check_availability_()}</Button>
    <Button href={user ? `/booking/${venue.id}` : undefined} onclick={(e) => { if (!user) { e.preventDefault(); pageState.showAuthPopup = true; } }} variant="primary" class="flex-1 font-bold text-xs shadow-gold">⚡ {m.venues_details_bookNow()}</Button>
  </div>

  {#if pageState.isLightboxOpen}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="fixed inset-0 z-[500] bg-black/95 backdrop-blur-md flex flex-col justify-between p-4 transition-all duration-300" onclick={() => (pageState.isLightboxOpen = false)}>
      <div class="flex justify-between items-center text-white py-2 px-4 z-50">
        <span class="text-xs md:text-sm font-semibold tracking-wider font-mono">{pageState.lightboxIndex + 1} / {venue.images.length}</span>
        <button onclick={() => (pageState.isLightboxOpen = false)} class="w-10 h-10 rounded-full bg-white/10 flex items-center justify-center text-white hover:bg-white/20 transition cursor-pointer text-lg" aria-label="Close Lightbox">✕</button>
      </div>
      <div class="flex-1 flex items-center justify-center relative select-none">
        <button onclick={(e) => { e.stopPropagation(); prevImage(); }} class="absolute start-2 md:start-6 w-12 h-12 rounded-full bg-white/10 hover:bg-white/20 text-white flex items-center justify-center transition cursor-pointer z-50 font-bold" aria-label="Previous Image">❮</button>
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div onclick={(e) => e.stopPropagation()}>
          <img src={venue.images[pageState.lightboxIndex]?.url} alt={venue.images[pageState.lightboxIndex]?.alt || name} class="max-w-full max-h-[75dvh] object-contain shadow-2xl rounded-lg" />
        </div>
        <button onclick={(e) => { e.stopPropagation(); nextImage(); }} class="absolute end-2 md:end-6 w-12 h-12 rounded-full bg-white/10 hover:bg-white/20 text-white flex items-center justify-center transition cursor-pointer z-50 font-bold" aria-label="Next Image">❯</button>
      </div>
      <div class="text-center text-white/80 py-4 px-6 max-w-2xl mx-auto z-50">
        <p class="text-xs md:text-sm font-medium">{venue.images[pageState.lightboxIndex]?.alt || name}</p>
      </div>
    </div>
  {/if}

  {#if pageState.showAuthPopup}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="fixed inset-0 z-[var(--z-modal)] bg-black/60 backdrop-blur-sm flex items-center justify-center p-4" onclick={() => (pageState.showAuthPopup = false)}>
      <div class="bg-white rounded-2xl border border-[var(--color-border)] w-full max-w-sm shadow-[var(--shadow-lg)] p-6 text-center" onclick={(e) => e.stopPropagation()}>
        <span class="text-4xl block mb-4">🔒</span>
        <h3 class="font-display text-lg font-bold text-[var(--color-secondary)] mb-2 flex flex-col gap-1"><span>Please login or register first</span><span class="text-xs font-semibold text-[var(--color-muted)]">يرجى تسجيل الدخول أو الاشتراك أولاً</span></h3>
        <div class="text-xs text-[var(--color-muted)] mb-6 flex flex-col gap-2"><p dir="ltr">You must be signed in to access direct venue bookings and rates inquiry.</p><p dir="rtl">يجب تسجيل الدخول إلى حسابك للوصول إلى الحجز المباشر واستعلام الأسعار.</p></div>
        <div class="flex flex-col gap-2">
          <Button href="/auth/login" variant="primary" size="sm" class="w-full font-bold">{m.auto_sign_in___()}</Button>
          <Button href="/auth/register" variant="outline" size="sm" class="w-full font-bold">{m.auto_register_now__()}</Button>
          <Button variant="ghost" size="sm" onclick={() => (pageState.showAuthPopup = false)} class="w-full mt-1 font-semibold text-xs">{m.auto_close__()}</Button>
        </div>
      </div>
    </div>
  {/if}

  {#if pageState.showInquiryModal}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="fixed inset-0 z-[var(--z-modal)] bg-black/60 backdrop-blur-sm flex items-center justify-center p-4" onclick={() => (pageState.showInquiryModal = false)}>
      <div class="bg-[var(--color-surface)] rounded-2xl border border-[var(--color-border)] w-full max-w-md shadow-[var(--shadow-lg)] overflow-hidden" onclick={(e) => e.stopPropagation()}>
        <div class="p-6 border-b border-[var(--color-border)] flex justify-between items-center bg-[var(--color-surface-alt)]">
          <div class="text-start"><h3 class="font-display text-lg font-bold text-[var(--color-secondary)]">{m.auto_check_availability_()}</h3><p class="text-xs text-[var(--color-muted)] mt-1 font-medium">{m.auto_send_a_direct_reques()}</p></div>
          <button onclick={() => (pageState.showInquiryModal = false)} class="text-[var(--color-muted)] hover:text-[var(--color-secondary)] cursor-pointer text-lg p-2">✕</button>
        </div>
        <form onsubmit={handleInquiry} class="p-6 flex flex-col gap-4 text-start">
          <div class="flex flex-col gap-1.5"><label for="inquiry-date" class="text-xs font-bold text-[var(--color-secondary)]">{m.auto_event_date()} *</label><input id="inquiry-date" type="date" bind:value={pageState.eventDate} required min={new Date().toISOString().split("T")[0]} class="w-full p-3 border border-[var(--color-border)] rounded-xl bg-white text-sm focus:outline-none focus:border-[var(--color-primary-contrast)] focus:ring-1 focus:ring-[var(--color-primary)] transition" /></div>
          <div class="flex flex-col gap-1.5"><label for="inquiry-guests" class="text-xs font-bold text-[var(--color-secondary)]">{m.auto_expected_guest_count()} {isVenue ? "*" : t("(اختياري)", "(Optional)")}</label><input id="inquiry-guests" type="number" min="1" step="1" bind:value={pageState.guestCount} required={isVenue} class="w-full p-3 border border-[var(--color-border)] rounded-xl bg-white text-sm focus:outline-none focus:border-[var(--color-primary-contrast)] focus:ring-1 focus:ring-[var(--color-primary)] transition" placeholder={m.auto_eg_150()} /></div>
          <div class="flex flex-col gap-1.5"><label for="inquiry-message" class="text-xs font-bold text-[var(--color-secondary)]">{m.auto_message_details()} *</label><textarea id="inquiry-message" bind:value={pageState.inquiryMessage} required rows="4" class="w-full p-3 border border-[var(--color-border)] rounded-xl bg-white text-sm resize-none focus:outline-none focus:border-[var(--color-primary-contrast)] focus:ring-1 focus:ring-[var(--color-primary)] transition" placeholder={m.auto_type_your_inquiries_()}></textarea></div>
          {#if pageState.inquiryError}<p class="text-xs text-[var(--color-error)] text-center font-bold bg-red-50 p-2.5 rounded-lg border border-red-100">{pageState.inquiryError}</p>{/if}
          <Button type="submit" loading={pageState.submittingInquiry} fullWidth size="lg" class="text-white bg-[#5b21b6] hover:bg-[#4c1d95] border-none shadow-md font-bold text-xs transition-colors duration-200 mt-2">{m.auto_submit_inquiry_now()}</Button>
        </form>
      </div>
    </div>
  {/if}
{/if}
