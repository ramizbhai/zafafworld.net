import { env } from "$env/dynamic/public";
import { fail, redirect, error } from '@sveltejs/kit';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ parent, fetch, cookies, params }) => {
    const sessionToken = cookies.get('zafaf_vendor_session');

    const { user } = await parent();
    if (!user) {
        throw redirect(303, '/login');
    }

    const promoId = params.id;

    // Fetch the specific promotion by ID + vendor's products concurrently (Phase 4: direct GET)
    const promotionPromise = fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/vendor/promotions/${promoId}`, {
        headers: {
            'Authorization': `Bearer ${sessionToken}`,
            'Cookie': `zafaf_vendor_session=${sessionToken}`
        }
    });

    const productsPromise = fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/vendor/products`, {
        headers: {
            'Authorization': `Bearer ${sessionToken}`,
            'Cookie': `zafaf_vendor_session=${sessionToken}`
        }
    });

    let promotion: any = null;
    let products: any[] = [];

    try {
        const [promoResponse, productsResponse] = await Promise.all([promotionPromise, productsPromise]);

        if (promoResponse.ok) {
            const data = await promoResponse.json();
            promotion = data.promotion || null;
        } else if (promoResponse.status === 404) {
            throw error(404, { message: 'Promotion not found.' });
        } else if (promoResponse.status === 403) {
            throw error(403, { message: 'Access denied.' });
        }

        if (productsResponse.ok) {
            const data = await productsResponse.json();
            products = (data.products || []).filter((p: any) => p.metadata?.status === 'active');
        }
    } catch (err: any) {
        if (err?.status === 404 || err?.status === 403) throw err;
        console.error('Failed to load promotion or products for edit:', err);
    }

    if (!promotion) {
        throw error(404, { message: 'Promotion not found.' });
    }

    return {
        promotion,
        products
    };
};

