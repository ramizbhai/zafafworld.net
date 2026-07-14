import { z } from 'zod';
import { baseInfoSchema, type Block } from '$lib/shared/builder/builder.schema';
import { triggerUpgrade } from "$lib/stores/upgradeStore";

export class BuilderState {
    // Current Wizard Step
    currentStep = $state(1);
    totalSteps = 4;

    // Base Info
    titleAr = $state("");
    titleEn = $state("");
    metaTitleAr = $state("");
    metaTitleEn = $state("");
    metaDescriptionAr = $state("");
    metaDescriptionEn = $state("");

    // Blocks
    featureBlocks = $state<Block[]>([]);
    mediaBlocks = $state<Block[]>([]);
    
    // UI State
    showAddMenuAtIndex = $state<{type: 'feature'|'media', index: number} | null>(null);

    // Initial bound callbacks for syncing to parent
    onSync?: (data: any) => void;

    constructor(initialData?: any) {
        if (initialData) {
            this.titleAr = initialData.titleAr || "";
            this.titleEn = initialData.titleEn || "";
            this.metaTitleAr = initialData.metaTitleAr || "";
            this.metaTitleEn = initialData.metaTitleEn || "";
            this.metaDescriptionAr = initialData.metaDescriptionAr || "";
            this.metaDescriptionEn = initialData.metaDescriptionEn || "";
            
            this.initBlocks(initialData.descriptionAr || "", initialData.descriptionEn || "");
        }
    }

    initBlocks(descriptionAr: string, descriptionEn: string) {
        try {
            const arParsed = this.parseBlocks(descriptionAr);
            const enParsed = this.parseBlocks(descriptionEn);
            const merged = this.mergeBlocks(arParsed, enParsed);
            
            this.featureBlocks = merged.filter(b => ['heading', 'subheading', 'text', 'list', 'button', 'contact', 'social', 'divider'].includes(b.type));
            this.mediaBlocks = merged.filter(b => ['image', 'gallery', 'video', 'map', 'image_text'].includes(b.type));
        } catch {
            this.featureBlocks = [];
            this.mediaBlocks = [];
        }
    }

    parseBlocks(val: string) {
        try {
            const parsed = JSON.parse(val || "[]");
            if (Array.isArray(parsed)) return parsed;
            return [{ type: "text", content: val }];
        } catch {
            return val ? [{ type: "text", content: val }] : [];
        }
    }

