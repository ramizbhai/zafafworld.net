import { env } from "$env/dynamic/public";
import { redirect, fail } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
    const sessionToken = cookies.get('zafaf_admin_session');

    if (!sessionToken) {
        throw redirect(303, '/login');
    }

    const authHeaders = {
        'Authorization': `Bearer ${sessionToken}`,
        'Cookie': `zafaf_admin_session=${sessionToken}`
    };

    try {
        const res = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/vendors/subscription/requests`, { headers: authHeaders });

        if (!res.ok) {
            if (res.status === 401 || res.status === 403) {
                cookies.delete('zafaf_admin_session', { path: '/' });
                throw redirect(303, '/login');
            }
            return { requests: [] };
        }

        const data = await res.json();
        return {
            requests: data.status === 'success' ? (data.requests || []) : []
        };
    } catch (err) {
        if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
            throw err;
        }
        console.error('Failed to load subscription requests:', err);
        return { requests: [] };
    }
};

export const actions: Actions = {
    approve: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) {
            return fail(401, { error: 'Unauthorized' });
        }

        const formData = await request.formData();
        const id = formData.get('id') as string;
        const notes = formData.get('admin_notes') as string | null;

        if (!id) {
            return fail(400, { error: 'Missing request ID' });
        }

        try {
            const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/vendors/subscription/requests/${id}/approve`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ admin_notes: notes || undefined })
            });

            if (!response.ok) {
                const errData = await response.json().catch(() => ({}));
                return fail(response.status, { error: errData.error || errData.message || 'Approval failed' });
            }

            return { success: true, approved: true };
        } catch (err: any) {
            return fail(500, { error: err.message || 'Internal connection error' });
        }
    },

    reject: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) {
            return fail(401, { error: 'Unauthorized' });
        }

        const formData = await request.formData();
        const id = formData.get('id') as string;
        const notes = formData.get('admin_notes') as string | null;

        if (!id) {
            return fail(400, { error: 'Missing request ID' });
        }

        try {
            const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/vendors/subscription/requests/${id}/reject`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ admin_notes: notes || undefined })
            });

            if (!response.ok) {
                const errData = await response.json().catch(() => ({}));
                return fail(response.status, { error: errData.error || errData.message || 'Rejection failed' });
            }

            return { success: true, rejected: true };
        } catch (err: any) {
            return fail(500, { error: err.message || 'Internal connection error' });
        }
    }
};
