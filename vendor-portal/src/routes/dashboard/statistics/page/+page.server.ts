import { apiClient } from '$lib/api/client';
import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ parent, fetch, cookies }) => {
    const sessionToken = cookies.get('zafaf_vendor_session');

    // Fetch pre-aggregated statistics dashboard payload
    const statsPromise = apiClient.withFetch(fetch).vendor.getDashboardStats(sessionToken || '');

    // Authenticate parent session
    const { user } = await parent();
    if (!user) {
        throw redirect(303, '/login');
    }

    try {
        const result = await statsPromise;
        if (result.success && result.data && result.data.status === 'success' && result.data.data) {
            const data = result.data;
            return {
                metrics: data.data.metrics ?? null,
                vendor: data.data.vendor ?? null,
                products: data.data.products ?? [],
                leadInquiries: data.data.leadInquiries ?? [],
                urgencyBreakdown: data.data.urgencyBreakdown ?? [],
                productStatusBreakdown: data.data.productStatusBreakdown ?? [],
                ratingAxes: data.data.ratingAxes ?? []
            };
        }
    } catch (err) {
        console.error('Failed to load statistics dashboard data:', err);
    }

    return {
        metrics: null,
        vendor: null,
        products: [],
        leadInquiries: [],
        urgencyBreakdown: [],
        productStatusBreakdown: [],
        ratingAxes: []
    };
};
