import type { Handle, HandleFetch } from '@sveltejs/kit';
import { env } from '$env/dynamic/public';

export const handle: Handle = async ({ event, resolve }) => {
    // Determine language from cookie, defaulting to Arabic ('ar')
    const lang = event.cookies.get('zafaf_lang') || 'ar';
    const dir = lang === 'ar' ? 'rtl' : 'ltr';

    // Inject language and direction into HTML during SSR to prevent blink
    const response = await resolve(event, {
        transformPageChunk: ({ html }) => html
            .replace('%lang%', lang)
            .replace('%dir%', dir)
    });

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

    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 10000);

    try {
        return await fetch(request, { signal: controller.signal });
    } catch (err: any) {
        if (err.name === 'AbortError') {
            console.error(`[admin-panel hooks.server.ts] Fetch timeout (10s) for URL: ${request.url}`);
            return new Response('Gateway Timeout', { status: 504 });
        }
        throw err;
    } finally {
        clearTimeout(timeoutId);
    }
};
