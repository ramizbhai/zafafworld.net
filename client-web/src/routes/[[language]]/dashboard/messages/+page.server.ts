import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { env } from '$env/dynamic/public';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
    const token = cookies.get('zafaf_client_session');
    if (!token) {
        throw redirect(303, '/auth/login');
    }

    const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';

    // Fetch user profile so the page has access to user.id without a client-side
    // fetch — this also prevents the runtime crash where data.user was undefined.
    let user: any = null;
    try {
        const res = await fetch(`${API_BASE}/api/v1/auth/me`, {
            headers: {
                'Authorization': `Bearer ${token}`,
                'Cookie': `zafaf_client_session=${token}`
            },
            signal: AbortSignal.timeout(5000)
        });
        if (res.ok) {
            const body = await res.json();
            if (body.status === 'success') {
                user = body.user;
            }
        }
    } catch (err) {
        // Non-fatal — the page degrades gracefully without the user object.
        // A missing user object won't block navigation.
        console.warn('[Messages Page Loader] Could not fetch user profile:', err);
    }

    return { token, user };
};
