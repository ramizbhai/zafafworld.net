<script lang="ts">
    import {
        ArrowUp,
        ArrowDown,
        Copy,
        Trash2,
        X,
        Link as LinkIcon,
        MapPin,
    } from "lucide-svelte";
    import { resolveMediaUrl } from "$lib/shared/utils/media";
    import { getVideoEmbedUrl } from "$lib/utils/mediaParser";
    import { getBlockDef } from "$lib/constants/blockTypes";

    let {
        block = $bindable(),
        index,
        isFirst,
        isLast,
        currentLang = "en",
        onMoveUp,
        onMoveDown,
        onDuplicate,
        onRemove,
        onSync,
    } = $props<{
        block: any;
        index: number;
        isFirst: boolean;
        isLast: boolean;
        currentLang?: "ar" | "en";
        onMoveUp: (index: number) => void;
        onMoveDown: (index: number) => void;
        onDuplicate: (index: number) => void;
        onRemove: (index: number) => void;
        onSync: () => void;
    }>();

    const def = $derived(getBlockDef(block.type));
    const DefIcon = $derived(def.icon);
    
    function clearMedia() {
        block.url = "";
        onSync();
    }
</script>

<div class="block-card-wrapper animate-slide-in">
    <!-- Block head banner -->
    <div class="block-card-head">
        <span class="block-index-label">
            <DefIcon size={16} />
            {currentLang === "ar" ? def.labelAr : def.labelEn}
        </span>
        <div class="block-actions">
            <button
                type="button"
                class="action-btn"
                onclick={() => onMoveUp(index)}
                disabled={isFirst}
                title={currentLang === "ar" ? "أعلى" : "Move Up"}
            >
                <ArrowUp size={14} />
            </button>
            <button
                type="button"
                class="action-btn"
                onclick={() => onMoveDown(index)}
                disabled={isLast}
                title={currentLang === "ar" ? "أسفل" : "Move Down"}
            >
                <ArrowDown size={14} />
            </button>
            <button
                type="button"
                class="action-btn"
                onclick={() => onDuplicate(index)}
                title={currentLang === "ar" ? "نسخ" : "Duplicate"}
            >
                <Copy size={14} />
            </button>
            <button
                type="button"
                class="action-btn delete-btn"
                onclick={() => onRemove(index)}
                title={currentLang === "ar" ? "حذف" : "Delete"}
            >
                <Trash2 size={14} />
            </button>
        </div>
    </div>

    <!-- Input areas -->
    <div
        class="block-card-inputs"
        class:grid-side-by-side={def.group !== "structure"}
    >
        {#if def.group === "text" || def.group === "interactive"}
            <!-- English inputs -->
            <div class="lang-input-pane ltr-pane">
                <div class="pane-header">
                    <span class="lang-flag-indicator">🇬🇧</span>
                    <span class="pane-title">English</span>
                </div>

                {#if block.type === "button"}
                    <input
                        type="text"
                        class="pane-text-input mb-2"
                        bind:value={block.contentEn}
                        oninput={onSync}
                        placeholder="Button Label (e.g. Visit Website)"
                    />
                    <div class="pane-url-input">
                        <LinkIcon size={14} class="pane-icon" />
                        <input
                            type="url"
                            bind:value={block.url}
                            oninput={onSync}
                            placeholder="Button URL (https://...)"
                        />
                    </div>
                {:else}
                    <textarea
                        class="pane-textarea"
                        class:is-heading={block.type === "heading"}
                        class:is-subheading={block.type === "subheading"}
                        bind:value={block.contentEn}
                        oninput={onSync}
                        placeholder="Write {def.labelEn} in English..."
                        rows={block.type === "heading" || block.type === "subheading" ? 2 : 4}
                    ></textarea>
                {/if}
            </div>

            <!-- Arabic inputs -->
            <div class="lang-input-pane rtl-pane">
                <div class="pane-header">
                    <span class="lang-flag-indicator">🇸🇦</span>
                    <span class="pane-title">العربية</span>
                </div>

                {#if block.type === "button"}
                    <input
                        type="text"
                        class="pane-text-input mb-2"
                        bind:value={block.contentAr}
                        oninput={onSync}
                        placeholder="نص الزر (مثال: زيارة الموقع)"
                    />
                    <div class="pane-url-input">
                        <LinkIcon size={14} class="pane-icon" />
                        <input
                            type="url"
                            bind:value={block.url}
                            oninput={onSync}
                            placeholder="رابط الزر (https://...)"
                        />
                    </div>
                {:else}
                    <textarea
                        class="pane-textarea"
                        class:is-heading={block.type === "heading"}
                        class:is-subheading={block.type === "subheading"}
                        bind:value={block.contentAr}
                        oninput={onSync}
                        placeholder="اكتب {def.labelAr} باللغة العربية..."
                        rows={block.type === "heading" || block.type === "subheading" ? 2 : 4}
                    ></textarea>
                {/if}
            </div>
        {:else if def.group === "media"}
            <!-- Media Left: Input Options -->
            <div class="lang-input-pane media-input-pane">
                <div class="pane-header">
                    <span class="pane-title">
                        {currentLang === "ar" ? "إضافة محتوى" : "Add Content"}
                    </span>
                </div>
                <div class="pane-url-input">
                    <DefIcon size={14} class="pane-icon" />
                    <input
                        type="url"
                        bind:value={block.url}
                        oninput={onSync}
                        placeholder={currentLang === "ar" ? "أدخل الرابط (URL)..." : "Enter URL..."}
                    />
                </div>
                <p class="media-help-text">
                    {currentLang === "ar"
                        ? "أدخل رابط الصورة، الفيديو، أو الخريطة وسيظهر العرض الجانبي تلقائياً."
                        : "Enter the URL for the image, video, or map and the preview will appear automatically."}
                </p>
            </div>
            <!-- Media Right: Preview -->
            <div class="lang-input-pane media-preview-pane">
                {#if block.url}
                    <div class="media-preview-container">
                        <button
                            class="clear-media-btn"
                            onclick={clearMedia}
                            title={currentLang === "ar" ? "إزالة المحتوى" : "Clear Content"}
                        >
                            <X size={14} />
                        </button>
                        {#if block.type === "image" || block.type === "gallery"}
                            <img
                                src={resolveMediaUrl(block.url)}
                                alt="Preview"
                                class="preview-img"
                            />
                        {:else if block.type === "video"}
                            {@const embedUrl = getVideoEmbedUrl(block.url)}
                            {#if embedUrl}
                                <iframe
                                    src={embedUrl}
                                    title="Preview Video"
                                    class="w-full h-full border-0 aspect-video"
                                    allowfullscreen
                                ></iframe>
                            {:else}
                                <!-- svelte-ignore a11y_media_has_caption -->
                                <video
                                    src={resolveMediaUrl(block.url)}
                                    controls
                                    playsinline
                                    preload="metadata"
                                    class="w-full h-full max-h-[200px] object-contain rounded-lg"
                                ></video>
                            {/if}
                        {:else if block.type === "map"}
                            <div class="preview-map-placeholder">
                                <MapPin size={32} />
                                <span>Map Preview</span>
                            </div>
                        {/if}
                    </div>
                {:else}
                    <div class="empty-preview">
                        <DefIcon size={24} />
                        <span>
                            {currentLang === "ar" ? "لا يوجد محتوى للعرض" : "No content to preview"}
                        </span>
                    </div>
                {/if}
            </div>
        {:else if block.type === "divider"}
            <div class="divider-pane">
                <hr class="preview-divider" />
                <span class="divider-label">
                    {currentLang === "ar" ? "خط فاصل سيظهر هنا" : "Divider line will appear here"}
                </span>
            </div>
        {/if}
    </div>
</div>
