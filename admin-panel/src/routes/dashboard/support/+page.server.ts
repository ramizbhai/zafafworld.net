import { env } from "$env/dynamic/public";
import { redirect } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
    const sessionToken = cookies.get('zafaf_admin_session');

    if (!sessionToken) {
        throw redirect(303, '/login');
    }

    try {
        const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/assistant/inquiries`, {
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
            return { inquiries: [], error: 'Failed to load inquiries' };
        }

        const data = await response.json();
        return {
            inquiries: data.inquiries || []
        };

    } catch (err) {
        console.error('Failed to fetch support inquiries:', err);
        return { inquiries: [], error: 'Internal connection error' };
    }
};

export const actions: Actions = {
    resolve: async ({ cookies, fetch, request }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) {
            throw redirect(303, '/login');
        }

        const formData = await request.formData();
        const id = formData.get('id');

        try {
            const response = await fetch(`${env.PUBLIC_API_URL || `${env.PUBLIC_API_URL || 'http://localhost:8080'}`}/api/v1/admin/assistant/inquiries/${id}/status`, {
                method: 'PATCH',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ status: 'resolved' })
            });

            if (!response.ok) {
                return { success: false, error: 'Failed to resolve inquiry' };
            }

            return { success: true };
        } catch (err) {
            console.error('Failed to resolve inquiry:', err);
            return { success: false, error: 'Connection error' };
        }
    }
};
