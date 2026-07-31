import { env } from '$env/dynamic/public';
import { publicApiBase } from '$lib/utils/env.js';

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

    const API_BASE = publicApiBase();
    
    // Normalize leading slash
    const normalizedUrl = url.startsWith('/') ? url : `/${url}`;
    
    // If it's an uploaded file from the backend, resolve it against the backend API
    return `${API_BASE}${normalizedUrl}`;
}

/**
 * Resolves the optimized version of a given image URL if it is stored in the backend upload path.
 * Appends the requested size suffix (e.g., _thumb, _card, _medium, _large) to the image name.
 * Handles null/undefined inputs, external URLs, static assets, and non-webp files gracefully.
 */
export function getOptimizedImage(
    url: string | null | undefined,
    size: 'thumb' | 'card' | 'medium' | 'large'
): string {
    if (!url) return '';

    // Ignore external URLs
    if (
        url.startsWith('http://') ||
        url.startsWith('https://') ||
        url.startsWith('blob:') ||
        url.startsWith('data:')
    ) {
        return url;
    }

    // Ignore static client assets
    if (
        url.startsWith('/images/') ||
        url.startsWith('/favicon') ||
        url.startsWith('/icons/')
    ) {
        return url;
    }

    // Normalize lowercase check to assert only webp files are modified
    if (url.toLowerCase().endsWith('.webp')) {
        const dotIndex = url.lastIndexOf('.');
        if (dotIndex !== -1) {
            const base = url.substring(0, dotIndex);
            const ext = url.substring(dotIndex);
            return `${base}_${size}${ext}`;
        }
    }

    return url;
}

export interface ResponsiveSources {
    avif: {
        original: string;
        large: string;
        medium: string;
        card: string;
        thumb: string;
    };
    webp: {
        original: string;
        large: string;
        medium: string;
        card: string;
        thumb: string;
    };
    fallback: string;
}

/**
 * Returns the complete set of resolved AVIF and WebP responsive sizes
 * for rendering high-performance <picture> tags on the client.
 */
export function getResponsiveSources(url: string | null | undefined): ResponsiveSources | null {
    if (!url) return null;

    const resolvedUrl = resolveMediaUrl(url);

    // Only apply suffix generation if the file matches our standard upload pattern (ends in .webp)
    if (!resolvedUrl.toLowerCase().endsWith('.webp')) {
        const fallback = resolvedUrl;
        return {
            avif: { original: fallback, large: fallback, medium: fallback, card: fallback, thumb: fallback },
            webp: { original: fallback, large: fallback, medium: fallback, card: fallback, thumb: fallback },
            fallback
        };
    }

    const dotIndex = resolvedUrl.lastIndexOf('.');
    const base = resolvedUrl.substring(0, dotIndex);

    return {
        avif: {
            original: `${base}.avif`,
            large: `${base}_large.avif`,
            medium: `${base}_medium.avif`,
            card: `${base}_card.avif`,
            thumb: `${base}_thumb.avif`
        },
        webp: {
            original: `${base}.webp`,
            large: `${base}_large.webp`,
            medium: `${base}_medium.webp`,
            card: `${base}_card.webp`,
            thumb: `${base}_thumb.webp`
        },
        fallback: `${base}.webp`
    };
}
