import { env } from "$env/dynamic/public";
import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
    const sessionToken = cookies.get('zafaf_admin_session');

    if (!sessionToken) {
        throw redirect(303, '/login');
    }

    try {
        const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/analytics/summary`, {
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
            return { summary: null, error: 'Failed to load platform financial telemetry' };
        }

        const data = await response.json();
        if (data.status === 'success') {
            return {
                summary: data.summary || null
            };
        }

        return { summary: null, error: data.message || 'Failed to parse financial data' };

    } catch (err) {
        console.error('Failed to fetch admin financial telemetry:', err);
        return { summary: null, error: 'Internal connection error' };
    }
};
