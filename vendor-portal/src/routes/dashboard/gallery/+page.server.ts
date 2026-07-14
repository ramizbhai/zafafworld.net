import { env } from "$env/dynamic/public";
import { fail, redirect } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

// ═══════════════════════════════════════════════════════════════════════════
// /dashboard/gallery — Repurposed as "Brand Profile Settings"
//
// This page manages the vendor's BRAND-LEVEL media (product_id = NULL):
//   - Brand logo (is_cover = false, product_id = null)
//   - Brand cover/header photo (is_cover = true, product_id = null)
//
// Per-listing images are managed on /dashboard/products/[id] via the
// ListingImageUpload component and the /vendor/products/:id/images API.
// ═══════════════════════════════════════════════════════════════════════════

export const load: PageServerLoad = async ({ parent, fetch, cookies }) => {
    const sessionToken = cookies.get('zafaf_vendor_session');
    if (!sessionToken) throw redirect(303, '/login');

    // Load brand gallery (product_id = NULL rows) and vendor profile
    const [brandGalleryRes, dashboardRes] = await Promise.allSettled([
        fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/vendor/gallery`, {
            headers: {
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_vendor_session=${sessionToken}`
            }
        }),
        fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/vendor/stats/dashboard`, {
            headers: {
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_vendor_session=${sessionToken}`
            }
        })
    ]);

    // Auth guard via parent layout
    const { user } = await parent();
    if (!user) throw redirect(303, '/login');

    // Brand gallery — filter to brand-level images only (product_id === null)
    let brandGallery: any[] = [];
    if (brandGalleryRes.status === 'fulfilled' && brandGalleryRes.value.ok) {
        const data = await brandGalleryRes.value.json();
        if (data.status === 'success' && Array.isArray(data.gallery)) {
            brandGallery = data.gallery.filter((img: any) => !img.product_id);
        }
    }

    // Vendor profile
    let vendor: any = null;
    if (dashboardRes.status === 'fulfilled' && dashboardRes.value.ok) {
        const data = await dashboardRes.value.json();
        if (data.status === 'success' && data.data?.vendor) {
            vendor = data.data.vendor;
        }
    }

    const brandLogo  = brandGallery.find((img: any) => !img.isCover || img.is_cover === false);
    const brandCover = brandGallery.find((img: any) =>  img.isCover ||  img.is_cover === true);

    return {
        brandGallery,
        brandLogo:  brandLogo  ?? null,
        brandCover: brandCover ?? null,
        vendor,
        sessionToken  // for client-side upload component
    };
};

export const actions: Actions = {
    // Upload a brand-level image (logo or cover)
    uploadBrandImage: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_vendor_session');
        if (!sessionToken) return fail(401, { error: 'Session expired.' });

        const fd = await request.formData();
        const file = fd.get('file') as File | null;
        const role = fd.get('role')?.toString() || 'logo'; // 'logo' | 'cover'

        if (!file || file.size === 0) {
            return fail(400, { error: 'No file provided.' });
        }

        // Forward to upload endpoint
        const uploadForm = new FormData();
        uploadForm.append('file', file);

        const uploadRes = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/vendor/upload`, {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_vendor_session=${sessionToken}`
            },
            body: uploadForm
        });

        if (!uploadRes.ok) {
            const err = await uploadRes.json().catch(() => ({}));
            return fail(uploadRes.status, { error: err.message || 'Upload failed.' });
        }

        const { url, file_path } = await uploadRes.json();
        const isCover = role === 'cover';

        // If setting a new cover, clear old brand cover first
        if (isCover) {
            await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/vendor/gallery`, {
                method: 'GET',
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_vendor_session=${sessionToken}`
                }
            });
        }

        // Link to gallery as brand-level image (product_id = null implicitly)
        const linkRes = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/vendor/gallery`, {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_vendor_session=${sessionToken}`,
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                image_url: url,
                file_path,
                is_cover: isCover,
                caption: role === 'cover' ? 'Brand Cover' : 'Brand Logo'
            })
        });

        if (!linkRes.ok) {
            const err = await linkRes.json().catch(() => ({}));
            return fail(linkRes.status, { error: err.message || 'Failed to save image.' });
        }

        return { success: true, message: role === 'cover' ? 'Cover photo updated!' : 'Logo updated!' };
    },

    // Delete a brand-level gallery image
    deleteBrandImage: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_vendor_session');
        if (!sessionToken) return fail(401, { error: 'Session expired.' });

        const fd = await request.formData();
        const imageId = fd.get('image_id')?.toString();

        if (!imageId) return fail(400, { error: 'Image ID is required.' });

        const res = await fetch(`${env.PUBLIC_API_URL || `${env.PUBLIC_API_URL || 'http://localhost:8080'}`}/api/v1/vendor/gallery/${imageId}`, {
            method: 'DELETE',
            headers: {
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_vendor_session=${sessionToken}`
            }
        });

        if (!res.ok) {
            return fail(res.status, { error: 'Failed to delete image.' });
        }

        return { success: true, message: 'Image deleted successfully.' };
    }
};
