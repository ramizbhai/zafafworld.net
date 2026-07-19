import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ url, cookies, params }) => {
    // Graceful fallback for incomplete paths or old query parameters
    const defaultCountry = cookies.get('zafaf_selected_country') || 'sa';
    const countryParam = url.searchParams.get('country') || defaultCountry;
    const categoryParam = url.searchParams.get('category') || 'all';
    
    const country = countryParam.toLowerCase();
    const category = categoryParam.toLowerCase();

    // Preserve any remaining query parameters
    const queryParams = new URLSearchParams(url.searchParams);
    queryParams.delete('country');
    queryParams.delete('category');
    
    const langPrefix = params.language ? `/${params.language}` : '';
    const queryString = queryParams.toString() ? `?${queryParams.toString()}` : '';
    
    throw redirect(301, `${langPrefix}/listings/${country}/${category}${queryString}`);
};
