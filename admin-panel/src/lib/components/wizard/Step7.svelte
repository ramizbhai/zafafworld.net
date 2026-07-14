<script lang="ts">
    import { getI18n } from "$lib/i18n/i18n.svelte";
    import { listingStore, type GalleryItem } from "$lib/stores/listingStore";
    import { vendorStore } from "$lib/stores/vendorStore";
    import { triggerUpgrade } from "$lib/stores/upgradeStore";
    import { Camera, Video } from "lucide-svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";
    import { env } from "$env/dynamic/public";

    const i18n = getI18n();
    let { data } = $props<{ data: { sessionToken: string } }>();
    const apiBase =
        env.PUBLIC_API_URL && env.PUBLIC_API_URL.includes("localhost")
            ? ""
            : env.PUBLIC_API_URL || "";

    let coverItem = $state<GalleryItem | null>(
        $listingStore.formData.coverItem,
    );
    let galleryItems = $state<GalleryItem[]>(
        $listingStore.formData.galleryItems || [],
    );
    let isDraggingCover = $state(false);

    let isValid = $derived(coverItem !== null && coverItem.status === "completed");
    $effect(() => {
        listingStore.setCanContinue(isValid);
    });

    const isUploadingAny = $derived(
        coverItem?.status === "uploading" ||
            coverItem?.status === "processing" ||
            galleryItems.some(
                (item) =>
                    item.status === "uploading" || item.status === "processing",
            ),
    );

    let maxCoverPhotos = $derived($vendorStore.policy_limits.cover_photos);
    let maxAdditionalPhotos = $derived($vendorStore.policy_limits.photos);
    let maxVideos = $derived($vendorStore.policy_limits.videos);

    let currentCoverCount = $derived(coverItem && coverItem.status !== "failed" ? 1 : 0);
    let currentPhotosCount = $derived(galleryItems.filter(i => i.mediaType !== 'video' && i.status !== "failed").length);
    let currentVideosCount = $derived(galleryItems.filter(i => i.mediaType === 'video' && i.status !== "failed").length);

    let canAddCover = true;
    let canAddAnyGallery = $derived(
        (maxAdditionalPhotos === -1 || currentPhotosCount < maxAdditionalPhotos) || 
        (maxVideos === -1 || currentVideosCount < maxVideos)
    );

    function updateItemState(id: string, updates: Partial<GalleryItem>) {
        if (coverItem && coverItem.id === id) {
            coverItem = { ...coverItem, ...updates };
        } else {
            const index = galleryItems.findIndex((x) => x.id === id);
            if (index !== -1) {
                galleryItems[index] = { ...galleryItems[index], ...updates };
            }
        }
    }

    function uploadFile(
        file: File,
        isCover: boolean,
        itemId: string,
    ): Promise<void> {
        return new Promise((resolve, reject) => {
            const xhr = new XMLHttpRequest();

            updateItemState(itemId, {
                xhr,
                status: "uploading",
                progress: 0,
                speed: "0 KB/s",
            });

            let startTime = Date.now();
            let lastTime = startTime;
            let lastBytes = 0;

            xhr.upload.onprogress = (event) => {
                if (event.lengthComputable) {
                    const percent = Math.round(
                        (event.loaded / event.total) * 100,
                    );

                    const now = Date.now();
                    const duration = (now - lastTime) / 1000;
                    let speed = "0 KB/s";
                    if (duration >= 0.5) {
                        const bytesSent = event.loaded - lastBytes;
                        const speedBytes = bytesSent / duration;
                        if (speedBytes > 1024 * 1024) {
                            speed =
                                (speedBytes / (1024 * 1024)).toFixed(1) +
                                " MB/s";
                        } else {
                            speed = (speedBytes / 1024).toFixed(0) + " KB/s";
                        }
                        lastTime = now;
                        lastBytes = event.loaded;
                    }
                    updateItemState(itemId, { progress: percent, speed });
                }
            };

            xhr.onload = async () => {
                if (xhr.status >= 200 && xhr.status < 300) {
                    try {
                        const res = JSON.parse(xhr.responseText);
                        if (res.status === "success") {
                            updateItemState(itemId, {
                                status: "completed",
                                url: res.url,
                                file_path: res.file_path,
                                mediaType: res.media_type || "image",
                                thumbnailUrl: res.thumbnail_url,
                                fileSize: res.file_size || file.size,
                                durationSeconds: res.duration_seconds,
                            });
                            resolve();
                        } else {
                            const errMsg = res.message || "Upload failed";
                            updateItemState(itemId, {
                                status: "failed",
                                error: errMsg,
                            });
                            reject(new Error(errMsg));
                        }
                    } catch (e: any) {
                        updateItemState(itemId, {
                            status: "failed",
                            error: "Invalid server response",
                        });
                        reject(e);
                    }
                } else if (xhr.status === 402) {
                    try {
                        const res = JSON.parse(xhr.responseText);
                        const limitType = res.meta?.limit_type || (isCover ? "cover_photos" : "photos");
                        triggerUpgrade(limitType);
                    } catch (e) {}
                    const errMsg = "Limit Reached. Please upgrade.";
                    updateItemState(itemId, {
                        status: "failed",
                        error: errMsg,
                    });
                    reject(new Error(errMsg));
                } else {
                    const errMsg = `HTTP ${xhr.status}`;
                    updateItemState(itemId, {
                        status: "failed",
                        error: errMsg,
                    });
                    reject(new Error(errMsg));
                }
            };

            xhr.onerror = () => {
                const errMsg = "Network connection error";
                updateItemState(itemId, { status: "failed", error: errMsg });
                reject(new Error(errMsg));
            };

            xhr.open("POST", `${apiBase}/api/v1/vendor/upload`);
            xhr.setRequestHeader(
                "Authorization",
                `Bearer ${data.sessionToken}`,
            );

            const fd = new FormData();
            fd.append("file", file);
            xhr.send(fd);
        });
    }

    function startCoverUpload(file: File) {
        coverItem = {
            id: Math.random().toString(36).substr(2, 9),
            url: "",
            previewUrl: URL.createObjectURL(file),
            isCover: true,
            sortOrder: 0,
            mediaType: "image",
            fileSize: file.size,
            file: file,
            status: "uploading",
            progress: 0,
            speed: "0 KB/s",
        };
        uploadFile(file, true, coverItem.id).catch(console.error);
    }

    function handleCoverDrop(e: DragEvent) {
        e.preventDefault();
        isDraggingCover = false;
        if (!canAddCover) {
            triggerUpgrade("cover_photos");
            return;
        }
        const file = e.dataTransfer?.files?.[0];
        if (file && file.type.startsWith("image/")) {
            startCoverUpload(file);
        }
    }

    function handleCoverClick(e: MouseEvent) {
        if (!canAddCover) {
            e.preventDefault();
            triggerUpgrade("cover_photos");
        }
    }

    function handleCoverSelect(e: Event) {
        const input = e.target as HTMLInputElement;
        if (!canAddCover) {
            input.value = "";
            return;
        }
        const file = input.files?.[0];
        if (file && file.type.startsWith("image/")) {
            startCoverUpload(file);
        }
    }

    function removeCover() {
        if (coverItem?.xhr) coverItem.xhr.abort();
        coverItem = null;
    }

    function startGalleryUpload(file: File) {
        const isVideo =
            file.type.startsWith("video/") ||
            file.name.endsWith(".mp4") ||
            file.name.endsWith(".webm") ||
            file.name.endsWith(".mov");
        const item: GalleryItem = {
            id: Math.random().toString(36).substr(2, 9),
            url: "",
            previewUrl: URL.createObjectURL(file),
            isCover: false,
            sortOrder: galleryItems.length + 1,
            mediaType: isVideo ? "video" : "image",
            fileSize: file.size,
            file: file,
            status: "uploading",
            progress: 0,
            speed: "0 KB/s",
        };
        galleryItems = [...galleryItems, item];
        uploadFile(file, false, item.id).catch(console.error);
    }

    function handleGalleryClick(e: MouseEvent) {
        if (!canAddAnyGallery) {
            e.preventDefault();
            triggerUpgrade("photos");
        }
    }

    function handleGallerySelect(e: Event) {
        const input = e.target as HTMLInputElement;
        const files = input.files;
        if (files) {
            let photoCount = currentPhotosCount;
            let videoCount = currentVideosCount;

            for (let i = 0; i < files.length; i++) {
                const file = files[i];
                const isVideo = file.type.startsWith("video/") || file.name.endsWith(".mp4") || file.name.endsWith(".mov") || file.name.endsWith(".webm");
                
                if (isVideo) {
                    if (maxVideos !== -1 && videoCount >= maxVideos) {
                        triggerUpgrade("videos");
                        continue;
                    }
                    videoCount++;
                } else if (file.type.startsWith("image/")) {
                    if (maxAdditionalPhotos !== -1 && photoCount >= maxAdditionalPhotos) {
                        triggerUpgrade("photos");
                        continue;
                    }
                    photoCount++;
                } else {
                    continue; // unsupported type
                }
                
                startGalleryUpload(file);
            }
        }
        input.value = ""; // Reset file input so same files can be selected again if failed
    }

    function removeGalleryItem(index: number) {
        const item = galleryItems[index];
        if (item.xhr) item.xhr.abort();
        galleryItems = galleryItems.filter((_, i) => i !== index);
    }

    function retryUpload(item: GalleryItem) {
        if (item.file) {
            uploadFile(item.file, item.isCover, item.id).catch(console.error);
        }
    }

    $effect(() => {
        listingStore.updateFormData({ coverItem, galleryItems });

        listingStore.setSubmitHandler(async () => {
            if (isUploadingAny) {
                listingStore.setError(
                    "Wait for all uploads to finish before continuing.",
                );
                return;
            }

            if (!coverItem || coverItem.status !== "completed") {
                listingStore.setError(
                    "Cover image is required and must be fully uploaded.",
                );
                return;
            }

            listingStore.setSubmitting(true);
            listingStore.setError("");

            try {
                // Build atomic gallery items payload
                const galleryPayload = [];
                if (coverItem && coverItem.status === "completed") {
                    galleryPayload.push({
                        imageUrl: coverItem.url,
                        filePath: coverItem.file_path || null,
                        isCover: true,
                        sortOrder: 0,
                        caption: coverItem.caption || null,
                        mediaType: coverItem.mediaType,
                        thumbnailUrl: coverItem.thumbnailUrl || null,
                        fileSize: coverItem.fileSize,
                        durationSeconds: coverItem.durationSeconds || null,
                    });
                }
                galleryItems
                    .filter((item) => item.status === "completed")
                    .forEach((item, idx) => {
                        galleryPayload.push({
                            imageUrl: item.url,
                            filePath: item.file_path || null,
                            isCover: false,
                            sortOrder: item.sortOrder || idx + 1,
                            caption: item.caption || null,
                            mediaType: item.mediaType,
                            thumbnailUrl: item.thumbnailUrl || null,
                            fileSize: item.fileSize,
                            durationSeconds: item.durationSeconds || null,
                        });
                    });

                const url = `${apiBase}/api/v1/vendor/products/${$listingStore.productId}`;
                const payload = {
                    version: $listingStore.version,
                    galleryItems: galleryPayload,
                };

                const res = await fetch(url, {
                    method: "PUT",
                    headers: {
                        "Content-Type": "application/json",
                        Authorization: `Bearer ${data.sessionToken}`,
                    },
                    body: JSON.stringify(payload),
                });

                if (!res.ok) {
                    const err = await res.json().catch(() => ({}));
                    throw new Error(err.error || `Server error ${res.status}`);
                }

                const responseData = await res.json();
                if (responseData.product?.version) {
                    listingStore.setVersion(responseData.product.version);
                }

                listingStore.setHighestStep(7);
                goto(`${$page.url.pathname.split("/step-")[0]}/step-8`);
            } catch (err: any) {
                listingStore.setError(
                    err.message || "Failed to save gallery items.",
                );
            } finally {
                listingStore.setSubmitting(false);
            }
        });
    });
