import { env } from "$env/dynamic/public";
import { redirect, fail } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
    const sessionToken = cookies.get('zafaf_admin_session');

    if (!sessionToken) {
        throw redirect(303, '/login');
    }

    try {
        // Fetch notifications log
        const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/notifications`, {
            headers: {
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_admin_session=${sessionToken}`
            }
        });

        // Fetch vendors list for targeted dropdown
        const vendorsResponse = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/vendors`, {
            headers: {
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_admin_session=${sessionToken}`
            }
        });

        if (!response.ok || !vendorsResponse.ok) {
            if (response.status === 401 || response.status === 403 || vendorsResponse.status === 401 || vendorsResponse.status === 403) {
                cookies.delete('zafaf_admin_session', { path: '/' });
                throw redirect(303, '/login');
            }
            return { notifications: [], vendors: [], error: 'Failed to load notifications page data' };
        }

        const notifData = await response.json();
        const vendorData = await vendorsResponse.json();

        return {
            notifications: notifData.notifications || [],
            vendors: vendorData.vendors || []
        };

    } catch (err) {
        console.error('Failed to fetch admin notifications data:', err);
        return { notifications: [], vendors: [], error: 'Internal connection error' };
    }
};

export const actions: Actions = {
    sendNotification: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');

        if (!sessionToken) {
            throw redirect(303, '/login');
        }

        const formData = await request.formData();
        const titleAr = formData.get('titleAr')?.toString() || '';
        const titleEn = formData.get('titleEn')?.toString() || '';
        const messageAr = formData.get('messageAr')?.toString() || '';
        const messageEn = formData.get('messageEn')?.toString() || '';
        const targetAudience = formData.get('targetAudience')?.toString() || 'all';

        if (!messageAr || !messageEn) {
            return fail(400, { success: false, error: 'Messages are mandatory in both Arabic and English' });
        }

        try {
            const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/notifications`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`
                },
                body: JSON.stringify({
                    titleAr,
                    titleEn,
                    messageAr,
                    messageEn,
                    targetAudience
                })
            });

            if (!response.ok) {
                const errData = await response.json().catch(() => ({}));
                return fail(response.status, {
                    success: false,
                    error: errData.message || 'Failed to dispatch notification'
                });
            }

            const data = await response.json();
            if (data.status === 'success') {
                return { success: true };
            }

            return fail(400, {
                success: false,
                error: data.message || 'Failed to dispatch notification'
            });

        } catch (err) {
            console.error('Failed to send notification action:', err);
            return fail(500, { success: false, error: 'Network error dispatching notification' });
        }
    }
};
