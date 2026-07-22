import { redirect } from '@sveltejs/kit';
import type { RequestEvent } from '@sveltejs/kit';

export const GET = async ({}: RequestEvent) => {
    throw redirect(301, '/sitemap.xml');
};
