import { env } from "$env/dynamic/public";
import { fail, redirect } from '@sveltejs/kit';
import { safeFetch } from '$lib/utils/api';
import type { PageServerLoad, Actions } from './$types';

export const load: PageServerLoad = async ({ fetch, cookies, depends, locals }) => {
    depends('app:vendor-data');
    const sessionToken = cookies.get('zafaf_vendor_session');
    
    // 1. Block access immediately if no session cookie exists
    if (!sessionToken) {
        throw redirect(303, '/login');
    }

    // Fetch vendor data and cities list in parallel
    const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';
    let cities: any[] = [];

    try {
        // 2. Fetch the active vendor details and cities concurrently
        const [response, citiesRes] = await Promise.all([
            fetch(`${API_BASE}/api/v1/vendor/stats/dashboard`, {
                headers: {
                    'Authorization': `Bearer ${sessionToken}`,
                    'Cookie': `zafaf_vendor_session=${sessionToken}`
                }
            }),
            fetch(`${API_BASE}/api/v1/public/cities`)
                .then(res => res.ok ? res.json() : null)
                .catch(() => null)
        ]);

        // Parse cities
        if (citiesRes && citiesRes.status === 'success' && Array.isArray(citiesRes.cities)) {
            cities = citiesRes.cities;
        }

        if (!response.ok) {
            if (response.status === 401 || response.status === 403) {
                throw redirect(303, '/login');
            }
            return {
                vendor: {
                    name_ar: locals.user?.first_name || 'Partner',
                    name_en: locals.user?.first_name || 'Partner',
                    category: '',
                    status: 'active'
                },
                cities,
                error: 'Encountered a server error or backend is offline.'
            };
        }

        const data = await response.json();
        
        if (data.status !== 'success' || !data.data?.vendor) {
            return {
                vendor: {
                    name_ar: locals.user?.first_name || 'Partner',
                    name_en: locals.user?.first_name || 'Partner',
                    category: '',
                    status: 'active'
                },
                cities,
                error: 'Encountered unexpected response from server.'
            };
        }

        return {
            vendor: data.data.vendor,
            cities
        };
    } catch (err) {
        // Re-throw SvelteKit redirect exceptions directly
        if (err && typeof err === 'object' && 'status' in err && 'location' in err) {
            throw err;
        }
        console.error('Settings page loader error:', err);
        return {
            vendor: {
                name_ar: locals.user?.first_name || 'Partner',
                name_en: locals.user?.first_name || 'Partner',
                category: '',
                status: 'active'
            },
            cities,
            error: 'Unable to connect to the backend server.'
        };
    }
};

export const actions: Actions = {
    updateProfile: async ({ request, fetch, cookies }) => {
        const sessionToken = cookies.get('zafaf_vendor_session');
        if (!sessionToken) {
            return fail(401, { error: 'Unauthorized: Session has expired or is invalid.' });
        }

        const formData = await request.formData();
        const name_ar = formData.get('name_ar')?.toString().trim();
        const name_en = formData.get('name_en')?.toString().trim();
        const description_ar = formData.get('description_ar')?.toString().trim() ?? '';
        const description_en = formData.get('description_en')?.toString().trim() ?? '';
        const address_ar = formData.get('address_ar')?.toString().trim() || undefined;
        const address_en = formData.get('address_en')?.toString().trim() || undefined;
        const phone = formData.get('phone')?.toString().trim() || undefined;
        const email = formData.get('email')?.toString().trim() || undefined;
        
        // Extract capacity ranges safely
        const capacity_min = formData.has('capacity_min') && formData.get('capacity_min') !== '' 
            ? Number(formData.get('capacity_min')) 
            : undefined;
        const capacity_max = formData.has('capacity_max') && formData.get('capacity_max') !== '' 
            ? Number(formData.get('capacity_max')) 
            : undefined;
            
        const latitude = formData.has('latitude') && formData.get('latitude') !== '' ? Number(formData.get('latitude')) : undefined;
        const longitude = formData.has('longitude') && formData.get('longitude') !== '' ? Number(formData.get('longitude')) : undefined;
        const area_sqm = formData.has('area_sqm') && formData.get('area_sqm') !== '' ? Number(formData.get('area_sqm')) : undefined;
        
        // Parse has_partition as boolean
        const has_partition = formData.has('has_partition') && (formData.get('has_partition') === 'on' || formData.get('has_partition') === 'true');
        
        // Extract amenities as an array of strings
        const amenities = formData.getAll('amenities').map(a => a.toString().trim()).filter(Boolean);
        
        const crm_venue_id = formData.get('crm_venue_id')?.toString().trim() || undefined;
        const star_rating = formData.has('star_rating') && formData.get('star_rating') !== '' ? Number(formData.get('star_rating')) : undefined;
        const event_spaces_available = formData.has('event_spaces_available') && formData.get('event_spaces_available') !== '' ? Number(formData.get('event_spaces_available')) : undefined;
        const event_type = formData.get('event_type')?.toString().trim() || undefined;
        const website = formData.get('website')?.toString().trim() || undefined;
        const maps_url = formData.get('maps_url')?.toString().trim() || undefined;
        const video_url_1 = formData.get('video_url_1')?.toString().trim() || undefined;
        const version = parseInt(formData.get('version')?.toString() || '1');
        const city_id = formData.get('city_id')?.toString().trim() || undefined;

        // Server-side boundary checks
        if (!name_ar || !name_en) {
            return fail(400, { error: 'Corporate name is required in both Arabic and English.' });
        }

        // Forward full profile payload downstream to Rust Axum API defensively via safeFetch
        const result = await safeFetch<any>(fetch, `${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/vendor/profile`, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${sessionToken}`,
                'Cookie': `zafaf_vendor_session=${sessionToken}`
            },
            body: JSON.stringify({
                name_ar,
                name_en,
                description_ar,
                description_en,
                address_ar,
                address_en,
                phone,
                email,
                capacity_min,
                capacity_max,
                latitude,
                longitude,
                area_sqm,
                amenities,
                crm_venue_id,
                star_rating,
                event_spaces_available,
                event_type,
                website,
                maps_url,
                video_url_1,
                has_partition,
                city_id,
                version
            })
        });

        if (!result.success) {
            return fail(result.status, { success: false, error: result.error?.message || 'Synchronization with registry failed.' });
        }

        return {
            success: true,
            message: result.data?.message || 'Corporate settings synchronized successfully!'
        };
    }
};
