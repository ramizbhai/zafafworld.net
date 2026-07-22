import { generateSitemapXml } from '$lib/services/sitemap.js';
import type { RequestEvent } from '@sveltejs/kit';

export const GET = async ({ fetch }: RequestEvent) => {
    return generateSitemapXml(fetch);
};
