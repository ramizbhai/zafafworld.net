import { env } from "$env/dynamic/public";
import { redirect, fail } from '@sveltejs/kit';
import { z } from 'zod';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
    const sessionToken = cookies.get('zafaf_admin_session');

    // 1. Block access immediately if no session cookie exists
    if (!sessionToken) {
        throw redirect(303, '/login');
    }

    const authHeaders = {
        'Authorization': `Bearer ${sessionToken}`,
        'Cookie': `zafaf_admin_session=${sessionToken}`
    };

    try {
        // 2. Fetch context from aggregated BFF endpoint
        const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/vendors-context`, {
            headers: authHeaders
        });

        if (!response.ok) {
            if (response.status === 401 || response.status === 403) {
                cookies.delete('zafaf_admin_session', { path: '/' });
                throw redirect(303, '/login');
            }
            return { vendors: [], pendingListings: [], tiers: [] };
        }

        const data = await response.json();

        return {
            vendors: data.status === 'success' ? (data.vendors || []) : [],
            pendingListings: data.status === 'success' ? (data.pendingListings || []) : [],
            tiers: data.status === 'success' ? (data.tiers || []) : []
        };

    } catch (err) {
        // Propagate redirects directly
        if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
            throw err;
        }

        console.error('Failed to load vendors/listings:', err);
        return { vendors: [], pendingListings: [], tiers: [] };
    }
};

export const actions: Actions = {
    reactivate: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) {
            return fail(401, { error: 'Unauthorized administrative session' });
        }

        const formData = await request.formData();
        const id = formData.get('id') as string;

        if (!id) {
            return fail(400, { error: 'Missing vendor identifier' });
        }

        try {
            // Reactivate a suspended/banned vendor account
            const response = await fetch(`${env.PUBLIC_API_URL || `${env.PUBLIC_API_URL || 'http://localhost:8080'}`}/api/v1/admin/vendors/${id}/approve`, {
                method: 'PATCH',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`,
                    'Content-Type': 'application/json'
                }
            });

            if (!response.ok) {
                const errData = await response.json().catch(() => ({}));
                return fail(response.status, {
                    error: errData.error?.message || `Failed to reactivate vendor (Status ${response.status})`
                });
            }

            const data = await response.json();
            if (data.status === 'success') {
                return { success: true, reactivatedId: id };
            }

            return fail(400, { error: data.message || 'Reactivation action failed' });

        } catch (err: any) {
            console.error('Error reactivating vendor account:', err);
            return fail(500, { error: err.message || 'Internal connection error' });
        }
    },

    updateStatus: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) {
            return fail(401, { error: 'Unauthorized administrative session' });
        }

        const formData = await request.formData();
        const id = formData.get('id') as string;
        const status = formData.get('status') as string;
        const reason = formData.get('reason') as string | null;

        if (!id || !status) {
            return fail(400, { error: 'Missing vendor identifier or target status' });
        }

        const allowedStatuses = ['active', 'suspended', 'banned'];
        if (!allowedStatuses.includes(status)) {
            return fail(400, { error: `Invalid status '${status}'. Must be one of: ${allowedStatuses.join(', ')}` });
        }

        try {
            const response = await fetch(`${env.PUBLIC_API_URL || `${env.PUBLIC_API_URL || 'http://localhost:8080'}`}/api/v1/admin/vendors/${id}/status`, {
                method: 'PATCH',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ status, reason: reason || undefined })
            });

            if (!response.ok) {
                const errData = await response.json().catch(() => ({}));
                return fail(response.status, {
                    error: errData.message || `Failed to update vendor status (Status ${response.status})`
                });
            }

            const data = await response.json();
            if (data.status === 'success') {
                return { success: true, statusUpdated: true, newStatus: status, vendorId: id };
            }

            return fail(400, { error: data.message || 'Status update action failed' });

        } catch (err: any) {
            console.error('Error updating vendor status:', err);
            return fail(500, { error: err.message || 'Internal connection error' });
        }
    },

    updateSubscription: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) {
            return fail(401, { error: 'Unauthorized administrative session' });
        }

        const formData = await request.formData();
        const id = formData.get('id') as string;
        const subscription_status = formData.get('subscription_status') as string;
        const subscription_tier_id = formData.get('subscription_tier_id') as string | null;

        if (!id || !subscription_status) {
            return fail(400, { error: 'Missing vendor identifier or subscription status' });
        }

        const allowedSubscriptionStatuses = ['trial', 'active', 'stopped'];
        if (!allowedSubscriptionStatuses.includes(subscription_status)) {
            return fail(400, { error: `Invalid subscription status '${subscription_status}'. Must be one of: ${allowedSubscriptionStatuses.join(', ')}` });
        }

        try {
            const response = await fetch(`${env.PUBLIC_API_URL || `${env.PUBLIC_API_URL || 'http://localhost:8080'}`}/api/v1/admin/vendors/${id}/subscription`, {
                method: 'PATCH',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ 
                    subscription_status,
                    subscription_tier_id: subscription_tier_id || undefined
                })
            });

            if (!response.ok) {
                const errData = await response.json().catch(() => ({}));
                return fail(response.status, {
                    error: errData.message || `Failed to update subscription status (Status ${response.status})`
                });
            }

            const data = await response.json();
            if (data.status === 'success') {
                return { success: true, subscriptionUpdated: true, newSubscriptionStatus: subscription_status, vendorId: id };
            }

            return fail(400, { error: data.message || 'Subscription status update action failed' });

        } catch (err: any) {
            console.error('Error updating subscription status:', err);
            return fail(500, { error: err.message || 'Internal connection error' });
        }
    },

    updateFeatured: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) {
            return fail(401, { error: 'Unauthorized administrative session' });
        }

        const formData = await request.formData();
        const id = formData.get('id') as string;
        const is_featured = formData.get('is_featured') === 'true';
        const expires_at_str = formData.get('expires_at') as string | null;

        if (!id) {
            return fail(400, { error: 'Missing vendor identifier' });
        }

        const expires_at = expires_at_str && expires_at_str.trim() !== '' ? new Date(expires_at_str).toISOString() : null;

        try {
            const response = await fetch(`${env.PUBLIC_API_URL || `${env.PUBLIC_API_URL || 'http://localhost:8080'}`}/api/v1/admin/vendors/${id}/featured`, {
                method: 'PATCH',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ is_featured, expires_at })
            });

            if (!response.ok) {
                const errData = await response.json().catch(() => ({}));
                return fail(response.status, {
                    error: errData.message || `Failed to update featured configuration (Status ${response.status})`
                });
            }

            const data = await response.json();
            if (data.status === 'success') {
                return { success: true, featuredUpdated: true, isFeatured: is_featured, vendorId: id };
            }

            return fail(400, { error: data.message || 'Featured placement update failed' });

        } catch (err: any) {
            console.error('Error updating featured configuration:', err);
            return fail(500, { error: err.message || 'Internal connection error' });
        }
    },

    moderateListing: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) {
            return fail(401, { error: 'Unauthorized administrative session' });
        }

        const formData = await request.formData();
        const vendor_id  = formData.get('vendor_id')  as string;
        const product_id = formData.get('product_id') as string;
        const status     = formData.get('status')     as string;
        const reason     = formData.get('reason')     as string | null;

        if (!vendor_id || !product_id || !status) {
            return fail(400, { error: 'Missing vendor_id, product_id, or status' });
        }

        if (status === 'rejected') {
            const rejectionSchema = z.object({
                reason: z.string().trim().min(5, "A descriptive rejection reason (minimum 5 characters) is required to notify the vendor.")
            });
            
            const validation = rejectionSchema.safeParse({ reason });
            if (!validation.success) {
                return fail(400, { error: validation.error.issues[0].message });
            }
        }

        const allowed = ['active', 'rejected', 'suspended', 'archived'];
        if (!allowed.includes(status)) {
            return fail(400, { error: `Invalid listing status '${status}'. Must be one of: ${allowed.join(', ')}` });
        }

        try {
            const response = await fetch(
                `${env.PUBLIC_API_URL || `${env.PUBLIC_API_URL || 'http://localhost:8080'}`}/api/v1/admin/vendors/${vendor_id}/products/${product_id}/status`,
                {
                    method: 'PATCH',
                    headers: {
                        'Authorization': `Bearer ${sessionToken}`,
                        'Cookie': `zafaf_admin_session=${sessionToken}`,
                        'Content-Type': 'application/json'
                    },
                    body: JSON.stringify({ status, reason: reason || undefined })
                }
            );

            if (!response.ok) {
                const errData = await response.json().catch(() => ({}));
                return fail(response.status, {
                    error: errData.message || `Failed to update listing status (HTTP ${response.status})`
                });
            }

            const data = await response.json();
            if (data.status === 'success') {
                return { success: true, listingModerated: true, newStatus: status, productId: product_id };
            }

            return fail(400, { error: data.message || 'Listing moderation action failed' });

        } catch (err: any) {
            console.error('Error moderating listing:', err);
            return fail(500, { error: err.message || 'Internal connection error' });
        }
    }
};
