import { env } from "$env/dynamic/public";
import { fail, redirect } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ parent, fetch, cookies, url }) => {
    // Authenticate through parent layout
    const { user } = await parent();
    if (!user) {
        throw redirect(303, '/login');
    }

    const sessionToken = cookies.get('zafaf_vendor_session');

    // Server-Side Pagination Prep
    const params = new URLSearchParams();
    if (url.searchParams.get('status')) params.set('status', url.searchParams.get('status')!);
    if (url.searchParams.get('page')) params.set('page', url.searchParams.get('page')!);
    if (url.searchParams.get('sort')) params.set('sort', url.searchParams.get('sort')!);
    if (url.searchParams.get('order')) params.set('order', url.searchParams.get('order')!);
    if (url.searchParams.get('q')) params.set('q', url.searchParams.get('q')!);
    
    const queryString = params.toString();

    try {
        const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/vendor/inquiries${queryString ? '?' + queryString : ''}`, {
            headers: {
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_vendor_session=${sessionToken}`
            }
        });

        if (response.ok) {
            const data = await response.json();
            if (data.status === 'success') {
                return {
                    inquiries: data.inquiries || [],
                    totalCount: data.totalCount || (data.inquiries ? data.inquiries.length : 0),
                    totalPages: data.totalPages || 1
                };
            }
        }
    } catch (err) {
        console.error('Failed to load B2B inquiries in server loader:', err);
    }

    return {
        inquiries: [],
        totalCount: 0,
        totalPages: 1
    };
};

export const actions: Actions = {
    markAsRead: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_vendor_session');
        if (!sessionToken) {
            return fail(401, { error: 'Your session has expired. Please sign in again.' });
        }

        const formData = await request.formData();
        const id = formData.get('id')?.toString();

        if (!id) {
            return fail(400, { error: 'Inquiry ID payload is missing.' });
        }

        try {
            const apiResponse = await fetch(`${env.PUBLIC_API_URL || `${env.PUBLIC_API_URL || 'http://localhost:8080'}`}/api/v1/vendor/inquiries/${id}/read`, {
                method: 'PATCH',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_vendor_session=${sessionToken}`
                }
            });

            const data = await apiResponse.json();

            if (!apiResponse.ok || data.status !== 'success') {
                return fail(apiResponse.status || 400, { error: data.message || 'Failed to update lead status.' });
            }

            return { success: true, message: 'Lead successfully marked as read.' };

        } catch (err) {
            console.error('Connection error updating inquiry state:', err);
            return fail(500, { error: 'Unable to communicate with the database server.' });
        }
    },
    addCouple: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_vendor_session');
        if (!sessionToken) {
            return fail(401, { error: 'Your session has expired. Please sign in again.' });
        }

        const formData = await request.formData();
        const customer_name = formData.get('customer_name')?.toString();
        const phone = formData.get('phone')?.toString();
        const wedding_date = formData.get('wedding_date')?.toString();
        const message = formData.get('message')?.toString() || '';

        if (!customer_name || !phone || !wedding_date) {
            return fail(400, { error: 'All fields (name, phone, event date) are required.' });
        }

        const transactionNonceToken = crypto.randomUUID();

        try {
            const apiResponse = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/vendor/inquiries`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_vendor_session=${sessionToken}`,
                    'Idempotency-Key': transactionNonceToken
                },
                body: JSON.stringify({
                    customer_name,
                    phone,
                    wedding_date,
                    message
                })
            });

            const data = await apiResponse.json();

            if (!apiResponse.ok || data.status !== 'success') {
                return fail(apiResponse.status || 400, { error: data.message || 'Failed to create inquiry.' });
            }

            return { success: true, message: 'Lead successfully created.' };

        } catch (err) {
            console.error('Connection error adding inquiry:', err);
            return fail(500, { error: 'Unable to communicate with the database server.' });
        }
    }
};
