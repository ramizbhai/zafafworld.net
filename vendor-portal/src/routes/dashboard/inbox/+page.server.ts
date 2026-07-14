import { apiClient } from '$lib/api/client';
import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch, cookies, depends, locals }) => {
    depends('app:vendor-inbox');
    const sessionToken = cookies.get('zafaf_vendor_session');

    if (!sessionToken) {
        throw redirect(303, '/login');
    }

    try {
        const result = await apiClient.withFetch(fetch).vendor.getConversations(sessionToken);

        if (!result.success || !result.data) {
            if (result.status === 401 || result.status === 403) {
                throw redirect(303, '/login');
            }
            return {
                conversations: [],
                vendorUserId: locals.user?.id || '',
                token: sessionToken,
                error: result.error?.message || 'Encountered a server error or backend is offline.'
            };
        }

        const json = result.data;

        return {
            conversations: json.data || [],
            vendorUserId: locals.user?.id || '',
            token: sessionToken
        };
    } catch (err) {
        if (err && typeof err === 'object' && 'status' in err && 'location' in err) throw err;
        console.error('Inbox page loader error:', err);
        return {
            conversations: [],
            vendorUserId: locals.user?.id || '',
            token: sessionToken
        };
    }
};
