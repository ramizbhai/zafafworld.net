/**
 * categoryIcons.ts — Canonical category slug → emoji icon map
 *
 * Single source of truth. Previously duplicated in:
 *  - ListingCard.svelte
 *  - +layout.server.ts
 *  - HeroSection.svelte
 *
 * Includes V2 canonical slugs + legacy aliases for backward-compat.
 */
export const CATEGORY_ICON_MAP: Record<string, string> = {
  // ── Venues ──────────────────────────────────────────────────────────────
  'wedding-palace':   '🏛️',
  'hotel-venue':      '🏨',
  'villa-resort':     '🏡',
  'restaurant-event': '🍽️',
  'outdoor-garden':   '🌿',
  'rooftop-venue':    '🌃',
  'private-beach':    '🏖️',
  'chalet':           '🏕️',
  // ── Fashion ─────────────────────────────────────────────────────────────
  'wedding-gown':     '👗',
  'haute-couture':    '✨',
  'abaya-jalabiya':   '🧕',
  'groom-attire':     '👘',
  // ── Beauty ──────────────────────────────────────────────────────────────
  'hair-makeup':      '💄',
  'beauty-skincare':  '🧴',
  'henna-art':        '🌿',
  'perfume-oud':      '🌸',
  'male-grooming':    '🪒',
  // ── Photography ─────────────────────────────────────────────────────────
  'photography-video': '📷',
  'photo-studio':      '📸',
  // ── Food ────────────────────────────────────────────────────────────────
  'catering':         '🍱',
  'wedding-cake':     '🎂',
  'wedding-sweets':   '🍬',
  // ── Entertainment ───────────────────────────────────────────────────────
  'entertainment-dj': '🎵',
  'zaffa':            '🥁',
  'nasheed-band':     '🎶',
  // ── Jewelry & Gifts ─────────────────────────────────────────────────────
  'wedding-jewelry':  '💍',
  'wedding-gifts':    '🎁',
  // ── Planning & Decor ────────────────────────────────────────────────────
  'wedding-planner':    '📋',
  'khosha-decor':       '🌺',
  'flowers-floral':     '💐',
  'wedding-invitation': '✉️',
  'lighting-av':        '💡',
  // ── Transportation ──────────────────────────────────────────────────────
  'wedding-car': '🚗',
  // ── Legacy aliases (back-compat with pre-V2 slugs) ──────────────────────
  'photographers-and-videographers':  '📸',
  'photographers_and_videographers':  '📸',
  'wedding-planning':                 '📋',
  'wedding-gowns':                    '👗',
  'hair-make-up':                     '💄',
  'wedding-cakes':                    '🎂',
  'band-dj-and-entertainment':        '🎵',
  'wedding-flowers-and-bouquets':     '💐',
  'henneh-art':                       '🌿',
  'wedding-rings-jewelry':            '💍',
  'wedding-treats-and-gifts':         '🎁',
  'wedding-invitations':              '✉️',
};

export const FALLBACK_CATEGORY_ICON = '✨';

export function getCategoryIcon(slug: string | null | undefined): string {
  if (!slug) return FALLBACK_CATEGORY_ICON;
  return CATEGORY_ICON_MAP[slug] ?? FALLBACK_CATEGORY_ICON;
}
