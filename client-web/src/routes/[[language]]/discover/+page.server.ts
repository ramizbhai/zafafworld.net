import { env } from '$env/dynamic/public';
import { getCached } from '$lib/services/api/cache';
import { getLocale } from '$lib/paraglide/runtime.js';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch }) => {
    const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';
    const lang = getLocale();
    console.log('[discover PageServerLoad] resolved lang from getLocale:', lang);

    const posts = await getCached('discover:blogs_list', async () => {
        try {
            const response = await fetch(`${API_BASE}/api/v1/public/blogs`);
            if (!response.ok) {
                console.warn(`[Blog Server Loader] Blogs endpoint returned status ${response.status}`);
                return [];
            }
            const data = await response.json();
            return data.status === 'success' && Array.isArray(data.data) ? data.data : [];
        } catch (err) {
            console.error('[Blog Server Loader] Failed to fetch blogs:', err);
            return [];
        }
    }, 60 * 1000); // Cache the blogs list for 60 seconds

    const filteredPosts = posts.filter((p: any) => p.lang === lang);

    return { posts: filteredPosts };
};
