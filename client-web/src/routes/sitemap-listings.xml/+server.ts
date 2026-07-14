import { env } from '$env/dynamic/public';
import type { RequestEvent } from '@sveltejs/kit';

const SITE_URL = 'https://zafafworld.net';

export const GET = async ({ fetch }: RequestEvent) => {
    const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';

    // Fetch dynamic listings from the backend
    let allListings: { slug: string }[] = [];
    try {
        let page = 1;
        let totalPages = 1;

        // Loop through paginated listings API to gather all slugs
        do {
            // limit=48 is the maximum allowed by backend
            const res = await fetch(`${API_BASE}/api/v1/public/listings?page=${page}&limit=48`);
            if (!res.ok) break;

            const data = await res.json();
            // Backend returns { listings: [...], totalPages: N } directly — no status wrapper
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
        // Fallback to empty array
    }

    // Generate XML entries for dynamic listings
    const listingEntries = allListings.map(listing => `
        <url>
            <loc>${SITE_URL}/ar/listings/${listing.slug}</loc>
            <xhtml:link rel="alternate" hreflang="ar" href="${SITE_URL}/ar/listings/${listing.slug}" />
            <xhtml:link rel="alternate" hreflang="en" href="${SITE_URL}/en/listings/${listing.slug}" />
            <xhtml:link rel="alternate" hreflang="x-default" href="${SITE_URL}/ar/listings/${listing.slug}" />
            <changefreq>weekly</changefreq>
            <priority>0.9</priority>
        </url>`).join('');

    const sitemap = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">
${listingEntries}
</urlset>`;

    return new Response(sitemap.trim(), {
        headers: {
            'Content-Type': 'application/xml',
            'Cache-Control': 'public, max-age=3600' // cache for 1 hour
        }
    });
};
