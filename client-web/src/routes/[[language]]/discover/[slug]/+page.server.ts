import { env } from '$env/dynamic/private';
import { error } from '@sveltejs/kit';
import DOMPurify from 'isomorphic-dompurify';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params, fetch }) => {
    const slug = params.slug;
    const apiBase = env.PUBLIC_API_URL || 'http://backend:8080';

    let engagementData: any = null;

    try {
        const engagementRes = await fetch(`${apiBase}/api/v1/public/blogs/${slug}`);
        if (engagementRes.ok) {
            const body = await engagementRes.json();
            if (body.status === 'success' && body.data) {
                engagementData = body.data;
            }
        } else if (engagementRes.status === 404) {
            throw error(404, 'Blog post not found');
        }
    } catch (err: any) {
        console.error(`[Public Blog Loader] API request exception for slug "${slug}":`, err);
        throw error(500, 'Internal Server Error');
    }

    if (!engagementData) {
        throw error(404, 'Blog post not found');
    }

    // Sanitize content HTML unconditionally
    const sanitizedHtml = DOMPurify.sanitize(
        engagementData.content_html ?? '',
        {
            ADD_TAGS: ['figure', 'figcaption', 'iframe'],
            ADD_ATTR: ['allow', 'allowfullscreen', 'frameborder', 'scrolling']
        }
    );

    // Fetch related posts (from Rust)
    let related: any[] = [];
    try {
        const relatedRes = await fetch(`${apiBase}/api/v1/public/blogs/related/${slug}`);
        if (relatedRes.ok) {
            const relatedBody = await relatedRes.json();
            related = relatedBody.data ?? [];
        }
    } catch {
        // Non-fatal
    }

    // Merge sanitized content into post payload
    const post = {
        ...engagementData,
        content_html: sanitizedHtml,
    };

    return {
        post,
        related,
    };
};
