import { apiClient } from '$lib/api/client';
import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
    const sessionToken = cookies.get('zafaf_vendor_session');
    
    // Sync logout with the backend to strictly revoke the session token
    if (sessionToken) {
        try {
            await apiClient.withFetch(fetch).auth.logout(sessionToken);
        } catch (err) {
            console.error('Failed to notify backend of logout:', err);
        }
    }

    // Flush the HttpOnly session cookie
    cookies.delete('zafaf_vendor_session', { path: '/' });
    
    // Redirect safely back to the login portal
    throw redirect(303, '/login');
};

