import { env } from "$env/dynamic/public";
import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ params, cookies, fetch }) => {
    const sessionToken = cookies.get('zafaf_admin_session');
    if (!sessionToken) {
        return json({ error: 'Unauthorized administrative session' }, { status: 401 });
    }

    try {
        const response = await fetch(`${env.PUBLIC_API_URL || `${env.PUBLIC_API_URL || 'http://localhost:8080'}`}/api/v1/admin/vendors/${params.id}/chat/messages`, {
            headers: {
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_admin_session=${sessionToken}`
            }
        });

        if (!response.ok) {
            const errData = await response.json().catch(() => ({}));
            return json({ error: errData.error?.message || `Failed to fetch messages (Status ${response.status})` }, { status: response.status });
        }

        const data = await response.json();
        return json(data);
    } catch (err: any) {
        console.error('Error fetching chat messages in proxy:', err);
        return json({ error: err.message || 'Internal connection error' }, { status: 500 });
    }
};

export const POST: RequestHandler = async ({ params, cookies, request, fetch }) => {
    const sessionToken = cookies.get('zafaf_admin_session');
    if (!sessionToken) {
        return json({ error: 'Unauthorized administrative session' }, { status: 401 });
    }

    try {
        const { body } = await request.json();
        if (!body || !body.trim()) {
            return json({ error: 'Message body cannot be empty' }, { status: 400 });
        }

        const response = await fetch(`${env.PUBLIC_API_URL || `${env.PUBLIC_API_URL || 'http://localhost:8080'}`}/api/v1/admin/vendors/${params.id}/chat/reply`, {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_admin_session=${sessionToken}`,
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ body })
        });

        if (!response.ok) {
            const errData = await response.json().catch(() => ({}));
            return json({ error: errData.error?.message || `Failed to send reply (Status ${response.status})` }, { status: response.status });
        }

        const data = await response.json();
        return json(data);
    } catch (err: any) {
        console.error('Error sending chat reply in proxy:', err);
        return json({ error: err.message || 'Internal connection error' }, { status: 500 });
    }
};
