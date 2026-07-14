import { env } from "$env/dynamic/public";
import { redirect, fail } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch, params }) => {
    const sessionToken = cookies.get('zafaf_admin_session');
    if (!sessionToken) throw redirect(303, '/login');

    try {
        const response = await fetch(`${env.PUBLIC_API_URL || `${env.PUBLIC_API_URL || 'http://localhost:8080'}`}/api/v1/admin/vendors/${params.id}`, {
            headers: {
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_admin_session=${sessionToken}`
            }
        });

        if (response.status === 401 || response.status === 403) {
            cookies.delete('zafaf_admin_session', { path: '/' });
            throw redirect(303, '/login');
        }

        if (response.status === 404) throw redirect(303, '/dashboard/vendors');

        if (!response.ok) return { vendor: null, tiers: [] };

        const data = await response.json();
        
        let tiers = [];
        try {
            const tiersRes = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/public/subscription/tiers`, {
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`
                }
            });
            if (tiersRes.ok) {
                const tiersData = await tiersRes.json();
                if (tiersData.status === 'success') {
                    tiers = tiersData.tiers;
                }
            }
        } catch (e) {
            console.error('Failed to load tiers:', e);
        }

        if (data.status === 'success') {
            return { vendor: data.vendor, tiers };
        }

        return { vendor: null };

    } catch (err) {
        if (err && typeof err === 'object' && 'status' in err && 'location' in err) throw err;
        console.error('Failed to load vendor detail:', err);
        return { vendor: null };
    }
};

export const actions: Actions = {
    updateStatus: async ({ request, cookies, fetch, params }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) return fail(401, { error: 'Unauthorized' });

        const fd = await request.formData();
        const status = fd.get('status')?.toString();
        const reason = fd.get('reason')?.toString() || undefined;

        if (!status) return fail(400, { error: 'Status is required' });

        try {
            const res = await fetch(`${env.PUBLIC_API_URL || `${env.PUBLIC_API_URL || 'http://localhost:8080'}`}/api/v1/admin/vendors/${params.id}/status`, {
                method: 'PATCH',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ status, reason })
            });

            const data = await res.json().catch(() => ({}));
            if (!res.ok) return fail(res.status, { error: data.message || 'Status update failed' });
            return { success: true, message: data.message || 'Vendor status updated.' };

        } catch (err: any) {
            return fail(500, { error: err.message || 'Connection error' });
        }
    },

    updateSubscription: async ({ request, cookies, fetch, params }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) return fail(401, { error: 'Unauthorized' });

        const fd = await request.formData();
        const subscription_status = fd.get('subscription_status')?.toString();
        const subscription_tier_id = fd.get('subscription_tier_id')?.toString() || undefined;
        let subscription_expires_at = fd.get('subscription_expires_at')?.toString() || undefined;

        if (!subscription_status) return fail(400, { error: 'Subscription status is required' });

        if (subscription_expires_at && subscription_expires_at.length === 16) {
            // HTML datetime-local provides YYYY-MM-DDTHH:mm
            subscription_expires_at = `${subscription_expires_at}:00Z`;
        }

        try {
            const res = await fetch(`${env.PUBLIC_API_URL || `${env.PUBLIC_API_URL || 'http://localhost:8080'}`}/api/v1/admin/vendors/${params.id}/subscription`, {
                method: 'PATCH',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    subscription_status,
                    subscription_tier_id,
                    subscription_expires_at
                })
            });

            const data = await res.json().catch(() => ({}));
            if (!res.ok) return fail(res.status, { error: data.message || 'Subscription update failed' });
            return { success: true, message: data.message || 'Subscription updated.' };

        } catch (err: any) {
            return fail(500, { error: err.message || 'Connection error' });
        }
    },

    updateProductStatus: async ({ request, cookies, fetch, params }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) return fail(401, { error: 'Unauthorized' });

        const fd = await request.formData();
        const productId = fd.get('product_id')?.toString();
        const status = fd.get('status')?.toString();
        const reason = fd.get('reason')?.toString() || undefined;

        if (!productId || !status) return fail(400, { error: 'Product ID and status are required' });

        try {
            const res = await fetch(
                `${env.PUBLIC_API_URL || `${env.PUBLIC_API_URL || 'http://localhost:8080'}`}/api/v1/admin/vendors/${params.id}/products/${productId}/status`,
                {
                    method: 'PATCH',
                    headers: {
                        'Authorization': `Bearer ${sessionToken}`,
                        'Cookie': `zafaf_admin_session=${sessionToken}`,
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify({ status, reason })
                }
            );

            const data = await res.json().catch(() => ({}));
            if (!res.ok) return fail(res.status, { error: data.message || 'Hall status update failed' });
            return { success: true, message: data.message || 'Hall status updated.' };

        } catch (err: any) {
            return fail(500, { error: err.message || 'Connection error' });
        }
    },

    promoteProduct: async ({ request, cookies, fetch, params }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) return fail(401, { error: 'Unauthorized' });

        const fd = await request.formData();
        const productId = fd.get('product_id')?.toString();
        const daysStr = fd.get('days')?.toString();

        if (!productId || !daysStr) return fail(400, { error: 'Product ID and days are required' });

        const days = parseInt(daysStr, 10);
        if (isNaN(days) || days <= 0) return fail(400, { error: 'Invalid promotion duration' });

        try {
            const res = await fetch(
                `${env.PUBLIC_API_URL || `${env.PUBLIC_API_URL || 'http://localhost:8080'}`}/api/v1/admin/products/${productId}/promote`,
                {
                    method: 'POST',
                    headers: {
                        'Authorization': `Bearer ${sessionToken}`,
                        'Cookie': `zafaf_admin_session=${sessionToken}`,
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify({ days })
                }
            );

            const data = await res.json().catch(() => ({}));
            if (!res.ok) return fail(res.status, { error: data.message || 'Product promotion failed' });
            return { success: true, message: data.message || 'Product promoted successfully.' };

        } catch (err: any) {
            return fail(500, { error: err.message || 'Connection error' });
        }
    }
};
