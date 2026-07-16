import { env } from '$env/dynamic/public';
import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async ({ request, cookies, fetch }) => {
    const sessionToken = cookies.get('zafaf_admin_session');
    
    if (!sessionToken) {
        return json({ status: 'error', message: 'Unauthorized' }, { status: 401 });
    }

    try {
        const apiBase = env.PUBLIC_API_URL || 'http://localhost:8080';
        
        const response = await fetch(`${apiBase}/api/v1/admin/blogs/upload`, {
            method: 'POST',
            body: request.body,
            headers: {
                'Authorization': `Bearer ${sessionToken}`,
                'Content-Type': request.headers.get('content-type') || ''
            },
            // @ts-expect-error Node fetch duplex mode required for stream bodies
            duplex: 'half'
        });

        const data = await response.json();
        return json(data, { status: response.status });
    } catch (e) {
        console.error("Upload proxy error:", e);
        return json({ status: 'error', message: 'Upload failed' }, { status: 500 });
    }
};
