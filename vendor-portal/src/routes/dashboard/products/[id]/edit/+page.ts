import { redirect } from '@sveltejs/kit';

export function load({ params }) {
    throw redirect(302, `/dashboard/products/${params.id}/edit/step-1`);
}
