import { apiClient } from '$lib/api/client';
import { fail } from '@sveltejs/kit';
import type { Actions } from './$types';

export const actions: Actions = {
    default: async ({ request, fetch }) => {
        const formData = await request.formData();
        const token = formData.get('token')?.toString().trim();
        const password = formData.get('password')?.toString();
        const confirmPassword = formData.get('confirmPassword')?.toString();

        if (!token) {
            return fail(400, {
                error: 'Invalid or missing recovery token.'
            });
        }

        if (!password || password.length < 8) {
            return fail(400, {
                error: 'Password must be at least 8 characters long.'
            });
        }

        if (password !== confirmPassword) {
            return fail(400, {
                error: 'Passwords do not match.'
            });
        }

        try {
            const result = await apiClient.withFetch(fetch).auth.resetPassword({ token, password });

            if (!result.success || !result.data || result.data.status !== 'success') {
                return fail(result.status || 500, {
                    error: result.error?.message || 'An error occurred while resetting your password.'
                });
            }

            return { success: true, message: result.data.message };
        } catch (err) {
            console.error('Vendor reset password connection error with Rust backend:', err);
            return fail(500, {
                error: 'An unexpected system error occurred.'
            });
        }
    }
};