    mergeBlocks(arBlocks: any[], enBlocks: any[]) {
        const merged: Block[] = [];
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
                layout: ar.layout || en.layout || "left",
            });
        }
        return merged;
    }

    sync() {
        const allBlocks = [...this.featureBlocks, ...this.mediaBlocks];
        const arBlocks = allBlocks.map((b) => {
            if (b.type === "image_text") return { type: b.type, content: b.contentAr, url: b.url, layout: b.layout || 'left' };
            if (["image", "map", "video", "gallery"].includes(b.type)) return { type: b.type, url: b.url };
            if (b.type === "divider") return { type: b.type };
            if (b.type === "button") return { type: b.type, content: b.contentAr, url: b.url };
            return { type: b.type, content: b.contentAr };
        });

        const enBlocks = allBlocks.map((b) => {
            if (b.type === "image_text") return { type: b.type, content: b.contentEn, url: b.url, layout: b.layout || 'left' };
            if (["image", "map", "video", "gallery"].includes(b.type)) return { type: b.type, url: b.url };
            if (b.type === "divider") return { type: b.type };
            if (b.type === "button") return { type: b.type, content: b.contentEn, url: b.url };
            return { type: b.type, content: b.contentEn };
        });

        this.autoGenerateSEO();

        if (this.onSync) {
            this.onSync({
                descriptionAr: JSON.stringify(arBlocks),
                descriptionEn: JSON.stringify(enBlocks),
                titleAr: this.titleAr,
                titleEn: this.titleEn,
                metaTitleAr: this.metaTitleAr,
                metaTitleEn: this.metaTitleEn,
                metaDescriptionAr: this.metaDescriptionAr,
                metaDescriptionEn: this.metaDescriptionEn,
            });
        }
    }

    autoGenerateSEO() {
        if (!this.metaTitleEn && this.titleEn) this.metaTitleEn = `${this.titleEn} | ZafafWorld`;
        if (!this.metaTitleAr && this.titleAr) this.metaTitleAr = `${this.titleAr} | زفاف وورلد`;

        const firstText = this.featureBlocks.find((b) => ["text", "heading", "subheading"].includes(b.type));
        if (firstText) {
            if (!this.metaDescriptionEn && firstText.contentEn) {
                this.metaDescriptionEn = firstText.contentEn.slice(0, 155).trim() + (firstText.contentEn.length > 155 ? "..." : "");
            }
            if (!this.metaDescriptionAr && firstText.contentAr) {
                this.metaDescriptionAr = firstText.contentAr.slice(0, 155).trim() + (firstText.contentAr.length > 155 ? "..." : "");
            }
        }
    }

    // Wizard Navigation
    nextStep() {
        if (this.currentStep < this.totalSteps) this.currentStep++;
    }

    prevStep() {
        if (this.currentStep > 1) this.currentStep--;
    }

    goToStep(step: number) {
        if (step >= 1 && step <= this.totalSteps) this.currentStep = step;
    }

    // Block Operations
    addBlock(type: string, isMedia: boolean, index: number, maxBlocks: number, isDiamond: boolean, locale: string) {
        const targetArr = isMedia ? this.mediaBlocks : this.featureBlocks;
        if (maxBlocks !== -1 && (this.featureBlocks.length + this.mediaBlocks.length) >= maxBlocks && !isDiamond) {
            const msgEn = `You have reached your plan limit of ${maxBlocks} description blocks. Upgrade required to add more.`;
            const msgAr = `لقد وصلت إلى الحد الأقصى لباقتك وهو ${maxBlocks} أقسام وصف. الترقية مطلوبة لإضافة المزيد.`;
            triggerUpgrade("description_blocks", "current", locale === "ar" ? msgAr : msgEn);
            return;
        }

        const newBlock: Block = { type, contentAr: "", contentEn: "", url: "", layout: "left" };
        targetArr.splice(index, 0, newBlock);
        this.showAddMenuAtIndex = null;
        this.sync();
    }

    duplicateBlock(isMedia: boolean, index: number, maxBlocks: number, isDiamond: boolean, locale: string) {
        const targetArr = isMedia ? this.mediaBlocks : this.featureBlocks;
        if (maxBlocks !== -1 && (this.featureBlocks.length + this.mediaBlocks.length) >= maxBlocks && !isDiamond) {
            const msgEn = `You have reached your plan limit of ${maxBlocks} description blocks. Upgrade required to duplicate blocks.`;
            const msgAr = `لقد وصلت إلى الحد الأقصى لباقتك وهو ${maxBlocks} أقسام وصف. الترقية مطلوبة لنسخ المزيد.`;
            triggerUpgrade("description_blocks", "current", locale === "ar" ? msgAr : msgEn);
            return;
        }
        const blockToCopy = JSON.parse(JSON.stringify(targetArr[index]));
        targetArr.splice(index + 1, 0, blockToCopy);
        this.sync();
    }

    removeBlock(isMedia: boolean, index: number) {
        if (isMedia) {
            this.mediaBlocks = this.mediaBlocks.filter((_, i) => i !== index);
        } else {
            this.featureBlocks = this.featureBlocks.filter((_, i) => i !== index);
        }
        this.sync();
    }

    clearMedia(index: number) {
        if (this.mediaBlocks[index]) {
            this.mediaBlocks[index].url = "";
        }
        this.sync();
    }

    moveUp(isMedia: boolean, index: number) {
        if (index === 0) return;
        const targetArr = isMedia ? this.mediaBlocks : this.featureBlocks;
        const temp = targetArr[index];
        targetArr[index] = targetArr[index - 1];
        targetArr[index - 1] = temp;
        this.sync();
    }

    moveDown(isMedia: boolean, index: number) {
        const targetArr = isMedia ? this.mediaBlocks : this.featureBlocks;
        if (index === targetArr.length - 1) return;
        const temp = targetArr[index];
        targetArr[index] = targetArr[index + 1];
        targetArr[index + 1] = temp;
        this.sync();
    }

    validateBaseInfo() {
        const result = baseInfoSchema.safeParse({
            titleAr: this.titleAr,
            titleEn: this.titleEn,
            metaTitleAr: this.metaTitleAr,
            metaTitleEn: this.metaTitleEn,
            metaDescriptionAr: this.metaDescriptionAr,
            metaDescriptionEn: this.metaDescriptionEn
        });
        return result;
    }
}
