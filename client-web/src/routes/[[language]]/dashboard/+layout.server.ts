import { redirect, error } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types';
import { env } from '$env/dynamic/public';

export const load: LayoutServerLoad = async ({ cookies, fetch, url }) => {
    const sessionToken = cookies.get('zafaf_client_session');
    
    // 1. Block access immediately if no session cookie exists
    if (!sessionToken) {
        throw redirect(303, '/auth/login');
    }

    const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';

    let profileResponse: Response;
    try {
        profileResponse = await fetch(`${API_BASE}/api/v1/auth/me`, {
            headers: {
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_client_session=${sessionToken}`
            }
        });
    } catch (err) {
        console.error('[Dashboard Layout Loader] Profile fetch connection failed:', err);
        throw error(503, { message: 'Service Temporarily Unavailable. Please try again later.' });
    }

    if (!profileResponse.ok) {
        if (profileResponse.status === 401 || profileResponse.status === 403) {
            console.warn(`[Dashboard Layout Loader] Session token invalid/expired (status ${profileResponse.status}). Clearing session.`);
            cookies.delete('zafaf_client_session', {
                path: '/',
                secure: url.protocol === 'https:',
                sameSite: 'lax'
            });
            throw redirect(303, '/auth/login');
        } else {
            console.error(`[Dashboard Layout Loader] Backend authentication returned error status: ${profileResponse.status}`);
            throw error(profileResponse.status, { message: 'Failed to verify session with backend server.' });
        }
    }

    let profileData: any;
    try {
        profileData = await profileResponse.json();
    } catch (e) {
        console.error('[Dashboard Layout Loader] Failed to parse profile JSON:', e);
        throw error(500, { message: 'Malformed response received from profile server.' });
    }

    if (profileData.status !== 'success') {
        console.warn('[Dashboard Layout Loader] Profile verification response status not success. Clearing session.');
        cookies.delete('zafaf_client_session', {
            path: '/',
            secure: url.protocol === 'https:',
            sameSite: 'lax'
        });
        throw redirect(303, '/auth/login');
    }

    const role = profileData.user?.role?.toLowerCase();
    if (role !== 'client') {
        console.warn(`[Dashboard Layout Loader] Role mismatch: expected 'client', got '${role}'. Redirecting to login with error param.`);
        throw redirect(303, `/auth/login?error=wrong_portal&role=${role}`);
    }

    let dashboardResponse: Response | null = null;
    try {
        dashboardResponse = await fetch(`${API_BASE}/api/v1/client/dashboard`, {
            headers: {
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_client_session=${sessionToken}`
            }
        });
    } catch (err) {
        console.error('[Dashboard Layout Loader] Dashboard fetch connection failed:', err);
    }

    let dashboardData: any = null;
    if (dashboardResponse && dashboardResponse.ok) {
        try {
            dashboardData = await dashboardResponse.json();
        } catch (e) {
            console.error('[Dashboard Layout Loader] Failed to parse dashboard JSON:', e);
        }
    }

    const user = profileData.user;
    const dashboard = dashboardData?.status === 'success' ? dashboardData.data : {
        weddingCountdown: { weddingDate: null, daysRemaining: null },
        budget: { totalBudget: 0, spentAmount: 0, remainingBudget: 0 },
        activeBookings: []
    };

    return {
        user,
        dashboard
    };
};
