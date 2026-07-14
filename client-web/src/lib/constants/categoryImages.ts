export const CATEGORY_IMAGE_MAP: Record<string, string> = {
  'wedding-invitation': '/categories/wedding-invitation.webp',
  'photography-video': '/categories/photography-video.webp',
  'wedding-palace': '/categories/wedding-palace.webp',
  'hair-makeup': '/categories/hair-makeup.webp',
  'henna-art': '/categories/henna-art.webp',
  'wedding-cake': '/categories/wedding-cake.webp',
  'khosha-decor': '/categories/khosha-decor.webp',
  'wedding-gown': '/categories/wedding-gown.webp',
  'hotel-venue': '/categories/hotel-venue.webp',
  'villa-resort': '/categories/villa-resort.webp',
  'restaurant-event': '/categories/restaurant-event.webp',
  'outdoor-garden': '/categories/outdoor-garden.webp',
  'rooftop-venue': '/categories/rooftop-venue.webp',
  'private-beach': '/categories/private-beach.webp',
  'chalet': '/categories/chalet.webp',
  'haute-couture': '/categories/haute-couture.webp',
  'abaya-jalabiya': '/categories/abaya-jalabiya.webp',
  'groom-attire': '/categories/groom-attire.webp',
  'beauty-skincare': '/categories/beauty-skincare.webp',
  'male-grooming': '/categories/male-grooming.webp',
  'photo-studio': '/categories/photo-studio.webp',
  'catering': '/categories/catering.webp',
  'wedding-sweets': '/categories/wedding-sweets.webp',
  'entertainment-dj': '/categories/entertainment-dj.webp',
  'zaffa': '/categories/zaffa.webp',
  'nasheed-band': '/categories/nasheed-band.webp',
  'wedding-jewelry': '/categories/wedding-jewelry.webp',
  'wedding-gifts': '/categories/wedding-gifts.webp',
  'wedding-planner': '/categories/wedding-planner.webp',
  'flowers-floral': '/categories/flowers-floral.webp',
  'lighting-av': '/categories/lighting-av.webp',
  'wedding-car': '/categories/wedding-car.webp'
};

export const FALLBACK_CATEGORY_IMAGE = '/categories/wedding-palace.webp';

export function getCategoryImageUrl(key: string | null | undefined): string {
  if (!key) return FALLBACK_CATEGORY_IMAGE;
  return CATEGORY_IMAGE_MAP[key] || FALLBACK_CATEGORY_IMAGE;
}
