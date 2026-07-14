import type { RequestEvent } from '@sveltejs/kit';

const SITE_URL = 'https://zafafworld.net';

// Only include routes that actually exist as SvelteKit pages
const staticRoutes = [
    '/',
    '/listings',
    '/vendors',
    '/venues',
    '/discover',
    '/about',
    '/contact',
    '/faq',
    '/help',
    '/privacy',
    '/terms',
    '/auth/login',
    '/auth/register'
];

export const GET = async ({}: RequestEvent) => {
    // Generate XML entries for static routes
    const staticEntries = staticRoutes.map(route => `
        <url>
            <loc>${SITE_URL}/ar${route === '/' ? '' : route}</loc>
            <xhtml:link rel="alternate" hreflang="ar" href="${SITE_URL}/ar${route === '/' ? '' : route}" />
            <xhtml:link rel="alternate" hreflang="en" href="${SITE_URL}/en${route === '/' ? '' : route}" />
            <xhtml:link rel="alternate" hreflang="x-default" href="${SITE_URL}/ar${route === '/' ? '' : route}" />
            <changefreq>daily</changefreq>
            <priority>${route === '/' ? '1.0' : '0.8'}</priority>
        </url>`).join('');

    const sitemap = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">
${staticEntries}
</urlset>`;

    return new Response(sitemap.trim(), {
        headers: {
            'Content-Type': 'application/xml',
            'Cache-Control': 'public, max-age=3600' // cache for 1 hour
        }
    });
};
