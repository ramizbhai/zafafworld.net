export function parseBlocks(val: string) {
    try {
        const parsed = JSON.parse(val || "[]");
        if (Array.isArray(parsed)) return parsed;
        return [{ type: "text", content: val }];
    } catch {
        return val ? [{ type: "text", content: val }] : [];
    }
}

export function mergeBlocks(arBlocks: any[], enBlocks: any[]) {
    const merged: any[] = [];
    const len = Math.max(arBlocks.length, enBlocks.length);
    for (let i = 0; i < len; i++) {
        const ar = arBlocks[i] || {};
        const en = enBlocks[i] || {};
        const type = ar.type || en.type || "text";

        merged.push({
            type: type,
            contentAr: ar.content || "",
            contentEn: en.content || "",
            url: ar.url || en.url || "",
        });
    }
    return merged;
}

export function syncBlocksToJSON(blocks: any[]) {
    const arBlocks = blocks.map((b) => {
        if (["image", "map", "video", "gallery"].includes(b.type))
            return { type: b.type, url: b.url };
        if (b.type === "divider") return { type: b.type };
        if (b.type === "button")
            return { type: b.type, content: b.contentAr, url: b.url };
        return { type: b.type, content: b.contentAr };
    });

    const enBlocks = blocks.map((b) => {
        if (["image", "map", "video", "gallery"].includes(b.type))
            return { type: b.type, url: b.url };
        if (b.type === "divider") return { type: b.type };
        if (b.type === "button")
            return { type: b.type, content: b.contentEn, url: b.url };
        return { type: b.type, content: b.contentEn };
    });

    return {
        newAr: JSON.stringify(arBlocks),
        newEn: JSON.stringify(enBlocks)
    };
}

export function generateSeoSuggestions(
    blocks: any[],
    currentMetaTitleEn: string,
    currentMetaTitleAr: string,
    currentMetaDescEn: string,
    currentMetaDescAr: string,
    titleEn: string,
    titleAr: string
) {
    let newMetaTitleEn = currentMetaTitleEn;
    let newMetaTitleAr = currentMetaTitleAr;
    let newMetaDescEn = currentMetaDescEn;
    let newMetaDescAr = currentMetaDescAr;

    if (!newMetaTitleEn && titleEn) {
        newMetaTitleEn = `${titleEn} | ZafafWorld`;
    }
    if (!newMetaTitleAr && titleAr) {
        newMetaTitleAr = `${titleAr} | زفاف وورلد`;
    }

    const firstText = blocks.find((b) =>
        ["text", "heading", "subheading"].includes(b.type)
    );
    if (firstText) {
        if (!newMetaDescEn && firstText.contentEn) {
            newMetaDescEn = firstText.contentEn.slice(0, 155).trim();
            if (firstText.contentEn.length > 155) newMetaDescEn += "...";
        }
        if (!newMetaDescAr && firstText.contentAr) {
            newMetaDescAr = firstText.contentAr.slice(0, 155).trim();
            if (firstText.contentAr.length > 155) newMetaDescAr += "...";
        }
    }

    return {
        metaTitleEn: newMetaTitleEn,
        metaTitleAr: newMetaTitleAr,
        metaDescriptionEn: newMetaDescEn,
        metaDescriptionAr: newMetaDescAr,
    };
}

export function generateSlugPreview(titleEn: string, titleAr: string) {
    const text = titleEn || titleAr || "new-listing";
    return text
        .toLowerCase()
        .replace(/[^a-z0-9\s-]/g, "")
        .trim()
        .replace(/\s+/g, "-")
        .replace(/-+/g, "-");
}
