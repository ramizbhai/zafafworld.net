<script lang="ts">
  import type { Venue } from '$lib/types/index.js';
  import { resolveMediaUrl, getOptimizedImage } from '$lib/shared/utils/media.js';
  import * as m from '$lib/paraglide/messages.js';
import { getLocale } from '$lib/paraglide/runtime.js';
import { getLocalizedField, formatCurrency, formatNumber, formatDate } from '$lib/utils/localize.js';
  import Badge from '$lib/components/ui/Badge.svelte';
  import StarRating from '$lib/components/ui/StarRating.svelte';

  interface Props {
    venue: Venue;
    layout?: 'grid' | 'list';
    class?: string;
  }

  let { venue, layout = 'grid', class: extraClass = '' }: Props = $props();

  let isWishlisted = $state(false);
  let imgError = $state(false);

  const name = $derived(getLocalizedField(venue, 'name', getLocale()));
  const city = $derived(venue.location.city);
  const district = $derived(venue.location.district);
  const primaryImage = $derived(venue.images.find((img) => img.isPrimary) ?? venue.images[0]);
</script>

<article
  class="
    group rounded-2xl overflow-hidden bg-white border border-[var(--color-border)]
    card-hover
    {layout === 'list' ? 'flex flex-row' : 'flex flex-col'}
    {venue.isFeatured ? 'featured-card' : ''}
    {extraClass}
  "
  aria-label={name}
