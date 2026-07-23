<script lang="ts">
    import { getI18n } from "$lib/i18n/i18n.svelte";
    import { Video } from "lucide-svelte";
    import type { WizardUploadState } from "../../features/vendor/wizard/wizardState.svelte";
    import { getApiUrl } from "$lib/utils/api";
    import { formatBytes } from "$lib/shared/utils/media";
    import UploadProgressOverlay from "./UploadProgressOverlay.svelte";

    let { state, maxAdditionalPhotos, maxVideos }: { state: WizardUploadState, maxAdditionalPhotos: number, maxVideos: number } = $props();
    const i18n = getI18n();

    function handleGalleryClick(e: MouseEvent) {
        if (!state.checkLimits(new File([], "dummy.jpg", { type: "image/jpeg" }), maxVideos, maxAdditionalPhotos)) {
            e.preventDefault();
        }
    }

    function handleGallerySelect(e: Event) {
        const input = e.target as HTMLInputElement;
        const files = input.files;
        if (files) {
            for (let i = 0; i < files.length; i++) {
                const file = files[i];
                if (state.checkLimits(file, maxVideos, maxAdditionalPhotos)) {
                    state.startGalleryUpload(file, i18n.locale);
                }
            }
        }
        input.value = "";
    }
</script>

<div class="gallery-side">
    <label for="gallery-input">
        Gallery Photos & Videos
        <div class="text-xs text-gray-500 font-normal mt-1">
            Photos: {state.currentPhotosCount}/{maxAdditionalPhotos === -1 ? '∞' : maxAdditionalPhotos} Used
            | Videos: {state.currentVideosCount}/{maxVideos === -1 ? '∞' : maxVideos} Used
        </div>
    </label>
    <div class="gallery-scroll-area custom-scrollbar">
        <div class="gallery-grid">
            {#each state.galleryItems as item, i}
                <div class="gallery-thumb group relative rounded-lg overflow-hidden border border-gray-200 shadow-sm bg-gray-50 h-24">
                    {#if item.status === "completed"}
                        {#if item.mediaType === "video"}
                            <video
                                src={item.previewUrl || (item.url.startsWith("http") ? item.url : getApiUrl(item.url))}
                                poster={item.thumbnailUrl ? (item.thumbnailUrl.startsWith("http") ? item.thumbnailUrl : getApiUrl(item.thumbnailUrl)) : ''}
                                muted
                                class="w-full h-full object-cover"
                            ></video>
                            <div class="absolute inset-0 bg-black/20 flex items-center justify-center">
                                <Video size={20} class="text-white" />
                            </div>
                        {:else}
                            <img
                                src={item.previewUrl || (item.url.startsWith("http") ? item.url : getApiUrl(item.url))}
                                alt="Gallery"
                                class="w-full h-full object-cover"
                            />
                        {/if}
                        <div class="absolute bottom-1 left-1 bg-black/60 text-white text-[0.6rem] px-1.5 py-0.5 rounded font-mono">
                            {formatBytes(item.fileSize)}
                        </div>
                        <button
                            type="button"
                            class="gallery-remove"
                            onclick={() => state.removeGalleryItem(i)}
                        >✕</button>
                    {:else}
                        <div class="absolute inset-0 flex flex-col justify-center items-center p-1.5 bg-black/70 text-center">
                            {#if item.status === "failed"}
                                <span class="text-red-400 text-[0.55rem] font-bold line-clamp-2 leading-tight mb-1" title={item.error}>
                                    {item.error || "Failed"}
                                </span>
                                <button
                                    class="bg-indigo-600 text-white text-[0.55rem] px-1.5 py-0.5 rounded hover:bg-indigo-700 font-medium"
                                    onclick={() => state.retryUpload(item, i18n.locale)}
                                >Retry</button>
                            {:else}
                                <UploadProgressOverlay {item} />
                            {/if}
                            <button
                                type="button"
                                class="gallery-remove"
                                onclick={() => state.removeGalleryItem(i)}
                            >✕</button>
                        </div>
                    {/if}
                </div>
            {/each}
        </div>
    </div>
    
    <div class="mt-4">
        <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
        <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
        <label 
            class="gallery-add-tile !w-full !h-24 flex flex-col justify-center items-center cursor-pointer bg-gray-50 border-2 border-dashed border-gray-300 rounded-lg hover:bg-indigo-50 hover:border-indigo-500 transition-colors" 
            for="gallery-input" 
            onclick={handleGalleryClick}
            onkeydown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                    handleGalleryClick(e as any);
                }
            }}
            tabindex="0"
        >
            <span class="text-3xl text-gray-400 mb-1">+</span>
            <span class="text-sm font-medium text-gray-600">Add Photos / Videos</span>
            <input
                id="gallery-input"
                type="file"
                accept="image/*,video/*"
                multiple
                class="hidden-input"
                onchange={handleGallerySelect}
                tabindex="-1"
            />
        </label>
    </div>
</div>

<style>
    .gallery-side {
        display: flex;
        flex-direction: column;
        border-inline-start: 1px solid var(--border-color, #e5e7eb);
        padding-inline-start: 2rem;
        height: 100%;
    }

    .gallery-scroll-area {
        flex-grow: 1;
        max-height: 380px;
        overflow-y: auto;
        padding-right: 0.5rem;
    }

    .custom-scrollbar::-webkit-scrollbar {
        width: 6px;
    }
    .custom-scrollbar::-webkit-scrollbar-track {
        background: #f1f1f1;
        border-radius: 4px;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb {
        background: #cbd5e1;
        border-radius: 4px;
    }
    .custom-scrollbar::-webkit-scrollbar-thumb:hover {
        background: #94a3b8;
    }
    
    .gallery-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
        gap: 1rem;
        padding-top: 1rem;
    }
    
    .gallery-remove {
        position: absolute;
        top: 0.25rem;
        right: 0.25rem;
        background: rgba(0,0,0,0.5);
        color: white;
        border: none;
        border-radius: 50%;
        width: 20px;
        height: 20px;
        font-size: 10px;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        opacity: 0;
        transition: opacity 0.2s;
    }
    .gallery-thumb:hover .gallery-remove {
        opacity: 1;
    }
    .hidden-input {
        display: none;
    }

    @media (max-width: 768px) {
        .gallery-side {
            border-inline-start: none;
            padding-inline-start: 0;
            border-top: 1px solid var(--border-color, #e5e7eb);
            padding-top: 1.5rem;
        }
        .gallery-scroll-area {
            max-height: 300px;
        }
    }
</style>
