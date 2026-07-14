import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ locals, cookies }) => {
    const locale = cookies.get('zafaf_locale') || 'ar';
    return {
        user: locals.user,
        locale
    };
};
