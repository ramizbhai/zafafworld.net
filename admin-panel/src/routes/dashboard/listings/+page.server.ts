import { env } from '$env/dynamic/public';
import { fail, redirect } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ fetch, cookies, depends }) => {
    depends('app:admin-listings');

    const sessionToken = cookies.get('zafaf_admin_session');
    if (!sessionToken) throw redirect(303, '/login');

    const res = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/vendors/all-listings?limit=1000`, {
        headers: {
            'Authorization': `Bearer ${sessionToken}`,
            'Cookie': `zafaf_admin_session=${sessionToken}`
        }
    });

    if (!res.ok) {
        return { listings: [] };
    }
    
    const data = await res.json();
    return {
        listings: data.listings || []
    };
};

export const actions: Actions = {
    approve: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) return fail(401, { error: 'Unauthorized' });

        const fd = await request.formData();
        const vendor_id = fd.get('vendor_id')?.toString();
        const product_id = fd.get('product_id')?.toString();

        if (!vendor_id || !product_id) {
            return fail(400, { error: 'Missing vendor or product ID' });
        }

        const res = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/vendors/${vendor_id}/products/${product_id}/status`, {
            method: 'PATCH',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_admin_session=${sessionToken}`
            },
            body: JSON.stringify({
                status: 'active',
                reason: 'Approved by admin'
            })
        });

        if (!res.ok) {
            return fail(res.status, {
                error: 'Failed to approve listing'
            });
        }

        return { success: true };
    },

    reject: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) return fail(401, { error: 'Unauthorized' });

        const fd = await request.formData();
        const vendor_id = fd.get('vendor_id')?.toString();
        const product_id = fd.get('product_id')?.toString();
        const reason = fd.get('reason')?.toString()?.trim();

        if (!vendor_id || !product_id) {
            return fail(400, { error: 'Missing vendor or product ID' });
        }
        if (!reason) {
            return fail(400, { error: 'Rejection reason is required' });
        }

        const res = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/vendors/${vendor_id}/products/${product_id}/status`, {
            method: 'PATCH',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_admin_session=${sessionToken}`
            },
            body: JSON.stringify({
                status: 'rejected',
                reason
            })
        });

        if (!res.ok) {
            return fail(res.status, { error: 'Failed to reject listing' });
        }

        return { success: true };
    }
};
