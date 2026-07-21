import type { ParamMatcher } from '@sveltejs/kit';

/**
 * Strict allowlist of known city slugs pulled from /api/v1/public/cities.
 *
 * ⚠️  This list must be updated any time a new city is added to the database.
 * Using a strict allowlist (not a loose regex) is what prevents collisions
 * with existing top-level routes like /about, /auth, /dashboard, /discover.
 *
 * city_slug → country_id mapping is intentionally embedded here so the
 * server-side load function can resolve country without an extra API call.
 */
export const CITY_SLUGS = new Set([
    'riyadh',
    'jeddah',
    'khobar',
    'dammam',
    'abha',
    'hafuf',
    'wajh',
    'rass',
    'arar',
    'bisha',
    'buraidah',
    'hail',
    'jizan',
    'jubail',
    'khamis-mushait',
    'makkah',
    'medina',
    'najran',
    'qatif',
    'sakaka',
    'tabuk',
    'taif',
    'turaif',
    'umluj',
    'yanbu',
]);

/**
 * Map of city slug → country_id for server-side country inference.
 * All cities are currently Saudi Arabia; add new entries when multi-country
 * support is expanded.
 */
export const CITY_COUNTRY_MAP: Record<string, string> = {
    riyadh: 'sa',
    jeddah: 'sa',
    khobar: 'sa',
    dammam: 'sa',
    abha: 'sa',
    hafuf: 'sa',
    wajh: 'sa',
    rass: 'sa',
    arar: 'sa',
    bisha: 'sa',
    buraidah: 'sa',
    hail: 'sa',
    jizan: 'sa',
    jubail: 'sa',
    'khamis-mushait': 'sa',
    makkah: 'sa',
    medina: 'sa',
    najran: 'sa',
    qatif: 'sa',
    sakaka: 'sa',
    tabuk: 'sa',
    taif: 'sa',
    turaif: 'sa',
    umluj: 'sa',
    yanbu: 'sa',
};

export const match: ParamMatcher = (param) => {
    return CITY_SLUGS.has(param.toLowerCase());
};
