import { env } from "$env/dynamic/public";
import { redirect, fail } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

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

        const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/cms/articles?${queryParams.toString()}`, {
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

export const actions: Actions = {
    createArticle: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) return fail(401, { error: 'Unauthorized' });

        const fd = await request.formData();
        const slug = fd.get('slug')?.toString().trim();
        const category = fd.get('category')?.toString().trim() || 'wedding-tips';
        const title_ar = fd.get('title_ar')?.toString().trim();
        const title_en = fd.get('title_en')?.toString().trim();
        const summary_ar = fd.get('summary_ar')?.toString().trim() || '';
        const summary_en = fd.get('summary_en')?.toString().trim() || '';
        const body_ar = fd.get('body_ar')?.toString().trim() || '';
        const body_en = fd.get('body_en')?.toString().trim() || '';
        const published = fd.get('published') === 'true';

        if (!slug || !title_ar || !title_en) {
            return fail(400, { error: 'Slug, Arabic Title, and English Title are required' });
        }

        try {
            const res = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/cms/articles`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    slug, category, title_ar, title_en,
                    summary_ar, summary_en, body_ar, body_en, published
                })
            });

            const data = await res.json().catch(() => ({}));
            if (!res.ok) return fail(res.status, { error: data.message || 'Failed to create article' });
            return { success: true, message: 'Article created successfully' };

        } catch (err: any) {
            return fail(500, { error: err.message || 'Connection error' });
        }
    },

    updateArticle: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) return fail(401, { error: 'Unauthorized' });

        const fd = await request.formData();
        const id = fd.get('id')?.toString();
        const slug = fd.get('slug')?.toString().trim();
        const category = fd.get('category')?.toString().trim() || 'wedding-tips';
        const title_ar = fd.get('title_ar')?.toString().trim();
        const title_en = fd.get('title_en')?.toString().trim();
        const summary_ar = fd.get('summary_ar')?.toString().trim() || '';
        const summary_en = fd.get('summary_en')?.toString().trim() || '';
        const body_ar = fd.get('body_ar')?.toString().trim() || '';
        const body_en = fd.get('body_en')?.toString().trim() || '';
        const published = fd.get('published') === 'true';

        if (!id || !slug || !title_ar || !title_en) {
            return fail(400, { error: 'Article ID, Slug, Arabic Title, and English Title are required' });
        }

        try {
            const res = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/cms/articles/${id}`, {
                method: 'PUT',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    slug, category, title_ar, title_en,
                    summary_ar, summary_en, body_ar, body_en, published
                })
            });

            const data = await res.json().catch(() => ({}));
            if (!res.ok) return fail(res.status, { error: data.message || 'Failed to update article' });
            return { success: true, message: 'Article updated successfully' };

        } catch (err: any) {
            return fail(500, { error: err.message || 'Connection error' });
        }
    },

    deleteArticle: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) return fail(401, { error: 'Unauthorized' });

        const fd = await request.formData();
        const id = fd.get('id')?.toString();

        if (!id) return fail(400, { error: 'Article ID is required' });

        try {
            const res = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/cms/articles/${id}`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`
                }
            });

            const data = await res.json().catch(() => ({}));
            if (!res.ok) return fail(res.status, { error: data.message || 'Failed to delete article' });
            return { success: true, message: 'Article deleted successfully' };

        } catch (err: any) {
            return fail(500, { error: err.message || 'Connection error' });
        }
    }
};

