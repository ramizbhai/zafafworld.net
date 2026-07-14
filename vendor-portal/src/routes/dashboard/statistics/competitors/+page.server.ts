import { apiClient } from '$lib/api/client';
import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ parent, fetch, cookies }) => {
    const sessionToken = cookies.get('zafaf_vendor_session');

    // Authenticate parent session
    const { user } = await parent();
    if (!user) {
        throw redirect(303, '/login');
    }

    try {
        const result = await apiClient.withFetch(fetch).vendor.getCompetitors(sessionToken || '');

        if (result.success && result.data && result.data.status === 'success') {
            return {
                competitorsCity: result.data.competitorsCity || [],
                competitorsService: result.data.competitorsService || []
            };
        }
    } catch (err) {
        console.error('Failed to load competitors in server loader:', err);
    }

    return {
        competitorsCity: [],
        competitorsService: []
    };
};
