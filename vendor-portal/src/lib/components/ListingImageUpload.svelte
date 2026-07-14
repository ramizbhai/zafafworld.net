<script lang="ts">
  import { resolveMediaUrl } from "$lib/shared/utils/media";
  import { apiClient } from "$lib/api/client";
    // ═══════════════════════════════════════════════════════════════════════
    // ListingImageUpload.svelte
    // Listing-scoped, drag-and-drop image gallery manager with cover setting.
    // Props:
    //   productId  — the UUID of the product/listing
    //   token      — auth JWT for API calls
    //   maxImages  — default 8 images per listing
    //   lang       — 'en' | 'ar' for RTL/LTR UI
    // ═══════════════════════════════════════════════════════════════════════

    interface Image {
        id: string;
        imageUrl: string;
        filePath: string | null;
        isCover: boolean;
        sortOrder: number;
        caption: string | null;
    }

    interface Props {
        productId: string;
        token: string;
        maxImages?: number;
        lang?: 'en' | 'ar';
        initialImages?: Image[];
    }

    let {
        productId,
        token,
        maxImages = 8,
        lang = 'en',
        initialImages = []
    }: Props = $props();

    const isRtl = $derived(lang === 'ar');

    let images    = $state<Image[]>([]);
    let isDragging = $state(false);

    $effect(() => {
        images = initialImages;
    });
    let isUploading = $state(false);
    let uploadError = $state('');
    let uploadSuccess = $state('');



    // ── Upload handler ────────────────────────────────────────────────────────

    async function uploadImage(file: File): Promise<void> {
        if (images.length >= maxImages) {
            uploadError = lang === 'ar'
                ? `الحد الأقصى ${maxImages} صور.`
                : `Maximum ${maxImages} images allowed.`;
            return;
        }

        const allowedTypes = ['image/jpeg', 'image/jpg', 'image/png', 'image/webp'];
        if (!allowedTypes.includes(file.type)) {
            uploadError = lang === 'ar'
                ? 'يُسمح فقط بصور JPG و PNG و WEBP.'
                : 'Only JPG, PNG, and WEBP images are accepted.';
            return;
        }

        if (file.size > 5 * 1024 * 1024) {
            uploadError = lang === 'ar'
                ? 'الحد الأقصى لحجم الصورة 5 ميغابايت.'
                : 'File size must not exceed 5 MB.';
            return;
        }

        uploadError = '';
        isUploading = true;

        try {
            // Phase 1: Upload file to get URL
            const formData = new FormData();
            formData.append('file', file);

            const uploadRes = await apiClient.vendor.uploadMedia(token, formData);

            if (uploadRes.error) {
                throw new Error(uploadRes.error.message || 'Upload failed.');
            }

            const { url, file_path } = uploadRes.data || {};
            if (!url) {
                throw new Error('Upload returned no URL.');
            }

            // Phase 2: Link image to listing (first image auto-set as cover)
            const isCover = images.length === 0;
            const linkRes = await apiClient.vendor.linkProductImage(token, productId, {
                image_url: url,
                file_path: file_path,
                is_cover: isCover,
                sort_order: images.length
            });

            if (linkRes.error) {
                throw new Error(linkRes.error.message || 'Failed to link image.');
            }

            const { id: newId } = linkRes.data || {};

            images = [...images, {
                id: newId,
                imageUrl: url,
                filePath: file_path ?? null,
                isCover,
                sortOrder: images.length,
                caption: null
            }];

            uploadSuccess = lang === 'ar' ? 'تمّ رفع الصورة بنجاح!' : 'Image uploaded successfully!';
            setTimeout(() => { uploadSuccess = ''; }, 3000);

        } catch (err: any) {
            uploadError = err.message || 'Upload failed. Please try again.';
        } finally {
            isUploading = false;
        }
    }

    // ── Delete image ──────────────────────────────────────────────────────────

    async function deleteImage(imgId: string) {
        const confirmed = confirm(
            lang === 'ar'
                ? 'هل أنت متأكد من حذف هذه الصورة؟ سيتم حذف الملف نهائياً.'
                : 'Are you sure you want to delete this image? The file will be permanently deleted.'
        );
        if (!confirmed) return;

        const res = await apiClient.vendor.deleteProductImage(token, productId, imgId);

        if (!res.error) {
            images = images.filter(img => img.id !== imgId);
            // If the deleted image was the cover, promote the first remaining image
            if (images.length > 0 && !images.some(img => img.isCover)) {
                await setCover(images[0].id);
            }
        } else {
            uploadError = lang === 'ar' ? 'فشل حذف الصورة.' : 'Failed to delete image.';
        }
    }

    // ── Set cover ──────────────────────────────────────────────────────────────

    async function setCover(imgId: string) {
        const res = await apiClient.vendor.setProductCoverImage(token, productId, imgId);

        if (!res.error) {
            images = images.map(img => ({ ...img, isCover: img.id === imgId }));
        } else {
            uploadError = lang === 'ar' ? 'فشل تعيين صورة الغلاف.' : 'Failed to set cover image.';
        }
    }

    // ── Drag-and-drop ──────────────────────────────────────────────────────────

    function onDragOver(e: DragEvent) {
        e.preventDefault();
        isDragging = true;
    }

    function onDragLeave() {
        isDragging = false;
    }

    async function onDrop(e: DragEvent) {
        e.preventDefault();
        isDragging = false;
        const files = Array.from(e.dataTransfer?.files ?? []);
        for (const file of files) {
            if (images.length < maxImages) {
                await uploadImage(file);
            }
        }
    }

    async function onFileInput(e: Event) {
        const input = e.target as HTMLInputElement;
        const files = Array.from(input.files ?? []);
        for (const file of files) {
            if (images.length < maxImages) {
                await uploadImage(file);
            }
        }
        input.value = '';
    }

    const t = {
        ar: {
            title:       'معرض الصور',
            dropzone:    'اسحب الصور هنا أو انقر للرفع',
            subtext:     'JPG · PNG · WEBP — حتى 5 ميغابايت لكل صورة',
            coverLabel:  'غلاف',
            setCover:    'تعيين غلاف',
            delete:      'حذف',
            uploading:   'جارٍ الرفع...',
            maxHint:     (n: number) => `${n} صور متبقية`
        },
        en: {
            title:       'Listing Gallery',
            dropzone:    'Drag & drop images here, or click to select',
            subtext:     'JPG · PNG · WEBP — max 5 MB per image',
            coverLabel:  'Cover',
            setCover:    'Set as Cover',
            delete:      'Delete',
            uploading:   'Uploading...',
            maxHint:     (n: number) => `${n} image slot(s) remaining`
        }
    };

    const tx = $derived(lang === 'ar' ? t.ar : t.en);
    const remaining = $derived(maxImages - images.length);
