import { apiClient } from '$lib/api/client';
import { fail, redirect } from '@sveltejs/kit';
import type { Actions } from './$types';

import { z } from 'zod';

const loginSchema = z.object({
    email: z.string().email('Invalid email address format.'),
    password: z.string().min(8, 'Password must be at least 8 characters long.')
});

export const actions: Actions = {
    default: async ({ request, cookies, fetch, url }) => {
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
            const result = await apiClient.withFetch(fetch).auth.login({ email, password });

            if (!result.success || !result.data || result.data.status !== 'success') {
                return fail(401, {
                    error: result.error?.message || 'Invalid ID or Password',
                    values: { email }
                });
            }

            const responseData = result.data;

            // Ensure the user role is vendor
            if (responseData.user?.role?.toLowerCase() !== 'vendor') {
                return fail(403, {
                    error: 'Invalid ID or Password',
                    values: { email }
                });
            }

            const isSecure = url.protocol === 'https:' || request.headers.get('x-forwarded-proto') === 'https';
            // Set the zafaf_vendor_session cookie securely based on protocol
            cookies.set('zafaf_vendor_session', responseData.token, {
                httpOnly: true,
                secure: isSecure,
                sameSite: 'lax',
                path: '/',
                maxAge: 60 * 60 * 24 // 1 day
            });

        } catch (err) {
            // Re-throw SvelteKit redirect exceptions
            if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
                throw err;
            }

            console.error('Login action connection error with Rust backend:', err);
            return fail(500, {
                error: 'Unable to communicate with the authentication service. Please verify the backend API is online.',
                values: { email }
            });
        }

        // Redirect to dashboard root on successful token set
        throw redirect(303, '/dashboard');
    }
};