>
  <!-- Image -->
  <div class="
    relative overflow-hidden bg-[var(--color-surface-alt)]
    {layout === 'list' ? 'w-64 flex-shrink-0' : 'aspect-[4/3] w-full'}
  ">
    {#if primaryImage && !imgError}
      <img
        src={resolveMediaUrl(getOptimizedImage(primaryImage.url, 'card'))}
        alt={primaryImage.alt || name || 'venue image'}
        loading="lazy"
        width="400"
        height="300"
        onerror={(e) => {
          const img = e.currentTarget as HTMLImageElement;
          const fallback = resolveMediaUrl(primaryImage.url);
          if (img.src !== fallback) {
            img.src = fallback;
          } else {
            imgError = true;
          }
        }}
        class="w-full h-full object-cover transition-transform duration-500 group-hover:scale-105"
      />
    {:else}
      <div class="w-full h-full flex items-center justify-center text-[var(--color-muted)]">
        <svg viewBox="0 0 24 24" class="w-12 h-12 opacity-30" fill="none" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M2.25 15.75l5.159-5.159a2.25 2.25 0 013.182 0l5.159 5.159m-1.5-1.5l1.409-1.409a2.25 2.25 0 013.182 0l2.909 2.909m-18 3.75h16.5a1.5 1.5 0 001.5-1.5V6a1.5 1.5 0 00-1.5-1.5H3.75A1.5 1.5 0 002.25 6v12a1.5 1.5 0 001.5 1.5zm10.5-11.25h.008v.008h-.008V8.25zm.375 0a.375.375 0 11-.75 0 .375.375 0 01.75 0z"/>
        </svg>
      </div>
    {/if}

    <!-- Gradient overlay -->
    <div class="absolute inset-0 gradient-dark opacity-0 group-hover:opacity-100 transition-opacity duration-300" aria-hidden="true"></div>

    <!-- Badges -->
    <div class="absolute top-3 start-3 flex flex-col gap-1.5">
      {#if venue.isFeatured}
        <div class="featured-ribbon">
          <svg viewBox="0 0 20 20" class="w-3 h-3 fill-current" aria-hidden="true">
            <path fill-rule="evenodd" d="M10.868 2.884c-.321-.772-1.415-.772-1.736 0l-1.83 4.401-4.753.381c-.833.067-1.171 1.107-.536 1.651l3.6 3.1-1.096 4.637c-.192.813.687 1.45 1.393 1.006l4.053-2.54 4.053 2.54c.706.444 1.585-.193 1.393-1.006l-1.096-4.637 3.6-3.1c.635-.544.297-1.584-.536-1.65l-4.752-.382-1.831-4.401z" clip-rule="evenodd"/>
          </svg>
          <span>{m.auto_featured()}</span>
        </div>
      {/if}
      {#if !venue.isAvailable}
        <Badge variant="error" size="sm">{m.venues_card_unavailable()}</Badge>
      {/if}
    </div>

    <!-- Wishlist -->
    <button
      onclick={() => { isWishlisted = !isWishlisted; }}
      class="
        absolute top-3 end-3 w-9 h-9 rounded-full bg-white/90 backdrop-blur-sm
        flex items-center justify-center shadow-[var(--shadow-sm)]
        transition-all duration-200 hover:scale-110
      "
      aria-label={m.venues_card_wishlist()}
      aria-pressed={isWishlisted}
    >
      <svg viewBox="0 0 24 24" class="w-5 h-5 {isWishlisted ? 'fill-[var(--color-error)] stroke-none' : 'fill-none stroke-[var(--color-muted)] stroke-2'}">
        <path stroke-linecap="round" stroke-linejoin="round" d="M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12z"/>
      </svg>
    </button>

    <!-- Image count indicator -->
    {#if venue.images.length > 1}
      <div class="absolute bottom-3 end-3 bg-black/50 backdrop-blur-sm rounded-full px-2 py-1 text-xs text-white flex items-center gap-1">
        <svg viewBox="0 0 24 24" class="w-3 h-3" fill="currentColor"><path d="M2.25 15.75l5.159-5.159a2.25 2.25 0 013.182 0l5.159 5.159m-1.5-1.5l1.409-1.409a2.25 2.25 0 013.182 0l2.909 2.909"/></svg>
        {venue.images.length}
      </div>
    {/if}
  </div>

  <!-- Content -->
  <div class="flex flex-col flex-1 p-5">
    <!-- Location -->
    <div class="flex items-center gap-1.5 text-xs text-[var(--color-muted)] mb-2">
      <svg viewBox="0 0 24 24" class="w-3.5 h-3.5 flex-shrink-0" fill="none" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M15 10.5a3 3 0 11-6 0 3 3 0 016 0z"/>
        <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 10.5c0 7.142-7.5 11.25-7.5 11.25S4.5 17.642 4.5 10.5a7.5 7.5 0 1115 0z"/>
      </svg>
      <span>{city}{district ? `، ${district}` : ''}</span>
    </div>

    <!-- Name -->
    <h3 class="font-display text-lg font-semibold text-[var(--color-secondary)] mb-2 line-clamp-1">
      <a href="/venues/{venue.slug}" class="hover:text-[var(--color-primary)] transition-colors">
        {name}
      </a>
    </h3>

    <!-- Rating -->
    <div class="mb-3">
      <StarRating
        rating={venue.rating}
        showValue
        reviewCount={venue.reviewCount}
        size="sm"
        label="{venue.rating} {m.common_stars()} — {venue.reviewCount} {m.common_reviews()}"
      />
    </div>

    <!-- Capacity -->
    <div class="flex items-center gap-1.5 text-xs text-[var(--color-muted)] mb-4">
      <svg viewBox="0 0 24 24" class="w-3.5 h-3.5" fill="none" stroke="currentColor" stroke-width="2">
        <path stroke-linecap="round" stroke-linejoin="round" d="M15 19.128a9.38 9.38 0 002.625.372 9.337 9.337 0 004.121-.952 4.125 4.125 0 00-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.106A12.318 12.318 0 018.624 21c-2.331 0-4.512-.645-6.374-1.766l-.001-.109a6.375 6.375 0 0111.964-3.07M12 6.375a3.375 3.375 0 11-6.75 0 3.375 3.375 0 016.75 0zm8.25 2.25a2.625 2.625 0 11-5.25 0 2.625 2.625 0 015.25 0z"/>
      </svg>
      <span>{venue.capacity.min} – {venue.capacity.max} {m.venues_card_guests()}</span>
    </div>

    <!-- Spacer -->
    <div class="mt-auto">
      <!-- Price + CTA -->
      <div class="flex items-end justify-between gap-3">
        <div>
          <p class="text-xs text-[var(--color-muted)]">{m.venues_card_perEvent()}</p>
          <p class="text-xl font-bold text-[var(--color-secondary)]">
            {formatCurrency(venue.pricing.basePrice)}
          </p>
        </div>

        <a
          href="/venues/{venue.slug}"
          class="
            flex-shrink-0 inline-flex items-center gap-1.5 px-4 py-2.5 rounded-xl
            bg-[var(--color-primary)] text-[var(--color-secondary)] text-sm font-semibold
            hover:bg-[var(--color-primary-dark)] transition-colors shadow-[var(--shadow-gold)]
          "
        >
          {m.venues_card_bookNow()}
          <svg viewBox="0 0 20 20" class="w-4 h-4 rtl:rotate-180" fill="currentColor">
            <path fill-rule="evenodd" d="M3 10a.75.75 0 01.75-.75h10.638L10.23 5.29a.75.75 0 111.04-1.08l5.5 5.25a.75.75 0 010 1.08l-5.5 5.25a.75.75 0 11-1.04-1.08l4.158-3.96H3.75A.75.75 0 013 10z" clip-rule="evenodd"/>
          </svg>
        </a>
      </div>
    </div>
  </div>
</article>

<style>
  .featured-card {
    border-color: #d97706 !important; /* Gold border */
    box-shadow: 0 10px 30px -10px rgba(217, 119, 6, 0.15), 0 1px 3px rgba(217, 119, 6, 0.05) !important;
    position: relative;
  }

  .featured-card::after {
    content: '';
    position: absolute;
    inset: 0;
    border-radius: 1rem;
    border: 1px solid rgba(217, 119, 6, 0.25);
    pointer-events: none;
    transition: border-color 0.3s;
  }

  .featured-card:hover::after {
    border-color: rgba(217, 119, 6, 0.55);
  }

  .featured-ribbon {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    background: linear-gradient(135deg, #d97706 0%, #b45309 100%);
    color: #ffffff;
    font-size: 0.65rem;
    font-weight: 800;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    padding: 0.25rem 0.65rem;
    border-radius: 30px;
    box-shadow: 0 4px 10px rgba(217, 119, 6, 0.25);
    border: 1px solid rgba(255, 255, 255, 0.15);
  }
</style>
