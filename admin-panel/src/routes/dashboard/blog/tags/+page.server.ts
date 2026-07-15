import { env } from '$env/dynamic/public';
import { redirect, fail } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
    const sessionToken = cookies.get('zafaf_admin_session');
    if (!sessionToken) throw redirect(303, '/login');

    try {
        const res = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/tags`, {
            headers: {
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_admin_session=${sessionToken}`
            }
        });

        if (!res.ok) {
            return { tags: [], error: 'Failed to load tags' };
        }

        const data = await res.json();
        return {
            tags: data.data || []
        };
    } catch (e) {
        console.error('Failed to load tags:', e);
        return { tags: [], error: 'Connection error' };
    }
};

export const actions: Actions = {
    create: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) throw redirect(303, '/login');

        const formData = await request.formData();
        const name = formData.get('name')?.toString()?.trim();
        const slug = formData.get('slug')?.toString()?.trim() || name?.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)+/g, '');

        if (!name) {
            return fail(400, { error: 'Tag name is required' });
        }

        try {
            const res = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/tags`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`
                },
                body: JSON.stringify({ name, slug })
            });

            if (!res.ok) {
                const err = await res.json();
                return fail(res.status, { error: err.message || 'Failed to create tag' });
            }

            return { success: true };
        } catch (e) {
            console.error(e);
            return fail(500, { error: 'Network error occurred' });
        }
    },

    delete: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) throw redirect(303, '/login');

        const formData = await request.formData();
        const id = formData.get('id')?.toString();

        if (!id) {
            return fail(400, { error: 'Tag ID is required' });
        }

        try {
            const res = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/tags/${id}`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`
                }
            });

            if (!res.ok) {
                const err = await res.json();
                return fail(res.status, { error: err.message || 'Failed to delete tag' });
            }

            return { success: true };
        } catch (e) {
            console.error(e);
            return fail(500, { error: 'Network error occurred' });
        }
    }
};