export const actions: Actions = {
    update: async ({ request, fetch, cookies, params }) => {
        const sessionToken = cookies.get('zafaf_vendor_session');
        if (!sessionToken) {
            return fail(401, { error: 'Your session has expired.' });
        }

        const promoId = params.id;
        const formData = await request.formData();

        const listing_id = formData.get('listing_id')?.toString() || '';
        const promo_type = formData.get('promo_type')?.toString() || 'discount';
        const discount_type = formData.get('discount_type')?.toString() || null;
        const discount_percentage = formData.get('discount_percentage') ? parseInt(formData.get('discount_percentage')?.toString() || '0') : null;
        const discount_fixed_amount = formData.get('discount_fixed_amount') ? parseFloat(formData.get('discount_fixed_amount')?.toString() || '0') : null;
        const benefit_description_en = formData.get('benefit_description_en')?.toString() || null;
        const benefit_description_ar = formData.get('benefit_description_ar')?.toString() || null;
        const use_listing_cover_image = formData.get('use_listing_cover_image')?.toString() === 'true';
        let banner_image_url = formData.get('banner_image_url')?.toString() || '';
        let banner_file_id: string | null = null;
        const banner_file = formData.get('banner_file') as File | null;

        const title_en = formData.get('title_en')?.toString() || '';
        const title_ar = formData.get('title_ar')?.toString() || '';
        const description_en = formData.get('description_en')?.toString() || '';
        const description_ar = formData.get('description_ar')?.toString() || '';
        const badge_text_en = formData.get('badge_text_en')?.toString() || '';
        const badge_text_ar = formData.get('badge_text_ar')?.toString() || '';
        const start_at = formData.get('start_at')?.toString() || '';
        const end_at = formData.get('end_at')?.toString() || '';

        // Validate required fields
        if (!listing_id) {
            return fail(400, { error: 'Listing selection is required.', errorAr: 'تحديد المنتج مطلوب.' });
        }

        if (!title_en.trim() || !title_ar.trim()) {
            return fail(400, { error: 'Both English and Arabic titles are required.', errorAr: 'العنوان بالعربية والإنجليزية مطلوب.' });
        }

        if (title_en.length > 255) {
            return fail(400, { error: 'Title (English) must not exceed 255 characters.', errorAr: 'يجب ألا يتجاوز العنوان (بالإنجليزية) 255 حرفاً.' });
        }
        if (title_ar.length > 255) {
            return fail(400, { error: 'Title (Arabic) must not exceed 255 characters.', errorAr: 'يجب ألا يتجاوز العنوان (بالعربية) 255 حرفاً.' });
        }

        if (promo_type !== 'discount' && promo_type !== 'benefit') {
            return fail(400, { error: 'Invalid promotion type.', errorAr: 'نوع العرض غير صالح.' });
        }

        if (promo_type === 'discount') {
            if (discount_type !== 'percentage' && discount_type !== 'fixed_amount') {
                return fail(400, { error: 'Discount type is required.', errorAr: 'نوع الخصم مطلوب.' });
            }

            if (discount_type === 'percentage') {
                if (!discount_percentage || discount_percentage < 5 || discount_percentage > 90) {
                    return fail(400, { error: 'Discount percentage must be between 5% and 90%.', errorAr: 'يجب أن تكون نسبة الخصم بين ٥٪ و ٩٠٪.' });
                }
            } else {
                if (!discount_fixed_amount || discount_fixed_amount <= 0) {
                    return fail(400, { error: 'Fixed discount amount must be greater than 0.', errorAr: 'يجب أن يكون مبلغ الخصم أكبر من 0.' });
                }
            }
        } else {
            if (!benefit_description_en?.trim() || !benefit_description_ar?.trim()) {
                return fail(400, { error: 'Benefit descriptions in both languages are required.', errorAr: 'وصف الميزة بكلا اللغتين مطلوب.' });
            }
            if ((benefit_description_en?.length ?? 0) > 255) {
                return fail(400, { error: 'Benefit description (English) must not exceed 255 characters.', errorAr: 'يجب ألا يتجاوز وصف الميزة (بالإنجليزية) 255 حرفاً.' });
            }
            if ((benefit_description_ar?.length ?? 0) > 255) {
                return fail(400, { error: 'Benefit description (Arabic) must not exceed 255 characters.', errorAr: 'يجب ألا يتجاوز وصف الميزة (بالعربية) 255 حرفاً.' });
            }
        }

        // Word count limit validation (max 2000 words after stripping HTML)
        const countWords = (html: string) => {
            const text = html.replace(/<[^>]*>/g, ' ');
            return text.trim().split(/\s+/).filter(w => w.length > 0).length;
        };

        if (countWords(description_en) > 2000) {
            return fail(400, { error: 'Description (English) must not exceed 2000 words.', errorAr: 'يجب ألا يتجاوز الوصف (بالإنجليزية) 2000 كلمة.' });
        }
        if (countWords(description_ar) > 2000) {
            return fail(400, { error: 'Description (Arabic) must not exceed 2000 words.', errorAr: 'يجب ألا يتجاوز الوصف (بالعربية) 2000 كلمة.' });
        }

        if (!start_at || !end_at) {
            return fail(400, { error: 'Start and end dates are required.', errorAr: 'تاريخ البداية والنهاية مطلوبان.' });
        }

        if (new Date(end_at) <= new Date(start_at)) {
            return fail(400, { error: 'End date must be after start date.', errorAr: 'تاريخ النهاية يجب أن يكون بعد تاريخ البداية.' });
        }

        // Upload banner image if a file was provided
        if (banner_file && banner_file.size > 0) {
            try {
                const uploadForm = new FormData();
                uploadForm.append('file', banner_file);

                const uploadRes = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/vendor/promotions/upload-banner`, {
                    method: 'POST',
                    headers: {
                        'Authorization': `Bearer ${sessionToken}`,
                        'Cookie': `zafaf_vendor_session=${sessionToken}`
                    },
                    body: uploadForm
                });

                if (uploadRes.ok) {
                    const uploadData = await uploadRes.json();
                    if (uploadData.status === 'success' && uploadData.url) {
                        banner_image_url = uploadData.url;
                        banner_file_id = uploadData.id || null;
                    }
                } else {
                    const err = await uploadRes.json().catch(() => ({}));
                    return fail(uploadRes.status || 400, {
                        error: err.message || 'Failed to upload banner image.',
                        errorAr: err.message || 'فشل رفع صورة البانر.'
                    });
                }
            } catch (uploadErr) {
                console.error('Banner upload error:', uploadErr);
                return fail(500, { error: 'Failed to upload banner image.', errorAr: 'فشل رفع صورة البانر.' });
            }
        }

        // Helper: best-effort cleanup of a newly-uploaded banner if the main update call fails
        // (Only clean up banners uploaded in THIS request, not the existing stored one)
        const newlyUploadedBanner = banner_file && banner_file.size > 0 ? banner_image_url : '';
        async function cleanupNewBanner() {
            if (!newlyUploadedBanner) return;
            try {
                await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/vendor/promotions/cleanup-banner`, {
                    method: 'DELETE',
                    headers: {
                        'Content-Type': 'application/json',
                        'Authorization': `Bearer ${sessionToken}`,
                        'Cookie': `zafaf_vendor_session=${sessionToken}`
                    },
                    body: JSON.stringify({ file_url: newlyUploadedBanner })
                });
            } catch (e) {
                console.error('Banner cleanup failed:', e);
            }
        }

        try {
            const apiResponse = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/vendor/promotions/${promoId}`, {
                method: 'PUT',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_vendor_session=${sessionToken}`
                },
                body: JSON.stringify({
                    listing_id,
                    promo_type,
                    discount_type,
                    discount_percentage,
                    discount_fixed_amount,
                    benefit_description_en: benefit_description_en?.trim() || null,
                    benefit_description_ar: benefit_description_ar?.trim() || null,
                    use_listing_cover_image,
                    custom_banner_image_url: banner_image_url.trim() || null,
                    file_id: banner_file_id,
                    title_en: title_en.trim(),
                    title_ar: title_ar.trim(),
                    description_en: description_en.trim() || null,
                    description_ar: description_ar.trim() || null,
                    badge_text_en: badge_text_en.trim() || null,
                    badge_text_ar: badge_text_ar.trim() || null,
                    start_at: new Date(start_at).toISOString(),
                    end_at: new Date(end_at).toISOString()
                })
            });

            const responseData = await apiResponse.json();

            if (!apiResponse.ok || responseData.status !== 'success') {
                // New banner was uploaded but update failed — clean it up
                await cleanupNewBanner();
                return fail(apiResponse.status || 400, {
                    error: responseData.message || 'Failed to update promotion.',
                    errorAr: responseData.message || 'فشل تحديث العرض الترويجي.'
                });
            }

            // Redirect to offers list on success
            throw redirect(303, '/dashboard/offers');

        } catch (err: any) {
            if (err?.status === 303) throw err;
            console.error('Connection error updating promotion:', err);
            // Best-effort banner cleanup on unexpected error
            await cleanupNewBanner().catch(() => {});
            return fail(500, { error: 'Unable to communicate with the server.', errorAr: 'تعذر الاتصال بالخادم.' });
        }
    }
};
