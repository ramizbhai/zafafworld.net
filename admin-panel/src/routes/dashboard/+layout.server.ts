import { env } from "$env/dynamic/public";
import { redirect, error } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types';

import { isAdmin, isValidProfileResponse } from '../../core/auth/sessionService';

export const load: LayoutServerLoad = async ({ cookies, fetch }) => {
    const sessionToken = cookies.get('zafaf_admin_session');

    // 1. Block access immediately if no session cookie exists
    if (!sessionToken) {
        throw redirect(303, '/login');
    }

    try {
        // 2. Validate session and fetch telemetry concurrently
        const [profileResponse, dashboardResponse, notificationsResponse, unreadResponse] = await Promise.all([
            fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/auth/me`, {
                method: 'GET',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`
                }
            }),
            fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/analytics/dashboard`, {
                method: 'GET',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`
                }
            }),
            fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/notifications/recent`, {
                method: 'GET',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`
                }
            }),
            fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/notifications/unread-count`, {
                method: 'GET',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`
                }
            })
        ]);

        if (!profileResponse.ok) {
            if (profileResponse.status === 401 || profileResponse.status === 403 || profileResponse.status === 404) {
                // 401 / 403 / 404 (invalid or expired session): Clear cookie and redirect
                cookies.delete('zafaf_admin_session', { path: '/' });
                throw redirect(303, '/login');
            }
            // 5xx / network error: Throw 503 instead of redirecting to /login
            throw error(503, 'Service temporarily unavailable');
        }

        const profileData = await profileResponse.json();

        // 3. Enforce strict administrative role check using core session service
        if (!isValidProfileResponse(profileData)) {
            // REVISIT LATER: Malformed response handling. Treating as invalid session for now.
            cookies.delete('zafaf_admin_session', { path: '/' });
            throw redirect(303, '/login');
        }

        if (!isAdmin(profileData.user)) {
            cookies.delete('zafaf_admin_session', { path: '/' });
            throw redirect(303, '/login');
        }

        const dashboardData = dashboardResponse.ok ? await dashboardResponse.json() : null;
        const notificationsData = notificationsResponse.ok ? await notificationsResponse.json() : { notifications: [] };
        const unreadData = unreadResponse.ok ? await unreadResponse.json() : { unreadCount: 0 };

        return {
            user: profileData.user,
            dashboard: dashboardData,
            notifications: notificationsData.notifications || [],
            unreadCount: unreadData.unreadCount || 0
        };

    } catch (err) {
        // Catch SvelteKit redirect exceptions and re-throw them directly
        if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
            throw err;
        }

        console.error('Admin Layout Guard Handshake anomaly:', err);

        // Do NOT delete the cookie here. A network error or transient backend
        // failure is NOT proof that the session is invalid. Throw a 503 error
        // so the user stays on the current URL and can refresh when the service recovers.
        throw error(503, 'Service unavailable or backend connection failed. Please refresh the page.');
    }
};
