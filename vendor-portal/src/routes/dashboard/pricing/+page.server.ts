import { env } from "$env/dynamic/public";
import { fail, redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ parent, fetch, cookies }) => {
    const sessionToken = cookies.get('zafaf_vendor_session');

    // Authenticate parent session
    const { user } = await parent();
    if (!user) {
        throw redirect(303, '/login');
    }

    try {
        const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/vendor/pricing`, {
            headers: {
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_vendor_session=${sessionToken}`
            }
        });

        if (response.ok) {
            const data = await response.json();
            if (data.status === 'success') {
                return {
                    plans: data.plans || [],
                    subscriptions: data.subscriptions || [],
                    pages: data.pages || []
                };
            }
        }
    } catch (err) {
        console.error('Failed to load pricing in server loader:', err);
    }

    return {
        plans: [],
        subscriptions: [],
        pages: []
    };
};
