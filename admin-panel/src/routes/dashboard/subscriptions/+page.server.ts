import { env } from "$env/dynamic/public";
import { redirect, fail } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
    const sessionToken = cookies.get('zafaf_admin_session');

    if (!sessionToken) {
        throw redirect(303, '/login');
    }

    const authHeaders = {
        'Authorization': `Bearer ${sessionToken}`,
        'Cookie': `zafaf_admin_session=${sessionToken}`
    };

    try {
        const [reqsRes, ctxRes] = await Promise.all([
            fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/vendors/subscription/requests`, { headers: authHeaders }),
            fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/vendors/context`, { headers: authHeaders })
        ]);

        if (reqsRes.status === 401 || reqsRes.status === 403 || ctxRes.status === 401 || ctxRes.status === 403) {
            cookies.delete('zafaf_admin_session', { path: '/' });
            throw redirect(303, '/login');
        }

        const reqsData = reqsRes.ok ? await reqsRes.json() : {};
        const ctxData = ctxRes.ok ? await ctxRes.json() : {};

        return {
            requests: reqsData.status === 'success' ? (reqsData.requests || []) : [],
            vendors: ctxData.status === 'success' ? (ctxData.vendors || []) : [],
            tiers: ctxData.status === 'success' ? (ctxData.tiers || []) : []
        };
    } catch (err) {
        if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
            throw err;
        }
        console.error('Failed to load subscription management data:', err);
        return { requests: [], vendors: [], tiers: [] };
    }
};

export const actions: Actions = {
    approve: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) return fail(401, { error: 'Unauthorized' });

        const formData = await request.formData();
        const id = formData.get('id') as string;
        const notes = formData.get('admin_notes') as string | null;

        if (!id) return fail(400, { error: 'Missing request ID' });

        try {
            const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/vendors/subscription/requests/${id}/approve`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ admin_notes: notes || undefined })
            });

            if (!response.ok) {
                const errData = await response.json().catch(() => ({}));
                return fail(response.status, { error: errData.error || errData.message || 'Approval failed' });
            }

            return { success: true, message: 'Subscription request approved successfully' };
        } catch (err: any) {
            return fail(500, { error: err.message || 'Internal connection error' });
        }
    },

    reject: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) return fail(401, { error: 'Unauthorized' });

        const formData = await request.formData();
        const id = formData.get('id') as string;
        const notes = formData.get('admin_notes') as string | null;

        if (!id) return fail(400, { error: 'Missing request ID' });

        try {
            const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/vendors/subscription/requests/${id}/reject`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ admin_notes: notes || undefined })
            });

            if (!response.ok) {
                const errData = await response.json().catch(() => ({}));
                return fail(response.status, { error: errData.error || errData.message || 'Rejection failed' });
            }

            return { success: true, message: 'Subscription request rejected' };
        } catch (err: any) {
            return fail(500, { error: err.message || 'Internal connection error' });
        }
    },

    assignSubscription: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) return fail(401, { error: 'Unauthorized' });

        const formData = await request.formData();
        const id = formData.get('id') as string;
        const subscription_tier_id = formData.get('subscription_tier_id') as string;
        let expires_at = formData.get('expires_at') as string | null;

        if (!id || !subscription_tier_id) {
            return fail(400, { error: 'Vendor ID and subscription tier are required' });
        }

        if (expires_at && expires_at.length === 10) {
            expires_at = `${expires_at}T23:59:59Z`;
        }

        try {
            const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/vendors/${id}/subscription`, {
                method: 'PATCH',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    subscription_status: 'active',
                    subscription_tier_id,
                    subscription_expires_at: expires_at || undefined
                })
            });

            if (!response.ok) {
                const errData = await response.json().catch(() => ({}));
                return fail(response.status, { error: errData.message || 'Failed to update vendor subscription' });
            }

            return { success: true, message: 'Vendor subscription assigned successfully' };
        } catch (err: any) {
            return fail(500, { error: err.message || 'Internal connection error' });
        }
    },

    removeSubscription: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) return fail(401, { error: 'Unauthorized' });

        const formData = await request.formData();
        const id = formData.get('id') as string;
        const free_tier_id = formData.get('free_tier_id') as string | null;

        if (!id) return fail(400, { error: 'Vendor ID is required' });

        try {
            const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/vendors/${id}/subscription`, {
                method: 'PATCH',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    subscription_status: 'active',
                    subscription_tier_id: free_tier_id || undefined,
                    subscription_expires_at: null
                })
            });

            if (!response.ok) {
                const errData = await response.json().catch(() => ({}));
                return fail(response.status, { error: errData.message || 'Failed to remove vendor subscription' });
            }

            return { success: true, message: 'Vendor moved to Free plan' };
        } catch (err: any) {
            return fail(500, { error: err.message || 'Internal connection error' });
        }
    }
};

