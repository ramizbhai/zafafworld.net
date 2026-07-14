<script lang="ts">
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import StarRating from "$lib/components/ui/StarRating.svelte";
  import { getLocale } from "$lib/paraglide/runtime.js";
  import * as m from "$lib/paraglide/messages.js";

  let {
    venue,
    name,
    totalReviews,
    isSaved,
    t,
    getCategoryInfo,
    toggleSave,
    shareProfile,
    onShowAuthPopup,
    onShowInquiryModal,
    activeImage = $bindable(0),
    activeTab = $bindable("overview")
  } = $props<{
    venue: any;
    name: string;
    totalReviews: number;
    isSaved: boolean;
    t: (ar: string, en: string) => string;
    getCategoryInfo: (slug: string) => any;
    toggleSave: () => void;
    shareProfile: () => void;
    onShowAuthPopup: () => void;
    onShowInquiryModal: () => void;
    activeImage: number;
    activeTab: string;
  }>();

</script>

<!-- Hero Section with Parallax Background and Avatar Overlap -->
<section
  class="relative bg-[var(--color-secondary)] overflow-hidden h-[280px] md:h-[400px]"
  aria-label="Cover Image"
>
  {#if venue.images && venue.images.length > 0}
    <img
      src={venue.images[activeImage]?.url}
      alt={venue.images[activeImage]?.alt || name}
      class="w-full h-full object-cover opacity-70 transition-transform duration-[8000ms] hover:scale-105"
      fetchpriority="high"
    />
  {:else}
    <div
      class="w-full h-full bg-gradient-to-br from-[var(--color-secondary)] to-[#4a3f35] flex items-center justify-center"
    >
      <span class="text-7xl opacity-10">⚜️</span>
    </div>
  {/if}
  <div
    class="absolute inset-0 bg-gradient-to-t from-black/85 via-black/45 to-transparent"
  ></div>

  <!-- Cover Image Quick Navigation -->
  {#if venue.images.length > 1}
    <div class="absolute bottom-6 start-6 z-10 flex gap-2 hidden md:flex">
      {#each venue.images.slice(0, 5) as img, i}
        <button
          onclick={() => (activeImage = i)}
          class="w-12 h-12 rounded-lg overflow-hidden border-2 transition-all duration-300 {activeImage === i
            ? 'border-[var(--color-primary)] scale-105 shadow-md'
            : 'border-white/40 hover:border-white/80'}"
          aria-label="Image {i + 1}"
        >
          <img
            src={img.url}
            alt={img.alt}
            class="w-full h-full object-cover"
            loading="lazy"
          />
        </button>
      {/each}
      {#if venue.images.length > 5}
        <button
          onclick={() => {
            activeTab = "gallery";
            document
              .getElementById("tab-navigation-bar")
              ?.scrollIntoView({ behavior: "smooth" });
          }}
          class="w-12 h-12 rounded-lg bg-black/75 border-2 border-white/40 flex items-center justify-center text-white text-[10px] font-bold hover:bg-black/90 transition"
        >
          +{venue.images.length - 5}
        </button>
      {/if}
    </div>
  {/if}

  <!-- Top Left Badges -->
  <div class="absolute top-6 start-6 flex gap-2 z-10">
    {#if venue.isFeatured}
      <Badge
        variant="primary"
        class="font-semibold text-xs tracking-wider shadow-gold"
        >{m.venues_card_featured()}</Badge
      >
    {/if}
  </div>
</section>

<!-- Profile Brand Overlap Area -->
<div class="bg-white border-b border-[var(--color-border)] shadow-sm">
  <div class="container-page pb-6 md:pb-8 relative">
    <!-- Profile Picture (Brand Avatar) -->
    <div class="absolute -top-16 md:-top-24 start-6 md:start-12 z-20">
      <div class="relative group">
        <div
          class="w-28 h-28 md:w-36 md:h-36 rounded-full overflow-hidden border-4 border-white bg-[var(--color-surface-alt)] shadow-[var(--shadow-lg)] flex items-center justify-center transition-all duration-300 group-hover:scale-[1.02] group-hover:shadow-gold"
        >
          {#if venue.coordinator?.avatar}
            <img
              src={venue.coordinator.avatar}
              alt={name}
              class="w-full h-full object-cover"
            />
          {:else}
            <div
              class="w-full h-full bg-gradient-to-br from-[var(--color-surface-alt)] to-[var(--color-border)] text-[var(--color-primary-contrast)] flex items-center justify-center text-3xl md:text-5xl font-display font-extrabold"
            >
              {name.charAt(0).toUpperCase()}
            </div>
          {/if}
        </div>

        <!-- Verification Badge -->
        {#if venue.vendor?.verified}
          <div
            class="absolute bottom-1 end-1 w-8 h-8 rounded-full bg-[var(--color-primary)] text-[var(--color-secondary)] border-2 border-white flex items-center justify-center text-base shadow-md"
            title={t("حساب موثق", "Verified Account")}
          >
            ✓
          </div>
        {/if}
      </div>
    </div>

    <!-- Header Information Block -->
    <div
      class="pt-16 md:pt-4 ps-0 md:ps-48 flex flex-col md:flex-row md:items-end md:justify-between gap-6"
    >
      <!-- Text details -->
      <div class="text-start">
        <div class="flex flex-wrap items-center gap-2 mb-2">
          {#each venue.category as cat}
            {@const catInfo = getCategoryInfo(cat)}
            <span
              class="inline-flex items-center gap-1 text-xs px-2.5 py-1 rounded-full bg-[var(--color-surface-alt)] text-[var(--color-secondary)] font-medium"
            >
              <span>{catInfo.icon}</span>
              <span>{catInfo.label}</span>
            </span>
          {/each}
        </div>

        <h1
          class="font-display text-2xl md:text-4xl font-extrabold text-[var(--color-secondary)] mb-2 flex items-center gap-2 flex-wrap"
        >
          {name}
        </h1>

        <div
          class="flex flex-wrap items-center gap-4 text-xs md:text-sm text-[var(--color-muted)] font-medium"
        >
          <!-- Location -->
          <p class="flex items-center gap-1">
            <span class="text-[var(--color-primary-contrast)] text-sm"
              >📍</span
            >
            {venue.location.city}، {venue.location.district}
          </p>

          <span
            class="w-1.5 h-1.5 rounded-full bg-[var(--color-border)] hidden sm:inline-block"
          ></span>

          <!-- Ratings summary -->
          <button
            onclick={() => {
              activeTab = "reviews";
              document
                .getElementById("tab-navigation-bar")
                ?.scrollIntoView({ behavior: "smooth" });
            }}
            class="flex items-center gap-1.5 hover:text-[var(--color-primary-contrast)] transition"
          >
            <StarRating rating={venue.rating} size="sm" />
            <span class="underline font-bold"
              >({totalReviews > 0 ? totalReviews : venue.reviewCount}
              {m.auto_reviews_1()})</span
            >
          </button>
        </div>
      </div>

      <!-- Desktop Quick Actions -->
      <div class="flex items-center gap-2 mt-2 md:mt-0 flex-wrap">
        <!-- Favorite -->
        <button
          onclick={toggleSave}
          class="flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl border border-[var(--color-border)] text-sm font-bold bg-white text-[var(--color-secondary)] hover:bg-[var(--color-surface-alt)] transition cursor-pointer"
          aria-pressed={isSaved}
        >
          <svg
            viewBox="0 0 24 24"
            class="w-4 h-4 {isSaved ? 'fill-[var(--color-error)] stroke-none'
              : 'fill-none stroke-current stroke-2'}"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12z"
            />
          </svg>
          <span
            >{isSaved ? t("مفضّل", "Saved") : t("حفظ للمفضلة", "Save")}</span
          >
        </button>

        <!-- Share -->
        <button
          onclick={shareProfile}
          class="flex items-center justify-center gap-2 px-4 py-2.5 rounded-xl border border-[var(--color-border)] text-sm font-bold bg-white text-[var(--color-secondary)] hover:bg-[var(--color-surface-alt)] transition cursor-pointer"
        >
          <span class="text-sm">🔗</span>
          <span>{t("مشاركة", "Share")}</span>
        </button>

        <!-- Inquire Direct -->
        <Button
          onclick={() => {
            onShowAuthPopup();
            onShowInquiryModal();
          }}
          variant="primary"
          size="md"
          class="shadow-[var(--shadow-gold)] font-bold text-xs"
        >
          ⚜️ {t("إرسال طلب استفسار", "Send Inquiry")}
        </Button>
      </div>
    </div>
  </div>
</div>
