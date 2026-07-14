import { env } from "$env/dynamic/public";
import { env as privateEnv } from '$env/dynamic/private';
import { redirect, type Handle, type HandleFetch } from '@sveltejs/kit';
import * as Sentry from '@sentry/sveltekit';

Sentry.init({
    dsn: privateEnv.SENTRY_DSN || '',
    tracesSampleRate: 1.0,
    environment: process.env.NODE_ENV === 'production' ? 'production' : 'development'
});

export const handleError = Sentry.handleErrorWithSentry();

export const handle: Handle = async ({ event, resolve }) => {
    const sessionToken = event.cookies.get('zafaf_vendor_session');
    const path = event.url.pathname;

    // Check if the route is a dashboard route
    const isDashboard = path.startsWith('/dashboard');
    // Check if the route is a public auth/gateway page
    const isAuthPage = path === '/' || path === '/login' || path === '/register';

    if (isDashboard) {
        if (!sessionToken) {
            throw redirect(303, '/login');
        }

        try {
            const response = await event.fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/auth/me`, {
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_vendor_session=${sessionToken}`
                }
            });

            if (response.ok) {
                const data = await response.json();
                if (data.status === 'success' && data.user?.role?.toLowerCase() === 'vendor') {
                    // Populate user in locals for loaders and actions
                    event.locals.user = data.user;
                } else {
                    // Invalid role or status, clean cookie and redirect
                    event.cookies.delete('zafaf_vendor_session', { path: '/' });
                    throw redirect(303, '/login');
                }
            } else if (response.status === 401 || response.status === 403) {
                // Explicitly unauthorized by backend, clear session and redirect
                event.cookies.delete('zafaf_vendor_session', { path: '/' });
                throw redirect(303, '/login');
            } else {
                // Other server error (e.g. 500) — do not grant access, redirect to login
                console.warn(`Backend returned server error status=${response.status} during session validation. Redirecting to login.`);
                event.cookies.delete('zafaf_vendor_session', { path: '/' });
                throw redirect(303, '/login');
            }
        } catch (err) {
            // Re-throw SvelteKit redirect exceptions
            if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
                throw err;
            }
            // Network failure or backend unreachable — clear session and redirect to login
            console.error('Session validation failed (backend offline/network issue):', err);
            event.cookies.delete('zafaf_vendor_session', { path: '/' });
            throw redirect(303, '/login');
        }
    } else if (isAuthPage) {
        if (sessionToken) {
            try {
                const response = await event.fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/auth/me`, {
                    headers: {
                        'Authorization': `Bearer ${sessionToken}`,
                        'Cookie': `zafaf_vendor_session=${sessionToken}`
                    }
                });

                if (response.ok) {
                    const data = await response.json();
                    if (data.status === 'success' && data.user?.role?.toLowerCase() === 'vendor') {
                        // User is already logged in, redirect them directly to the dashboard
                        event.locals.user = data.user;
                        throw redirect(303, '/dashboard');
                    }
                }
            } catch (err) {
                if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
                    throw err;
                }
                console.error('Hook session validation skipped (backend offline):', err);
            }
        }
    }

    const locale = event.cookies.get('zafaf_locale') || 'ar';
    const dir = locale === 'ar' ? 'rtl' : 'ltr';

    return resolve(event, {
        transformPageChunk: ({ html }) => {
            return html.replace('<html lang="en">', `<html lang="${locale}" dir="${dir}">`);
        }
    });
};

export const handleFetch: HandleFetch = async ({ request, fetch }) => {
    const apiUrl = env.PUBLIC_API_URL || 'https://api.zafafworld.net';
    if (request.url.startsWith(apiUrl)) {
        const originalUrl = request.url;
        
        // When rewriting to backend, explicitly preserve headers from the original request
        const newUrl = request.url.replace(apiUrl, 'http://backend:8080');
        
        const newHeaders = new Headers(request.headers);
        // Explicitly set the Host header to api.zafafworld.net if it was stripped
        // or keep the one provided. But we MUST ensure the backend sees the correct host.
        newHeaders.set('Host', 'api.zafafworld.net');
        
        request = new Request(newUrl, {
            method: request.method,
            headers: newHeaders,
            body: request.body,
            duplex: 'half'
        } as any);
    }

    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 10000);

    try {
        return await fetch(request, { signal: controller.signal });
    } catch (err: any) {
        if (err.name === 'AbortError') {
            console.error(`[vendor-portal hooks.server.ts] Fetch timeout (10s) for URL: ${request.url}`);
            return new Response('Gateway Timeout', { status: 504 });
        }
        throw err;
    } finally {
        clearTimeout(timeoutId);
    }
};
