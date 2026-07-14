<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { apiClient } from '$lib/api/client';
    import { Sparkles, Upload, Loader, AlertCircle, RefreshCw, FileText } from 'lucide-svelte';

    // ═══════════════════════════════════════════════════════════════════════
    // MediaUpload.svelte
    // Reusable Enterprise-grade file upload component with status tracking.
    // Supports images and video transcoding progress via polling.
    // ═══════════════════════════════════════════════════════════════════════

    interface Props {
        token: string;
        lang?: 'en' | 'ar';
        maxSizeBytes?: number; // default 200MB (limit for videos)
        allowedTypes?: string[]; // MIME types
        targetDir?: string;
        onSuccess?: (media: {
            id: string;
            url: string;
            file_path: string;
            media_type: string;
            thumbnail_url?: string;
            file_size: number;
        }) => void;
        onFailure?: (err: string) => void;
    }

    let {
        token,
        lang = 'en',
        maxSizeBytes = 200 * 1024 * 1024,
        allowedTypes = [
            'image/jpeg', 'image/jpg', 'image/png', 'image/webp',
            'video/mp4', 'video/webm', 'video/quicktime', 'application/mp4'
        ],
        onSuccess,
        onFailure
    }: Props = $props();

    const isRtl = $derived(lang === 'ar');

    // States
    type UploadStatus = 'idle' | 'uploaded' | 'processing' | 'ready' | 'failed';
    let status = $state<UploadStatus>('idle');
    let progressPercent = $state(0);
    let errorMessage = $state('');
    let isDragging = $state(false);
    let currentFile = $state<File | null>(null);
    let mediaId = $state<string | null>(null);
    let pollInterval = $state<any>(null);

    // Clean up timers on destroy
    onDestroy(() => {
        clearPoll();
    });

    function clearPoll() {
        if (pollInterval) {
            clearInterval(pollInterval);
            pollInterval = null;
        }
    }

    // Keyboard handlers
    let fileInputRef = $state<HTMLInputElement | null>(null);

    function triggerSelect() {
        fileInputRef?.click();
    }

    function handleKeyDown(e: KeyboardEvent) {
        if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            triggerSelect();
        }
    }

    // Validation
    function validateFile(file: File): boolean {
        if (file.size > maxSizeBytes) {
            errorMessage = isRtl
                ? `الملف كبير جداً. الحد الأقصى المسموح به هو ${Math.round(maxSizeBytes / (1024 * 1024))} ميغابايت.`
                : `File is too large. Maximum allowed size is ${Math.round(maxSizeBytes / (1024 * 1024))} MB.`;
            status = 'failed';
            onFailure?.(errorMessage);
            return false;
        }

        // Check file extension as fallback if MIME type is empty
        const ext = file.name.substring(file.name.lastIndexOf('.')).toLowerCase();
        const validExtensions = ['.jpg', '.jpeg', '.png', '.webp', '.mp4', '.webm', '.mov'];
        const isValidMime = allowedTypes.includes(file.type);
        const isValidExt = validExtensions.includes(ext);

        if (!isValidMime && !isValidExt) {
            errorMessage = isRtl
                ? 'نوع الملف غير مدعوم. يرجى رفع صورة أو مقطع فيديو.'
                : 'Unsupported file type. Please upload a valid image or video file.';
            status = 'failed';
            onFailure?.(errorMessage);
            return false;
        }

        return true;
    }

    // Drag-and-drop triggers
    function onDragOver(e: DragEvent) {
        e.preventDefault();
        isDragging = true;
    }

    function onDragLeave(e: DragEvent) {
        e.preventDefault();
        isDragging = false;
    }

    function onDrop(e: DragEvent) {
        e.preventDefault();
        isDragging = false;
        const files = e.dataTransfer?.files;
        if (files && files.length > 0) {
            handleFileUpload(files[0]);
        }
    }

    function onFileSelect(e: Event) {
        const target = e.target as HTMLInputElement;
        const files = target.files;
        if (files && files.length > 0) {
            handleFileUpload(files[0]);
        }
    }

    // Polling background transcoder status
    async function startPolling(id: string) {
        clearPoll();
        status = 'processing';
        pollInterval = setInterval(async () => {
            try {
                const res = await apiClient.vendor.getUploadStatus(token, id);
                if (res.data && res.data.status === 'success') {
                    const upData = res.data.data;
                    if (upData.status === 'ready') {
                        clearPoll();
                        status = 'ready';
                        onSuccess?.({
                            id: upData.id,
                            url: upData.file_url,
                            file_path: upData.file_url, // fallback
                            media_type: upData.mime_type.startsWith('video/') ? 'video' : 'image',
                            file_size: upData.file_size
                        });
                    } else if (upData.status === 'failed') {
                        clearPoll();
                        status = 'failed';
                        errorMessage = upData.error_message || (isRtl ? 'فشلت معالجة الوسائط.' : 'Media processing failed.');
                        onFailure?.(errorMessage);
                    }
                }
            } catch (err: any) {
                // Ignore transient network errors during polling
                tracing: console.warn('Error polling status:', err);
            }
        }, 2000);
    }

    // Core upload function
    async function handleFileUpload(file: File) {
        if (!validateFile(file)) return;

        currentFile = file;
        errorMessage = '';
        status = 'uploaded';
        progressPercent = 0;

        const formData = new FormData();
        formData.append('file', file);
        formData.append('media_type', file.type.startsWith('video/') ? 'video' : 'image');

        try {
            const res = await apiClient.vendor.uploadMedia(token, formData);
            if (res.error) {
                throw new Error(res.error.message || 'Upload failed');
            }

            const data = res.data;
            if (data && data.status === 'success') {
                mediaId = data.id;
                if (data.status_state === 'processing') {
                    startPolling(data.id);
                } else {
                    status = 'ready';
                    onSuccess?.({
                        id: data.id,
                        url: data.url,
                        file_path: data.file_path,
                        media_type: data.media_type,
                        thumbnail_url: data.thumbnail_url,
                        file_size: data.file_size
                    });
                }
            }
        } catch (err: any) {
            status = 'failed';
            errorMessage = err.message || (isRtl ? 'حدث خطأ أثناء الرفع.' : 'Upload failed due to connection error.');
            onFailure?.(errorMessage);
        }
    }

    function retry() {
        if (currentFile) {
            handleFileUpload(currentFile);
        } else {
            status = 'idle';
        }
    }

    function reset() {
        clearPoll();
        status = 'idle';
        currentFile = null;
        errorMessage = '';
        mediaId = null;
    }
