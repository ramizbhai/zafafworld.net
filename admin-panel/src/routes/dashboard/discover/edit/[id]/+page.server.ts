import { env } from "$env/dynamic/public";
import { redirect } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ params, cookies, fetch }) => {
    const sessionToken = cookies.get('zafaf_admin_session');
    if (!sessionToken) throw redirect(303, '/login');

    const { id } = params;
    
    if (id === 'new') {
        return {
            blog: {
                id: null,
                slug: '',
                title: '',
                title_ar: '',
                title_en: '',
                excerpt: '',
                content_html: '',
                content_markdown: '',
                cover_image_url: '',
                meta_title: '',
                meta_title_ar: '',
                meta_title_en: '',
                meta_description: '',
                meta_description_ar: '',
                meta_description_en: '',
                focus_keywords: '',
                read_time_minutes: 5,
                is_published: false,
                published_at: null,
                categories: [],
                tags: []
            },
            allCategories: [],
            allTags: []
        };
    }

    try {
        const [res, tagsRes, catsRes] = await Promise.all([
            fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/blogs/${id}`, {
                headers: { 'Authorization': `Bearer ${sessionToken}` }
            }),
            fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/tags`, {
                headers: { 'Authorization': `Bearer ${sessionToken}` }
            }),
            fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/categories`, {
                headers: { 'Authorization': `Bearer ${sessionToken}` }
            })
        ]);

        let allTags = [];
        let allCategories = [];

        if (tagsRes.ok) {
            const tData = await tagsRes.json();
            if (tData.status === 'success') allTags = tData.data;
        }

        if (catsRes.ok) {
            const cData = await catsRes.json();
            if (cData.status === 'success') allCategories = cData.data;
        }

        if (!res.ok) {
            return { error: 'Failed to load blog' };
        }
        
        const data = await res.json();
        if (data.status === 'success') {
            return { 
                blog: data.data,
                allTags,
                allCategories
            };
        }
        
        return { error: 'Failed to load blog', allTags, allCategories };
    } catch (e) {
        return { error: 'Network error' };
    }
};

export const actions: Actions = {
    save: async ({ request, cookies, params, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) throw redirect(303, '/login');

        const formData = await request.formData();
        const payload = {
            slug: formData.get('slug')?.toString() || '',
            title: formData.get('title')?.toString() || '',
            excerpt: formData.get('excerpt')?.toString() || '',
            content_html: formData.get('content_html')?.toString() || '',
            content_markdown: formData.get('content_markdown')?.toString() || '',
            cover_image_url: formData.get('cover_image_url')?.toString() || '',
            meta_title: formData.get('meta_title')?.toString() || '',
            meta_description: formData.get('meta_description')?.toString() || '',
            focus_keywords: formData.get('focus_keywords')?.toString() || '',
            read_time_minutes: parseInt(formData.get('read_time_minutes')?.toString() || '5'),
            is_published: formData.get('is_published') === 'on' || formData.get('is_published') === 'true',
            published_at: formData.get('published_at')?.toString() ? new Date(formData.get('published_at') as string).toISOString() : null,
            categories: formData.getAll('categories'),
            tags: formData.getAll('tags')
        };

        const { id } = params;
        const isNew = id === 'new';
        const url = isNew 
            ? `${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/blogs`
            : `${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/blogs/${id}`;

        try {
            const res = await fetch(url, {
                method: isNew ? 'POST' : 'PUT',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(payload)
            });

            if (!res.ok) {
                const err = await res.json();
                return { success: false, error: err.message || 'Failed to save blog post' };
            }

            const data = await res.json();
            if (isNew && data.data?.id) {
                throw redirect(303, `/dashboard/discover/edit/${data.data.id}?saved=true`);
            }

            return { success: true };
        } catch (e) {
            if (e && typeof e === 'object' && 'status' in e && 'location' in e) {
                throw e; // re-throw redirects
            }
            return { success: false, error: 'Network error' };
        }
    }
};
