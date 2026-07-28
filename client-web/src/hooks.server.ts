import { i18n } from '$lib/i18n.js';
import { sequence } from '@sveltejs/kit/hooks';
import { env } from '$env/dynamic/public';
import type { HandleFetch } from '@sveltejs/kit';
import { generateSitemapXml } from '$lib/services/sitemap.js';
import { requestContextStore } from '$lib/utils/requestContext.js';
import http from 'node:http';

// Global interception of Node http responses to apply optimal Cache-Control for static assets
const originalWriteHead = http.ServerResponse.prototype.writeHead;
(http.ServerResponse.prototype as any).writeHead = function (
    this: http.ServerResponse,
    statusCode: number,
    ...args: any[]
) {
    const req = this.req;
    if (req && req.url) {
        const cleanUrl = req.url.split('?')[0];
        if (
            cleanUrl.endsWith('.webp') ||
            cleanUrl.startsWith('/categories/') ||
            cleanUrl.startsWith('/images/')
        ) {
            this.setHeader('Cache-Control', 'public, max-age=31536000, immutable');
            this.removeHeader('expires');
            this.removeHeader('pragma');
        }
    }
    return (originalWriteHead as any).apply(this, [statusCode, ...args]);
};

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

    const country = event.cookies.get('zafaf_selected_country') || 'SA';

    const response = await requestContextStore.run({ countryCode: country.toUpperCase() }, () => {
        return sequence(
            i18n.handle()
        )({ event, resolve });
    });

    response.headers.set('X-Content-Type-Options', 'nosniff');
    response.headers.set('X-Frame-Options', 'DENY');
    response.headers.set('Referrer-Policy', 'strict-origin-when-cross-origin');
    response.headers.set('Permissions-Policy', 'geolocation=(), camera=(), microphone=()');
    response.headers.set('Strict-Transport-Security', 'max-age=31536000; includeSubDomains; preload');

    return response;
};

export const handleFetch: HandleFetch = async ({ request, fetch }) => {
    const apiUrl = env.PUBLIC_API_URL || 'https://api.zafafworld.net';
    if (request.url.startsWith(apiUrl)) {
        request = new Request(
            request.url.replace(apiUrl, 'http://backend:8080'),
            request
        );
    }

    const context = requestContextStore.getStore();
    if (context?.countryCode) {
        request.headers.set('X-Country-ID', context.countryCode);
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