</script>

<div class="gallery-manager" dir={isRtl ? 'rtl' : 'ltr'}>
    <!-- Header -->
    <div class="gallery-header">
        <h3 class="gallery-title">{tx.title}</h3>
        <span class="slots-remaining" class:warning={remaining <= 2}>
            {tx.maxHint(remaining)}
        </span>
    </div>

    <!-- Feedback -->
    {#if uploadError}
        <div class="feedback error" role="alert">{uploadError}</div>
    {/if}
    {#if uploadSuccess}
        <div class="feedback success" role="status">{uploadSuccess}</div>
    {/if}

    <!-- Image Grid -->
    {#if images.length > 0}
        <div class="image-grid">
            {#each images as img (img.id)}
                <div class="image-card" class:is-cover={img.isCover}>
                    {#if img.isCover}
                        <div class="cover-badge">{tx.coverLabel}</div>
                    {/if}
                    <img src={resolveMediaUrl(img.imageUrl)} alt={img.caption ?? 'Listing image'} loading="lazy" />
                    <div class="image-actions">
                        {#if !img.isCover}
                            <button
                                type="button"
                                class="btn-action set-cover"
                                onclick={() => setCover(img.id)}
                                title={tx.setCover}
                            >
                                ⭐
                            </button>
                        {/if}
                        <button
                            type="button"
                            class="btn-action delete"
                            onclick={() => deleteImage(img.id)}
                            title={tx.delete}
                        >
                            🗑
                        </button>
                    </div>
                </div>
            {/each}
        </div>
    {/if}

    <!-- Drop Zone (shown when below max) -->
    {#if images.length < maxImages}
        <div
            class="dropzone"
            class:dragging={isDragging}
            class:uploading={isUploading}
            role="button"
            tabindex="0"
            aria-label={tx.dropzone}
            ondragover={onDragOver}
            ondragleave={onDragLeave}
            ondrop={onDrop}
            onclick={() => !isUploading && (document.getElementById(`file-input-${productId}`) as HTMLInputElement)?.click()}
            onkeydown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                    e.preventDefault();
                    if (!isUploading) {
                        (document.getElementById(`file-input-${productId}`) as HTMLInputElement)?.click();
                    }
                }
            }}
        >
            {#if isUploading}
                <div class="upload-spinner"></div>
                <span class="dz-text">{tx.uploading}</span>
            {:else}
                <div class="upload-icon">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M12 16.5V9.75m0 0l3 3m-3-3l-3 3M6.75 19.5a4.5 4.5 0 01-1.41-8.775 5.25 5.25 0 0110.338-2.32 5.75 5.75 0 011.985 4.93A4.5 4.5 0 0117.25 19.5H6.75z" />
                    </svg>
                </div>
                <span class="dz-text">{tx.dropzone}</span>
                <span class="dz-sub">{tx.subtext}</span>
            {/if}
        </div>

        <input
            id="file-input-{productId}"
            type="file"
            accept=".jpg,.jpeg,.png,.webp"
            multiple
            style="display:none"
            onchange={onFileInput}
        />
    {/if}
</div>

<style>
.gallery-manager {
    display: flex;
    flex-direction: column;
    gap: 1rem;
}

.gallery-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
}

.gallery-title {
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-text-primary, #1a1a2e);
    margin: 0;
}

.slots-remaining {
    font-size: 0.75rem;
    color: var(--color-text-muted, #888);
    background: rgba(99,102,241,0.08);
    padding: 0.2rem 0.6rem;
    border-radius: 20px;
    transition: all 0.3s ease;
}
.slots-remaining.warning {
    color: #e67e22;
    background: rgba(230,126,34,0.12);
}

.feedback {
    padding: 0.75rem 1rem;
    border-radius: 10px;
    font-size: 0.875rem;
    font-weight: 500;
}
.feedback.error   { background: rgba(239,68,68,0.1);  color: #dc2626; border: 1px solid rgba(239,68,68,0.2); }
.feedback.success { background: rgba(34,197,94,0.1);  color: #16a34a; border: 1px solid rgba(34,197,94,0.2); }

/* Image grid */
.image-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
    gap: 0.75rem;
}

.image-card {
    position: relative;
    border-radius: 12px;
    overflow: hidden;
    aspect-ratio: 4/3;
    border: 2px solid transparent;
    transition: all 0.25s ease;
    cursor: default;
}

.image-card:hover { transform: translateY(-2px); box-shadow: 0 8px 24px rgba(0,0,0,0.15); }
.image-card.is-cover { border-color: #6366f1; box-shadow: 0 0 0 3px rgba(99,102,241,0.2); }

.image-card img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
}

.cover-badge {
    position: absolute;
    top: 6px;
    left: 6px;
    background: #6366f1;
    color: white;
    font-size: 0.65rem;
    font-weight: 700;
    text-transform: uppercase;
    padding: 2px 8px;
    border-radius: 20px;
    letter-spacing: 0.05em;
    z-index: 2;
}

.image-actions {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    display: flex;
    justify-content: center;
    gap: 0.4rem;
    padding: 0.4rem;
    background: linear-gradient(to top, rgba(0,0,0,0.7), transparent);
    opacity: 0;
    transition: opacity 0.2s ease;
}
.image-card:hover .image-actions { opacity: 1; }

.btn-action {
    border: none;
    border-radius: 8px;
    padding: 0.3rem 0.5rem;
    font-size: 0.85rem;
    cursor: pointer;
    transition: all 0.15s ease;
    line-height: 1;
}
.btn-action.set-cover {
    background: rgba(255,255,255,0.9);
    color: #6366f1;
}
.btn-action.set-cover:hover { background: white; transform: scale(1.1); }
.btn-action.delete {
    background: rgba(239,68,68,0.8);
    color: white;
}
.btn-action.delete:hover { background: #ef4444; transform: scale(1.1); }

/* Drop zone */
.dropzone {
    border: 2px dashed rgba(99,102,241,0.3);
    border-radius: 16px;
    padding: 2.5rem 1.5rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;
    transition: all 0.25s ease;
    background: rgba(99,102,241,0.02);
    text-align: center;
}

.dropzone:hover,
.dropzone.dragging {
    border-color: #6366f1;
    background: rgba(99,102,241,0.06);
    transform: translateY(-1px);
}

.dropzone.uploading {
    cursor: not-allowed;
    opacity: 0.8;
}

.upload-icon {
    width: 48px;
    height: 48px;
    color: #6366f1;
    opacity: 0.7;
    transition: all 0.3s ease;
}
.dropzone:hover .upload-icon { opacity: 1; transform: scale(1.1); }
.upload-icon svg { width: 100%; height: 100%; }

.dz-text {
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--color-text-primary, #1a1a2e);
}

.dz-sub {
    font-size: 0.75rem;
    color: var(--color-text-muted, #888);
}

/* Spinner */
.upload-spinner {
    width: 32px;
    height: 32px;
    border: 3px solid rgba(99,102,241,0.2);
    border-top-color: #6366f1;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
}
@keyframes spin { to { transform: rotate(360deg); } }
</style>
