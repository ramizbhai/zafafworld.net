import { env } from '$env/dynamic/public';
import { redirect, fail } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
    const sessionToken = cookies.get('zafaf_admin_session');
    if (!sessionToken) throw redirect(303, '/login');

    try {
        const res = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/comments`, {
            headers: {
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_admin_session=${sessionToken}`
            }
        });

        if (!res.ok) {
            return { comments: [], error: 'Failed to load comments' };
        }

        const data = await res.json();
        return {
            comments: data.data || []
        };
    } catch (e) {
        console.error('Failed to load comments:', e);
        return { comments: [], error: 'Connection error' };
    }
};

export const actions: Actions = {
    approve: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) throw redirect(303, '/login');

        const formData = await request.formData();
        const id = formData.get('id')?.toString();

        if (!id) {
            return fail(400, { error: 'Comment ID is required' });
        }

        try {
            const res = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/comments/${id}/approve`, {
                method: 'PATCH',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`
                }
            });

            if (!res.ok) {
                const err = await res.json();
                return fail(res.status, { error: err.message || 'Failed to approve comment' });
            }

            return { success: true };
        } catch (e) {
            console.error(e);
            return fail(500, { error: 'Network error occurred' });
        }
    },

    reject: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) throw redirect(303, '/login');

        const formData = await request.formData();
        const id = formData.get('id')?.toString();

        if (!id) {
            return fail(400, { error: 'Comment ID is required' });
        }

        try {
            const res = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/comments/${id}/reject`, {
                method: 'PATCH',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`
                }
            });

            if (!res.ok) {
                const err = await res.json();
                return fail(res.status, { error: err.message || 'Failed to reject comment' });
            }

            return { success: true };
        } catch (e) {
            console.error(e);
            return fail(500, { error: 'Network error occurred' });
        }
    }
};
