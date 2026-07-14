export function isEmbeddableUrl(url: string) {
    if (!url) return false;
    return url.includes("google.com/maps/embed") || url.includes("google.com/maps/d/embed") || url.includes("openstreetmap.org/export/embed");
}

export function getEmbedUrl(url: string) {
    if (!url) return "";
    if (url.includes("<iframe")) {
        const match = url.match(/src=["']([^"']+)["']/);
        if (match && match[1]) {
            return match[1];
        }
    }
    return url;
}

export function getGalleryUrls(url: string) {
    if (!url) return [];
    return url.split(/[\n,]/).map(u => u.trim()).filter(Boolean);
}

export function getYouTubeId(url: string): string | null {
    if (!url) return null;
    const regExp = /^.*(youtu.be\/|v\/|u\/\w\/|embed\/|watch\?v=|\&v=|shorts\/)([^#\&\?]*).*/;
    const match = url.match(regExp);
    return (match && match[2].length === 11) ? match[2] : null;
}

export function getVimeoId(url: string): string | null {
    if (!url) return null;
    const regExp = /^.*(vimeo\.com\/|video\/)(clip\/)?([0-9]+).*/;
    const match = url.match(regExp);
    return match ? match[3] : null;
}

export function getVideoEmbedUrl(url: string): string | null {
    if (!url) return null;
    
    if (url.includes("<iframe")) {
        const match = url.match(/src=["']([^"']+)["']/);
        if (match && match[1]) {
            url = match[1];
        }
    }

    const ytId = getYouTubeId(url);
    if (ytId) {
        return `https://www.youtube.com/embed/${ytId}`;
    }

    const vimeoId = getVimeoId(url);
    if (vimeoId) {
        return `https://player.vimeo.com/video/${vimeoId}`;
    }

    return null;
}
