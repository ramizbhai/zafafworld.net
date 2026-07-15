import { env } from "$env/dynamic/public";
import { redirect, fail } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch, url }) => {
    const sessionToken = cookies.get('zafaf_admin_session');
    if (!sessionToken) throw redirect(303, '/login');

    const page = url.searchParams.get('page') || '1';
    const limit = url.searchParams.get('limit') || '20';

    try {
        const queryParams = new URLSearchParams();
        queryParams.set('page', page);
        queryParams.set('limit', limit);
        
        const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/blogs?${queryParams.toString()}`, {
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
            return { blogs: [], total: 0, page: 1, totalPages: 1, error: 'Failed to load blog posts' };
        }

        const data = await response.json();
        if (data.status === 'success') {
            const limitNum = parseInt(data.limit || limit);
            const totalNum = parseInt(data.total || 0);
            
            return {
                blogs: data.data || [],
                total: totalNum,
                page: data.page || 1,
                totalPages: Math.ceil(totalNum / limitNum) || 1,
                publicClientUrl: env.PUBLIC_CLIENT_URL || 'https://zafafworld.net'
            };
        }

        return { blogs: [], total: 0, page: 1, totalPages: 1, error: data.message || 'Failed to parse blogs' };

    } catch (err) {
        if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
            throw err;
        }
        console.error('Failed to fetch admin blogs list:', err);
        return { blogs: [], total: 0, page: 1, totalPages: 1, error: 'Internal connection error' };
    }
};

export const actions: Actions = {
    deletePost: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) throw redirect(303, '/login');

        const formData = await request.formData();
        const id = formData.get('id')?.toString();

        if (!id) return fail(400, { error: 'Post ID is required' });

        try {
            const res = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/blogs/${id}`, {
                method: 'DELETE',
                headers: { 'Authorization': `Bearer ${sessionToken}` }
            });

            if (!res.ok) {
                const err = await res.json();
                return fail(res.status, { error: err.message || 'Failed to delete blog post' });
            }

            return { success: true };
        } catch (e) {
            console.error(e);
            return fail(500, { error: 'Network error occurred' });
        }
    }
};
