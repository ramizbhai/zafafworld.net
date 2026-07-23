import { env } from "$env/dynamic/public";
import { fail, redirect } from '@sveltejs/kit';
import { safeFetch } from '$lib/utils/api';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ fetch, cookies, depends }) => {
    console.log("ENTER products +page.server.ts");
    depends('app:vendor-products');
    const sessionToken = cookies.get('zafaf_vendor_session');

    if (!sessionToken) {
        console.log("products +page.server.ts REDIRECT: No session token found. Redirecting to /login");
        throw redirect(303, '/login');
    }

    const headers = {
        'Authorization': `Bearer ${sessionToken}`,
        'Cookie': `zafaf_vendor_session=${sessionToken}`
    };

    try {
        const productsUrl = `${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/vendor/products`;
        const promosUrl = `${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/vendor/promotions?limit=100`;
        console.log(`products +page.server.ts: Fetching products from: ${productsUrl} and promos from: ${promosUrl}`);

        // Fetch products and promotions in parallel
        const [productsResponse, promosResponse] = await Promise.all([
            fetch(productsUrl, { headers }),
            fetch(promosUrl, { headers }).catch(err => {
                console.error("products +page.server.ts promos fetch CRASHED:", err);
                return null;
            })
        ]);

        console.log(`products +page.server.ts: products response status=${productsResponse.status}, promos response status=${promosResponse ? promosResponse.status : 'FAILED'}`);

        if (!productsResponse.ok) {
            console.error(`products +page.server.ts error: Products fetch failed with status=${productsResponse.status}`);
            if (productsResponse.status === 401 || productsResponse.status === 403) {
                console.log("products +page.server.ts REDIRECT: Unauthorized/Forbidden. Redirecting to /login");
                throw redirect(303, '/login');
            }
            const resObj = {
                products: [],
                total: 0,
                promoMap: {},
                error: 'Encountered a server error or backend is offline.'
            };
            console.log("EXIT products +page.server.ts (fetch failed, returned fallback)");
            return resObj;
        }

        const productsData = await productsResponse.json();
        console.log(`products +page.server.ts: Products data parsed successfully. Found ${productsData?.products?.length ?? 0} products.`);

        // Build a map of listing_id → promo info for cross-linking
        let promoMap: Record<string, { status: string; discount: number; promoId: string }> = {};
        if (promosResponse && promosResponse.ok) {
            const promosData = await promosResponse.json();
            const promos = promosData.promotions || [];
            for (const promo of promos) {
                // Only show Active, Scheduled, or Pending promos
                if (!['Active', 'Scheduled', 'Pending'].includes(promo.derived_status)) continue;
                for (const listingId of (promo.listing_ids || [])) {
                    // Prefer Active > Scheduled > Pending in display priority
                    const existing = promoMap[listingId];
                    const priority: Record<string, number> = { Active: 3, Scheduled: 2, Pending: 1 };
                    if (!existing || (priority[promo.derived_status] || 0) > (priority[existing.status] || 0)) {
                        promoMap[listingId] = {
                            status: promo.derived_status,
                            discount: promo.discount_percentage,
                            promoId: promo.id
                        };
                    }
                }
            }
        }

        const resObj = {
            products: productsData.products ?? [],
            total: productsData.total ?? 0,
            promoMap
        };
        console.log("EXIT products +page.server.ts (success)");
        return resObj;
    } catch (err: any) {
        if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
            console.log(`products +page.server.ts REDIRECT caught: to location=${err.location} with status=${err.status}`);
            throw err;
        }
        console.error('ERROR products +page.server.ts products page loader error:', err?.stack || err);
        return {
            products: [],
            total: 0,
            promoMap: {},
            error: 'Unable to connect to the backend server.'
        };
    }
};


export const actions: Actions = {
    deleteProduct: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_vendor_session');
        if (!sessionToken) return fail(401, { error: 'Unauthorized' });

        const formData = await request.formData();
        const productId = formData.get('product_id')?.toString();

        if (!productId) return fail(400, { error: 'Product ID is required' });

        const result = await safeFetch<any>(fetch, `${env.PUBLIC_API_URL || `${env.PUBLIC_API_URL || 'http://localhost:8080'}`}/api/v1/vendor/products/${productId}`, {
            method: 'DELETE',
            headers: {
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_vendor_session=${sessionToken}`
            }
        });

        if (!result.success) {
            return fail(result.status, { error: result.error?.message || 'Failed to delete product' });
        }

        return { success: true, message: 'Hall deleted successfully' };
    },

    toggleAvailability: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_vendor_session');
        if (!sessionToken) return fail(401, { error: 'Unauthorized' });

        const formData = await request.formData();
        const productId = formData.get('product_id')?.toString();
        const isAvailable = formData.get('is_available') === 'true';

        if (!productId) return fail(400, { error: 'Product ID is required' });

        const result = await safeFetch<any>(fetch, `${env.PUBLIC_API_URL || `${env.PUBLIC_API_URL || 'http://localhost:8080'}`}/api/v1/vendor/products/${productId}/availability`, {
            method: 'PATCH',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_vendor_session=${sessionToken}`
            },
            body: JSON.stringify({ is_available: !isAvailable })
        });

        if (!result.success) {
            return fail(result.status, { error: result.error?.message || 'Failed to update availability' });
        }

        return { success: true };
    }
};
