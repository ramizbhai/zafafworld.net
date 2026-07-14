import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ url }) => {
    const params = url.searchParams.toString();
    const target = params ? `/listings?${params}` : '/listings';
    throw redirect(301, target);
};
