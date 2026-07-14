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
    const published = url.searchParams.get('published') || 'all';

    try {
        const queryParams = new URLSearchParams();
        queryParams.set('page', page);
        queryParams.set('limit', '10');
        if (search) {
            queryParams.set('search', search);
        }
        if (published && published !== 'all') {
            queryParams.set('published', published);
        }

        const response = await fetch(`${env.PUBLIC_API_URL || `${env.PUBLIC_API_URL || 'http://localhost:8080'}`}/api/v1/admin/cms/articles?${queryParams.toString()}`, {
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
            return { articles: [], total: 0, page: 1, totalPages: 1, search, published, error: 'Failed to load CMS articles list' };
        }

        const data = await response.json();
        if (data.status === 'success') {
            return {
                articles: data.articles || [],
                total: data.total || 0,
                page: data.page || 1,
                totalPages: data.totalPages || 1,
                search,
                published
            };
        }

        return { articles: [], total: 0, page: 1, totalPages: 1, search, published, error: data.message || 'Failed to parse CMS articles' };

    } catch (err) {
        if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
            throw err;
        }
        console.error('Failed to fetch admin CMS articles list:', err);
        return { articles: [], total: 0, page: 1, totalPages: 1, search, published, error: 'Internal connection error' };
    }
};
