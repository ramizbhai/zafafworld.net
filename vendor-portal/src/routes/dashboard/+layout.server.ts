import { apiClient } from '$lib/api/client';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ locals, cookies, fetch, depends }) => {
    console.log("ENTER dashboard +layout.server.ts");
    depends('app:vendor-data');
    const user = locals.user;
    const sessionToken = cookies.get('zafaf_vendor_session');

    try {
        if (sessionToken) {
            const client = apiClient.withFetch(fetch);

            console.log("dashboard +layout.server.ts: Session token found. Initiating parallel api fetches...");

            const telemetryPromise = client.vendor.getDashboardStats(sessionToken).then(res => {
                console.log(`dashboard +layout.server.ts telemetry: success=${res.success}, status=${res.status}`);
                if (res.status === 402) return { status: 'stopped', data: null };
                if (res.success && res.data) return res.data;
                return { status: 'error', data: null };
            }).catch((err) => {
                console.error("dashboard +layout.server.ts telemetry ERROR:", err);
                return { status: 'error', data: null };
            });

            const metadataPromise = client.vendor.getMetadataConfig(sessionToken).then(res => {
                console.log(`dashboard +layout.server.ts metadata: success=${res.success}`);
                if (res.success && res.data) return res.data;
                return { data: null };
            }).catch((err) => {
                console.error("dashboard +layout.server.ts metadata ERROR:", err);
                return { data: null };
            });

            const notificationsPromise = client.vendor.getNotifications(sessionToken).then(res => {
                console.log(`dashboard +layout.server.ts notifications: success=${res.success}`);
                if (res.success && res.data) return res.data;
                return { notifications: [] };
            }).catch((err) => {
                console.error("dashboard +layout.server.ts notifications ERROR:", err);
                return { notifications: [] };
            });

            const tiersPromise = client.vendor.getTiers(sessionToken).then(res => {
                console.log(`dashboard +layout.server.ts tiers: success=${res.success}`);
                if (res.success && res.data) return res.data;
                return { tiers: [] };
            }).catch((err) => {
                console.error("dashboard +layout.server.ts tiers ERROR:", err);
                return { tiers: [] };
            });

            const resObj = {
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
            console.log("EXIT dashboard +layout.server.ts (with session token)");
            return resObj;
        }

        const locale = cookies.get('zafaf_locale') || 'ar';
        const resObj = {
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
        console.log("EXIT dashboard +layout.server.ts (no session token)");
        return resObj;
    } catch (err: any) {
        console.error("ERROR dashboard +layout.server.ts:", err?.stack || err);
        throw err;
    }
};
