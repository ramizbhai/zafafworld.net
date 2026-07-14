import { env } from '$env/dynamic/public';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch }) => {
    const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';
    let promotions = [];

    try {
        const response = await fetch(`${API_BASE}/api/v1/public/promotions`);
        if (response.ok) {
            const data = await response.json();
            if (data.status === 'success') {
                promotions = data.promotions || [];
            }
        }
    } catch (err) {
        console.error('[Offers SSR] Failed to load public promotions:', err);
    }

    return {
        promotions
    };
};
