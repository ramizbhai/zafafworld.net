import type { PageServerLoad } from './$types';
import { DashboardService } from '$lib/features/admin/dashboard/dashboard.service.js';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
    return await DashboardService.loadDashboardData(cookies, fetch);
};
