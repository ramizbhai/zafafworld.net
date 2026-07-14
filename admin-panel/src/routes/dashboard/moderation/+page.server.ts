import { env } from "$env/dynamic/public";
import { redirect } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
    const sessionToken = cookies.get('zafaf_admin_session');

    if (!sessionToken) {
        throw redirect(303, '/login');
    }

    try {
        const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/reviews/pending`, {
            headers: {
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_admin_session=${sessionToken}`
            }
        });

        if (!response.ok) {
            if (response.status === 401 || response.status === 403) {
                cookies.delete('zafaf_admin_session', { path: '/' });
                throw redirect(303, '/login');
            }
            return { reviews: [], error: 'Failed to load reviews' };
        }

        const data = await response.json();
        return {
            reviews: data.reviews || []
        };

    } catch (err) {
        console.error('Failed to fetch pending reviews for moderation:', err);
        return { reviews: [], error: 'Internal connection error' };
    }
};

export const actions: Actions = {
    moderate: async ({ cookies, fetch, request }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) {
            throw redirect(303, '/login');
        }

        const formData = await request.formData();
        const id = formData.get('id');
        const approve = formData.get('approve') === 'true';

        try {
            const response = await fetch(`${env.PUBLIC_API_URL || `${env.PUBLIC_API_URL || 'http://localhost:8080'}`}/api/v1/admin/reviews/${id}/approve`, {
                method: 'PATCH',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ approve })
            });

            if (!response.ok) {
                return { success: false, error: 'Failed to moderate review' };
            }

            return { success: true };
        } catch (err) {
            console.error('Failed to moderate review:', err);
            return { success: false, error: 'Connection error' };
        }
    }
};