</script>

<div class="media-upload-container {isRtl ? 'rtl' : 'ltr'}">
    <!-- Dropzone -->
    {#if status === 'idle' || isDragging}
        <div
            class="dropzone {isDragging ? 'dragging' : ''}"
            role="button"
            tabindex="0"
            aria-label={isRtl ? "منطقة رفع الملفات" : "File Upload Dropzone"}
            onclick={triggerSelect}
            onkeydown={handleKeyDown}
            ondragover={onDragOver}
            ondragleave={onDragLeave}
            ondrop={onDrop}
        >
            <input
                type="file"
                bind:this={fileInputRef}
                class="hidden-file-input"
                accept={allowedTypes.join(',')}
                onchange={onFileSelect}
            />
            <div class="dropzone-content">
                <div class="icon-circle">
                    <Upload size={28} class="upload-icon" />
                </div>
                <div class="text-group">
                    <p class="main-text">
                        {isRtl ? 'اسحب وأفلت الملف هنا أو انقر للاختيار' : 'Drag & drop file here or click to browse'}
                    </p>
                    <p class="sub-text">
                        {isRtl ? 'يدعم الصور ومقاطع الفيديو حتى 200 ميغابايت' : 'Supports image & video formats up to 200 MB'}
                    </p>
                </div>
            </div>
        </div>
    {/if}

    <!-- Uploading / Processing State -->
    {#if status === 'uploaded' || status === 'processing'}
        <div class="status-card glass-card">
            <div class="status-header">
                <div class="spinner-container">
                    <Loader size={32} class="animate-spin text-primary" />
                </div>
                <div class="status-info">
                    <h4 class="status-title">
                        {status === 'uploaded' 
                            ? (isRtl ? 'جاري رفع الملف...' : 'Uploading file...') 
                            : (isRtl ? 'جاري معالجة وتحويل الوسائط...' : 'Processing and optimization pipeline...')
                        }
                    </h4>
                    <p class="status-description">
                        {isRtl 
                            ? 'يرجى الانتظار، جاري تحضير ملفك بجودة محسنة.' 
                            : 'Please wait while we transcode and optimize files for web delivery.'
                        }
                    </p>
                </div>
            </div>

            <!-- Fake Progress Loader bar -->
            <div class="progress-container">
                <div class="progress-bar-bg">
                    <div class="progress-bar-fill animated-shimmer"></div>
                </div>
            </div>
        </div>
    {/if}

    <!-- Ready / Success State -->
    {#if status === 'ready'}
        <div class="status-card success glass-card">
            <div class="status-header">
                <div class="icon-circle-success">
                    <Sparkles size={24} class="success-icon" />
                </div>
                <div class="status-info">
                    <h4 class="status-title">
                        {isRtl ? 'اكتملت المعالجة بنجاح!' : 'Media processing completed!'}
                    </h4>
                    <p class="status-description">
                        {isRtl 
                            ? 'تم ضغط وتحسين الوسائط بنجاح للويب.' 
                            : 'All web-optimized media variants successfully generated and saved.'
                        }
                    </p>
                </div>
                <button type="button" class="btn-reset" onclick={reset}>
                    {isRtl ? 'رفع ملف آخر' : 'Upload another'}
                </button>
            </div>
        </div>
    {/if}

    <!-- Failed / Error State -->
    {#if status === 'failed'}
        <div class="status-card error glass-card">
            <div class="status-header">
                <div class="icon-circle-error">
                    <AlertCircle size={24} class="error-icon" />
                </div>
                <div class="status-info">
                    <h4 class="status-title">
                        {isRtl ? 'فشلت معالجة الملف' : 'Upload or processing failed'}
                    </h4>
                    <p class="status-description error-msg">{errorMessage}</p>
                </div>
            </div>
            <div class="action-buttons">
                <button type="button" class="btn-retry" onclick={retry}>
                    <RefreshCw size={16} class="btn-icon" />
                    {isRtl ? 'إعادة المحاولة' : 'Retry'}
                </button>
                <button type="button" class="btn-cancel" onclick={reset}>
                    {isRtl ? 'إلغاء' : 'Cancel'}
                </button>
            </div>
        </div>
    {/if}
</div>

<style>
    .media-upload-container {
        font-family: 'Cairo', 'Outfit', sans-serif;
        width: 100%;
        margin-top: 0.5rem;
    }

    .dropzone {
        border: 2px dashed rgba(124, 58, 237, 0.3);
        background: rgba(255, 255, 255, 0.02);
        border-radius: 12px;
        padding: 2.5rem 1.5rem;
        text-align: center;
        cursor: pointer;
        transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
        position: relative;
        outline: none;
    }

    .dropzone:focus-visible {
        border-color: #7c3aed;
        box-shadow: 0 0 0 3px rgba(124, 58, 237, 0.25);
    }

    .dropzone:hover, .dropzone.dragging {
        border-color: #7c3aed;
        background: rgba(124, 58, 237, 0.04);
        transform: translateY(-2px);
    }

    .hidden-file-input {
        display: none;
    }

    .dropzone-content {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1rem;
    }

    .icon-circle {
        width: 60px;
        height: 60px;
        border-radius: 50%;
        background: rgba(124, 58, 237, 0.08);
        display: flex;
        align-items: center;
        justify-content: center;
        color: #7c3aed;
        transition: transform 0.3s ease;
    }

    .dropzone:hover .icon-circle {
        transform: scale(1.1);
    }

    .text-group .main-text {
        font-size: 1.05rem;
        font-weight: 600;
        color: var(--text-color, #1f2937);
        margin: 0 0 0.25rem 0;
    }

    .text-group .sub-text {
        font-size: 0.85rem;
        color: #6b7280;
        margin: 0;
    }

    /* Glass Cards */
    .glass-card {
        background: rgba(255, 255, 255, 0.03);
        backdrop-filter: blur(10px);
        -webkit-backdrop-filter: blur(10px);
        border: 1px solid rgba(255, 255, 255, 0.05);
        border-radius: 12px;
        padding: 1.5rem;
    }

    .status-card {
        display: flex;
        flex-direction: column;
        gap: 1.25rem;
        animation: fadeIn 0.3s ease-out;
    }

    .status-header {
        display: flex;
        align-items: flex-start;
        gap: 1rem;
    }

    .status-info {
        flex: 1;
    }

    .status-title {
        font-size: 1.05rem;
        font-weight: 700;
        margin: 0 0 0.25rem 0;
        color: #f3f4f6;
    }

    .status-description {
        font-size: 0.88rem;
        color: #9ca3af;
        margin: 0;
        line-height: 1.4;
    }

    .error-msg {
        color: #ef4444 !important;
    }

    /* Progress bar styling */
    .progress-container {
        width: 100%;
    }

    .progress-bar-bg {
        width: 100%;
        height: 6px;
        background: rgba(255, 255, 255, 0.06);
        border-radius: 9999px;
        overflow: hidden;
    }

    .progress-bar-fill {
        height: 100%;
        width: 60%; /* Shimmer placeholder */
        background: linear-gradient(90deg, #7c3aed, #ec4899);
        border-radius: 9999px;
        animation: loadingShimmer 2.5s infinite linear;
    }

    /* Shimmer Animation */
    @keyframes loadingShimmer {
        0% {
            transform: translateX(-100%);
        }
        100% {
            transform: translateX(200%);
        }
    }

    /* Icon circles */
    .icon-circle-success {
        width: 48px;
        height: 48px;
        border-radius: 50%;
        background: rgba(16, 185, 129, 0.1);
        display: flex;
        align-items: center;
        justify-content: center;
        color: #10b981;
    }

    .icon-circle-error {
        width: 48px;
        height: 48px;
        border-radius: 50%;
        background: rgba(239, 68, 68, 0.1);
        display: flex;
        align-items: center;
        justify-content: center;
        color: #ef4444;
    }

    /* Action Buttons */
    .action-buttons {
        display: flex;
        gap: 0.75rem;
    }

    .btn-retry {
        background: #7c3aed;
        color: white;
        border: none;
        border-radius: 8px;
        padding: 0.5rem 1.25rem;
        font-weight: 600;
        font-size: 0.88rem;
        cursor: pointer;
        display: inline-flex;
        align-items: center;
        gap: 0.5rem;
        transition: background 0.2s;
    }

    .btn-retry:hover {
        background: #6d28d9;
    }

    .btn-cancel {
        background: rgba(255, 255, 255, 0.05);
        color: #d1d5db;
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 8px;
        padding: 0.5rem 1.25rem;
        font-weight: 600;
        font-size: 0.88rem;
        cursor: pointer;
        transition: background 0.2s;
    }

    .btn-cancel:hover {
        background: rgba(255, 255, 255, 0.1);
    }

    .btn-reset {
        background: rgba(16, 185, 129, 0.1);
        color: #10b981;
        border: 1px solid rgba(16, 185, 129, 0.2);
        border-radius: 8px;
        padding: 0.4rem 1rem;
        font-size: 0.85rem;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s;
    }

    .btn-reset:hover {
        background: rgba(16, 185, 129, 0.18);
    }

    .animate-spin {
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        from { transform: rotate(0deg); }
        to { transform: rotate(360deg); }
    }

    @keyframes fadeIn {
        from { opacity: 0; transform: translateY(4px); }
        to { opacity: 1; transform: translateY(0); }
    }
</style>
