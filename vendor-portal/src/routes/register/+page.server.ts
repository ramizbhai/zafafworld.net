import type { PageServerLoad, Actions } from './$types';
import { fail, redirect, isRedirect } from '@sveltejs/kit';
import { apiClient } from '$lib/api/client';
import { z } from 'zod';

const registerSchema = z.object({
    official_name: z.string().trim().min(1, 'Official Name is required.'),
    email: z.string().trim().email('Invalid email address format.'),
    password: z.string().min(8, 'Your password must be at least 8 characters long for secure vendor access.'),
    phone: z.string().trim().min(1, 'Mobile Number is required.')
});

export const load: PageServerLoad = async () => {
    return {};
};

export const actions: Actions = {
    default: async ({ request, fetch, cookies, url }) => {
        const formData = await request.formData();
        const rawData = Object.fromEntries(formData);
        
        const parseResult = registerSchema.safeParse(rawData);
        if (!parseResult.success) {
            return fail(400, {
                error: parseResult.error.issues[0].message,
                values: {
                    official_name: rawData.official_name?.toString(),
                    email: rawData.email?.toString(),
                    phone: rawData.phone?.toString()
                }
            });
        }
        
        const { official_name, email, password, phone } = parseResult.data;

        // 3. Dispatch payload directly to the Rust backend Registration endpoint
        try {
            const result = await apiClient.withFetch(fetch).auth.register({
                official_name,
                email,
                password,
                phone
            });

            if (!result.success || !result.data) {
                return fail(result.status || 400, {
                    error: result.error?.message || 'The registration server rejected this submission. Please verify details.',
                    values: { official_name, email, phone }
                });
            }

            const data = result.data;

            if (data.status !== 'success') {
                return fail(result.status || 400, {
                    error: data.message || 'The registration server rejected this submission. Please verify details.',
                    values: { official_name, email, phone }
                });
            }

            // ── Frictionless Entry: backend now returns JWT immediately ──────────────
            // Set the session cookie and redirect straight to the dashboard.
            // No success card, no "Go to Login" — the vendor is already logged in.
            if (data.token) {
                const isSecure = url.protocol === 'https:' || request.headers.get('x-forwarded-proto') === 'https';
                cookies.set('zafaf_vendor_session', data.token, {
                    path: '/',
                    httpOnly: true,
                    sameSite: 'lax',
                    secure: isSecure,
                    maxAge: 60 * 60 * 24  // 1 day, matching JWT expiry
                });
            }

        } catch (err: any) {
            // Re-throw SvelteKit redirects as-is
            if (isRedirect(err)) {
                throw err;
            }
            console.error('Connection error with Rust register endpoint:', err);
            return fail(500, {
                error: 'Unable to communicate with the core registration service. Please verify the backend API is online.',
                values: { official_name, email, phone }
            });
        }

        throw redirect(303, '/dashboard');
    }
};
