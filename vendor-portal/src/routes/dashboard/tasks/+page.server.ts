import { apiClient } from '$lib/api/client';
import { fail, redirect } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ parent, fetch, cookies }) => {
    const sessionToken = cookies.get('zafaf_vendor_session');

    // Fetch tasks using apiClient
    const tasksPromise = apiClient.withFetch(fetch).vendor.getTasks(sessionToken || '');

    // Authenticate parent session
    const { user } = await parent();
    if (!user) {
        throw redirect(303, '/login');
    }

    try {
        const result = await tasksPromise;
        if (result.success && result.data && result.data.status === 'success') {
            return {
                tasks: result.data.tasks
            };
        }
    } catch (err) {
        console.error('Failed to load tasks in server loader:', err);
    }

    return {
        tasks: []
    };
};

export const actions: Actions = {
    create: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_vendor_session');
        if (!sessionToken) {
            return fail(401, { error: 'Your session has expired.' });
        }

        const formData = await request.formData();
        const title = formData.get('title')?.toString().trim();
        const category = formData.get('category')?.toString().trim() || 'عام';

        if (!title) {
            return fail(400, { error: 'Task title is required.' });
        }

        // Map categories for LTR/RTL title prefixing
        const categoryMapEn: Record<string, string> = {
            'عام': 'General',
            'تزيين': 'Decoration',
            'ضيافة': 'Hospitality',
            'صوتيات': 'Audio',
            'حسابات': 'Finance',
            'تواصل': 'Communication'
        };

        const categoryEn = categoryMapEn[category] || 'General';
        const title_ar = `[${category}] ${title}`;
        const title_en = `[${categoryEn}] ${title}`;

        try {
            const result = await apiClient.withFetch(fetch).vendor.createTask(sessionToken, {
                title_ar,
                title_en,
                due_date: null
            });

            if (!result.success || !result.data || result.data.status !== 'success') {
                return fail(result.status || 400, { error: result.error?.message || 'Failed to create task.' });
            }

            return { success: true, message: 'Task created successfully!' };

        } catch (err) {
            console.error('Error creating task:', err);
            return fail(500, { error: 'Failed to communicate with backend.' });
        }
    },

    toggle: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_vendor_session');
        if (!sessionToken) {
            return fail(401, { error: 'Your session has expired.' });
        }

        const formData = await request.formData();
        const id = formData.get('id')?.toString();
        const title_ar = formData.get('title_ar')?.toString();
        const title_en = formData.get('title_en')?.toString();
        const is_completed = formData.get('is_completed')?.toString() === 'true';

        if (!id || !title_ar || !title_en) {
            return fail(400, { error: 'Task ID and title parameters are required.' });
        }

        try {
            const result = await apiClient.withFetch(fetch).vendor.updateTask(sessionToken, id, {
                title_ar,
                title_en,
                is_completed,
                due_date: null
            });

            if (!result.success || !result.data || result.data.status !== 'success') {
                return fail(result.status || 400, { error: result.error?.message || 'Failed to toggle task.' });
            }

            return { success: true };

        } catch (err) {
            console.error('Error toggling task:', err);
            return fail(500, { error: 'Failed to update task state.' });
        }
    },

    delete: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_vendor_session');
        if (!sessionToken) {
            return fail(401, { error: 'Your session has expired.' });
        }

        const formData = await request.formData();
        const id = formData.get('id')?.toString();

        if (!id) {
            return fail(400, { error: 'Task ID is missing.' });
        }

        try {
            const result = await apiClient.withFetch(fetch).vendor.deleteTask(sessionToken, id);

            if (!result.success || !result.data || result.data.status !== 'success') {
                return fail(result.status || 400, { error: result.error?.message || 'Failed to delete task.' });
            }

            return { success: true, message: 'Task deleted successfully!' };

        } catch (err) {
            console.error('Error deleting task:', err);
            return fail(500, { error: 'Failed to delete task.' });
        }
    }
};
