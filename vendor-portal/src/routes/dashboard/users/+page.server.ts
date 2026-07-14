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
        const result = await apiClient.withFetch(fetch).vendor.getStaff(sessionToken || '');

        if (result.success && result.data && result.data.status === 'success') {
            return {
                staff: result.data.staff || []
            };
        }
    } catch (err) {
        console.error('Failed to load staff in server loader:', err);
    }

    return {
        staff: []
    };
};

export const actions: Actions = {
    add: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_vendor_session');
        if (!sessionToken) {
            return fail(401, { error: 'Your session has expired.' });
        }

        const formData = await request.formData();
        const name = formData.get('name')?.toString().trim();
        const email = formData.get('email')?.toString().trim();
        const role = formData.get('role')?.toString().trim();
        const status = formData.get('status')?.toString().trim() || 'active';

        if (!name || !email || !role) {
            return fail(400, { error: 'Name, email, and role are required.' });
        }

        try {
            const result = await apiClient.withFetch(fetch).vendor.createStaff(sessionToken, {
                name,
                email,
                role,
                status
            });

            if (!result.success || !result.data || result.data.status !== 'success') {
                return fail(result.status || 400, { error: result.error?.message || 'Failed to create staff member.' });
            }

            return { success: true, message: 'Staff member created successfully!' };

        } catch (err) {
            console.error('Error creating staff member:', err);
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
        const name = formData.get('name')?.toString().trim();
        const email = formData.get('email')?.toString().trim();
        const role = formData.get('role')?.toString().trim();
        const status = formData.get('status')?.toString().trim() || 'active';

        if (!id || !name || !email || !role) {
            return fail(400, { error: 'ID, name, email, and role are required.' });
        }

        try {
            const result = await apiClient.withFetch(fetch).vendor.updateStaff(sessionToken, id, {
                name,
                email,
                role,
                status
            });

            if (!result.success || !result.data || result.data.status !== 'success') {
                return fail(result.status || 400, { error: result.error?.message || 'Failed to update staff member.' });
            }

            return { success: true, message: 'Staff member updated successfully!' };

        } catch (err) {
            console.error('Error updating staff member:', err);
            return fail(500, { error: 'Failed to update staff details.' });
        }
    },

    toggleStatus: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_vendor_session');
        if (!sessionToken) {
            return fail(401, { error: 'Your session has expired.' });
        }

        const formData = await request.formData();
        const id = formData.get('id')?.toString();
        const status = formData.get('status')?.toString();

        if (!id || !status) {
            return fail(400, { error: 'Staff ID and status are required.' });
        }

        try {
            const result = await apiClient.withFetch(fetch).vendor.updateStaffStatus(sessionToken, id, status);

            if (!result.success || !result.data || result.data.status !== 'success') {
                return fail(result.status || 400, { error: result.error?.message || 'Failed to toggle status.' });
            }

            return { success: true, message: 'Staff status updated successfully!' };

        } catch (err) {
            console.error('Error toggling staff status:', err);
            return fail(500, { error: 'Failed to update status.' });
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
            return fail(400, { error: 'Staff ID is missing.' });
        }

        try {
            const result = await apiClient.withFetch(fetch).vendor.deleteStaff(sessionToken, id);

            if (!result.success || !result.data || result.data.status !== 'success') {
                return fail(result.status || 400, { error: result.error?.message || 'Failed to delete staff member.' });
            }

            return { success: true, message: 'Staff member deleted successfully!' };

        } catch (err) {
            console.error('Error deleting staff member:', err);
            return fail(500, { error: 'Failed to delete staff member.' });
        }
    }
};
