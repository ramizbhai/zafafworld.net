<script lang="ts">
    import { getLocale } from '$lib/paraglide/runtime.js';
    import { resolveMediaUrl } from "$lib/shared/utils/media.js";
    import { Sparkles, MapPin, Phone, Share2, Compass, ChevronDown, ChevronUp, Link as LinkIcon } from 'lucide-svelte';

    let { descriptionAr = "", descriptionEn = "" } = $props<{
        descriptionAr?: string;
        descriptionEn?: string;
    }>();

    const isAr = $derived(getLocale() === 'ar');
    let isExpanded = $state(false);

    // Block Parsing helpers
    function parseBlocks(val: string) {
        try {
            const parsed = JSON.parse(val || "[]");
            if (Array.isArray(parsed)) return parsed;
            return [{ type: "text", content: val }];
        } catch {
            return val ? [{ type: "text", content: val }] : [];
        }
    }

    function mergeBlocks(arBlocks: any[], enBlocks: any[]) {
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
                layout: ar.layout || en.layout || "left",
            });
        }
        return merged;
    }

    // Process blocks reactively
    const blocks = $derived.by(() => {
        const arParsed = parseBlocks(descriptionAr);
        const enParsed = parseBlocks(descriptionEn);
        return mergeBlocks(arParsed, enParsed);
    });

    // Content Anchors (Headings)
    const headings = $derived(
        blocks
            .filter(b => b.type === "heading" && (isAr ? b.contentAr : b.contentEn))
            .map((b, idx) => ({
                id: `heading-anchor-${idx}`,
                text: isAr ? b.contentAr : b.contentEn
            }))
    );

    // Highlights (subheadings or list items that seem important)
    const highlights = $derived(
        blocks
            .filter(b => b.type === "subheading" && (isAr ? b.contentAr : b.contentEn))
            .slice(0, 4)
            .map(b => (isAr ? b.contentAr : b.contentEn))
    );

    // Scroll helper for anchors
    function scrollToAnchor(id: string) {
        const element = document.getElementById(id);
        if (element) {
            element.scrollIntoView({ behavior: "smooth", block: "start" });
        }
    }

    // Embed and Video helpers
    function getGalleryUrls(url: string) {
        if (!url) return [];
        return url.split(/[\n,]/).map(u => u.trim()).filter(Boolean);
    }

    function getEmbedUrl(url: string) {
        if (!url) return "";
        if (url.includes("<iframe")) {
            const match = url.match(/src=["']([^"']+)["']/);
            if (match && match[1]) return match[1];
        }
        return url;
    }

    function isEmbeddableUrl(url: string) {
        if (!url) return false;
        return url.includes("google.com/maps/embed") || url.includes("google.com/maps/d/embed") || url.includes("openstreetmap.org/export/embed");
    }

    function getVideoEmbedUrl(url: string): string | null {
        if (!url) return null;
        if (url.includes("<iframe")) {
            const match = url.match(/src=["']([^"']+)["']/);
            if (match && match[1]) url = match[1];
        }
        const ytMatch = url.match(/^.*(youtu.be\/|v\/|u\/\w\/|embed\/|watch\?v=|\&v=|shorts\/)([^#\&\?]*).*/);
        if (ytMatch && ytMatch[2].length === 11) return `https://www.youtube.com/embed/${ytMatch[2]}`;
        const vimeoMatch = url.match(/^.*(vimeo\.com\/|video\/)(clip\/)?([0-9]+).*/);
        if (vimeoMatch) return `https://player.vimeo.com/video/${vimeoMatch[3]}`;
        return null;
    }
</script>

{#if blocks.length > 0}
    <!-- Structured V2 Description Section -->
    <section class="bg-white rounded-2xl border border-slate-100 p-6 sm:p-8 shadow-sm space-y-6">
        
        <!-- Header title -->
        <div class="flex items-center gap-3 border-b border-slate-100 pb-4">
            <div class="p-2.5 bg-amber-50 text-amber-600 rounded-xl">
                <Sparkles size={22} />
            </div>
            <h3 class="font-display text-xl font-bold text-slate-900">
                {isAr ? "تفاصيل إضافية والوصف" : "Detailed Description & Details"}
            </h3>
        </div>

        <!-- Highlights Row -->
        {#if highlights.length > 0}
            <div class="flex flex-wrap gap-2 pt-1">
                {#each highlights as hl}
                    <span class="inline-flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold rounded-xl text-amber-800 bg-amber-50 border border-amber-100">
                        <Sparkles size={12} class="text-amber-600" />
                        <span>{hl}</span>
                    </span>
                {/each}
            </div>
        {/if}

        <!-- Dynamic Content Anchors navigation bar -->
        {#if headings.length > 1}
            <div class="bg-slate-50 p-4 rounded-xl border border-slate-100/80 flex flex-wrap items-center gap-x-4 gap-y-2 text-xs sm:text-sm font-semibold text-slate-500">
                <span class="text-slate-400 font-bold">{isAr ? "فهرس المحتوى:" : "Index:"}</span>
                {#each headings as heading}
                    <button 
                        onclick={() => scrollToAnchor(heading.id)}
                        class="text-amber-600 hover:text-amber-700 hover:underline cursor-pointer flex items-center gap-0.5"
                    >
                        <LinkIcon size={12} />
                        <span>{heading.text}</span>
                    </button>
                {/each}
            </div>
        {/if}

        <!-- Collapsible Content Wrapper -->
        <div class="relative overflow-hidden transition-all duration-500 {isExpanded ? 'max-h-[5000px]' : 'max-h-[320px]'}" aria-expanded={isExpanded}>
            
            <div class="prose max-w-none text-slate-800 space-y-6 text-start">
                {#each blocks as block, idx}
                    {@const content = isAr ? (block.contentAr || block.contentEn) : (block.contentEn || block.contentAr)}
                    
                    {#if block.type === "heading" && content}
                        <h4 id={`heading-anchor-${blocks.filter((b, i) => i < idx && b.type === "heading").length}`} class="text-xl sm:text-2xl font-extrabold text-slate-900 tracking-tight mt-8 mb-4 border-b border-slate-50 pb-2">
                            {content}
                        </h4>
                    {:else if block.type === "subheading" && content}
                        <h5 class="text-lg font-bold text-slate-800 tracking-tight mt-6 mb-3">
                            {content}
                        </h5>
                    {:else if block.type === "text" && content}
                        <p class="text-sm sm:text-base text-slate-600 leading-relaxed whitespace-pre-wrap">
                            {content}
                        </p>
                    {:else if block.type === "list" && content}
                        <ul class="list-disc pl-5 rtl:pl-0 rtl:pr-5 space-y-2 my-4">
                            {#each content.split("\n") as item}
                                {#if item.trim()}
                                    <li class="text-sm sm:text-base text-slate-600">{item}</li>
                                {/if}
                            {/each}
                        </ul>
                    {:else if block.type === "image" && block.url}
                        <img src={resolveMediaUrl(block.url)} alt="" loading="lazy" class="rounded-2xl max-w-full h-auto my-6 shadow-xs border border-slate-100" />
                    {:else if block.type === "gallery" && block.url}
                        {@const urls = getGalleryUrls(block.url)}
                        {#if urls.length > 1}
                            <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-4 my-6">
                                {#each urls as imageUrl}
                                    <div class="relative aspect-video rounded-2xl overflow-hidden shadow-xs border border-slate-100">
                                        <img src={resolveMediaUrl(imageUrl)} alt="" loading="lazy" class="w-full h-full object-cover" />
                                    </div>
                                {/each}
                            </div>
                        {:else if urls.length === 1}
                            <img src={resolveMediaUrl(urls[0])} alt="" loading="lazy" class="rounded-2xl max-w-full h-auto my-6 shadow-xs border border-slate-100" />
                        {/if}
                    {:else if block.type === "video" && block.url}
                        {@const embedUrl = getVideoEmbedUrl(block.url)}
                        {#if embedUrl}
                            <div class="my-6 rounded-2xl overflow-hidden border border-slate-100 shadow-xs aspect-video w-full bg-slate-100">
                                <iframe src={embedUrl} title="Video" class="w-full h-full border-0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" allowfullscreen loading="lazy"></iframe>
                            </div>
                        {:else}
                            <!-- svelte-ignore a11y_media_has_caption -->
                            <video src={resolveMediaUrl(block.url)} controls playsinline preload="metadata" class="w-full rounded-2xl my-6 shadow-xs border border-slate-100"></video>
                        {/if}
                    {:else if block.type === "map" && block.url}
                        {@const embedUrl = getEmbedUrl(block.url)}
                        {#if isEmbeddableUrl(embedUrl)}
                            <div class="my-6 rounded-2xl overflow-hidden border border-slate-100 shadow-xs aspect-video w-full bg-slate-100">
                                <iframe src={embedUrl} title="Map" class="w-full h-full border-0" allowfullscreen loading="lazy"></iframe>
                            </div>
                        {:else}
                            <div class="my-6 p-6 bg-slate-50 border border-slate-200/60 rounded-2xl flex flex-col sm:flex-row items-center justify-between gap-4 shadow-xs">
                                <div class="flex items-center gap-4 text-start w-full sm:w-auto">
                                    <div class="p-3 bg-red-50 text-red-500 rounded-xl shrink-0"><MapPin size={24} /></div>
                                    <div class="flex-1">
                                        <h4 class="font-semibold text-slate-800">{isAr ? 'موقع الإعلان على الخريطة' : 'Listing Location on Map'}</h4>
                                    </div>
                                </div>
                                <a href={block.url} target="_blank" rel="noopener noreferrer" class="w-full sm:w-auto text-center bg-slate-900 text-white px-5 py-2.5 rounded-lg font-medium hover:bg-slate-800 transition-colors text-sm shadow-xs">{isAr ? 'عرض على الخريطة' : 'View on Map'}</a>
                            </div>
                        {/if}
                    {:else if block.type === "contact" && content}
                        <div class="my-6 p-6 bg-emerald-50/40 border border-emerald-100/60 rounded-2xl flex items-start gap-4 shadow-xs text-start">
                            <div class="p-3 bg-emerald-100/80 text-emerald-600 rounded-xl mt-1 shrink-0"><Phone size={20} /></div>
                            <div class="flex-1">
                                <h4 class="font-semibold text-slate-800 mb-1">{isAr ? 'تفاصيل الاتصال' : 'Contact Details'}</h4>
                                <p class="text-slate-600 whitespace-pre-wrap leading-relaxed text-sm">{content}</p>
                            </div>
                        </div>
                    {:else if block.type === "social" && content}
                        <div class="my-6 p-6 bg-blue-50/40 border border-blue-100/60 rounded-2xl flex items-start gap-4 shadow-xs text-start">
                            <div class="p-3 bg-blue-100/80 text-blue-600 rounded-xl mt-1 shrink-0"><Share2 size={20} /></div>
                            <div class="flex-1">
                                <h4 class="font-semibold text-slate-800 mb-1">{isAr ? 'قنوات التواصل الاجتماعي' : 'Social Media Channels'}</h4>
                                <p class="text-slate-600 whitespace-pre-wrap leading-relaxed text-sm">{content}</p>
                            </div>
                        </div>
                    {:else if block.type === "divider"}
                        <hr class="my-8 border-slate-100" />
                    {:else if block.type === "image_text"}
                        <div class="flex flex-col sm:flex-row gap-6 my-8 items-start bg-slate-50/50 p-6 rounded-3xl border border-slate-100">
                            {#if block.layout === 'right'}
                                <div class="flex-1 text-slate-700 leading-relaxed text-sm whitespace-pre-wrap order-2 sm:order-1">{content}</div>
                                {#if block.url}
                                    <div class="w-full sm:w-1/2 rounded-2xl overflow-hidden shadow-xs border border-slate-100 order-1 sm:order-2 shrink-0">
                                        <img src={block.url} alt="" class="w-full h-full object-cover" />
                                    </div>
                                {/if}
                            {:else}
                                {#if block.url}
                                    <div class="w-full sm:w-1/2 rounded-2xl overflow-hidden shadow-xs border border-slate-100 shrink-0">
                                        <img src={block.url} alt="" class="w-full h-full object-cover" />
                                    </div>
                                {/if}
                                <div class="flex-1 text-slate-700 leading-relaxed text-sm whitespace-pre-wrap">{content}</div>
                            {/if}
                        </div>
                    {:else if block.type === "button" && block.url && content}
                        <a href={block.url} target="_blank" rel="noopener noreferrer" class="inline-block bg-slate-900 text-white px-6 py-3 rounded-lg font-medium hover:bg-slate-800 transition-colors my-4">{content}</a>
                    {/if}
                {/each}
            </div>

            <!-- Fade backdrop mask when description is collapsed -->
            {#if !isExpanded}
                <div class="absolute bottom-0 left-0 right-0 h-28 bg-gradient-to-t from-white via-white/80 to-transparent pointer-events-none"></div>
            {/if}
        </div>

        <!-- Read More Expand / Collapse Action Trigger -->
        <div class="flex justify-center border-t border-slate-50 pt-4 mt-2">
            <button 
                onclick={() => isExpanded = !isExpanded}
                class="px-5 py-2 rounded-xl bg-slate-50 border border-slate-100 hover:bg-slate-100 text-slate-600 hover:text-slate-900 font-bold text-xs cursor-pointer flex items-center gap-1.5 transition-all focus:outline-hidden focus:ring-2 focus:ring-amber-500"
            >
                {#if isExpanded}
                    <span>{isAr ? "عرض أقل" : "Show Less"}</span>
                    <ChevronUp size={14} />
                {:else}
                    <span>{isAr ? "عرض المزيد" : "Read More"}</span>
                    <ChevronDown size={14} />
                {/if}
            </button>
        </div>

    </section>
{/if}
