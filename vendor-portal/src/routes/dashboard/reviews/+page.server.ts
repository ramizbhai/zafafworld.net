import { apiClient } from '$lib/api/client';
import { fail, redirect } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ parent, fetch, cookies, url }) => {
    const sessionToken = cookies.get('zafaf_vendor_session');
    
    const params = new URLSearchParams();
    const statusFilter = url.searchParams.get('status');
    if (statusFilter && statusFilter !== 'all') {
        params.set('status', statusFilter);
    }
    const queryString = params.toString();

    // Dispatch the reviews fetch concurrently
    const reviewsPromise = apiClient.withFetch(fetch).vendor.getReviews(sessionToken || '', queryString).then((result) => {
        if (result.success && result.data && result.data.status === 'success') {
            return result.data.reviews || [];
        }
        return [];
    }).catch((err) => {
        console.error('Failed to stream reviews:', err);
        return [];
    });

    // Authenticate parent session
    const { user } = await parent();
    if (!user) {
        throw redirect(303, '/login');
    }

    return {
        heavyReviews: reviewsPromise
    };
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
            const result = await apiClient.withFetch(fetch).vendor.updateReviewStatus(sessionToken, id, status);

            if (result.success) {
                return { success: true };
            } else {
                return fail(result.status || 400, { error: result.error?.message || 'Failed to update status' });
            }
        } catch (err) {
            console.error('Error in review updateStatus action:', err);
            return fail(500, { error: 'Network error occurred' });
        }
    }
};
