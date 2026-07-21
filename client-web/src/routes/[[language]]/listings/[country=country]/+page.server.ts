import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { SORT_API_TO_URL, DEFAULT_SORT_SLUG } from '$params/sort.js';
import { CITY_SLUGS } from '$params/city.js';
import { CATEGORY_SLUGS } from '$params/category.js';

/**
 * Legacy /listings/[country] redirect shim.
 *
 * Catches URLs of the form /ar/listings/sa (old country-level browse).
 * 301-redirects to /{lang}/ since country is no longer a URL segment.
 *
 * Also handles /ar/listings/sa?category=X&city=Y by forwarding city+category
 * into the new clean path.
 */
export const load: PageServerLoad = async ({ url, params }) => {
    const langPrefix = params.language ? `/${params.language}` : '';

    const cityParam     = url.searchParams.get('city')     || '';
    const categoryParam = url.searchParams.get('category') || '';
    const sortRaw       = url.searchParams.get('sort')     || '';

    const validCity     = cityParam     && CITY_SLUGS.has(cityParam.toLowerCase())     ? cityParam.toLowerCase()     : '';
    const validCategory = categoryParam && CATEGORY_SLUGS.has(categoryParam.toLowerCase()) ? categoryParam.toLowerCase() : '';

    let sortSlug = '';
    if (sortRaw) {
        const mapped = SORT_API_TO_URL[sortRaw.toLowerCase()];
        if (mapped && mapped !== DEFAULT_SORT_SLUG) sortSlug = mapped;
    }

    const segments: string[] = [];
    if (validCity)     segments.push(validCity);
    if (validCategory) segments.push(validCategory);
    if (sortSlug && validCategory) segments.push(sortSlug);

    const newPath = segments.length > 0
        ? `${langPrefix}/${segments.join('/')}/`
        : `${langPrefix}/`;

    const keepParams = new URLSearchParams(url.searchParams);
    keepParams.delete('city');
    keepParams.delete('category');
    keepParams.delete('sort');
    const queryString = keepParams.toString() ? `?${keepParams.toString()}` : '';

    throw redirect(301, `${newPath}${queryString}`);
};
