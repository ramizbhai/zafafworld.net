import { env } from "$env/dynamic/public";
import { redirect, fail } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch, params }) => {
    const sessionToken = cookies.get('zafaf_admin_session');
    if (!sessionToken) {
        throw redirect(303, '/login');
    }

    const { id } = params;
    const apiBase = env.PUBLIC_API_URL || 'http://localhost:8080';

    try {
        const response = await fetch(`${apiBase}/api/v1/admin/inquiries/${id}`, {
            headers: {
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_admin_session=${sessionToken}`
            }
        });

        if (!response.ok) {
            if (response.status === 401 || response.status === 403) {
                cookies.delete('zafaf_admin_session', { path: '/' });
                throw redirect(303, '/login');
            }
            throw redirect(303, '/dashboard/inquiries');
        }

        const data = await response.json();
        return {
            inquiry: data.inquiry,
            client: data.client,
            vendor: data.vendor,
            listing: data.listing,
            city: data.city,
            conversation: data.conversation || [],
            adminNotes: data.adminNotes || [],
            management: data.management || {}
        };
    } catch (err) {
        if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
            throw err;
        }
        console.error('Failed to load inquiry detail:', err);
        throw redirect(303, '/dashboard/inquiries');
    }
};

export const actions: Actions = {
    addNote: async ({ request, cookies, fetch, params }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) return fail(401, { error: 'Unauthorized session' });

        const formData = await request.formData();
        const note = formData.get('note') as string;
        const noteType = formData.get('note_type') as string || 'internal';

        if (!note || !note.trim()) return fail(400, { error: 'Note content is required' });

        try {
            const res = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/inquiries/${params.id}/notes`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ note: note.trim(), note_type: noteType, is_internal: true })
            });

            if (!res.ok) {
                const err = await res.json().catch(() => ({}));
                return fail(res.status, { error: err.message || 'Failed to add note' });
            }
            return { success: true };
        } catch (err: any) {
            return fail(500, { error: err.message || 'Server error' });
        }
    },

    deleteNote: async ({ request, cookies, fetch, params }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) return fail(401, { error: 'Unauthorized session' });

        const formData = await request.formData();
        const noteId = formData.get('note_id') as string;
        if (!noteId) return fail(400, { error: 'Missing note ID' });

        try {
            const res = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/inquiries/${params.id}/notes/${noteId}`, {
                method: 'DELETE',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`
                }
            });

            if (!res.ok) {
                const err = await res.json().catch(() => ({}));
                return fail(res.status, { error: err.message || 'Failed to delete note' });
            }
            return { success: true };
        } catch (err: any) {
            return fail(500, { error: err.message || 'Server error' });
        }
    },

    updateManagement: async ({ request, cookies, fetch, params }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) return fail(401, { error: 'Unauthorized session' });

        const formData = await request.formData();
        const priority = formData.get('priority') as string;
        const escalationStatus = formData.get('escalation_status') as string;
        const resolutionStatus = formData.get('resolution_status') as string;

        try {
            const res = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/inquiries/${params.id}/management`, {
                method: 'PATCH',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({
                    priority: priority || undefined,
                    escalation_status: escalationStatus || undefined,
                    resolution_status: resolutionStatus || undefined
                })
            });

            if (!res.ok) {
                const err = await res.json().catch(() => ({}));
                return fail(res.status, { error: err.message || 'Management update failed' });
            }
            return { success: true };
        } catch (err: any) {
            return fail(500, { error: err.message || 'Server error' });
        }
    },

    updateStatus: async ({ request, cookies, fetch, params }) => {
        const sessionToken = cookies.get('zafaf_admin_session');
        if (!sessionToken) return fail(401, { error: 'Unauthorized session' });

        const formData = await request.formData();
        const status = formData.get('status') as string;

        try {
            const res = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/inquiries/${params.id}/status`, {
                method: 'PATCH',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ status })
            });

            if (!res.ok) {
                const err = await res.json().catch(() => ({}));
                return fail(res.status, { error: err.message || 'Status update failed' });
            }
            return { success: true };
        } catch (err: any) {
            return fail(500, { error: err.message || 'Server error' });
        }
    }
};
