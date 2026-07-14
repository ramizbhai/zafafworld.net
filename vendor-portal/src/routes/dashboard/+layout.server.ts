import { apiClient } from '$lib/api/client';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ locals, cookies, fetch, depends }) => {
    depends('app:vendor-data');
    const user = locals.user;
    const sessionToken = cookies.get('zafaf_vendor_session');

    let gateLock: 'none' | 'stopped' = 'none';
    let tiers: any[] = [];
    
    if (sessionToken) {
        const client = apiClient.withFetch(fetch);

        const telemetryPromise = client.vendor.getDashboardStats(sessionToken).then(res => {
            if (res.status === 402) return { status: 'stopped', data: null };
            if (res.success && res.data) return res.data;
            return { status: 'error', data: null };
        }).catch(() => ({ status: 'error', data: null }));

        const metadataPromise = client.vendor.getMetadataConfig(sessionToken).then(res => {
            if (res.success && res.data) return res.data;
            return { data: null };
        }).catch(() => ({ data: null }));

        const notificationsPromise = client.vendor.getNotifications(sessionToken).then(res => {
            if (res.success && res.data) return res.data;
            return { notifications: [] };
        }).catch(() => ({ notifications: [] }));

        const tiersPromise = client.vendor.getTiers(sessionToken).then(res => {
            if (res.success && res.data) return res.data;
            return { tiers: [] };
        }).catch(() => ({ tiers: [] }));

        return {
            user,
            locale: cookies.get('zafaf_locale') || 'ar',
            sessionToken,
            streamed: {
                telemetry: telemetryPromise,
                metadata: metadataPromise,
                notifications: notificationsPromise,
                tiers: tiersPromise
            }
        };
    }

    const locale = cookies.get('zafaf_locale') || 'ar';

    return {
        user,
        locale,
        sessionToken,
        streamed: {
            telemetry: Promise.resolve({ data: null }),
            metadata: Promise.resolve({ data: null }),
            notifications: Promise.resolve([]),
            tiers: Promise.resolve([])
        }
    };
};
