import { env } from '$env/dynamic/public';
import type { LayoutServerLoad } from './$types';
import { getCached } from '$lib/services/api/cache';
import { getCategoryIcon } from '$lib/constants/categoryIcons.js';
import { apiClient } from '$lib/services/api/client.js';

export const load: LayoutServerLoad = async ({ fetch: svelteFetch, cookies }) => {
    const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';
    const sessionToken = cookies.get('zafaf_client_session');
    let user = null;

    // Dynamic fallback arrays
    let categories: any[] = [];
    let cities: any[] = [];
    let countries: any[] = [];
    let amenities: any[] = [];
    let venueTypes: any[] = [];

    const country = cookies.get('zafaf_selected_country') || 'sa';

    try {
        const [meData, categoriesRes, citiesRes, countriesRes, amenitiesRes, venueTypesRes] = await Promise.all([
            sessionToken ? apiClient.get<any>('/api/v1/auth/me', {
                fetch: svelteFetch,
                token: sessionToken,
                isServer: true
            }).catch((err) => {
                console.error('[Root Layout Loader] Handshake error:', err);
                return null;
            }) : Promise.resolve(null),
            getCached(`layout:categories:${country.toLowerCase()}`, () =>
                apiClient.get<any>('/api/v1/public/categories', {
                    fetch: svelteFetch,
                    isServer: true,
                    headers: { 'X-Country-ID': country }
                }).catch((err) => {
                    console.error('[Layout Loader] Failed to fetch categories:', err);
                    return null;
                }),
                10 * 1000
            ),
            getCached('layout:cities', () =>
                apiClient.get<any>('/api/v1/public/cities', { fetch: svelteFetch, isServer: true })
                    .catch((err) => {
                        console.error('[Layout Loader] Failed to fetch cities:', err);
                        return null;
                    })
            ),
            getCached('layout:countries', () =>
                apiClient.get<any>('/api/v1/public/countries', { fetch: svelteFetch, isServer: true }).catch(() => null)
            ),
            getCached('layout:amenities', () =>
                apiClient.get<any>('/api/v1/public/amenities', { fetch: svelteFetch, isServer: true }).catch(() => null)
            ),
            getCached('layout:venue-types', () =>
                apiClient.get<any>('/api/v1/public/venue-types', { fetch: svelteFetch, isServer: true }).catch(() => null)
            )
        ]);

        if (meData?.status === 'success') {
            user = {
                ...meData.user,
                session: {
                    access_token: sessionToken
                }
            };
        }

        // Transform partitioned categories map (venues, services) into a flat array
        if (categoriesRes) {
            let flattened: any[] = [];
            if (Array.isArray(categoriesRes.allCategories)) {
                flattened = categoriesRes.allCategories;
            } else if (categoriesRes.categories && typeof categoriesRes.categories === 'object') {
                // Flatten all category groups dynamically (venues, fashion, beauty, etc.)
                for (const groupKey of Object.keys(categoriesRes.categories)) {
                    const list = categoriesRes.categories[groupKey];
                    if (Array.isArray(list)) {
                        flattened.push(...list);
                    }
                }
            }

            categories = flattened.map((c: any) => ({
                key: c.slug,
                icon: getCategoryIcon(c.slug),
                labelAr: c.ar,
                labelEn: c.en,
                listingsCount: c.listingsCount ?? 0
            }));
        }


        // Handle cities mapping — normalize fields for consistent downstream access
        if (citiesRes && Array.isArray(citiesRes.cities)) {
            cities = citiesRes.cities.map((c: any) => ({
                id: c.id,
                slug: c.slug,
                name_ar: c.name_ar || c.ar || '',
                name_en: c.name_en || c.en || '',
                country_id: c.country_id || ''
            }));
        } else if (citiesRes && Array.isArray(citiesRes.data)) {
            cities = citiesRes.data.map((c: any) => ({
                id: c.id,
                slug: c.slug,
                name_ar: c.name_ar || c.ar || '',
                name_en: c.name_en || c.en || '',
                country_id: c.country_id || ''
            }));
        }

        // Handle countries mapping
        if (countriesRes && Array.isArray(countriesRes.countries)) {
            countries = countriesRes.countries;
        } else if (countriesRes && Array.isArray(countriesRes.data)) {
            countries = countriesRes.data;
        }

        // Handle amenities mapping
        if (amenitiesRes && Array.isArray(amenitiesRes.amenities)) {
            amenities = amenitiesRes.amenities;
        } else if (amenitiesRes && Array.isArray(amenitiesRes.data)) {
            amenities = amenitiesRes.data;
        }

        // Handle venue types mapping
        if (venueTypesRes && Array.isArray(venueTypesRes.venueTypes)) {
            venueTypes = venueTypesRes.venueTypes;
        } else if (venueTypesRes && Array.isArray(venueTypesRes.data)) {
            venueTypes = venueTypesRes.data;
        }

        return {
            user,
            metadata: {
                categories,
                cities,
                countries,
                amenities,
                venueTypes
            }
        };
    } catch (err: unknown) {
        console.error('[Layout Loader] Recoverable error in root layout loader, returning structural fallbacks:', err);
        return {
            user: null,
            metadata: {
                categories: [],
                cities: [],
                countries: [],
                amenities: [],
                venueTypes: []
            }
        };
    }
};
