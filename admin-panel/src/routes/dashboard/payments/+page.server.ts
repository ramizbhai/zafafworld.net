import { env } from "$env/dynamic/public";
import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch, url }) => {
    const sessionToken = cookies.get('zafaf_admin_session');

    if (!sessionToken) {
        throw redirect(303, '/login');
    }

    const page = url.searchParams.get('page') || '1';
    const search = url.searchParams.get('search') || '';

    try {
        const queryParams = new URLSearchParams();
        queryParams.set('page', page);
        queryParams.set('limit', '10');
        if (search) {
            queryParams.set('search', search);
        }

        const response = await fetch(`${env.PUBLIC_API_URL || `${env.PUBLIC_API_URL || 'http://localhost:8080'}`}/api/v1/admin/bookings?${queryParams.toString()}`, {
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
            return { bookings: [], total: 0, page: 1, totalPages: 1, search, error: 'Failed to load bookings database for payment projection' };
        }

        const data = await response.json();
        if (data.status === 'success') {
            return {
                bookings: data.bookings || [],
                total: data.total || 0,
                page: data.page || 1,
                totalPages: data.totalPages || 1,
                search
            };
        }

        return { bookings: [], total: 0, page: 1, totalPages: 1, search, error: data.message || 'Failed to parse payments projection dataset' };

    } catch (err) {
        console.error('Failed to project payments ledger:', err);
        return { bookings: [], total: 0, page: 1, totalPages: 1, search, error: 'Internal connection error' };
    }
};
