import { env } from "$env/dynamic/public";
import { redirect, fail } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
    const sessionToken = cookies.get('zafaf_admin_session');

    if (!sessionToken) {
        throw redirect(303, '/login');
    }

    try {
        const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/settings`, {
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
            return { settings: {}, error: 'Failed to load platform configurations' };
        }

        const data = await response.json();
        if (data.status === 'success') {
            return {
                settings: data.settings || {}
            };
        }

        return { settings: {}, error: data.message || 'Failed to parse configurations' };

    } catch (err) {
        console.error('Failed to fetch admin settings:', err);
        return { settings: {}, error: 'Internal connection error' };
    }
};

export const actions: Actions = {
    saveSettings: async ({ request, cookies, fetch }) => {
        const sessionToken = cookies.get('zafaf_admin_session');

        if (!sessionToken) {
            throw redirect(303, '/login');
        }

        const formData = await request.formData();
        const settingsMap: Record<string, string> = {};

        // Parse key-value strings from form submission
        for (const [key, value] of formData.entries()) {
            // Svelte action adds Svelte specific fields occasionally, skip javascript helpers
            if (key.startsWith('__')) continue;
            settingsMap[key] = value.toString();
        }

        try {
            const response = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/admin/settings`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_admin_session=${sessionToken}`
                },
                body: JSON.stringify({ settings: settingsMap })
            });

            if (!response.ok) {
                const errData = await response.json().catch(() => ({}));
                return fail(response.status, {
                    success: false,
                    error: errData.message || 'Failed to save configurations'
                });
            }

            const data = await response.json();
            if (data.status === 'success') {
                return { success: true };
            }

            return fail(400, {
                success: false,
                error: data.message || 'Failed to save settings'
            });

        } catch (err) {
            console.error('Failed to save settings action:', err);
            return fail(500, { success: false, error: 'Network error saving settings' });
        }
    }
};
