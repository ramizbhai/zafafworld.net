import { env } from '$env/dynamic/public';
import { getCached } from '$lib/services/api/cache';
import { getLocale } from '$lib/paraglide/runtime.js';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch }) => {
    const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';
    const lang = getLocale();
    console.log('[discover PageServerLoad] resolved lang from getLocale:', lang);

    const cacheKey = `discover:blogs_list:${lang}:1`;
    const dataResponse = await getCached(cacheKey, async () => {
        try {
            const response = await fetch(`${API_BASE}/api/v1/public/blogs?lang=${lang}&page=1&limit=12`);
            if (!response.ok) {
                console.warn(`[Blog Server Loader] Blogs endpoint returned status ${response.status}`);
                return { data: [] };
            }
            const json = await response.json();
            return json;
        } catch (err) {
            console.error('[Blog Server Loader] Failed to fetch blogs:', err);
            return { data: [] };
        }
    }, 60 * 1000); // Cache the blogs list for 60 seconds

    const posts = dataResponse.status === 'success' && Array.isArray(dataResponse.data) ? dataResponse.data : [];

    return { posts };
};
