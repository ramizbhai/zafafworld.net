import { env } from '$env/dynamic/public';
import type { RequestEvent } from '@sveltejs/kit';
import { CITY_SLUGS } from '$params/city.js';
import { CATEGORY_SLUGS } from '$params/category.js';

const SITE_URL = 'https://zafafworld.net';

/**
 * Generates two types of sitemap entries:
 *
 * 1. Individual listing detail pages  (/ar/listings/{slug})
 *    — unchanged, one per active listing.
 *
 * 2. Listings search/filter pages  (/ar/{city}/{category}/)
 *    — Only city × category combinations that have at least one active listing.
 *    — Only the default sort ("weighted") — other sort variants use a canonical
 *      tag pointing here, so they are NOT separately indexed.
 *    — Generated for both /ar/ and /en/ language prefixes.
 */
export const GET = async ({ fetch }: RequestEvent) => {
    const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';

    // ── 1. Individual listing slugs ──────────────────────────────────────────
    let allListings: { slug: string }[] = [];
    try {
        let page = 1;
        let totalPages = 1;
        do {
            const res = await fetch(`${API_BASE}/api/v1/public/listings?page=${page}&limit=48`);
            if (!res.ok) break;
            const data = await res.json();
            if (data.listings && Array.isArray(data.listings)) {
                allListings.push(...data.listings.map((l: any) => ({ slug: l.slug })));
                totalPages = data.totalPages || 1;
                page++;
            } else {
                break;
            }
        } while (page <= totalPages);
    } catch (err) {
        console.error('Failed to fetch listings for sitemap:', err);
    }

    const listingEntries = allListings.map(listing => `
        <url>
            <loc>${SITE_URL}/ar/listings/${listing.slug}</loc>
            <xhtml:link rel="alternate" hreflang="ar" href="${SITE_URL}/ar/listings/${listing.slug}" />
            <xhtml:link rel="alternate" hreflang="en" href="${SITE_URL}/en/listings/${listing.slug}" />
            <xhtml:link rel="alternate" hreflang="x-default" href="${SITE_URL}/ar/listings/${listing.slug}" />
            <changefreq>weekly</changefreq>
            <priority>0.9</priority>
        </url>`).join('');

    // ── 2. City × Category search page combinations ──────────────────────────
    // Fetch summary data from the backend to find which combinations have listings.
    // We use the aggregations endpoint if available, otherwise fall back to sampling.
    let cityCategoCombos: Array<{ city: string; category: string }> = [];
    try {
        // Fetch one page per city+category combination to check if results > 0.
        // This is intentionally parallel but batched to avoid overwhelming the API.
        const cities     = Array.from(CITY_SLUGS);
        const categories = Array.from(CATEGORY_SLUGS);

        // Build tasks for all combinations, run in parallel batches of 10
        const allTasks: Array<() => Promise<void>> = [];

        for (const city of cities) {
            for (const cat of categories) {
                allTasks.push(async () => {
                    try {
                        const res = await fetch(
                            `${API_BASE}/api/v1/public/listings?city=${encodeURIComponent(city)}&category=${encodeURIComponent(cat)}&page=1&limit=1`
                        );
                        if (!res.ok) return;
                        const data = await res.json();
                        const total = data.total ?? (data.listings?.length ?? 0);
                        if (total > 0) {
                            cityCategoCombos.push({ city, category: cat });
                        }
                    } catch {
                        // Skip this combination on error
                    }
                });
            }
        }

        // Run in batches of 10 to avoid rate limiting
        const BATCH_SIZE = 10;
        for (let i = 0; i < allTasks.length; i += BATCH_SIZE) {
            await Promise.all(allTasks.slice(i, i + BATCH_SIZE).map(t => t()));
        }
    } catch (err) {
        console.error('Failed to generate city×category sitemap combos:', err);
    }

    const langs = ['ar', 'en'];
    const searchPageEntries = cityCategoCombos.flatMap(({ city, category }) =>
        langs.map(lang => {
            const loc  = `${SITE_URL}/${lang}/search/${category}/${city}/`;
            const locAr = `${SITE_URL}/ar/search/${category}/${city}/`;
            const locEn = `${SITE_URL}/en/search/${category}/${city}/`;
            return `
        <url>
            <loc>${loc}</loc>
            <xhtml:link rel="alternate" hreflang="ar" href="${locAr}" />
            <xhtml:link rel="alternate" hreflang="en" href="${locEn}" />
            <xhtml:link rel="alternate" hreflang="x-default" href="${locAr}" />
            <changefreq>daily</changefreq>
            <priority>0.7</priority>
        </url>`;
        })
    ).join('');

    const sitemap = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">
${listingEntries}
${searchPageEntries}
</urlset>`;

    return new Response(sitemap.trim(), {
        headers: {
            'Content-Type': 'application/xml',
            'Cache-Control': 'public, max-age=3600' // cache for 1 hour
        }
    });
};
