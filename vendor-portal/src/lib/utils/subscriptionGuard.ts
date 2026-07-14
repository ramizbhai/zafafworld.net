import { get } from 'svelte/store';
import { goto } from '$app/navigation';

export async function checkSubscriptionQuota(actionFn: () => Promise<Response | any>) {
    try {
        const response = await actionFn();
        
        // If response is a native Response object
        if (response instanceof Response && response.status === 402) {
            return { blocked: true, status: 402 };
        }
        
        // If it's a SvelteKit action result or JSON
        if (response && response.status === 402) {
            return { blocked: true, status: 402 };
        }
        if (response && response.type === 'failure' && response.status === 402) {
             return { blocked: true, status: 402 };
        }

        return { blocked: false, response };
    } catch (e: any) {
        if (e?.status === 402 || e?.response?.status === 402) {
            return { blocked: true, status: 402 };
        }
        throw e;
    }
}
