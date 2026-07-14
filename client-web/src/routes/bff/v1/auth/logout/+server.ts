import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { env } from '$env/dynamic/public';
import { dev } from '$app/environment';

export const POST: RequestHandler = async ({ cookies, fetch }) => {
    const sessionToken = cookies.get('zafaf_client_session');
    
    // Clear the cookie immediately on the client side
    cookies.delete('zafaf_client_session', { path: '/' });
    
    if (sessionToken) {
        try {
            const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';
            // Call Rust backend logout handler to invalidate the session on the server side
            await fetch(`${API_BASE}/api/v1/auth/logout`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`
                }
            });
        } catch (err) {
            console.error('[Logout Endpoint] Error invalidating session on backend:', err);
        }
    }
    
    return json({ status: 'success' });
};
