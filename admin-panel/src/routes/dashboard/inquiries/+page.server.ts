import { env } from "$env/dynamic/public";
import { redirect, fail } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch, url }) => {
    const sessionToken = cookies.get('zafaf_admin_session');
    if (!sessionToken) {
        throw redirect(303, '/login');
    }

    const page = url.searchParams.get('page') || '1';
    const limit = url.searchParams.get('limit') || '20';
    const q = url.searchParams.get('q') || '';
    const status = url.searchParams.get('status') || '';
    const priority = url.searchParams.get('priority') || '';
    const escalationStatus = url.searchParams.get('escalation_status') || '';
    const resolutionStatus = url.searchParams.get('resolution_status') || '';

    const apiBase = env.PUBLIC_API_URL || 'http://localhost:8080';
    const queryParams = new URLSearchParams({
        page,
        limit,
        ...(q ? { q } : {}),
        ...(status ? { status } : {}),
        ...(priority ? { priority } : {}),
        ...(escalationStatus ? { escalation_status: escalationStatus } : {}),
        ...(resolutionStatus ? { resolution_status: resolutionStatus } : {})
    });

    try {
        const [inqRes, metricsRes] = await Promise.all([
            fetch(`${apiBase}/api/v1/admin/inquiries?${queryParams.toString()}`, {
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`
                }
            }),
            fetch(`${apiBase}/api/v1/admin/inquiries/metrics`, {
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`
                }
            })
        ]);

        if (!inqRes.ok) {
            if (inqRes.status === 401 || inqRes.status === 403) {
                cookies.delete('zafaf_admin_session', { path: '/' });
                throw redirect(303, '/login');
            }
            return { inquiries: [], pagination: { page: 1, limit: 20, totalItems: 0, totalPages: 0 }, metrics: null };
        }

        const inqData = await inqRes.json();
        const metricsData = metricsRes.ok ? await metricsRes.json() : null;

        return {
            inquiries: inqData.inquiries || [],
            pagination: inqData.pagination || { page: 1, limit: 20, totalItems: 0, totalPages: 0 },
            metrics: metricsData?.metrics || null,
            filters: { q, status, priority, escalationStatus, resolutionStatus }
        };
    } catch (err) {
        if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
            throw err;
        }
        console.error('Failed to load system inquiries:', err);
        return { inquiries: [], pagination: { page: 1, limit: 20, totalItems: 0, totalPages: 0 }, metrics: null, filters: { q: '', status: '', priority: '', escalationStatus: '', resolutionStatus: '' } };
    }
};

export const actions: Actions = {
    updateStatus: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) return fail(401, { error: 'Unauthorized session' });

        const formData = await request.formData();
        const id = formData.get('id') as string;
        const status = formData.get('status') as string;

        if (!id || !status) return fail(400, { error: 'Missing identifier or status' });

        try {
            const res = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/inquiries/${id}/status`, {
                method: 'PATCH',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ status })
            });

            if (!res.ok) {
                const err = await res.json().catch(() => ({}));
                return fail(res.status, { error: err.message || 'Status update failed' });
            }

            return { success: true };
        } catch (err: any) {
            return fail(500, { error: err.message || 'Server error' });
        }
    }
};
