import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ locals, cookies }) => {
    console.log("ENTER root +layout.server.ts");
    try {
        const locale = cookies.get('zafaf_locale') || 'ar';
        const res = {
            user: locals.user,
            locale
        };
        console.log("EXIT root +layout.server.ts");
        return res;
    } catch (err: any) {
        console.error("ERROR root +layout.server.ts:", err?.stack || err);
        throw err;
    }
};
