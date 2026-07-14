import { env } from "$env/dynamic/public";
import { fail, redirect } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ fetch, cookies }) => {
    const sessionToken = cookies.get('zafaf_admin_session');

    if (!sessionToken) {
        throw redirect(303, '/login');
    }

    try {
        const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/promotions`, {
            headers: {
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_admin_session=${sessionToken}`
            }
        });

        if (response.ok) {
            const data = await response.json();
            if (data.status === 'success') {
                return {
                    campaigns: data.promotions || []
                };
            }
        }
    } catch (err) {
        console.error('Failed to load admin promotions:', err);
    }

    return {
        campaigns: []
    };
};

export const actions: Actions = {
    approve: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) {
            return fail(401, { error: 'Your session has expired. Please sign in again.' });
        }

        const formData = await request.formData();
        const id = formData.get('id')?.toString();

        if (!id) {
            return fail(400, { error: 'Promotion ID is required.' });
        }

        try {
            const apiResponse = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/promotions/${id}/approve`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`
                }
            });

            const data = await apiResponse.json();

            if (!apiResponse.ok || data.status !== 'success') {
                return fail(apiResponse.status || 400, { error: data.message || 'Failed to approve promotion.' });
            }

            return { success: true, message: 'Promotion approved successfully!' };

        } catch (err) {
            console.error('Error approving promotion:', err);
            return fail(500, { error: 'Unable to communicate with the database server.' });
        }
    },

    reject: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) {
            return fail(401, { error: 'Your session has expired. Please sign in again.' });
        }

        const formData = await request.formData();
        const id = formData.get('id')?.toString();
        const reason = formData.get('rejection_reason')?.toString();

        if (!id) {
            return fail(400, { error: 'Promotion ID is required.' });
        }
        if (!reason || reason.trim() === '') {
            return fail(400, { error: 'Rejection reason is required.' });
        }

        try {
            const apiResponse = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/promotions/${id}/reject`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`
                },
                body: JSON.stringify({
                    rejection_reason: reason.trim()
                })
            });

            const data = await apiResponse.json();

            if (!apiResponse.ok || data.status !== 'success') {
                return fail(apiResponse.status || 400, { error: data.message || 'Failed to reject promotion.' });
            }

            return { success: true, message: 'Promotion rejected successfully!' };

        } catch (err) {
            console.error('Error rejecting promotion:', err);
            return fail(500, { error: 'Unable to communicate with the database server.' });
        }
    },

    bulkApprove: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) {
            return fail(401, { error: 'Your session has expired. Please sign in again.' });
        }

        const formData = await request.formData();
        const idsString = formData.get('ids')?.toString();
        if (!idsString) {
            return fail(400, { error: 'No promotions selected.' });
        }

        const ids = idsString.split(',').map(x => x.trim()).filter(Boolean);

        let successCount = 0;
        let errors = [];

        for (const id of ids) {
            try {
                const apiResponse = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/promotions/${id}/approve`, {
                    method: 'POST',
                    headers: {
                        'Authorization': `Bearer ${sessionToken}`,
                        'Cookie': `zafaf_admin_session=${sessionToken}`
                    }
                });
                if (apiResponse.ok) {
                    successCount++;
                } else {
                    const data = await apiResponse.json();
                    errors.push(`${id}: ${data.message || 'Error'}`);
                }
            } catch (err) {
                errors.push(`${id}: Connection failed`);
            }
        }

        return { 
            success: true, 
            message: `Successfully approved ${successCount} of ${ids.length} promotions.`, 
            errors: errors.length > 0 ? errors : undefined 
        };
    },

    bulkReject: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) {
            return fail(401, { error: 'Your session has expired. Please sign in again.' });
        }

        const formData = await request.formData();
        const idsString = formData.get('ids')?.toString();
        const reason = formData.get('rejection_reason')?.toString();

        if (!idsString) {
            return fail(400, { error: 'No promotions selected.' });
        }
        if (!reason || reason.trim() === '') {
            return fail(400, { error: 'Rejection reason is required for bulk rejection.' });
        }

        const ids = idsString.split(',').map(x => x.trim()).filter(Boolean);

        let successCount = 0;
        let errors = [];

        for (const id of ids) {
            try {
                const apiResponse = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/promotions/${id}/reject`, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                        'Authorization': `Bearer ${sessionToken}`,
                        'Cookie': `zafaf_admin_session=${sessionToken}`
                    },
                    body: JSON.stringify({
                        rejection_reason: reason.trim()
                    })
                });
                if (apiResponse.ok) {
                    successCount++;
                } else {
                    const data = await apiResponse.json();
                    errors.push(`${id}: ${data.message || 'Error'}`);
                }
            } catch (err) {
                errors.push(`${id}: Connection failed`);
            }
        }

        return { 
            success: true, 
            message: `Successfully rejected ${successCount} of ${ids.length} promotions.`, 
            errors: errors.length > 0 ? errors : undefined 
        };
    }
};
