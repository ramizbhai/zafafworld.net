import { apiClient } from '$lib/api/client';
import { fail, redirect } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
    const sessionToken = cookies.get('zafaf_vendor_session');
    if (!sessionToken) throw redirect(303, '/login');

    let requests: any[] = [];

    try {
        const result = await apiClient.withFetch(fetch).vendor.getSubscriptionRequests(sessionToken);
        
        if (result.success && result.data && result.data.status === 'success') {
            requests = result.data.requests || [];
        }
    } catch (err) {
        console.error('Error fetching subscription requests:', err);
    }

    return { requests };
};

export const actions: Actions = {
    requestUpgrade: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_vendor_session');
        if (!sessionToken) {
            return fail(401, { message: 'Session expired' });
        }

        const formData = await request.formData();
        const requested_tier_id = formData.get('requested_tier_id')?.toString();

        if (!requested_tier_id) {
            return fail(400, { message: 'Missing requested tier ID' });
        }

        try {
            const result = await apiClient.withFetch(fetch).vendor.createSubscriptionRequest(sessionToken, requested_tier_id);

            if (result.success) {
                return { success: true };
            } else {
                return fail(result.status || 400, { message: result.error?.message || 'Failed to request upgrade' });
            }
        } catch (err) {
            console.error('Error in requestUpgrade action:', err);
            return fail(500, { message: 'Network error occurred' });
        }
    }
};
