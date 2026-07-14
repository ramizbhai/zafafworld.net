import { env } from '$env/dynamic/public';

export type MediaVariant = 'large' | 'original' | 'medium' | 'card' | 'thumb';

/**
 * Resolves a potentially relative media URL (like /uploads/...) against the backend API URL.
 * Leaves absolute URLs or static client assets (like /images/fallbacks/...) unchanged.
 * Supports image variants if specified.
 */
export function resolveMediaUrl(
    url: string | null | undefined, 
    variant?: MediaVariant
): string {
    if (!url) return '';
    
    let resolvedUrl = url;

    // Apply variant if specified and it's an uploaded asset path
    if (variant && (url.includes('/uploads/') || url.includes('/assets/uploads/'))) {
        // Strip query parameters if any
        const [cleanPath, queryString] = url.split('?');
        const extIndex = cleanPath.lastIndexOf('.');
        if (extIndex !== -1) {
            const basePath = cleanPath.substring(0, extIndex);
            const ext = cleanPath.substring(extIndex + 1).toLowerCase();
            
            // Only apply variant format to image extensions
            if (['jpg', 'jpeg', 'png', 'webp', 'heic', 'gif'].includes(ext)) {
                resolvedUrl = `${basePath}_${variant}.webp${queryString ? '?' + queryString : ''}`;
            }
        }
    }

    if (resolvedUrl.startsWith('http://') || resolvedUrl.startsWith('https://') || resolvedUrl.startsWith('blob:') || resolvedUrl.startsWith('data:')) {
        return resolvedUrl;
    }

    // Client-side static assets
    if (resolvedUrl.startsWith('/images/') || resolvedUrl.startsWith('/favicon') || resolvedUrl.startsWith('/icons/')) {
        return resolvedUrl;
    }

    const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';
    
    // Normalize leading slash
    const normalizedUrl = resolvedUrl.startsWith('/') ? resolvedUrl : `/${resolvedUrl}`;
    
    // If it's an uploaded file from the backend, resolve it against the backend API
    return `${API_BASE}${normalizedUrl}`;
}

export function resolveMediaType(file: File): "image" | "video" {
    const ext = file.name.split(".").pop()?.toLowerCase();
    if (file.type.startsWith("video/") || (ext && ["mp4", "mov", "webm"].includes(ext))) {
        return "video";
    }
    return "image";
}

export function formatBytes(bytes: number | undefined, decimals = 1): string {
    if (!bytes) return "0 B";
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + " " + sizes[i];
}
