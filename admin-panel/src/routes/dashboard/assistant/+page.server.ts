import { env } from "$env/dynamic/public";
import { redirect, fail } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
    const sessionToken = cookies.get('zafaf_admin_session');

    // 1. Block access immediately if no session cookie exists
    if (!sessionToken) {
        throw redirect(303, '/login');
    }

    try {
        // 2. Load assistant inquiries from Axum backend
        const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/assistant/inquiries`, {
            headers: {
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_admin_session=${sessionToken}`
            }
        });

        if (!response.ok) {
            if (response.status === 401 || response.status === 403) {
                // Terminate session on token mismatch
                cookies.delete('zafaf_admin_session', { path: '/' });
                throw redirect(303, '/login');
            }
            return { inquiries: [] };
        }

        const data = await response.json();
        if (data.status === 'success') {
            return {
                inquiries: data.inquiries || []
            };
        }
        return { inquiries: [] };

    } catch (err) {
        // Propagate redirects directly
        if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
            throw err;
        }

        console.error('Failed to load assistant inquiries:', err);
        return { inquiries: [] };
    }
};

export const actions: Actions = {
    toggleStatus: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) {
            return fail(401, { error: 'Unauthorized administrative session' });
        }

        const formData = await request.formData();
        const id = formData.get('id') as string;
        const status = formData.get('status') as string;

        if (!id || !status) {
            return fail(400, { error: 'Missing inquiry identifier or target status' });
        }

        try {
            const response = await fetch(`${env.PUBLIC_API_URL || `${env.PUBLIC_API_URL || 'http://localhost:8080'}`}/api/v1/admin/assistant/inquiries/${id}/status`, {
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
                return fail(response.status, {
                    error: errData.message || `Failed to update inquiry status (Status ${response.status})`
                });
            }

            const data = await response.json();
            if (data.status === 'success') {
                return { success: true, id, status };
            }

            return fail(400, { error: data.message || 'Status transition failed' });

        } catch (err: any) {
            console.error('Error updating assistant inquiry status:', err);
            return fail(500, { error: err.message || 'Internal connection error' });
        }
    }
};
