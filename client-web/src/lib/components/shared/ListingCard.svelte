<script lang="ts">
    import type { Listing } from "$lib/types/index.js";
    import * as m from "$lib/paraglide/messages.js";
    import { getLocale } from "$lib/paraglide/runtime.js";
    import {
        getLocalizedField,
        formatCurrency,
        formatNumber,
        formatDate,
    } from "$lib/utils/localize.js";
    import { resolveMediaUrl, getOptimizedImage } from "$lib/shared/utils/media.js";
    import { getCategoryIcon } from "$lib/constants/categoryIcons.js";

    interface Props {
        listing: Listing;
        layout?: "grid" | "list";
        aspectRatio?: "landscape" | "portrait" | "square";
        size?: "sm" | "md" | "lg";
        class?: string;
        promotion?: any;
    }

    let {
        listing,
        layout = "grid",
        aspectRatio = "landscape",
        size = "md",
        class: extraClass = "",
        promotion = null,
    }: Props = $props();

    let isWishlisted = $state(false);
    let imgError = $state(false);
    // Fallback URL: if the sized variant (e.g. _card.webp) 404s, try the original cover image
    let imgFallbackSrc = $derived(
        listing.coverImage && !listing.coverImage.includes('_card')
            ? resolveMediaUrl(listing.coverImage)
            : ''
    );

    // ── Derived display values ────────────────────────────────────────────────
    const name = $derived(getLocalizedField(listing, "title", getLocale()));
    const vendorName = $derived(
        getLocalizedField(listing.vendor, "name", getLocale()),
    );
    const city = $derived(
        getLocale() === "ar" ? (listing.cityAr ?? "") : (listing.cityEn ?? ""),
    );

    const badgeText = $derived(() => {
        if (!promotion) return null;
        return getLocale() === "ar"
            ? (promotion.badge_text_ar || promotion.badge_text_en || null)
            : (promotion.badge_text_en || promotion.badge_text_ar || null);
    });

    const price = $derived(() => {
        const raw =
            listing.startingPrice ??
            (listing.basePriceSar ? parseFloat(listing.basePriceSar) : null);
        if (!raw) return null;
        if (promotion && promotion.promo_type === 'discount') {
            if (promotion.discount_type === 'percentage' && promotion.discount_percentage) {
                const discounted = raw * (1 - promotion.discount_percentage / 100);
                return formatCurrency(discounted);
            } else if (promotion.discount_type === 'fixed_amount' && promotion.discount_fixed_amount) {
                const discounted = Math.max(0, raw - parseFloat(promotion.discount_fixed_amount));
                return formatCurrency(discounted);
            }
        }
        return formatCurrency(raw);
    });

    const slashedPrice = $derived(() => {
        if (promotion && promotion.promo_type === 'discount') {
            if ((promotion.discount_type === 'percentage' && promotion.discount_percentage) || 
                (promotion.discount_type === 'fixed_amount' && promotion.discount_fixed_amount)) {
                const raw =
                    listing.startingPrice ??
                    (listing.basePriceSar ? parseFloat(listing.basePriceSar) : null);
                if (raw) return formatCurrency(raw);
            }
        }
        return null;
    });

    /** Gender section label */
    const genderLabel = $derived(() => {
        const genderSec =
            listing.genderSection ??
            listing.attributes?.genderSection ??
            listing.attributes?.gender_section;
        if (!genderSec) return null;
        const map: Record<string, { ar: string; en: string }> = {
            women_only: { ar: "نساء فقط", en: "Ladies Only" },
            men_only: { ar: "رجال فقط", en: "Gents Only" },
            mixed: { ar: "مختلط", en: "Mixed" },
            dual_parallel: { ar: "قاعتان منفصلتان", en: "Dual Halls" },
            family: { ar: "عائلي", en: "Family" },
        };
        const entry = map[genderSec];
        return entry ? getLocalizedField(entry, "", getLocale()) : null;
    });

    /** Category icon — uses shared canonical map from $lib/constants/categoryIcons.ts */
    const categoryIcon = $derived(() => getCategoryIcon(listing.category));

    const capacityText = $derived(() => {
        const menCap =
            listing.attributes?.menCapacity ?? listing.attributes?.men_capacity;
        const womenCap =
            listing.attributes?.womenCapacity ??
            listing.attributes?.women_capacity;
        const genderSec =
            listing.genderSection ??
            listing.attributes?.genderSection ??
            listing.attributes?.gender_section;

        if (genderSec === "dual_parallel" && menCap && womenCap) {
            const menLabel = m.auto_gents();
            const womenLabel = m.auto_ladies();
            return `${menCap} ${menLabel} + ${womenCap} ${womenLabel}`;
        }
        const cap = menCap ?? womenCap ?? (menCap || 0) + (womenCap || 0);
        if (!cap) return null;
        const guestWord = m.auto_guests();
        return `${cap.toLocaleString()} ${guestWord}`;
    });
