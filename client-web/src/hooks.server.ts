import { i18n } from '$lib/i18n.js';
import { sequence } from '@sveltejs/kit/hooks';
import { env } from '$env/dynamic/public';
import type { HandleFetch } from '@sveltejs/kit';
import { generateSitemapXml } from '$lib/services/sitemap.js';

import * as runtime from '$lib/paraglide/runtime.js';

export const handle = async ({ event, resolve }: any) => {
    const pathname = event.url.pathname;
    console.error('[HOOKS SERVER ENTRY] pathname:', pathname);

    // Direct sitemap serving for ANY path ending in sitemap.xml (bypasses Paraglide 302 i18n redirects)
    if (pathname.endsWith('sitemap.xml')) {
        console.error('[HOOKS SERVER ENTRY] Generating sitemap XML directly for:', pathname);
        return await generateSitemapXml(event.fetch);
    }

    // 301 Redirect legacy sitemap URLs directly to unified /sitemap.xml
    if (pathname.includes('sitemap-') || pathname.includes('sitemap_')) {
        return new Response(null, {
            status: 301,
            headers: {
                'Location': '/sitemap.xml',
                'Cache-Control': 'public, max-age=3600'
            }
        });
    }

    return sequence(
        async ({ event, resolve }) => {
            const lang = pathname.startsWith('/en') ? 'en' : 'ar';
            const r = runtime as any;
            if (typeof r.setLanguageTag === 'function') {
                r.setLanguageTag(lang);
            } else if (typeof r.setLocale === 'function') {
                r.setLocale(lang);
            }
            return resolve(event);
        },
        i18n.handle()
    )({ event, resolve });
};

export const handleFetch: HandleFetch = async ({ request, fetch }) => {
    const apiUrl = env.PUBLIC_API_URL || 'https://api.zafafworld.net';
    if (request.url.startsWith(apiUrl)) {
        request = new Request(
            request.url.replace(apiUrl, 'http://backend:8080'),
            request
        );
    }

    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 4000);

    try {
        return await fetch(request, { signal: controller.signal });
    } catch (err: any) {
        if (err.name === 'AbortError') {
            console.error(`[client-web hooks.server.ts] Fetch timeout (4s) for URL: ${request.url}`);
            return new Response('Gateway Timeout', { status: 504 });
        }
        throw err;
    } finally {
        clearTimeout(timeoutId);
    }
};
