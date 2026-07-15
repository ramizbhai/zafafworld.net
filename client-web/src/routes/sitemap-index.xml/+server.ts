import type { RequestEvent } from '@sveltejs/kit';

const SITE_URL = 'https://zafafworld.net';

export const GET = async ({}: RequestEvent) => {
    const sitemapIndex = `<?xml version="1.0" encoding="UTF-8"?>
<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
    <sitemap>
        <loc>${SITE_URL}/sitemap-static.xml</loc>
    </sitemap>
    <sitemap>
        <loc>${SITE_URL}/sitemap-listings.xml</loc>
    </sitemap>
    <sitemap>
        <loc>${SITE_URL}/sitemap-blogs.xml</loc>
    </sitemap>
</sitemapindex>`;

    return new Response(sitemapIndex.trim(), {
        headers: {
            'Content-Type': 'application/xml',
            'Cache-Control': 'public, max-age=3600' // cache for 1 hour
        }
    });
};
