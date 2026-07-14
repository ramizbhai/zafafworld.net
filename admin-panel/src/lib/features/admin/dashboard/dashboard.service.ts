import { env } from "$env/dynamic/public";
import { redirect, type Cookies } from '@sveltejs/kit';

export class DashboardService {
    static async loadDashboardData(cookies: Cookies, fetchFn: typeof fetch) {
        const sessionToken = cookies.get('zafaf_admin_session');

        if (!sessionToken) {
            throw redirect(303, '/login');
        }

        try {
            const baseUrl = env.PUBLIC_API_URL || 'http://localhost:8080';
            const headers = {
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_admin_session=${sessionToken}`
            };

            const [analyticsRes, dashboardRes, approvalsRes] = await Promise.all([
                fetchFn(`${baseUrl}/api/v1/admin/analytics/summary`, { method: 'GET', headers }),
                fetchFn(`${baseUrl}/api/v1/admin/analytics/dashboard`, { method: 'GET', headers }),
                fetchFn(`${baseUrl}/api/v1/admin/approvals`, { method: 'GET', headers })
            ]);

            if (!analyticsRes.ok || !dashboardRes.ok || !approvalsRes.ok) {
                const status = analyticsRes.status || dashboardRes.status || approvalsRes.status;
                if (status === 401 || status === 403) {
                    cookies.delete('zafaf_admin_session', { path: '/' });
                    throw redirect(303, '/login');
                }
                return {
                    summary: null,
                    dashboard: null,
                    approvals: [],
                    error: 'Backend API responded with error status code.'
                };
            }

            const [analyticsData, dashboardData, approvalsData] = await Promise.all([
                analyticsRes.json(),
                dashboardRes.json(),
                approvalsRes.json()
            ]);

            return {
                summary: analyticsData.status === 'success' ? analyticsData.summary : null,
                dashboard: dashboardData,
                approvals: approvalsData.status === 'success' ? approvalsData.listings : [],
                error: null
            };

        } catch (err: any) {
            if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
                throw err;
            }
            console.error('Failed to load dashboard data:', err);
            return {
                summary: null,
                dashboard: null,
                approvals: [],
                error: err.message || 'Internal connection error'
            };
        }
    }
}
