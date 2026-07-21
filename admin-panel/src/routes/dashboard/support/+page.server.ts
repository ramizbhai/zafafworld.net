import { env } from "$env/dynamic/public";
import { redirect } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

const getApiBase = () => {
    return process.env.NODE_ENV === 'production' 
        ? 'http://backend:8080' 
        : (env.PUBLIC_API_URL || 'http://localhost:8080');
};

export const load: PageServerLoad = async ({ cookies, fetch, url }) => {
    const sessionToken = cookies.get('zafaf_admin_session');

    if (!sessionToken) {
        throw redirect(303, '/login');
    }

    const page = url.searchParams.get('page') || '1';
    const status = url.searchParams.get('status') || '';
    const search = url.searchParams.get('search') || '';

    try {
        const queryParams = new URLSearchParams();
        queryParams.set('page', page);
        queryParams.set('limit', '50');
        if (status && status !== 'all') {
            queryParams.set('status', status);
        }
        if (search) {
            queryParams.set('search', search);
        }

        const apiBase = getApiBase();
        const response = await fetch(`${apiBase}/api/v1/admin/support?${queryParams.toString()}`, {
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
            return { items: [], pagination: { total: 0, page: 1, limit: 50, pages: 1 }, error: 'Failed to load support messages' };
        }

        const resData = await response.json();
        if (resData.status === 'success') {
            return {
                items: resData.data?.items || [],
                pagination: resData.data?.pagination || { total: 0, page: 1, limit: 50, pages: 1 },
                statusFilter: status,
                searchFilter: search
            };
        }

        return { items: [], pagination: { total: 0, page: 1, limit: 50, pages: 1 }, error: resData.message || 'Failed to parse data' };
    } catch (err) {
        if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
            throw err;
        }
        console.error('Failed to fetch support messages:', err);
        return { items: [], pagination: { total: 0, page: 1, limit: 50, pages: 1 }, error: 'Internal connection error' };
    }
};

export const actions: Actions = {
    updateStatus: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) {
            return { success: false, error: 'Unauthorized administrative session' };
        }

        const formData = await request.formData();
        const id = formData.get('id') as string;
        const status = formData.get('status') as string;

        if (!id || !status) {
            return { success: false, error: 'Missing message identifier or status value' };
        }

        try {
            const apiBase = getApiBase();
            const response = await fetch(`${apiBase}/api/v1/admin/support/${id}/status`, {
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
                return { success: false, error: errData.message || `Failed to update status (Status ${response.status})` };
            }

            const resData = await response.json();
            if (resData.status === 'success') {
                return { success: true, message: resData.message };
            }

            return { success: false, error: resData.message || 'Action failed' };
        } catch (err: any) {
            console.error('Error updating support message status:', err);
            return { success: false, error: err.message || 'Internal connection error' };
        }
    },

    deleteMessage: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) {
            return { success: false, error: 'Unauthorized administrative session' };
        }

        const formData = await request.formData();
        const id = formData.get('id') as string;

        if (!id) {
            return { success: false, error: 'Missing message identifier' };
        }

        try {
            const apiBase = getApiBase();
            const response = await fetch(`${apiBase}/api/v1/admin/support/${id}`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`
                }
            });

            if (!response.ok) {
                const errData = await response.json().catch(() => ({}));
                return { success: false, error: errData.message || `Failed to delete message (Status ${response.status})` };
            }

            const resData = await response.json();
            if (resData.status === 'success') {
                return { success: true, message: resData.message };
            }

            return { success: false, error: resData.message || 'Action failed' };
        } catch (err: any) {
            console.error('Error deleting support message:', err);
            return { success: false, error: err.message || 'Internal connection error' };
        }
    }
};
