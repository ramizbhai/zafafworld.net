import type { ParamMatcher } from '@sveltejs/kit';

export const match: ParamMatcher = (param) => {
    // Match strict two-letter ISO country codes
    return /^[a-z]{2}$/.test(param.toLowerCase());
};
