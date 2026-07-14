import { apiClient } from '$lib/api/client';
import { fail, redirect } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ parent, fetch, cookies }) => {
    // Authenticate parent session
    const { user } = await parent();
    if (!user) {
        throw redirect(303, '/login');
    }

    const sessionToken = cookies.get('zafaf_vendor_session');

    try {
        const result = await apiClient.withFetch(fetch).vendor.getTemplates(sessionToken || '');

        if (result.success && result.data && result.data.status === 'success') {
            return {
                templates: result.data.templates
            };
        }
    } catch (err) {
        console.error('Failed to load templates in server loader:', err);
    }

    return {
        templates: []
    };
};

export const actions: Actions = {
    create: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_vendor_session');
        if (!sessionToken) {
            return fail(401, { error: 'Your session has expired.' });
        }

        const formData = await request.formData();
        const template_name = formData.get('template_name')?.toString().trim();
        const body_text_ar = formData.get('body_text_ar')?.toString().trim() || '';
        const body_text_en = formData.get('body_text_en')?.toString().trim() || '';

        if (!template_name) {
            return fail(400, { error: 'Template name is required.' });
        }

        try {
            const result = await apiClient.withFetch(fetch).vendor.createTemplate(sessionToken, {
                template_name,
                body_text_ar: body_text_ar || null,
                body_text_en: body_text_en || null
            });

            if (!result.success || !result.data || result.data.status !== 'success') {
                return fail(result.status || 400, { error: result.error?.message || 'Failed to create template.' });
            }

            return { success: true, message: 'Template created successfully!' };

        } catch (err) {
            console.error('Error creating template:', err);
            return fail(500, { error: 'Failed to communicate with backend.' });
        }
    },

    update: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_vendor_session');
        if (!sessionToken) {
            return fail(401, { error: 'Your session has expired.' });
        }

        const formData = await request.formData();
        const id = formData.get('id')?.toString();
        const template_name = formData.get('template_name')?.toString().trim();
        const body_text_ar = formData.get('body_text_ar')?.toString().trim() || '';
        const body_text_en = formData.get('body_text_en')?.toString().trim() || '';

        if (!id || !template_name) {
            return fail(400, { error: 'Template ID and name are required.' });
        }

        try {
            const result = await apiClient.withFetch(fetch).vendor.updateTemplate(sessionToken, id, {
                template_name,
                body_text_ar: body_text_ar || null,
                body_text_en: body_text_en || null
            });

            if (!result.success || !result.data || result.data.status !== 'success') {
                return fail(result.status || 400, { error: result.error?.message || 'Failed to update template.' });
            }

            return { success: true, message: 'Template updated successfully!' };

        } catch (err) {
            console.error('Error updating template:', err);
            return fail(500, { error: 'Failed to update template details.' });
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
            return fail(400, { error: 'Template ID is missing.' });
        }

        try {
            const result = await apiClient.withFetch(fetch).vendor.deleteTemplate(sessionToken, id);

            if (!result.success || !result.data || result.data.status !== 'success') {
                return fail(result.status || 400, { error: result.error?.message || 'Failed to delete template.' });
            }

            return { success: true, message: 'Template deleted successfully!' };

        } catch (err) {
            console.error('Error deleting template:', err);
            return fail(500, { error: 'Failed to delete template.' });
        }
    }
};