</script>

<div class="step-pane fade-in">
    <div class="step-heading">
        <Camera class="step-icon" size={28} />
        <div>
            <h1>{i18n.t("listingsWizard.galleryMedia") || "Gallery Media"}</h1>
            <p>
                {i18n.t("listingsWizard.galleryMediaDesc") ||
                    "Upload high-quality photos and videos."}
            </p>
        </div>
    </div>

    {#if $listingStore.submitError}
        <div class="error-banner">{$listingStore.submitError}</div>
    {/if}

    <div class="form-card gallery-section split-layout">
        <!-- Cover Image (Left Side - 60%) -->
        <div class="cover-side">
        <!-- Cover Image -->
        <div class="form-group">
            <label for="cover-input">
                <div class="flex items-center gap-2">
                    {i18n.t("listingsWizard.coverImage") || "Cover Image"}
                    <span class="required">*</span>
                </div>
                <div class="text-xs text-gray-500 font-normal mt-1">
                    {currentCoverCount}/{maxCoverPhotos === -1 ? '∞' : maxCoverPhotos} Used
                </div>
            </label>
            {#if coverItem}
                <div
                    class="relative w-full h-[450px] rounded-xl overflow-hidden shadow-sm group border border-gray-200 bg-gray-50"
                >
                    <img
                        src={coverItem.status === "completed"
                            ? coverItem.url.startsWith("http")
                                ? coverItem.url
                                : `${apiBase}${coverItem.url}`
                            : coverItem.previewUrl}
                        alt="Cover Preview"
                        class="w-full h-full object-cover {coverItem.status !==
                        'completed'
                            ? 'opacity-50 blur-[2px]'
                            : ''}"
                    />

                    <div
                        class="absolute inset-0 flex flex-col justify-center items-center p-4 bg-black/40"
                    >
                        {#if coverItem.status === "uploading"}
                            <div class="text-white text-xs font-semibold mb-1">
                                {i18n.locale === "ar"
                                    ? "جاري الرفع..."
                                    : "Uploading..."}
                            </div>
                            <div
                                class="w-2/3 bg-white/30 h-2 rounded-full overflow-hidden mb-1"
                            >
                                <div
                                    class="bg-indigo-500 h-full transition-all duration-150"
                                    style="width: {coverItem.progress || 0}%"
                                ></div>
                            </div>
                            <div class="text-white text-[0.7rem] flex gap-2">
                                <span>{coverItem.progress || 0}%</span>
                                <span>•</span>
                                <span>{coverItem.speed || "0 KB/s"}</span>
                            </div>
                        {:else if coverItem.status === "processing"}
                            <div
                                class="text-white text-xs font-semibold mb-1 animate-pulse"
                            >
                                Processing...
                            </div>
                        {:else if coverItem.status === "completed"}
                            <button
                                type="button"
                                class="absolute top-2 right-2 bg-white hover:bg-red-500 hover:text-white text-red-500 p-1.5 rounded-full shadow-md transition-colors"
                                onclick={removeCover}
                            >
                                ✕
                            </button>
                        {:else if coverItem.status === "failed"}
                            <div class="text-red-400 text-xs font-bold mb-1">
                                Failed
                            </div>
                            <button
                                class="bg-indigo-600 text-white text-xs px-3 py-1 rounded"
                                onclick={() => retryUpload(coverItem!)}
                                >Retry</button
                            >
                            <button
                                class="bg-gray-600 text-white text-xs px-3 py-1 rounded ml-2"
                                onclick={removeCover}>Remove</button
                            >
                        {/if}
                    </div>
                </div>
            {:else}
                <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
                <label
                    class="drop-zone"
                    style="min-height: 450px;"
                    class:dragging={isDraggingCover}
                    for="cover-input"
                    ondragover={(e) => {
                        e.preventDefault();
                        isDraggingCover = true;
                    }}
                    ondragleave={() => (isDraggingCover = false)}
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
        </div>

        <!-- Gallery Images (Right Side - 40%) -->
        <div class="gallery-side">
            <label for="gallery-input">
                Gallery Photos & Videos
                <div class="text-xs text-gray-500 font-normal mt-1">
                    Photos: {currentPhotosCount}/{maxAdditionalPhotos === -1 ? '∞' : maxAdditionalPhotos} Used
                    | Videos: {currentVideosCount}/{maxVideos === -1 ? '∞' : maxVideos} Used
                </div>
            </label>
            <div class="gallery-scroll-area custom-scrollbar">
                <div class="gallery-grid">
                    {#each galleryItems as item, i}
                    <div class="gallery-thumb">
                        {#if item.status === "completed"}
                            {#if item.mediaType === "video"}
                                <video
                                    src={item.previewUrl ||
                                        (item.url.startsWith("http")
                                            ? item.url
                                            : `${apiBase}${item.url}`)}
                                    muted
                                    class="w-full h-full object-cover"
                                ></video>
                                <div
                                    class="absolute inset-0 bg-black/20 flex items-center justify-center"
                                >
                                    <Video size={20} class="text-white" />
                                </div>
                            {:else}
                                <img
                                    src={item.previewUrl ||
                                        (item.url.startsWith("http")
                                            ? item.url
                                            : `${apiBase}${item.url}`)}
                                    alt="Gallery"
                                    class="w-full h-full object-cover"
                                />
                            {/if}
                            <button
                                type="button"
                                class="gallery-remove"
                                onclick={() => removeGalleryItem(i)}>✕</button
                            >
                        {:else}
                            <div
                                class="absolute inset-0 flex flex-col justify-center items-center p-2 bg-black/50"
                            >
                                {#if item.status === "uploading"}
                                    <span class="text-white text-[0.6rem]"
                                        >Uploading... {item.progress}%</span
                                    >
                                {:else if item.status === "failed"}
                                    <button
                                        class="bg-indigo-600 text-white text-[0.5rem] px-1 py-0.5 rounded"
                                        onclick={() => retryUpload(item)}
                                        >Retry</button
                                    >
                                {/if}
                                <button
                                    type="button"
                                    class="gallery-remove"
                                    onclick={() => removeGalleryItem(i)}
                                    >✕</button
                                >
                            </div>
                        {/if}
                    </div>
                {/each}
                </div>
            </div>
            
            <!-- Fixed Add Button at the bottom -->
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
    </div>
</div>

<style>
    .split-layout {
        display: grid;
        grid-template-columns: 60% 1fr;
        gap: 2rem;
        align-items: start;
    }

    .cover-side {
        display: flex;
        flex-direction: column;
    }

    .gallery-side {
        display: flex;
        flex-direction: column;
        border-left: 1px solid var(--border-color, #e5e7eb);
        padding-left: 2rem;
        height: 100%;
    }

    .gallery-scroll-area {
        flex-grow: 1;
        max-height: 380px;
        overflow-y: auto;
        padding-right: 0.5rem;
    }

    /* Custom Scrollbar for Gallery */
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

    @media (max-width: 768px) {
        .split-layout {
            grid-template-columns: 1fr;
            gap: 1.5rem;
        }
        .gallery-side {
            border-left: none;
            padding-left: 0;
            border-top: 1px solid var(--border-color, #e5e7eb);
            padding-top: 1.5rem;
        }
        .gallery-scroll-area {
            max-height: 300px;
        }
    }
</style>
