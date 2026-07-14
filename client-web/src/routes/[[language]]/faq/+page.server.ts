import { env } from '$env/dynamic/public';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch }) => {
    const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';
    try {
        const response = await fetch(`${API_BASE}/api/v1/public/faqs`);
        if (!response.ok) {
            console.warn(`[FAQ Server Loader] FAQs endpoint returned status ${response.status}`);
            return { faqs: [] };
        }
        const data = await response.json();
        return { faqs: data.status === 'success' && Array.isArray(data.data) ? data.data : [] };
    } catch (err) {
        console.error('[FAQ Server Loader] Failed to fetch faqs:', err);
        return { faqs: [] };
    }
};