</script>

<!-- ═══════════════════════════════════════════════════════════════════════════
  ListingCard — Premium listing card for the listing-centric catalog.
  Displays listing-specific images, price, capacity, and gender section.
  Fully RTL-safe with bilingual support.
══════════════════════════════════════════════════════════════════════════════ -->
<article
    class="
    listing-card group relative
    listing-card--{size}
    {layout === 'list' ? 'flex flex-col sm:flex-row' : 'flex flex-col'}
    {listing.isFeatured ? 'listing-card--featured' : ''}
    {extraClass}
  "
    aria-label={name}
>
    <!-- ── Image ─────────────────────────────────────────────────────────────── -->
    <div
        class="
    listing-card__image relative overflow-hidden
    {layout === 'list'
            ? 'w-full sm:w-56 aspect-[4/3] sm:aspect-auto flex-shrink-0'
            : aspectRatio === 'portrait'
              ? 'aspect-[3/4] w-full'
              : aspectRatio === 'square'
                ? 'aspect-square w-full'
                : 'aspect-[4/3] w-full'}
  "
    >
        {#if listing.coverImage && !imgError}
            <img
                src={resolveMediaUrl(getOptimizedImage(listing.coverImage, 'card'))}
                alt={name}
                loading="lazy"
                width="400"
                height="300"
                onerror={(e) => {
                    // Two-stage fallback: sized variant 404 → original URL → emoji placeholder
                    const img = e.currentTarget as HTMLImageElement;
                    if (imgFallbackSrc && img.src !== imgFallbackSrc) {
                        img.src = imgFallbackSrc;
                    } else {
                        imgError = true;
                    }
                }}
                class="w-full h-full object-cover transition-transform duration-700 group-hover:scale-105"
            />
        {:else}
            <div
                class="w-full h-full flex flex-col items-center justify-center bg-gradient-to-br from-[var(--color-surface-alt)] to-[var(--color-border)] gap-2"
            >
                <span class={size === "sm" ? "text-2xl" : "text-4xl"}
                    >{categoryIcon()}</span
                >
                <span
                    class="text-[10px] text-[var(--color-muted)] font-medium max-w-[90%] truncate"
                    >{name}</span
                >
            </div>
        {/if}

        <!-- Gradient overlay for text legibility on hover -->
        <div
            class="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300 pointer-events-none"
            aria-hidden="true"
        ></div>

        <!-- Top-left badges -->
        <div
            class="absolute top-2 start-2 sm:top-3 sm:start-3 flex flex-col gap-1 z-10"
        >
            {#if promotion}
                <div
                    class="promo-ribbon {size === 'sm'
                        ? 'text-[9px] px-1.5 py-0.5'
                        : 'text-[10px] sm:text-xs'}"
                >
                    🏷️ {badgeText() || (
                        promotion.promo_type === 'discount'
                            ? (promotion.discount_type === 'percentage'
                                ? `${promotion.discount_percentage}% OFF`
                                : `${Number(promotion.discount_fixed_amount).toLocaleString()} SAR OFF`)
                            : (getLocale() === 'ar' ? 'عرض ميزة مضافة' : 'Added Value')
                    )}
                </div>
            {/if}
            {#if listing.isFeatured}
                <div
                    class="featured-ribbon {size === 'sm'
                        ? 'text-[9px] px-1.5 py-0.5'
                        : 'text-[10px] sm:text-xs'}"
                >
                    <svg
                        viewBox="0 0 20 20"
                        class="w-2.5 h-2.5 fill-current"
                        aria-hidden="true"
                    >
                        <path
                            fill-rule="evenodd"
                            d="M10.868 2.884c-.321-.772-1.415-.772-1.736 0l-1.83 4.401-4.753.381c-.833.067-1.171 1.107-.536 1.651l3.6 3.1-1.096 4.637c-.192.813.687 1.45 1.393 1.006l4.053-2.54 4.053 2.54c.706.444 1.585-.193 1.393-1.006l-1.096-4.637 3.6-3.1c.635-.544.297-1.584-.536-1.65l-4.752-.382-1.831-4.401z"
                            clip-rule="evenodd"
                        />
                    </svg>
                    {#if size !== "sm"}
                        <span>{m.auto_featured()}</span>
                    {/if}
                </div>
            {/if}

            {#if genderLabel()}
                <div
                    class="gender-badge {size === 'sm'
                        ? 'text-[9px] px-1.5 py-0.5'
                        : 'text-[10px] sm:text-xs'}"
                >
                    <span>{genderLabel()}</span>
                </div>
            {/if}

            {#if !listing.isAvailable}
                <div
                    class="unavailable-badge {size === 'sm'
                        ? 'text-[9px] px-1.5 py-0.5'
                        : 'text-[10px] sm:text-xs'}"
                >
                    {m.auto_unavailable()}
                </div>
            {/if}
        </div>

        <!-- Wishlist button -->
        <button
            onclick={() => {
                isWishlisted = !isWishlisted;
            }}
            class="
        absolute top-2 end-2 sm:top-3 sm:end-3 z-10
        {size === 'sm' ? 'w-7.5 h-7.5' : 'w-9 h-9'} rounded-full
        bg-white/90 backdrop-blur-sm
        flex items-center justify-center
        shadow-sm transition-all duration-200 hover:scale-110
      "
            style={size === "sm" ? "width: 1.875rem; height: 1.875rem;" : ""}
            aria-label={m.auto_add_to_wishlist()}
            aria-pressed={isWishlisted}
        >
            <svg
                viewBox="0 0 24 24"
                class="{size === 'sm' ? 'w-4 h-4' : 'w-5 h-5'} {isWishlisted
                    ? 'fill-rose-500 stroke-none'
                    : 'fill-none stroke-[var(--color-muted)] stroke-2'}"
            >
                <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    d="M21 8.25c0-2.485-2.099-4.5-4.688-4.5-1.935 0-3.597 1.126-4.312 2.733-.715-1.607-2.377-2.733-4.313-2.733C5.1 3.75 3 5.765 3 8.25c0 7.22 9 12 9 12s9-4.78 9-12z"
                />
            </svg>
        </button>

        <!-- Image count badge -->
        {#if listing.imageCount > 1}
            <div
                class="absolute bottom-2 end-2 sm:bottom-3 sm:end-3 z-10 flex items-center gap-1 bg-black/50 backdrop-blur-sm rounded-full px-2 py-0.5 text-[10px] text-white"
            >
                <svg
                    viewBox="0 0 24 24"
                    class="w-2.5 h-2.5"
                    fill="currentColor"
                >
                    <path
                        d="M2.25 15.75l5.159-5.159a2.25 2.25 0 013.182 0l5.159 5.159m-1.5-1.5l1.409-1.409a2.25 2.25 0 013.182 0l2.909 2.909"
                    />
                </svg>
                {listing.imageCount}
            </div>
        {/if}
    </div>

    <!-- ── Content ────────────────────────────────────────────────────────────── -->
    <div class="flex flex-col flex-1 {size === 'sm' ? 'p-3.5' : 'p-5'}">
        <!-- Vendor brand + city -->
        <div
            class="flex items-center justify-between {size === 'sm'
                ? 'mb-1 gap-1.5'
                : 'mb-2 gap-2'}"
        >
            <span
                class="text-xs font-medium text-[var(--color-primary)] truncate max-w-[65%]"
                >{vendorName}</span
            >
            {#if city}
                <div
                    class="flex items-center gap-1 text-[10px] sm:text-xs text-[var(--color-muted)] shrink-0"
                >
                    <svg
                        viewBox="0 0 24 24"
                        class="w-3 h-3 flex-shrink-0"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M15 10.5a3 3 0 11-6 0 3 3 0 016 0z"
                        />
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M19.5 10.5c0 7.142-7.5 11.25-7.5 11.25S4.5 17.642 4.5 10.5a7.5 7.5 0 1115 0z"
                        />
                    </svg>
                    <span>{city}</span>
                </div>
            {/if}
        </div>

        <!-- Listing name -->
        <h3
            class="font-display {size === 'sm'
                ? 'text-[0.92rem] mb-1.5 line-clamp-1'
                : 'text-[1.05rem] mb-2.5 line-clamp-2'} font-bold text-[var(--color-secondary)] leading-snug"
        >
            <a
                href={listing.detailUrl}
                class="before:absolute before:inset-0 hover:text-[var(--color-primary)] transition-colors focus-visible:outline-none focus-visible:underline"
            >
                {name}
            </a>
        </h3>

        <!-- Rating -->
        <div
            class="flex items-center {size === 'sm'
                ? 'gap-1 mb-2'
                : 'gap-1.5 mb-3'} relative z-10"
        >
            {#each Array(5) as _, i}
                <svg
                    viewBox="0 0 20 20"
                    class="{size === 'sm' ? 'w-3 h-3' : 'w-3.5 h-3.5'} {i <
                    Math.round(listing.rating.overall)
                        ? 'text-amber-400'
                        : 'text-gray-200'}"
                    fill="currentColor"
                >
                    <path
                        d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z"
                    />
                </svg>
            {/each}
            <span class="text-xs font-semibold text-[var(--color-secondary)]"
                >{listing.rating.overall.toFixed(1)}</span
            >
            {#if listing.rating.count > 0 && size !== "sm"}
                <span class="text-xs text-[var(--color-muted)]"
                    >({listing.rating.count})</span
                >
            {/if}
        </div>

        <!-- Capacity chip (when available) -->
        {#if capacityText() && size !== "sm"}
            <div
                class="flex items-center gap-1.5 text-xs text-[var(--color-muted)] mb-4 relative z-10"
            >
                <svg
                    viewBox="0 0 24 24"
                    class="w-3.5 h-3.5 flex-shrink-0"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        d="M15 19.128a9.38 9.38 0 002.625.372 9.337 9.337 0 004.121-.952 4.125 4.125 0 00-7.533-2.493M15 19.128v-.003c0-1.113-.285-2.16-.786-3.07M15 19.128v.106A12.318 12.318 0 018.624 21c-2.331 0-4.512-.645-6.374-1.766l-.001-.109a6.375 6.375 0 0111.964-3.07M12 6.375a3.375 3.375 0 11-6.75 0 3.375 3.375 0 016.75 0zm8.25 2.25a2.625 2.625 0 11-5.25 0 2.625 2.625 0 015.25 0z"
                    />
                </svg>
                <span>{capacityText()}</span>
            </div>
        {/if}

        <!-- Spacer + CTA row -->
        <div class="mt-auto pt-2">
            <div
                class="flex items-center justify-between gap-2.5 flex-wrap sm:flex-nowrap"
            >
                <!-- Price -->
                <div>
                    {#if price()}
                        <p class="text-[10px] text-[var(--color-muted)] mb-0.5">
                            {m.auto_starting()}
                        </p>
                        <div class="flex items-baseline gap-2 flex-wrap">
                            <p
                                class="{size === 'sm'
                                    ? 'text-sm'
                                    : 'text-lg sm:text-xl'} font-bold text-[var(--color-secondary)] leading-tight"
                            >
                                {price()}
                            </p>
                            {#if slashedPrice()}
                                <p class="text-xs text-red-500 line-through font-medium">
                                    {slashedPrice()}
                                </p>
                            {/if}
                        </div>
                    {:else}
                        <p class="text-xs text-[var(--color-muted)] italic">
                            {m.auto_on_request()}
                        </p>
                    {/if}
                </div>

                <!-- Contact Vendor CTA -->
                <a
                    href={listing.detailUrl}
                    class="
            flex-shrink-0 inline-flex items-center gap-1
            {size === 'sm'
                        ? 'px-2.5 py-1.5 rounded-lg text-[10px]'
                        : 'px-3.5 py-2 sm:px-4 sm:py-2.5 rounded-xl text-xs sm:text-sm'}
            bg-[var(--color-primary)] text-[var(--color-secondary)]
            font-bold
            hover:bg-[var(--color-primary-dark)]
            transition-all duration-200
            shadow-[0_4px_12px_rgba(217,119,6,0.25)]
            hover:shadow-[0_6px_18px_rgba(217,119,6,0.4)]
            hover:-translate-y-0.5
          "
                >
                    <span
                        >{size === "sm"
                            ? m.auto_view()
                            : m.auto_contact_vendor()}</span
                    >
                    <svg
                        viewBox="0 0 20 20"
                        class="{size === 'sm'
                            ? 'w-3 h-3'
                            : 'w-3.5 h-3.5 sm:w-4 sm:h-4'} rtl:rotate-180"
                        fill="currentColor"
                    >
                        <path
                            fill-rule="evenodd"
                            d="M3 10a.75.75 0 01.75-.75h10.638L10.23 5.29a.75.75 0 111.04-1.08l5.5 5.25a.75.75 0 010 1.08l-5.5 5.25a.75.75 0 11-1.04-1.08l4.158-3.96H3.75A.75.75 0 013 10z"
                            clip-rule="evenodd"
                        />
                    </svg>
                </a>
            </div>
        </div>
    </div>
</article>

<style>
    /* ── Base card ──────────────────────────────────────────────────────────── */
    .listing-card {
        border-radius: 1rem;
        overflow: hidden;
        background: #ffffff;
        border: 1px solid var(--color-border);
        transition:
            box-shadow 0.25s ease,
            transform 0.25s ease;
    }
    .listing-card:hover {
        box-shadow:
            0 12px 40px -12px rgba(0, 0, 0, 0.15),
            0 2px 8px rgba(0, 0, 0, 0.06);
        transform: translateY(-2px);
    }

    /* ── Size variations ────────────────────────────────────────────────────── */
    .listing-card--sm {
        border-radius: 0.75rem;
    }
    .listing-card--sm .featured-ribbon {
        padding: 0.15rem 0.45rem;
    }
    .listing-card--sm .gender-badge {
        padding: 0.12rem 0.4rem;
    }

    /* ── Featured variant ────────────────────────────────────────────────────── */
    .listing-card--featured {
        border-color: #d97706;
        box-shadow:
            0 0 0 1px rgba(217, 119, 6, 0.2),
            0 8px 30px -8px rgba(217, 119, 6, 0.15);
        position: relative;
    }
    .listing-card--featured::after {
        content: "";
        position: absolute;
        inset: 0;
        border-radius: 1rem;
        border: 1px solid rgba(217, 119, 6, 0.3);
        pointer-events: none;
        transition: border-color 0.3s;
    }
    .listing-card--featured.listing-card--sm::after {
        border-radius: 0.75rem;
    }
    .listing-card--featured:hover::after {
        border-color: rgba(217, 119, 6, 0.6);
    }

    /* ── Badges ──────────────────────────────────────────────────────────────── */
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
        box-shadow: 0 4px 10px rgba(217, 119, 6, 0.3);
        border: 1px solid rgba(255, 255, 255, 0.2);
    }
    .gender-badge {
        display: inline-flex;
        align-items: center;
        background: rgba(15, 23, 42, 0.7);
        backdrop-filter: blur(8px);
        -webkit-backdrop-filter: blur(8px);
        color: #ffffff;
        font-size: 0.65rem;
        font-weight: 600;
        padding: 0.2rem 0.55rem;
        border-radius: 30px;
        border: 1px solid rgba(255, 255, 255, 0.15);
    }
    .unavailable-badge {
        display: inline-flex;
        align-items: center;
        background: rgba(220, 38, 38, 0.85);
        backdrop-filter: blur(6px);
        color: #fff;
        font-size: 0.65rem;
        font-weight: 700;
        padding: 0.2rem 0.55rem;
        border-radius: 30px;
    }
    .promo-ribbon {
        display: inline-flex;
        align-items: center;
        gap: 0.25rem;
        background: linear-gradient(135deg, hsl(162, 72%, 36%) 0%, hsl(162, 72%, 26%) 100%);
        color: #ffffff;
        font-size: 0.65rem;
        font-weight: 800;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        padding: 0.25rem 0.65rem;
        border-radius: 30px;
        box-shadow: 0 4px 10px rgba(0, 166, 120, 0.3);
        border: 1px solid rgba(255, 255, 255, 0.2);
    }
</style>
