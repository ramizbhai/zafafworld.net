import { env } from "$env/dynamic/public";
import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
    const sessionToken = cookies.get('zafaf_admin_session');

    if (!sessionToken) {
        throw redirect(303, '/login');
    }

    try {
        // Fetch pre-calculated commission ledgers from secure finance endpoint
        const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/finance/commissions`, {
            headers: {
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_admin_session=${sessionToken}`
            }
        });

        // Fetch platform configurations for settings display
        const settingsResponse = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/settings`, {
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
            return { commissions: [], summary: {}, settings: {}, error: 'Failed to load commission ledgers' };
        }

        const data = await response.json();
        const settingsData = await settingsResponse.json().catch(() => ({}));

        if (data.status === 'success') {
            return {
                commissions: data.commissions || [],
                summary: data.summary || { total_commissions: 0, paid_commissions: 0, pending_commissions: 0 },
                settings: settingsData.settings || {}
            };
        }

        return { commissions: [], summary: {}, settings: {}, error: data.message || 'Failed to parse commissions dataset' };

    } catch (err) {
        console.error('Failed to fetch admin commissions aggregation dataset:', err);
        return { bookings: [], settings: {}, error: 'Internal connection error' };
    }
};
