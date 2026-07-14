import { redirect, fail } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';
import { env } from '$env/dynamic/public';

export const load: PageServerLoad = async ({ cookies, fetch, depends }) => {
    // Register a granular invalidation key so form actions can call
    // invalidateAll() or invalidate('app:dashboard') to re-fetch this data
    // without a full page reload.
    depends('app:dashboard');

    const sessionToken = cookies.get('zafaf_client_session');
    if (!sessionToken) {
        throw redirect(303, '/auth/login');
    }

    const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';
    const headers = {
        'Authorization': `Bearer ${sessionToken}`,
        'Cookie': `zafaf_client_session=${sessionToken}`
    };

    try {
        const response = await fetch(`${API_BASE}/api/v1/client/dashboard-context`, {
            headers,
            signal: AbortSignal.timeout(8000)
        });

        if (!response.ok) {
            console.error('[Dashboard Page Server Load] BFF error status:', response.status);
            throw new Error(`BFF Request Failed: ${response.status}`);
        }

        const result = await response.json();
        const data = result.status === 'success' ? result.data : null;

        return {
            bookings: data?.bookings || [],
            profile: data?.profile || {
                weddingDate: null,
                daysRemaining: null,
                budget: { totalBudget: 0, spentAmount: 0, remainingBudget: 0 }
            },
            inquiries: data?.inquiries || [],
            activities: data?.activities || [],
            notifications: data?.notifications || [],
            conversations: data?.conversations || [],
            serverTimestamp: Date.now()
        };
    } catch (err) {
        console.error('[Dashboard Page Server Load] Error:', err);
        return {
            bookings: [],
            profile: {
                weddingDate: null,
                daysRemaining: null,
                budget: { totalBudget: 0, spentAmount: 0, remainingBudget: 0 }
            },
            inquiries: [],
            activities: [],
            notifications: [],
            conversations: [],
            serverTimestamp: Date.now()
        };
    }
};

export const actions: Actions = {
    cancelBooking: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_client_session');
        if (!sessionToken) {
            return fail(401, { success: false, message: 'Unauthorized' });
        }

        const formData = await request.formData();
        const bookingId = formData.get('bookingId');
        if (!bookingId) {
            return fail(400, { success: false, message: 'Booking ID is required' });
        }

        const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';

        try {
            const response = await fetch(`${API_BASE}/api/v1/client/bookings/${bookingId}/transition`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_client_session=${sessionToken}`
                },
                body: JSON.stringify({ toStatus: 'cancelled' })
            });

            const result = await response.json();
            if (!response.ok || result.status !== 'success') {
                return fail(400, { success: false, message: result.message || 'Cancellation failed' });
            }

            return { success: true, message: 'Booking cancelled successfully' };
        } catch (err) {
            console.error('Cancel booking error:', err);
            return fail(500, { success: false, message: 'Failed to request cancellation due to server error' });
        }
    },

    readNotifications: async ({ cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_client_session');
        if (!sessionToken) {
            return fail(401, { success: false, message: 'Unauthorized' });
        }

        const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';

        try {
            const response = await fetch(`${API_BASE}/api/v1/client/notifications/read`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_client_session=${sessionToken}`
                }
            });

            const result = await response.json();
            if (!response.ok || result.status !== 'success') {
                return fail(400, { success: false, message: result.message || 'Failed to mark notifications read' });
            }

            return { success: true };
        } catch (err) {
            console.error('Mark notifications read error:', err);
            return fail(500, { success: false, message: 'Failed to mark notifications read due to server error' });
        }
    }
};
