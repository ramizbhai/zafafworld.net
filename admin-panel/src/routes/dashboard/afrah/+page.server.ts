import { redirect, fail } from '@sveltejs/kit';
import { env } from '$env/dynamic/public';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
    const token = cookies.get('zafaf_admin_session');
    if (!token) {
        throw redirect(303, '/login');
    }

    const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';
    let inquiries: any[] = [];

    try {
        const res = await fetch(`${API_BASE}/api/v1/admin/afrah/inquiries`, {
            headers: { Authorization: `Bearer ${token}` },
        });
        if (res.ok) {
            const body = await res.json();
            if (body.status === 'success') {
                inquiries = body.inquiries || [];
            }
        }
    } catch (e) {
        console.error('Failed to fetch Afrah inquiries:', e);
    }

    return { inquiries };
};

export const actions: Actions = {
    updateStatus: async ({ request, cookies, fetch }) => {
        const token = cookies.get('zafaf_admin_session');
        if (!token) return fail(401, { error: 'Unauthorized' });

        const fd = await request.formData();
        const id = fd.get('id')?.toString();
        const status = fd.get('status')?.toString();

        if (!id || !status) return fail(400, { error: 'Inquiry ID and status are required' });

        const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';

        try {
            const res = await fetch(`${API_BASE}/api/v1/admin/afrah/inquiries/${id}/status`, {
                method: 'PATCH',
                headers: {
                    'Authorization': `Bearer ${token}`,
                    'Cookie': `zafaf_admin_session=${token}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ status })
            });

            const data = await res.json().catch(() => ({}));
            if (!res.ok) return fail(res.status, { error: data.message || 'Failed to update Afrah inquiry status' });

            return { success: true, message: 'Status updated successfully' };

        } catch (err: any) {
            return fail(500, { error: err.message || 'Connection error' });
        }
    }
};

