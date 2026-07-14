import { env } from "$env/dynamic/public";
import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

    export const load: PageServerLoad = async ({ cookies, fetch, url }) => {
    const sessionToken = cookies.get('zafaf_admin_session');

    // 1. Enforce strict administrative session authentication validation
    if (!sessionToken) {
        throw redirect(303, '/login');
    }

    try {
        const range = url.searchParams.get('range') || 'last_12_months';
        // 2. Fetch aggregated business telemetry from Axum backend
        const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/analytics/summary?range=${range}`, {
            headers: {
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_admin_session=${sessionToken}`
            }
        });

        if (!response.ok) {
            if (response.status === 401 || response.status === 403) {
                // Wipe local session cookie on authentication violation
                cookies.delete('zafaf_admin_session', { path: '/' });
                throw redirect(303, '/login');
            }
            return { summary: null, error: `Backend API responded with status code: ${response.status}` };
        }

        const data = await response.json();
        if (data.status === 'success') {
            return {
                summary: data.summary,
                error: null
            };
        }
        return { summary: null, error: data.message || 'Failed to load telemetry summary metrics' };

    } catch (err: any) {
        // Propagate redirects directly
        if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
            throw err;
        }

        console.error('Failed to load global administrative analytics:', err);
        return { summary: null, error: err.message || 'Internal connection error' };
    }
};
