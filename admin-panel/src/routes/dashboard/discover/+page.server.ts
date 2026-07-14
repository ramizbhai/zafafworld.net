import { env } from "$env/dynamic/public";
import { redirect } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch, url }) => {
    const sessionToken = cookies.get('zafaf_admin_session');

    if (!sessionToken) {
        throw redirect(303, '/login');
    }

    const page = url.searchParams.get('page') || '1';

    try {
        const queryParams = new URLSearchParams();
        queryParams.set('page', page);
        queryParams.set('limit', '20');
        
        const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/blogs?${queryParams.toString()}`, {
            headers: {
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_admin_session=${sessionToken}`
            }
        });

        // Fixed route mismatch: change /blogs/analytics to /analytics/discover
        const analyticsResponse = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/analytics/discover`, {
            headers: { 'Authorization': `Bearer ${sessionToken}` }
        });

        // Fetch comments list for moderation
        const commentsResponse = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/comments`, {
            headers: { 'Authorization': `Bearer ${sessionToken}` }
        });

        if (!response.ok) {
            if (response.status === 401 || response.status === 403) {
                cookies.delete('zafaf_admin_session', { path: '/' });
                throw redirect(303, '/login');
            }
            return { blogs: [], total: 0, page: 1, totalPages: 1, error: 'Failed to load discover blogs list', comments: [] };
        }

        const data = await response.json();
        if (data.status === 'success') {
            const limit = parseInt(data.limit || 20);
            const total = parseInt(data.total || 0);
            
            let analytics = { total_views: 0, pending_comments: 0 };
            if (analyticsResponse.ok) {
                const aData = await analyticsResponse.json();
                if (aData.status === 'success') {
                    analytics = aData.data;
                }
            }

            let comments = [];
            if (commentsResponse.ok) {
                const cData = await commentsResponse.json();
                if (cData.status === 'success') {
                    comments = cData.data || [];
                }
            }

            return {
                blogs: data.data || [],
                total: total,
                page: data.page || 1,
                totalPages: Math.ceil(total / limit) || 1,
                analytics,
                comments,
                publicClientUrl: env.PUBLIC_CLIENT_URL || 'https://zafafworld.net'
            };
        }

        return { blogs: [], total: 0, page: 1, totalPages: 1, error: data.message || 'Failed to parse blogs', comments: [] };

    } catch (err) {
        if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
            throw err;
        }
        console.error('Failed to fetch admin blogs list:', err);
        return { blogs: [], total: 0, page: 1, totalPages: 1, error: 'Internal connection error', comments: [] };
    }
};

export const actions: Actions = {
    deletePost: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) throw redirect(303, '/login');

        const formData = await request.formData();
        const id = formData.get('id')?.toString();

        if (!id) return { success: false, error: 'Post ID is required' };

        try {
            const res = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/blogs/${id}`, {
                method: 'DELETE',
                headers: { 'Authorization': `Bearer ${sessionToken}` }
            });

            if (!res.ok) {
                const err = await res.json();
                return { success: false, error: err.message || 'Failed to delete blog post' };
            }

            return { success: true, message: 'Blog post deleted successfully' };
        } catch (e) {
            return { success: false, error: 'Network error' };
        }
    },
    approveComment: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) throw redirect(303, '/login');

        const formData = await request.formData();
        const id = formData.get('id')?.toString();

        if (!id) return { success: false, error: 'Comment ID is required' };

        try {
            const res = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/comments/${id}/approve`, {
                method: 'PATCH',
                headers: { 'Authorization': `Bearer ${sessionToken}` }
            });

            if (!res.ok) {
                const err = await res.json();
                return { success: false, error: err.message || 'Failed to approve comment' };
            }

            return { success: true, message: 'Comment approved successfully' };
        } catch (e) {
            return { success: false, error: 'Network error' };
        }
    },
    deleteComment: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) throw redirect(303, '/login');

        const formData = await request.formData();
        const id = formData.get('id')?.toString();

        if (!id) return { success: false, error: 'Comment ID is required' };

        try {
            const res = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/comments/${id}/reject`, {
                method: 'PATCH',
                headers: { 'Authorization': `Bearer ${sessionToken}` }
            });

            if (!res.ok) {
                const err = await res.json();
                return { success: false, error: err.message || 'Failed to delete comment' };
            }

            return { success: true, message: 'Comment deleted successfully' };
        } catch (e) {
            return { success: false, error: 'Network error' };
        }
    },
    replyComment: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) throw redirect(303, '/login');

        const formData = await request.formData();
        const slug = formData.get('slug')?.toString();
        const parentId = formData.get('parentId')?.toString();
        const comment = formData.get('comment')?.toString();

        if (!slug || !parentId || !comment) {
            return { success: false, error: 'Missing required fields' };
        }

        try {
            const res = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/public/blogs/${slug}/comments`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ comment, parent_id: parentId })
            });

            if (!res.ok) {
                const err = await res.json();
                return { success: false, error: err.message || 'Failed to post reply' };
            }

            return { success: true, message: 'Reply posted successfully' };
        } catch (e) {
            return { success: false, error: 'Network error' };
        }
    },
    togglePublish: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) throw redirect(303, '/login');

        const formData = await request.formData();
        const id = formData.get('id')?.toString();

        if (!id) return { success: false, error: 'Post ID is required' };

        try {
            // 1. Fetch current post details
            const getRes = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/blogs/${id}`, {
                headers: { 'Authorization': `Bearer ${sessionToken}` }
            });

            if (!getRes.ok) {
                return { success: false, error: 'Failed to fetch blog post' };
            }

            const getData = await getRes.json();
            if (getData.status !== 'success' || !getData.data) {
                return { success: false, error: 'Failed to fetch blog post details' };
            }

            const blog = getData.data;

            // 2. Toggle published state
            const newIsPublished = !blog.is_published;

            // 3. Prepare payload for PUT update
            const payload = {
                slug: blog.slug,
                title: blog.title_en || blog.title || '',
                excerpt: blog.excerpt || '',
                content_html: blog.content_html || '',
                content_markdown: blog.content_markdown || '',
                cover_image_url: blog.cover_image_url || '',
                meta_title: blog.meta_title || '',
                meta_description: blog.meta_description || '',
                focus_keywords: blog.focus_keywords || '',
                read_time_minutes: blog.read_time_minutes || 5,
                is_published: newIsPublished,
                published_at: newIsPublished ? new Date().toISOString() : null,
                categories: blog.categories || [],
                tags: blog.tags || []
            };

            // 4. Update post via PUT
            const putRes = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/blogs/${id}`, {
                method: 'PUT',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(payload)
            });

            if (!putRes.ok) {
                const err = await putRes.json();
                return { success: false, error: err.message || 'Failed to update publish status' };
            }

            const actionMsg = newIsPublished ? 'Blog post published successfully' : 'Blog post reverted to draft';
            return { success: true, message: actionMsg };
        } catch (e) {
            console.error('Error toggling publish status:', e);
            return { success: false, error: 'Network error' };
        }
    }
};
