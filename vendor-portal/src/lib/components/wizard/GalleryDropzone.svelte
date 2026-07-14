<script lang="ts">
    import { getI18n } from "$lib/i18n/i18n.svelte";
    import type { WizardUploadState } from "../../features/vendor/wizard/wizardState.svelte";
    import { getApiUrl } from "$lib/utils/api";
    import UploadProgressOverlay from "./UploadProgressOverlay.svelte";

    let { state }: { state: WizardUploadState } = $props();
    const i18n = getI18n();

    let canAddCover = $derived(state.checkLimits(new File([], "dummy.jpg", { type: "image/jpeg" }), -1, -1)); // Always allowed initially unless custom logic

    function handleCoverSelect(e: Event) {
        const input = e.target as HTMLInputElement;
        const file = input.files?.[0];
        if (file) {
            const ext = file.name.split(".").pop()?.toLowerCase();
            const isImage = file.type.startsWith("image/") || (ext && ["jpg", "jpeg", "png", "webp", "gif"].includes(ext));
            if (isImage) {
                state.startCoverUpload(file, i18n.locale);
            }
        }
        input.value = "";
    }

    function handleCoverDrop(e: DragEvent) {
        e.preventDefault();
        state.isDraggingCover = false;
        const file = e.dataTransfer?.files?.[0];
        if (file && file.type.startsWith("image/")) {
            state.startCoverUpload(file, i18n.locale);
        }
    }
</script>

<div class="form-group">
    <label for="cover-input">
        <div class="flex items-center gap-2">
            {i18n.t("listingsWizard.coverImage") || "Cover Image"}
            <span class="required">*</span>
        </div>
        <div class="text-xs text-gray-500 font-normal mt-1">
            {state.currentCoverCount}/1 Used
        </div>
    </label>
    {#if state.coverItem}
        <div class="relative w-full h-[450px] rounded-xl overflow-hidden shadow-sm group border border-gray-200 bg-gray-50">
            <img
                src={state.coverItem.status === "completed"
                    ? state.coverItem.url.startsWith("http")
                        ? state.coverItem.url
                        : getApiUrl(state.coverItem.url)
                    : state.coverItem.previewUrl}
                alt="Cover Preview"
                class="w-full h-full object-cover {state.coverItem.status !== 'completed' ? 'opacity-50 blur-[2px]' : ''}"
            />

            <div class="absolute inset-0 flex flex-col justify-center items-center p-4 bg-black/40">
                {#if state.coverItem.status === "completed"}
                    <button
                        type="button"
                        class="absolute top-2 right-2 bg-white hover:bg-red-500 hover:text-white text-red-500 p-1.5 rounded-full shadow-md transition-colors"
                        onclick={() => state.removeCover()}
                    >
                        ✕
                    </button>
                {:else if state.coverItem.status === "failed"}
                    <div class="text-red-400 text-xs font-bold mb-1">
                        Failed: {state.coverItem.error}
                    </div>
                    <div>
                        <button
                            class="bg-indigo-600 text-white text-xs px-3 py-1 rounded"
                            onclick={() => state.retryUpload(state.coverItem!, i18n.locale)}
                        >
                            Retry
                        </button>
                        <button
                            class="bg-gray-600 text-white text-xs px-3 py-1 rounded ml-2"
                            onclick={() => state.removeCover()}
                        >
                            Remove
                        </button>
                    </div>
                {:else}
                    <UploadProgressOverlay item={state.coverItem} />
                {/if}
            </div>
        </div>
    {:else}
        <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
        <label
            class="drop-zone"
            style="min-height: 450px;"
            class:dragging={state.isDraggingCover}
            for="cover-input"
            ondragover={(e) => {
                e.preventDefault();
                state.isDraggingCover = true;
            }}
            ondragleave={() => (state.isDraggingCover = false)}
            ondrop={handleCoverDrop}
        >
            <div class="drop-zone-content">
                <div class="drop-icon">📸</div>
                <p>Drag & drop or <strong>click to upload</strong></p>
            </div>
            <input
                id="cover-input"
                type="file"
                accept="image/*"
                class="hidden-input"
                onchange={handleCoverSelect}
            />
        </label>
    {/if}
</div>

<style>
    .drop-zone {
        display: flex;
        align-items: center;
        justify-content: center;
        border: 2px dashed #d1d5db;
        border-radius: 0.75rem;
        background-color: #f9fafb;
        cursor: pointer;
        transition: all 0.2s ease-in-out;
    }
    .drop-zone.dragging {
        border-color: #6366f1;
        background-color: #e0e7ff;
    }
    .drop-zone-content {
        text-align: center;
        color: #6b7280;
    }
    .drop-icon {
        font-size: 3rem;
        margin-bottom: 1rem;
    }
    .hidden-input {
        display: none;
    }
    .form-group label {
        display: block;
        margin-bottom: 0.5rem;
        font-weight: 600;
        color: #374151;
    }
    .required {
        color: #ef4444;
    }
</style>
