import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { SORT_API_TO_URL, DEFAULT_SORT_SLUG } from '$params/sort.js';
import { CITY_SLUGS } from '$params/city.js';
import { CATEGORY_SLUGS } from '$params/category.js';
import { buildListingsUrl } from '$lib/utils/navigation.js';

export const load: PageServerLoad = async ({ url, cookies, params }) => {
    // Read the old query-param filter values
    const cityParam     = url.searchParams.get('city')     || '';
    const categoryParam = url.searchParams.get('category') || '';
    const sortRaw       = url.searchParams.get('sort')     || '';

    // Validate city and category against allowlists
    const validCity     = cityParam     && CITY_SLUGS.has(cityParam.toLowerCase())     ? cityParam.toLowerCase()     : '';
    const validCategory = categoryParam && CATEGORY_SLUGS.has(categoryParam.toLowerCase()) ? categoryParam.toLowerCase() : '';

    // Normalise sort
    let sortSlug = '';
    if (sortRaw) {
        const mapped = SORT_API_TO_URL[sortRaw.toLowerCase()];
        if (mapped) {
            sortSlug = mapped === DEFAULT_SORT_SLUG ? '' : mapped;
        } else {
            const urlSlugs = Object.keys(SORT_API_TO_URL);
            if (urlSlugs.includes(sortRaw.toLowerCase()) && sortRaw.toLowerCase() !== DEFAULT_SORT_SLUG) {
                sortSlug = sortRaw.toLowerCase();
            }
        }
    }

    const cleanPath = buildListingsUrl({
        city:     validCity     || undefined,
        category: validCategory || undefined,
        sort:     sortSlug      || undefined,
    });

    // Preserve secondary query params
    const keepParams = new URLSearchParams(url.searchParams);
    keepParams.delete('city');
    keepParams.delete('category');
    keepParams.delete('country');
    keepParams.delete('sort');
    keepParams.delete('q');
    keepParams.delete('tier');

    const queryString = keepParams.toString() ? `?${keepParams.toString()}` : '';

    throw redirect(301, `${cleanPath}${queryString}`);
};
