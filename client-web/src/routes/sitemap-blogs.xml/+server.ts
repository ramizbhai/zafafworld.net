import { env } from '$env/dynamic/public';
import type { RequestEvent } from '@sveltejs/kit';

const SITE_URL = 'https://zafafworld.net';

export const GET = async ({ fetch }: RequestEvent) => {
    const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';

    let allBlogs: { slug: string }[] = [];
    try {
        let page = 1;
        let hasMore = true;

        while (hasMore) {
            const res = await fetch(`${API_BASE}/api/v1/public/blogs?page=${page}&limit=50`);
            if (!res.ok) break;

            const data = await res.json();
            if (data.data && Array.isArray(data.data) && data.data.length > 0) {
                allBlogs.push(...data.data.map((b: any) => ({ slug: b.slug })));
                page++;
                // If we got fewer than 50, we reached the end
                if (data.data.length < 50) {
                    hasMore = false;
                }
            } else {
                hasMore = false;
            }
        }
    } catch (err) {
        console.error('Failed to fetch blogs for sitemap:', err);
    }

    const blogEntries = allBlogs.map(blog => `
        <url>
            <loc>${SITE_URL}/ar/discover/${blog.slug}</loc>
            <xhtml:link rel="alternate" hreflang="ar" href="${SITE_URL}/ar/discover/${blog.slug}" />
            <xhtml:link rel="alternate" hreflang="en" href="${SITE_URL}/en/discover/${blog.slug}" />
            <xhtml:link rel="alternate" hreflang="x-default" href="${SITE_URL}/ar/discover/${blog.slug}" />
            <changefreq>weekly</changefreq>
            <priority>0.8</priority>
        </url>`).join('');

    const sitemap = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">
${blogEntries}
</urlset>`;

    return new Response(sitemap.trim(), {
        headers: {
            'Content-Type': 'application/xml',
            'Cache-Control': 'public, max-age=3600'
        }
    });
};
