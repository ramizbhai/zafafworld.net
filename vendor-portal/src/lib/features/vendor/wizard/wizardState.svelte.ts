import { listingStore, type GalleryItem } from "$lib/stores/listingStore";
import { vendorStore } from "$lib/stores/vendorStore";
import { triggerUpgrade } from "$lib/stores/upgradeStore";
import { getApiUrl } from "$lib/utils/api";
import { resolveMediaType } from "$lib/shared/utils/media";

export class WizardUploadState {
    private sessionToken: string = "";
    
    coverItem = $state<GalleryItem | null>(null);
    galleryItems = $state<GalleryItem[]>([]);
    
    isDraggingCover = $state(false);
    
    uploadQueue = $state<Array<{ file: File, isCover: boolean, itemId: string }>>([]);
    activeUploadsCount = $state(0);
    MAX_CONCURRENT_UPLOADS = 3;

    constructor() {
    }

    setSessionToken(token: string) {
        this.sessionToken = token;
    }

    init(cover: GalleryItem | null, gallery: GalleryItem[]) {
        this.coverItem = cover;
        this.galleryItems = gallery;
    }

    get isValid() {
        return this.coverItem !== null && this.coverItem.status === "completed";
    }

    get isUploadingAny() {
        return (
            this.coverItem?.status === "uploading" ||
            this.coverItem?.status === "queued" ||
            this.coverItem?.status === "processing" ||
            this.galleryItems.some(
                (item) => item.status === "uploading" || item.status === "queued" || item.status === "processing"
            )
        );
    }

    get currentPhotosCount() {
        return this.galleryItems.filter(i => i.mediaType !== 'video' && i.status !== "failed").length;
    }

    get currentVideosCount() {
        return this.galleryItems.filter(i => i.mediaType === 'video' && i.status !== "failed").length;
    }

    get currentCoverCount() {
        return this.coverItem && this.coverItem.status !== "failed" ? 1 : 0;
    }

    checkLimits(file: File, maxVideos: number, maxAdditionalPhotos: number): boolean {
        const isVideo = resolveMediaType(file) === "video";
        if (isVideo) {
            if (maxVideos !== -1 && this.currentVideosCount >= maxVideos) {
                triggerUpgrade("videos");
                return false;
            }
        } else {
            if (maxAdditionalPhotos !== -1 && this.currentPhotosCount >= maxAdditionalPhotos) {
                triggerUpgrade("photos");
                return false;
            }
        }
        return true;
    }

    updateItemState(id: string, updates: Partial<GalleryItem>) {
        if (this.coverItem && this.coverItem.id === id) {
            this.coverItem = { ...this.coverItem, ...updates };
        } else {
            const index = this.galleryItems.findIndex((x) => x.id === id);
            if (index !== -1) {
                this.galleryItems[index] = { ...this.galleryItems[index], ...updates };
            }
        }
    }

    enqueueUpload(file: File, isCover: boolean, itemId: string, i18nLocale: string) {
        const mediaType = resolveMediaType(file);
        const maxLimit = mediaType === "video" ? 200 * 1024 * 1024 : 100 * 1024 * 1024;
        const maxLimitMB = mediaType === "video" ? 200 : 100;
        
        if (file.size > maxLimit) {
            this.updateItemState(itemId, {
                status: "failed",
                error: i18nLocale === "ar"
                    ? `حجم الملف يتجاوز الحد الأقصى (${maxLimitMB} ميجابايت)`
                    : `File exceeds ${maxLimitMB}MB size limit.`
            });
            listingStore.setError(
                i18nLocale === "ar"
                    ? `حجم الملف يتجاوز الحد الأقصى المسموح به وهو ${maxLimitMB} ميجابايت.`
                    : `File size exceeds the maximum allowed limit of ${maxLimitMB}MB.`
            );
            return;
        }

        this.updateItemState(itemId, { status: "queued" });
        this.uploadQueue = [...this.uploadQueue, { file, isCover, itemId }];
        this.processQueue();
    }

    processQueue() {
        if (this.activeUploadsCount >= this.MAX_CONCURRENT_UPLOADS || this.uploadQueue.length === 0) {
            return;
        }

        const next = this.uploadQueue[0];
        this.uploadQueue = this.uploadQueue.slice(1);
        this.activeUploadsCount++;

        this.uploadFile(next.file, next.isCover, next.itemId)
            .then(() => {
                this.activeUploadsCount--;
                this.processQueue();
            })
            .catch((err) => {
                this.activeUploadsCount--;
                this.processQueue();
                console.error("Queue upload failed:", err);
            });
    }

