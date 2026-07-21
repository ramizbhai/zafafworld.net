import { env } from "$env/dynamic/public";
import { redirect, fail } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch, url }) => {
    const sessionToken = cookies.get('zafaf_admin_session');

    if (!sessionToken) {
        throw redirect(303, '/login');
    }

    const page = url.searchParams.get('page') || '1';
    const search = url.searchParams.get('search') || '';
    const status = url.searchParams.get('status') || 'all';

    try {
        const queryParams = new URLSearchParams();
        queryParams.set('page', page);
        queryParams.set('limit', '10');
        if (search) {
            queryParams.set('search', search);
        }
        if (status && status !== 'all') {
            queryParams.set('status', status);
        }

        const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/bookings?${queryParams.toString()}`, {
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
            return { bookings: [], total: 0, page: 1, totalPages: 1, status, search, error: 'Failed to load bookings ledger' };
        }

        const data = await response.json();
        if (data.status === 'success') {
            return {
                bookings: data.bookings || [],
                total: data.total || 0,
                page: data.page || 1,
                totalPages: data.totalPages || 1,
                status,
                search
            };
        }

        return { bookings: [], total: 0, page: 1, totalPages: 1, status, search, error: data.message || 'Failed to parse bookings ledger data' };

    } catch (err) {
        if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
            throw err;
        }
        console.error('Failed to fetch admin bookings ledger:', err);
        return { bookings: [], total: 0, page: 1, totalPages: 1, status, search, error: 'Internal connection error' };
    }
};

export const actions: Actions = {
    updateStatus: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) return fail(401, { error: 'Unauthorized' });

        const fd = await request.formData();
        const id = fd.get('id')?.toString();
        const to_status = fd.get('to_status')?.toString();
        const reason = fd.get('reason')?.toString() || 'Status transitioned by admin';

        if (!id || !to_status) return fail(400, { error: 'Booking ID and target status are required' });

        try {
            const res = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/bookings/${id}/transition`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ to_status, reason })
            });

            const data = await res.json().catch(() => ({}));
            if (!res.ok) return fail(res.status, { error: data.message || 'Failed to transition booking status' });

            return { success: true, message: 'Booking status updated successfully' };

        } catch (err: any) {
            return fail(500, { error: err.message || 'Connection error' });
        }
    }
};

