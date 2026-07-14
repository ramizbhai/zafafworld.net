import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies }) => {
    // Clear session cookie securely
    cookies.delete('zafaf_admin_session', { path: '/' });
    
    // Redirect instantly to login route
    throw redirect(303, '/login');
};

