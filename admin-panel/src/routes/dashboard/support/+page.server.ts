import { env } from "$env/dynamic/public";
import { redirect } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ cookies }) => {
    const sessionToken = cookies.get('zafaf_admin_session');

    if (!sessionToken) {
        throw redirect(303, '/login');
    }

    return {
        inquiries: []
    };
};

export const actions: Actions = {};
