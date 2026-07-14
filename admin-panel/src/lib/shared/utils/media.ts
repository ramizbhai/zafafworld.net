import { env } from '$env/dynamic/public';

/**
 * Resolves a potentially relative media URL (like /uploads/...) against the backend API URL.
 * Leaves absolute URLs or static client assets (like /images/fallbacks/...) unchanged.
 */
export function resolveMediaUrl(url: string | null | undefined): string {
    if (!url) return '';
    
    if (url.startsWith('http://') || url.startsWith('https://') || url.startsWith('blob:') || url.startsWith('data:')) {
        return url;
    }

    // Client-side static assets
    if (url.startsWith('/images/') || url.startsWith('/favicon') || url.startsWith('/icons/')) {
        return url;
    }

    const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';
    
    // Normalize leading slash
    const normalizedUrl = url.startsWith('/') ? url : `/${url}`;
    
    // If it's an uploaded file from the backend, resolve it against the backend API
    return `${API_BASE}${normalizedUrl}`;
}
