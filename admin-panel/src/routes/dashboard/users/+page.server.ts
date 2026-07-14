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

        const response = await fetch(`${env.PUBLIC_API_URL || `${env.PUBLIC_API_URL || 'http://localhost:8080'}`}/api/v1/admin/users?${queryParams.toString()}`, {
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
            return { users: [], total: 0, page: 1, totalPages: 1, error: 'Failed to load users' };
        }

        const data = await response.json();
        if (data.status === 'success') {
            return {
                users: data.users || [],
                total: data.total || 0,
                page: data.page || 1,
                totalPages: data.totalPages || 1,
                stats: data.stats || {
                    totalUsers: 0,
                    totalClients: 0,
                    totalVendors: 0,
                    totalAdmins: 0,
                    newUsersThisMonth: 0
                },
                search
            };
        }

        return { users: [], total: 0, page: 1, totalPages: 1, error: data.message || 'Failed to parse user data' };

    } catch (err) {
        if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
            throw err;
        }
        console.error('Failed to fetch admin users directory:', err);
        return { users: [], total: 0, page: 1, totalPages: 1, error: 'Internal connection error' };
    }
};

export const actions = {
    updateStatus: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) {
            return { success: false, error: 'Unauthorized administrative session' };
        }

        const formData = await request.formData();
        const id = formData.get('id') as string;
        const status = formData.get('status') as string;

        if (!id || !status) {
            return { success: false, error: 'Missing user identifier or target status' };
        }

        try {
            const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/users/${id}/status`, {
                method: 'PATCH',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ status })
            });

            if (!response.ok) {
                const errData = await response.json().catch(() => ({}));
                return { success: false, error: errData.message || `Failed to update user status (Status ${response.status})` };
            }

            const data = await response.json();
            if (data.status === 'success') {
                return { success: true, message: data.message };
            }

            return { success: false, error: data.message || 'Status update action failed' };
        } catch (err: any) {
            console.error('Error updating user status:', err);
            return { success: false, error: err.message || 'Internal connection error' };
        }
    }
};
