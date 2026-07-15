import { env } from "$env/dynamic/public";
import { fail, redirect, error } from '@sveltejs/kit';
import { z } from 'zod';
import type { PageServerLoad, Actions } from './$types';

const loginSchema = z.object({
    email: z.string().email('Invalid email address format.'),
    password: z.string().min(8, 'Password must be at least 8 characters long.')
});

export const load: PageServerLoad = async ({ cookies, fetch }) => {
    const sessionToken = cookies.get('zafaf_admin_session');
    if (sessionToken) {
        try {
            // Check if active session is already a valid admin
            const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/auth/me`, {
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`
                }
            });

            if (!response.ok) {
                if (response.status !== 401 && response.status !== 403 && response.status !== 404) {
                    // 5xx / network error: Throw 503
                    throw error(503, 'Service temporarily unavailable');
                }
                // Clear the stale/invalid session cookie and let it fall through to login page
                cookies.delete('zafaf_admin_session', { path: '/' });
            } else {
                const data = await response.json();
                if (data.status === 'success' && data.user?.role?.toLowerCase() === 'admin') {
                    throw redirect(303, '/dashboard');
                }
            }
        } catch (err) {
            // If redirect or error is thrown by SvelteKit, rethrow it
            if (err && typeof err === 'object' && 'status' in err) {
                throw err;
            }
            console.error('Session pre-check connection anomaly:', err);
            // Network failure / backend unreachable: Throw 503
            throw error(503, 'Service temporarily unavailable');
        }
    }
};

export const actions: Actions = {
    default: async ({ request, cookies, fetch, url }) => {
        // CSRF Verification logging
        console.log('--- ADMIN LOGIN ACTION HEADERS ---');
        request.headers.forEach((value, key) => {
            console.log(`${key}: ${value}`);
        });
        console.log('---------------------------------');

        const formData = await request.formData();
        const data = Object.fromEntries(formData);
        
        const parseResult = loginSchema.safeParse(data);
        if (!parseResult.success) {
            return fail(400, {
                error: parseResult.error.issues[0].message,
                values: { email: data.email?.toString() }
            });
        }
        
        const { email, password } = parseResult.data;

        try {
            const apiResponse = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/auth/login`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ email, password, domain_type: 'Admin' })
            });

            const data = await apiResponse.json();

            if (!apiResponse.ok || data.status !== 'success') {
                return fail(401, {
                    error: 'Invalid ID or Password',
                    values: { email }
                });
            }

            // Enforce strict administrative authorization
            if (data.user?.role?.toLowerCase() !== 'admin') {
                return fail(403, {
                    error: 'Invalid ID or Password',
                    values: { email }
                });
            }

            const isSecure = url.protocol === 'https:' || request.headers.get('x-forwarded-proto') === 'https';

            // Write session token to HttpOnly secure cookie.
            // maxAge: 86400 = 24 hours, matching JWT expiry (Duration::days(1) in auth.rs).
            // Without maxAge this is a session cookie; some browsers/enhance flows
            // may not commit it before the dashboard redirect fires.
            cookies.set('zafaf_admin_session', data.token, {
                httpOnly: true,
                secure: isSecure,
                sameSite: 'lax',
                path: '/',
                maxAge: 60 * 60 * 24  // 24 hours
            });

            throw redirect(303, '/dashboard');

        } catch (err) {
            // Rethrow redirects if caught
            if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
                throw err;
            }

            console.error('Admin Auth connection failure with Axum API:', err);
            return fail(500, {
                error: 'Invalid ID or Password',
                values: { email }
            });
        }

        // Successfully authenticated admin: redirect to dashboard
        throw redirect(303, '/dashboard');
    }
};

