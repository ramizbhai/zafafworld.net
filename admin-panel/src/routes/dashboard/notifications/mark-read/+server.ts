import { env } from "$env/dynamic/public";
import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async ({ request, cookies, fetch }) => {
    const sessionToken = cookies.get('zafaf_admin_session');
    if (!sessionToken) {
        return json({ status: 'error', message: 'Unauthorized' }, { status: 401 });
    }

    try {
        const payload = await request.json();
        const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/notifications/mark-read`, {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_admin_session=${sessionToken}`,
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(payload)
        });

        if (!response.ok) {
            const errData = await response.json().catch(() => ({}));
            return json({ status: 'error', message: errData.message || 'Failed to mark notifications as read' }, { status: response.status });
        }

        const data = await response.json();
        return json(data);
    } catch (err: any) {
        console.error('Error proxying mark-read notifications:', err);
        return json({ status: 'error', message: err.message || 'Internal error' }, { status: 500 });
    }
};
