import { env } from "$env/dynamic/public";
import { redirect } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ fetch, cookies, params, url, depends }) => {
    depends('app:vendor-products');
    const sessionToken = cookies.get('zafaf_vendor_session');
    if (!sessionToken) throw redirect(303, '/login');

    const productId = params.id;
    const justCreated = url.searchParams.get('created') === 'true';

    // Fetch aggregated context from the BFF endpoint
    const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/vendor/products/${productId}/edit-context`, {
        headers: {
            'Authorization': `Bearer ${sessionToken}`,
            'Cookie': `zafaf_vendor_session=${sessionToken}`
        }
    });

    if (!response.ok) {
        if (response.status === 401 || response.status === 403) {
            throw redirect(303, '/login');
        }
        throw redirect(303, '/dashboard/products');
    }

    const data = await response.json();
    if (data.status !== 'success' || !data.product) {
        throw redirect(303, '/dashboard/products');
    }

    let cities = data.cities || [];
    let vendorCategory = data.vendorCategory || '';
    let listingImages = data.images || [];

    return {
        product: data.product,
        cities,
        vendorCategory,
        listingImages,
        justCreated,
        sessionToken
    };
};


