import { i18n } from '$lib/i18n.js';
import { sequence } from '@sveltejs/kit/hooks';
import { env } from '$env/dynamic/public';
import type { HandleFetch } from '@sveltejs/kit';

import * as runtime from '$lib/paraglide/runtime.js';

/**
 * The i18n.handle() middleware from @inlang/paraglide-sveltekit:
 *  1. Reads the PARAGLIDE_LOCALE cookie (and other configured strategies)
 *  2. Resolves the correct locale BEFORE the page is rendered
 *  3. Injects that locale into the %paraglide.lang% and %paraglide.textDirection%
 *     placeholders in app.html — so the browser receives <html lang dir> already set.
 *
 * This is what eliminates the RTL/LTR flicker. The client must NOT re-set these
 * attributes after hydration (the $effect in +layout.svelte has been removed).
 */
export const handle = sequence(
    async ({ event, resolve }) => {
        const requestUrl = new URL(event.request.url);
        const lang = requestUrl.pathname.startsWith('/en') ? 'en' : 'ar';
        const r = runtime as any;
        if (typeof r.setLanguageTag === 'function') {
            r.setLanguageTag(lang);
        } else if (typeof r.setLocale === 'function') {
            r.setLocale(lang);
        }
        return resolve(event);
    },
    i18n.handle()
);

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
            console.error(`[client-web hooks.server.ts] Fetch timeout (10s) for URL: ${request.url}`);
            return new Response('Gateway Timeout', { status: 504 });
        }
        throw err;
    } finally {
        clearTimeout(timeoutId);
    }
};
