import { env } from '$env/dynamic/public';

const SITE_URL = 'https://zafafworld.net';

// Explicit Whitelist of public static routes (excludes /auth/*, /account/*, /vendor/*, /admin/*)
const STATIC_ROUTES = [
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
    '/terms'
];

interface ListingItem {
    slug: string;
    updated_at?: string;
}

interface BlogItem {
    slug: string;
    updated_at?: string;
}

interface ComboItem {
    city: string;
    category: string;
    updated_at?: string;
}

interface SitemapBackendData {
    listings: ListingItem[];
    blogs: BlogItem[];
    combos: ComboItem[];
}

function formatDate(isoStr?: string): string {
    if (!isoStr) return new Date().toISOString().split('T')[0];
    try {
        return new Date(isoStr).toISOString().split('T')[0];
    } catch {
        return new Date().toISOString().split('T')[0];
    }
}

export async function generateSitemapXml(customFetch: any): Promise<Response> {
    const API_BASE = env.PUBLIC_API_URL || 'http://backend:8080';
    let sitemapData: SitemapBackendData = { listings: [], blogs: [], combos: [] };

    try {
        const fetchFn = customFetch || fetch;
        const res = await fetchFn(`${API_BASE}/api/v1/public/sitemap-data`, {
            headers: { 'Accept': 'application/json' }
        });
        if (res.ok) {
            const json = await res.json();
            if (json.status === 'success' && json.data) {
                sitemapData = json.data;
            }
        }
    } catch (err) {
        console.error('[Sitemap Service] Failed to fetch sitemap data from backend:', err);
    }

    const todayStr = new Date().toISOString().split('T')[0];

    // 1. Whitelisted Static Page Entries
    const staticEntries = STATIC_ROUTES.map(route => {
        const pathAr = route === '/' ? '/ar' : `/ar${route}`;
        const pathEn = route === '/' ? '/en' : `/en${route}`;
        const priority = route === '/' ? '1.0' : '0.8';

        return `  <url>
    <loc>${SITE_URL}${pathAr}</loc>
    <lastmod>${todayStr}</lastmod>
    <changefreq>daily</changefreq>
    <priority>${priority}</priority>
    <xhtml:link rel="alternate" hreflang="ar" href="${SITE_URL}${pathAr}" />
    <xhtml:link rel="alternate" hreflang="en" href="${SITE_URL}${pathEn}" />
    <xhtml:link rel="alternate" hreflang="x-default" href="${SITE_URL}${pathAr}" />
  </url>`;
    }).join('\n');

    // 2. Active Listing Entries
    const listingEntries = (sitemapData.listings || []).map(item => {
        const lastmod = formatDate(item.updated_at);
        const locAr = `${SITE_URL}/ar/listings/${item.slug}`;
        const locEn = `${SITE_URL}/en/listings/${item.slug}`;

        return `  <url>
    <loc>${locAr}</loc>
    <lastmod>${lastmod}</lastmod>
    <changefreq>weekly</changefreq>
    <priority>0.9</priority>
    <xhtml:link rel="alternate" hreflang="ar" href="${locAr}" />
    <xhtml:link rel="alternate" hreflang="en" href="${locEn}" />
    <xhtml:link rel="alternate" hreflang="x-default" href="${locAr}" />
  </url>`;
    }).join('\n');

    // 3. Published Blog Entries
    const blogEntries = (sitemapData.blogs || []).map(item => {
        const lastmod = formatDate(item.updated_at);
        const locAr = `${SITE_URL}/ar/discover/${item.slug}`;
        const locEn = `${SITE_URL}/en/discover/${item.slug}`;

        return `  <url>
    <loc>${locAr}</loc>
    <lastmod>${lastmod}</lastmod>
    <changefreq>weekly</changefreq>
    <priority>0.8</priority>
    <xhtml:link rel="alternate" hreflang="ar" href="${locAr}" />
    <xhtml:link rel="alternate" hreflang="en" href="${locEn}" />
    <xhtml:link rel="alternate" hreflang="x-default" href="${locAr}" />
  </url>`;
    }).join('\n');

    // 4. City x Category Search Combos (only active combinations)
    const comboEntries = (sitemapData.combos || []).map(item => {
        const lastmod = formatDate(item.updated_at);
        const locAr = `${SITE_URL}/ar/search/${item.category}/${item.city}/`;
        const locEn = `${SITE_URL}/en/search/${item.category}/${item.city}/`;

        return `  <url>
    <loc>${locAr}</loc>
    <lastmod>${lastmod}</lastmod>
    <changefreq>daily</changefreq>
    <priority>0.7</priority>
    <xhtml:link rel="alternate" hreflang="ar" href="${locAr}" />
    <xhtml:link rel="alternate" hreflang="en" href="${locEn}" />
    <xhtml:link rel="alternate" hreflang="x-default" href="${locAr}" />
  </url>`;
    }).join('\n');

    const xml = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">
${staticEntries}
${listingEntries}
${blogEntries}
${comboEntries}
</urlset>`;

    return new Response(xml.trim(), {
        headers: {
            'Content-Type': 'application/xml; charset=utf-8',
            'Cache-Control': 'public, max-age=3600'
        }
    });
}
