import { env } from "$env/dynamic/public";
import { redirect, fail } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ params, cookies, fetch }) => {
    const sessionToken = cookies.get('zafaf_admin_session');
    if (!sessionToken) throw redirect(303, '/login');

    const { id } = params;

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
            return { error: 'Failed to load blog post from server', allTags, allCategories };
        }
        
        const data = await res.json();
        if (data.status === 'success') {
            // Ensure categories and tags are initialized as arrays
            const blog = data.data;
            if (!blog.categories) blog.categories = [];
            if (!blog.tags) blog.tags = [];

            return { 
                blog,
                allTags,
                allCategories
            };
        }
        
        return { error: 'Failed to load blog post details', allTags, allCategories };
    } catch (e) {
        console.error(e);
        return { error: 'Network connection error' };
    }
};

export const actions: Actions = {
    save: async ({ request, cookies, params, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) throw redirect(303, '/login');

        const { id } = params;
        const formData = await request.formData();
        
        const payload = {
            slug: formData.get('slug')?.toString() || '',
            title: formData.get('title')?.toString() || '',
            title_ar: formData.get('title_ar')?.toString() || '',
            title_en: formData.get('title_en')?.toString() || '',
            excerpt: formData.get('excerpt')?.toString() || '',
            content_html: formData.get('content_html')?.toString() || '',
            content_markdown: formData.get('content_markdown')?.toString() || '',
            cover_image_url: formData.get('cover_image_url')?.toString() || '',
            meta_title: formData.get('meta_title')?.toString() || '',
            meta_title_ar: formData.get('meta_title_ar')?.toString() || '',
            meta_title_en: formData.get('meta_title_en')?.toString() || '',
            meta_description: formData.get('meta_description')?.toString() || '',
            meta_description_ar: formData.get('meta_description_ar')?.toString() || '',
            meta_description_en: formData.get('meta_description_en')?.toString() || '',
            focus_keywords: formData.get('focus_keywords')?.toString() || '',
            read_time_minutes: parseInt(formData.get('read_time_minutes')?.toString() || '5'),
            is_published: formData.get('is_published') === 'true' || formData.get('is_published') === 'on',
            published_at: formData.get('published_at')?.toString() ? new Date(formData.get('published_at') as string).toISOString() : null,
            categories: formData.getAll('categories'),
            tags: formData.getAll('tags'),
            canonical_url: formData.get('canonical_url')?.toString() || ''
        };

        try {
            const res = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/blogs/${id}`, {
                method: 'PUT',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Content-Type': 'application/json',
                    'Cookie': `zafaf_admin_session=${sessionToken}`
                },
                body: JSON.stringify(payload)
            });

            if (!res.ok) {
                const err = await res.json();
                return fail(res.status, { error: err.message || 'Failed to update blog post' });
            }

            throw redirect(303, '/dashboard/blog');
        } catch (e) {
            if (e && typeof e === 'object' && 'status' in e && 'location' in e) {
                throw e; // Propagate redirect
            }
            console.error('Update blog post error:', e);
            return fail(500, { error: 'Network connection error. Failed to save post.' });
        }
    }
};
