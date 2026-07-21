import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { buildListingsUrl } from '$lib/utils/navigation.js';

export const load: PageServerLoad = async ({ url, params }) => {
    const cleanPath = buildListingsUrl({
        city:     params.city     || undefined,
        category: params.category || undefined,
        sort:     params.sort     || undefined,
    });

    const keepParams = new URLSearchParams(url.searchParams);
    const queryString = keepParams.toString() ? `?${keepParams.toString()}` : '';

    throw redirect(301, `${cleanPath}${queryString}`);
};
