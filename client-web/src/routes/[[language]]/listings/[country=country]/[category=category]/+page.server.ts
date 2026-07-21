import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { SORT_API_TO_URL, DEFAULT_SORT_SLUG } from '$params/sort.js';
import { CITY_SLUGS } from '$params/city.js';
import { buildListingsUrl } from '$lib/utils/navigation.js';

export const load: PageServerLoad = async ({ url, params }) => {
    // The category comes from the OLD path segment
    const categoryFromPath = params.category !== 'all' ? params.category : '';

    // City comes from the query string (old hybrid format)
    const cityParam = url.searchParams.get('city') || '';
    const validCity = cityParam && CITY_SLUGS.has(cityParam.toLowerCase())
        ? cityParam.toLowerCase()
        : '';

    // Sort: read from query param, convert API enum → URL slug
    const sortRaw = url.searchParams.get('sort') || '';
    let sortSlug = '';
    if (sortRaw) {
        const mapped = SORT_API_TO_URL[sortRaw.toLowerCase()];
        if (mapped && mapped !== DEFAULT_SORT_SLUG) sortSlug = mapped;
    }

    const cleanPath = buildListingsUrl({
        city:     validCity        || undefined,
        category: categoryFromPath || undefined,
        sort:     sortSlug         || undefined,
    });

    const keepParams = new URLSearchParams(url.searchParams);
    keepParams.delete('city');
    keepParams.delete('sort');
    const queryString = keepParams.toString() ? `?${keepParams.toString()}` : '';

    throw redirect(301, `${cleanPath}${queryString}`);
};
