import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ url, params }) => {
    const categoryParam = url.searchParams.get('category') || 'all';
    const category = categoryParam.toLowerCase();

    const queryParams = new URLSearchParams(url.searchParams);
    queryParams.delete('category');
    
    const langPrefix = params.language ? `/${params.language}` : '';
    const queryString = queryParams.toString() ? `?${queryParams.toString()}` : '';
    
    throw redirect(301, `${langPrefix}/listings/${params.country}/${category}${queryString}`);
};
