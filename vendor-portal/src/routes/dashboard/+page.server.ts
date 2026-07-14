import { apiClient } from '$lib/api/client';
import { fail, redirect } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
    const sessionToken = cookies.get('zafaf_vendor_session');
    if (!sessionToken) {
        return { inquiries: [] };
    }

    try {
        const result = await apiClient.withFetch(fetch).vendor.getInquiries(sessionToken);
        if (result.success && result.data && result.data.status === 'success') {
            return {
                inquiries: result.data.inquiries
            };
        }
    } catch (err) {
        console.error('Error fetching inquiries for dashboard:', err);
    }

    return { inquiries: [] };
};

export const actions: Actions = {
    updateStatus: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_vendor_session');
        if (!sessionToken) {
            return fail(401, { error: 'Session expired' });
        }

        const formData = await request.formData();
        const id = formData.get('id')?.toString();
        const status = formData.get('status')?.toString();

        if (!id || !status) {
            return fail(400, { error: 'Missing id or status' });
        }

        try {
            const result = await apiClient.withFetch(fetch).vendor.updateInquiryStatus(sessionToken, id, status);

            if (result.success) {
                return { success: true };
            } else {
                return fail(result.status || 400, { error: result.error?.message || 'Failed to update status' });
            }
        } catch (err) {
            console.error('Error in updateStatus action:', err);
            return fail(500, { error: 'Network error occurred' });
        }
    },
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
