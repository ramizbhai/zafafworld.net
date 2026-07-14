import { redirect } from '@sveltejs/kit';
import { env } from '$env/dynamic/public';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
    const token = cookies.get('zafaf_admin_session');
    if (!token) {
        throw redirect(303, '/login');
    }

    const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';
    let inquiries: any[] = [];

    try {
        const res = await fetch(`${API_BASE}/api/v1/admin/afrah/inquiries`, {
            headers: { Authorization: `Bearer ${token}` },
        });
        if (res.ok) {
            const body = await res.json();
            if (body.status === 'success') {
                inquiries = body.inquiries || [];
            }
        }
    } catch (e) {
        console.error('Failed to fetch Afrah inquiries:', e);
    }

    return { inquiries };
};
