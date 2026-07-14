import { env } from "$env/dynamic/public";
import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch, url }) => {
    const sessionToken = cookies.get('zafaf_admin_session');

    if (!sessionToken) {
        throw redirect(303, '/login');
    }

    const page = url.searchParams.get('page') || '1';
    const operator = url.searchParams.get('operator') || '';
    const eventType = url.searchParams.get('eventType') || 'all';

    try {
        const queryParams = new URLSearchParams();
        queryParams.set('page', page);
        queryParams.set('limit', '10');
        if (operator) {
            queryParams.set('operator_search', operator);
        }
        if (eventType && eventType !== 'all') {
            queryParams.set('event_type', eventType);
        }

        const response = await fetch(`${env.PUBLIC_API_URL || `${env.PUBLIC_API_URL || 'http://localhost:8080'}`}/api/v1/admin/audit/logs?${queryParams.toString()}`, {
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
            return { logs: [], total: 0, page: 1, totalPages: 1, operator, eventType, error: 'Failed to load system audit logs' };
        }

        const data = await response.json();
        if (data.status === 'success') {
            return {
                logs: data.logs || [],
                total: data.total || 0,
                page: data.page || 1,
                totalPages: data.totalPages || 1,
                operator,
                eventType
            };
        }

        return { logs: [], total: 0, page: 1, totalPages: 1, operator, eventType, error: data.message || 'Failed to parse audit logging feed' };

    } catch (err) {
        if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
            throw err;
        }
        console.error('Failed to fetch admin audit logging feed:', err);
        return { logs: [], total: 0, page: 1, totalPages: 1, operator, eventType, error: 'Internal connection error' };
    }
};
