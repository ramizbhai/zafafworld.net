import type { ParamMatcher } from '@sveltejs/kit';

/**
 * Strict allowlist of known category slugs from /api/v1/public/categories.
 *
 * ⚠️  Update when new categories are added to the database.
 */
export const CATEGORY_SLUGS = new Set([
    'wedding-palace',
    'hotel-venue',
    'villa-resort',
    'restaurant-event',
    'outdoor-garden',
    'rooftop-venue',
    'private-beach',
    'chalet',
    'wedding-gown',
    'haute-couture',
    'abaya-jalabiya',
    'groom-attire',
    'hair-makeup',
    'beauty-skincare',
    'henna-art',
    'photography-video',
    'photo-studio',
    'catering',
    'wedding-cake',
    'wedding-sweets',
    'entertainment-dj',
    'zaffa',
    'nasheed-band',
    'wedding-jewelry',
    'wedding-gifts',
    'wedding-planner',
    'khosha-decor',
    'flowers-floral',
    'wedding-invitation',
    'lighting-av',
    'wedding-car',
    'male-grooming',
]);

export const match: ParamMatcher = (param) => {
    return CATEGORY_SLUGS.has(param.toLowerCase());
};