    uploadFile(file: File, isCover: boolean, itemId: string): Promise<void> {
        return new Promise((resolve, reject) => {
            const xhr = new XMLHttpRequest();

            this.updateItemState(itemId, {
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
                    const percent = Math.round((event.loaded / event.total) * 100);
                    const now = Date.now();
                    const duration = (now - lastTime) / 1000;
                    let speed = "0 KB/s";
                    if (duration >= 0.5) {
                        const bytesSent = event.loaded - lastBytes;
                        const speedBytes = bytesSent / duration;
                        if (speedBytes > 1024 * 1024) {
                            speed = (speedBytes / (1024 * 1024)).toFixed(1) + " MB/s";
                        } else {
                            speed = (speedBytes / 1024).toFixed(0) + " KB/s";
                        }
                        lastTime = now;
                        lastBytes = event.loaded;
                    }
                    this.updateItemState(itemId, { progress: percent, speed });
                }
            };

            xhr.onload = async () => {
                if (xhr.status >= 200 && xhr.status < 300) {
                    try {
                        const res = JSON.parse(xhr.responseText);
                        if (res.status === "success") {
                            this.updateItemState(itemId, {
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
                            this.updateItemState(itemId, { status: "failed", error: errMsg });
                            reject(new Error(errMsg));
                        }
                    } catch (e: any) {
                        this.updateItemState(itemId, { status: "failed", error: "Invalid server response" });
                        reject(e);
                    }
                } else if (xhr.status === 402) {
                    try {
                        const res = JSON.parse(xhr.responseText);
                        const limitType = res.errors?.limit_type || (isCover ? "cover_photos" : "photos");
                        triggerUpgrade(limitType);
                    } catch (e) {}
                    const errMsg = "Limit Reached. Please upgrade.";
                    this.updateItemState(itemId, { status: "failed", error: errMsg });
                    reject(new Error(errMsg));
                } else {
                    let errMsg = `HTTP ${xhr.status}`;
                    try {
                        const res = JSON.parse(xhr.responseText);
                        if (res.message || res.error) {
                            errMsg = res.message || res.error;
                        }
                    } catch (e) {}
                    this.updateItemState(itemId, { status: "failed", error: errMsg });
                    reject(new Error(errMsg));
                }
            };

            xhr.onerror = () => {
                const errMsg = "Network connection error";
                this.updateItemState(itemId, { status: "failed", error: errMsg });
                reject(new Error(errMsg));
            };

            xhr.open("POST", getApiUrl("/api/v1/vendor/upload"));
            xhr.setRequestHeader("Authorization", `Bearer ${this.sessionToken}`);

            const fd = new FormData();
            // Since this runs client-side and we use listingStore.productId, we must subscribe or read it directly:
            let pid = null;
            listingStore.subscribe(state => { pid = state.productId })();
            if (pid) {
                fd.append("product_id", pid);
            }
            
            fd.append("is_cover", isCover ? "true" : "false");
            fd.append("media_type", resolveMediaType(file));
            fd.append("file", file);
            xhr.send(fd);
        });
    }

    startCoverUpload(file: File, locale: string) {
        this.coverItem = {
            id: Math.random().toString(36).substr(2, 9),
            url: "",
            previewUrl: URL.createObjectURL(file),
            isCover: true,
            sortOrder: 0,
            mediaType: "image",
            fileSize: file.size,
            file: file,
            status: "queued",
            progress: 0,
            speed: "0 KB/s",
        };
        this.enqueueUpload(file, true, this.coverItem.id, locale);
    }

    startGalleryUpload(file: File, locale: string) {
        const mediaType = resolveMediaType(file);
        const item: GalleryItem = {
            id: Math.random().toString(36).substr(2, 9),
            url: "",
            previewUrl: URL.createObjectURL(file),
            isCover: false,
            sortOrder: this.galleryItems.length + 1,
            mediaType: mediaType,
            fileSize: file.size,
            file: file,
            status: "queued",
            progress: 0,
            speed: "0 KB/s",
        };
        this.galleryItems = [...this.galleryItems, item];
        this.enqueueUpload(file, false, item.id, locale);
    }

    removeCover() {
        if (this.coverItem?.xhr) this.coverItem.xhr.abort();
        this.coverItem = null;
    }

    removeGalleryItem(index: number) {
        const item = this.galleryItems[index];
        if (item.xhr) item.xhr.abort();
        this.galleryItems = this.galleryItems.filter((_, i) => i !== index);
    }

    retryUpload(item: GalleryItem, locale: string) {
        if (item.file) {
            this.updateItemState(item.id, { status: "queued", progress: 0, speed: "0 KB/s" });
            this.enqueueUpload(item.file, item.isCover, item.id, locale);
        }
    }
}
