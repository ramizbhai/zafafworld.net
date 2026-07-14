import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies }) => {
    const token = cookies.get('zafaf_admin_session');
    if (!token) {
        throw redirect(303, '/login');
    }
    return {};
};
