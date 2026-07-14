import { redirect } from '@sveltejs/kit';

export function GET({ params }) {
    throw redirect(303, `/dashboard/products/${params.id}/edit/step-1`);
}
