import type { ParamMatcher } from '@sveltejs/kit';

export const match: ParamMatcher = (param) => {
    // Category should be alphanumeric tokens optionally separated by hyphens (e.g. hotel-venue, all)
    return /^[a-z0-9-]+$/.test(param.toLowerCase());
};
