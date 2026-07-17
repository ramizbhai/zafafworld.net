import { redirect, error } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types';
import { env } from '$env/dynamic/public';

export const load: LayoutServerLoad = async ({ cookies, fetch, url, parent }) => {
    const sessionToken = cookies.get('zafaf_client_session');
    
    // 1. Block access immediately if no session cookie exists
    if (!sessionToken) {
        throw redirect(303, '/auth/login');
    }

    const { user: parentUser } = await parent();

    if (!parentUser) {
        // Parent failed to authenticate
        cookies.delete('zafaf_client_session', {
            path: '/',
            secure: url.protocol === 'https:',
            sameSite: 'lax'
        });
        throw redirect(303, '/auth/login');
    }

    const role = parentUser.role?.toLowerCase();
    if (role !== 'client') {
        console.warn(`[Dashboard Layout Loader] Role mismatch: expected 'client', got '${role}'. Redirecting to login with error param.`);
        throw redirect(303, `/auth/login?error=wrong_portal&role=${role}`);
    }

    const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';

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

    const user = parentUser;
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
