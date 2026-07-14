import { env } from "$env/dynamic/public";
import { fail, redirect } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ parent, fetch, cookies }) => {
    // Authenticate through parent layout
    const { user } = await parent();
    if (!user) {
        throw redirect(303, '/login');
    }

    const sessionToken = cookies.get('zafaf_vendor_session');

    try {
        const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/vendor/vendor_inquiries`, {
            headers: {
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_vendor_session=${sessionToken}`
            }
        });

        if (response.ok) {
            const data = await response.json();
            if (data.status === 'success') {
                return {
                    inquiries: data.inquiries
                };
            }
        }
    } catch (err) {
        console.error('Failed to load direct vendor inquiries in server loader:', err);
    }

    return {
        inquiries: []
    };
};

export const actions: Actions = {
    updateStatus: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_vendor_session');
        if (!sessionToken) {
            return fail(401, { error: 'Your session has expired. Please sign in again.' });
        }

        const formData = await request.formData();
        const id = formData.get('id')?.toString();
        const status = formData.get('status')?.toString();

        if (!id || !status) {
            return fail(400, { error: 'Inquiry ID or target status is missing.' });
        }

        try {
            const apiResponse = await fetch(`${env.PUBLIC_API_URL || `${env.PUBLIC_API_URL || 'http://localhost:8080'}`}/api/v1/vendor/vendor_inquiries/${id}/status`, {
                method: 'PATCH',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_vendor_session=${sessionToken}`
                },
                body: JSON.stringify({ status })
            });

            const data = await apiResponse.json();

            if (!apiResponse.ok || data.status !== 'success') {
                return fail(apiResponse.status || 400, { error: data.message || 'Failed to update lead status.' });
            }

            return { success: true, message: 'Lead status successfully updated.' };

        } catch (err) {
            console.error('Connection error updating inquiry state:', err);
            return fail(500, { error: 'Unable to communicate with the database server.' });
        }
    }
};
