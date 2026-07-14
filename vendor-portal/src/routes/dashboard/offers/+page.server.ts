import { env } from "$env/dynamic/public";
import { fail, redirect } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ parent, fetch, cookies, url }) => {
    const sessionToken = cookies.get('zafaf_vendor_session');

    const { user } = await parent();
    if (!user) {
        throw redirect(303, '/login');
    }

    // Read filter params from URL
    const status = url.searchParams.get('status') || undefined;
    const search = url.searchParams.get('search') || undefined;
    const page = parseInt(url.searchParams.get('page') || '1') || 1;
    const limit = 12;

    // Build API query string
    const params = new URLSearchParams();
    if (status) params.set('status', status);
    if (search) params.set('search', search);
    params.set('page', page.toString());
    params.set('limit', limit.toString());
    params.set('sort_by', 'created_at');
    params.set('sort_order', 'desc');

    // Fetch promotions and products concurrently
    const promotionsPromise = fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/vendor/promotions?${params.toString()}`, {
        headers: {
            'Authorization': `Bearer ${sessionToken}`,
            'Cookie': `zafaf_vendor_session=${sessionToken}`
        }
    });

    const productsPromise = fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/vendor/products`, {
        headers: {
            'Authorization': `Bearer ${sessionToken}`,
            'Cookie': `zafaf_vendor_session=${sessionToken}`
        }
    });

    let offers: any[] = [];
    let products: any[] = [];
    let total = 0;

    try {
        const [promoResponse, productsResponse] = await Promise.all([promotionsPromise, productsPromise]);
        
        if (promoResponse.ok) {
            const data = await promoResponse.json();
            if (data.status === 'success') {
                offers = data.promotions || [];
                total = data.total || offers.length;
            }
        }
        
        if (productsResponse.ok) {
            const data = await productsResponse.json();
            products = data.products || [];
        }
    } catch (err) {
        console.error('Failed to load promotions or products in server loader:', err);
    }

    return {
        offers,
        products,
        total,
        filters: {
            status: status || '',
            search: search || '',
            page,
            limit
        }
    };
};

export const actions: Actions = {
    delete: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_vendor_session');
        if (!sessionToken) {
            return fail(401, { error: 'Your session has expired.' });
        }

        const formData = await request.formData();
        const id = formData.get('id')?.toString();

        if (!id) {
            return fail(400, { error: 'Promotion ID is missing.' });
        }

        try {
            const apiResponse = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/vendor/promotions/${id}`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_vendor_session=${sessionToken}`
                }
            });

            const responseData = await apiResponse.json();

            if (!apiResponse.ok || responseData.status !== 'success') {
                return fail(apiResponse.status || 400, { 
                    error: responseData.message || 'Failed to delete/cancel promotion.' 
                });
            }

            return { success: true, message: 'Promotion cancelled successfully!' };

        } catch (err) {
            console.error('Connection error deleting promotion:', err);
            return fail(500, { error: 'Unable to communicate with the database server.' });
        }
    },

    pause: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_vendor_session');
        if (!sessionToken) {
            return fail(401, { error: 'Your session has expired.' });
        }

        const formData = await request.formData();
        const id = formData.get('id')?.toString();

        if (!id) {
            return fail(400, { error: 'Promotion ID is missing.' });
        }

        try {
            const apiResponse = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/vendor/promotions/${id}/pause`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_vendor_session=${sessionToken}`
                }
            });

            const responseData = await apiResponse.json();

            if (!apiResponse.ok || responseData.status !== 'success') {
                return fail(apiResponse.status || 400, { 
                    error: responseData.message || 'Failed to pause promotion.' 
                });
            }

            return { success: true, message: 'Promotion paused successfully!' };

        } catch (err) {
            console.error('Connection error pausing promotion:', err);
            return fail(500, { error: 'Unable to communicate with the database server.' });
        }
    },

    resume: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_vendor_session');
        if (!sessionToken) {
            return fail(401, { error: 'Your session has expired.' });
        }

        const formData = await request.formData();
        const id = formData.get('id')?.toString();

        if (!id) {
            return fail(400, { error: 'Promotion ID is missing.' });
        }

        try {
            const apiResponse = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/vendor/promotions/${id}/resume`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_vendor_session=${sessionToken}`
                }
            });

            const responseData = await apiResponse.json();

            if (!apiResponse.ok || responseData.status !== 'success') {
                return fail(apiResponse.status || 400, { 
                    error: responseData.message || 'Failed to resume promotion.' 
                });
            }

            return { success: true, message: 'Promotion resumed successfully!' };

        } catch (err) {
            console.error('Connection error resuming promotion:', err);
            return fail(500, { error: 'Unable to communicate with the database server.' });
        }
    },

    duplicate: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_vendor_session');
        if (!sessionToken) {
            return fail(401, { error: 'Your session has expired.' });
        }

        const formData = await request.formData();
        const id = formData.get('id')?.toString();

        if (!id) {
            return fail(400, { error: 'Promotion ID is missing.' });
        }

        try {
            const apiResponse = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/vendor/promotions/${id}/duplicate`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_vendor_session=${sessionToken}`
                }
            });

            const responseData = await apiResponse.json();

            if (!apiResponse.ok || responseData.status !== 'success') {
                return fail(apiResponse.status || 400, { 
                    error: responseData.message || 'Failed to duplicate promotion.' 
                });
            }

            return { success: true, message: 'Promotion duplicated successfully as a draft!', duplicatedId: responseData.promotion_id };

        } catch (err) {
            console.error('Connection error duplicating promotion:', err);
            return fail(500, { error: 'Unable to communicate with the database server.' });
        }
    },

    renew: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_vendor_session');
        if (!sessionToken) {
            return fail(401, { error: 'Your session has expired.' });
        }

        const formData = await request.formData();
        const id = formData.get('id')?.toString();
        const days = parseInt(formData.get('days')?.toString() || '14') || 14;

        if (!id) {
            return fail(400, { error: 'Promotion ID is missing.' });
        }

        try {
            const apiResponse = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/vendor/promotions/${id}/renew`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_vendor_session=${sessionToken}`
                },
                body: JSON.stringify({ days })
            });

            const responseData = await apiResponse.json();

            if (!apiResponse.ok || responseData.status !== 'success') {
                return fail(apiResponse.status || 400, { 
                    error: responseData.message || 'Failed to renew promotion.' 
                });
            }

            return { success: true, message: 'Promotion renewed and submitted for review successfully!' };

        } catch (err) {
            console.error('Connection error renewing promotion:', err);
            return fail(500, { error: 'Unable to communicate with the database server.' });
        }
    }
};
