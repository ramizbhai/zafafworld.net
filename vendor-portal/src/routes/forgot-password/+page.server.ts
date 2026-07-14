import { apiClient } from '$lib/api/client';
import { fail } from '@sveltejs/kit';
import type { Actions } from './$types';

export const actions: Actions = {
    default: async ({ request, fetch }) => {
        const formData = await request.formData();
        const email = formData.get('email')?.toString().trim().toLowerCase();

        if (!email || !email.includes('@')) {
            return fail(400, {
                error: 'A valid email address is required.',
                values: { email }
            });
        }

        try {
            const result = await apiClient.withFetch(fetch).auth.forgotPassword(email);

            if (!result.success || !result.data || result.data.status !== 'success') {
                return fail(result.status || 500, {
                    error: result.error?.message || 'An unexpected error occurred during password recovery.',
                    values: { email }
                });
            }

            return { success: true, message: result.data.message };
        } catch (err) {
            console.error('Vendor forgot password connection error with Rust backend:', err);
            return fail(500, {
                error: 'An unexpected system error occurred.',
                values: { email }
            });
        }
    }
};
