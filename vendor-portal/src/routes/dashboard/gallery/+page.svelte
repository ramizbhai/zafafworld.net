<script lang="ts">
    import { enhance } from '$app/forms';
    import { invalidateAll } from '$app/navigation';
    import { Building2, Upload, Trash2, Globe, Image, CheckCircle2, AlertCircle } from 'lucide-svelte';
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import { resolveMediaUrl } from '$lib/shared/utils/media';

    interface BrandImage {
        id: string;
        image_url: string;
        is_cover: boolean;
        caption: string | null;
    }

    interface Props {
        data: {
            brandGallery: BrandImage[];
            brandLogo:    BrandImage | null;
            brandCover:   BrandImage | null;
            vendor:       any;
            sessionToken: string;
        };
        form: any;
    }

    let { data, form }: Props = $props();
    const i18n = getI18n();

    let isUploadingLogo  = $state(false);
    let isUploadingCover = $state(false);

    // Local drag states
    let draggingLogo  = $state(false);
    let draggingCover = $state(false);

    function dropFile(
        e: DragEvent,
        role: 'logo' | 'cover'
    ) {
        e.preventDefault();
        if (role === 'logo')  draggingLogo  = false;
        if (role === 'cover') draggingCover = false;

        const file = e.dataTransfer?.files?.[0];
        if (!file) return;

        triggerUpload(file, role);
    }

    async function triggerUpload(file: File, role: 'logo' | 'cover') {
        const allowed = ['image/jpeg', 'image/png', 'image/webp'];
        if (!allowed.includes(file.type)) {
            alert(i18n.locale === 'ar' ? 'يتم قبول صور JPG أو PNG أو WEBP فقط.' : 'Only JPG, PNG, or WEBP images are accepted.');
            return;
        }
        if (file.size > 5 * 1024 * 1024) {
            alert(i18n.locale === 'ar' ? 'يجب أن يكون حجم الصورة أقل من 5 ميجابايت.' : 'Image must be under 5 MB.');
            return;
        }

        if (role === 'logo')  isUploadingLogo  = true;
        if (role === 'cover') isUploadingCover = true;

        try {
            const fd = new FormData();
            fd.append('file', file);
            fd.append('role', role);

            const res = await fetch('?/uploadBrandImage', {
                method: 'POST',
                body: fd
            });

            if (res.ok) await invalidateAll();
        } finally {
            if (role === 'logo')  isUploadingLogo  = false;
            if (role === 'cover') isUploadingCover = false;
        }
    }

    function onFileInput(e: Event, role: 'logo' | 'cover') {
        const file = (e.target as HTMLInputElement).files?.[0];
        if (file) triggerUpload(file, role);
        (e.target as HTMLInputElement).value = '';
    }
</script>

<svelte:head>
    <title>{i18n.t.gallery.title} | {i18n.t.common.appName}</title>
    <meta name="description" content={i18n.t.gallery.subtitle} />
</svelte:head>

<div class="brand-page">
    <!-- Page Header -->
    <div class="page-header">
        <div class="header-icon"><Building2 size={22} /></div>
        <div>
            <h1 class="page-title">{i18n.t.gallery.title}</h1>
            <p class="page-subtitle">
                {i18n.t.gallery.subtitle}
            </p>
        </div>
    </div>

    <!-- Feedback Banner -->
    {#if form?.success}
        <div class="feedback success" role="status">
            <CheckCircle2 size={16} />
            {form.message ?? i18n.t.gallery.successSave}
        </div>
    {/if}
    {#if form?.error}
        <div class="feedback error" role="alert">
            <AlertCircle size={16} />
            {form.error}
        </div>
    {/if}

    <!-- Info callout -->
    <div class="info-callout">
        <Globe size={16} />
        <div>
            <strong>{i18n.t.gallery.infoCalloutTitle}</strong>
            {i18n.t.gallery.infoCalloutText}
            <a href="/dashboard/products">{i18n.t.gallery.myListings}</a>
            {i18n.t.gallery.andOpenEditor}
        </div>
    </div>

    <div class="brand-grid">
        <!-- ── Logo Card ──────────────────────────────────────────────────── -->
        <div class="brand-card">
            <div class="brand-card-header">
                <div class="card-icon logo-icon">
                    <Image size={18} />
                </div>
                <div>
                    <h2 class="card-title">{i18n.t.gallery.brandLogo}</h2>
                    <p class="card-desc">{i18n.t.gallery.brandLogoDesc}</p>
                </div>
            </div>

            <!-- Current logo preview -->
            {#if data.brandLogo}
                <div class="current-image logo-preview">
                    <img src={resolveMediaUrl(data.brandLogo.image_url)} alt={i18n.t.gallery.brandLogo} />
                    <form method="POST" action="?/deleteBrandImage" use:enhance>
                        <input type="hidden" name="image_id" value={data.brandLogo.id} />
                        <button type="submit" class="delete-btn" title={i18n.t.gallery.removeLogo} aria-label={i18n.t.gallery.removeLogo}>
                            <Trash2 size={14} />
                        </button>
                    </form>
                </div>
            {/if}

            <!-- Drop zone -->
            <div
                class="dropzone"
                class:dragging={draggingLogo}
                class:uploading={isUploadingLogo}
                role="button"
                tabindex="0"
                aria-label={i18n.t.gallery.brandLogo}
                ondragover={(e) => { e.preventDefault(); draggingLogo = true; }}
                ondragleave={() => draggingLogo = false}
                ondrop={(e) => dropFile(e, 'logo')}
                onclick={() => !isUploadingLogo && (document.getElementById('logo-file') as HTMLInputElement)?.click()}
                onkeydown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                        e.preventDefault();
                        if (!isUploadingLogo) {
                            (document.getElementById('logo-file') as HTMLInputElement)?.click();
                        }
                    }
                }}
            >
                {#if isUploadingLogo}
                    <div class="dz-spinner"></div>
                    <span class="dz-text">{i18n.t.gallery.uploading}</span>
                {:else}
                    <Upload size={24} class="dz-upload-icon" />
                    <span class="dz-text">{data.brandLogo ? i18n.t.gallery.replaceLogo : i18n.t.gallery.uploadLogo}</span>
                    <span class="dz-sub">{i18n.t.gallery.dropzoneSub}</span>
                {/if}
            </div>
            <input id="logo-file" type="file" accept=".jpg,.jpeg,.png,.webp" style="display:none" onchange={(e) => onFileInput(e, 'logo')} />
        </div>

        <!-- ── Cover Photo Card ───────────────────────────────────────────── -->
        <div class="brand-card cover-card">
            <div class="brand-card-header">
                <div class="card-icon cover-icon">
                    <Globe size={18} />
                </div>
                <div>
                    <h2 class="card-title">{i18n.t.gallery.brandCover}</h2>
                    <p class="card-desc">{i18n.t.gallery.brandCoverDesc}</p>
                </div>
            </div>

            {#if data.brandCover}
                <div class="current-image cover-preview">
                    <img src={resolveMediaUrl(data.brandCover.image_url)} alt={i18n.t.gallery.brandCover} />
                    <form method="POST" action="?/deleteBrandImage" use:enhance>
                        <input type="hidden" name="image_id" value={data.brandCover.id} />
                        <button type="submit" class="delete-btn" title={i18n.t.gallery.removeCover} aria-label={i18n.t.gallery.removeCover}>
                            <Trash2 size={14} />
                        </button>
                    </form>
                </div>
            {/if}

            <div
                class="dropzone wide"
                class:dragging={draggingCover}
                class:uploading={isUploadingCover}
                role="button"
                tabindex="0"
                aria-label={i18n.t.gallery.brandCover}
                ondragover={(e) => { e.preventDefault(); draggingCover = true; }}
                ondragleave={() => draggingCover = false}
                ondrop={(e) => dropFile(e, 'cover')}
                onclick={() => !isUploadingCover && (document.getElementById('cover-file') as HTMLInputElement)?.click()}
                onkeydown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                        e.preventDefault();
                        if (!isUploadingCover) {
                            (document.getElementById('cover-file') as HTMLInputElement)?.click();
                        }
                    }
                }}
            >
                {#if isUploadingCover}
                    <div class="dz-spinner"></div>
                    <span class="dz-text">{i18n.t.gallery.uploading}</span>
                {:else}
                    <Upload size={24} class="dz-upload-icon" />
                    <span class="dz-text">{data.brandCover ? i18n.t.gallery.replaceCover : i18n.t.gallery.uploadCover}</span>
                    <span class="dz-sub">{i18n.t.gallery.dropzoneSub}</span>
                {/if}
            </div>
            <input id="cover-file" type="file" accept=".jpg,.jpeg,.png,.webp" style="display:none" onchange={(e) => onFileInput(e, 'cover')} />
        </div>
    </div>

    <!-- Brand Gallery (all brand-level images) -->
    {#if data.brandGallery.length > 0}
        <section class="all-brand-images">
            <h2 class="section-title">{i18n.t.gallery.allBrandMedia}</h2>
            <div class="image-grid">
                {#each data.brandGallery as img (img.id)}
                    <div class="image-card" class:is-cover={img.is_cover}>
                        {#if img.is_cover}
                            <div class="cover-badge">{i18n.t.gallery.cover}</div>
                        {:else}
                            <div class="logo-badge">{i18n.t.gallery.logo}</div>
                        {/if}
                        <img src={resolveMediaUrl(img.image_url)} alt={img.caption ?? (img.is_cover ? i18n.t.gallery.brandCover : i18n.t.gallery.brandLogo)} loading="lazy" />
                        <form method="POST" action="?/deleteBrandImage" use:enhance class="delete-form">
                            <input type="hidden" name="image_id" value={img.id} />
                            <button type="submit" class="delete-btn overlay-delete" title={i18n.t.gallery.delete}>
                                <Trash2 size={13} />
                            </button>
                        </form>
                    </div>
                {/each}
            </div>
        </section>
    {/if}
</div>

<style>
.brand-page {
    max-width: 960px;
    margin: 0 auto;
    padding: 1.5rem 1rem 4rem;
    font-family: var(--font-body, 'Inter', system-ui, sans-serif);
}

/* ── Header ──────────────────────────────────────────────────────────────── */
.page-header {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1.5rem;
}

.header-icon {
    width: 48px;
    height: 48px;
    background: linear-gradient(135deg, rgba(99,102,241,0.12), rgba(139,92,246,0.12));
    border-radius: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #6366f1;
    flex-shrink: 0;
}

.page-title {
    font-size: 1.6rem;
    font-weight: 700;
    color: var(--color-text-primary, #1a1a2e);
    margin: 0 0 0.2rem;
}

.page-subtitle {
    color: var(--color-text-muted, #666);
    font-size: 0.875rem;
    margin: 0;
}

/* ── Feedback ────────────────────────────────────────────────────────────── */
.feedback {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.8rem 1.2rem;
    border-radius: 12px;
    font-size: 0.875rem;
    font-weight: 500;
    margin-bottom: 1.25rem;
}
.feedback.success { background: rgba(34,197,94,0.1); color: #16a34a; border: 1px solid rgba(34,197,94,0.2); }
.feedback.error   { background: rgba(239,68,68,0.1); color: #dc2626; border: 1px solid rgba(239,68,68,0.2); }

/* ── Info Callout ────────────────────────────────────────────────────────── */
.info-callout {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    background: rgba(99,102,241,0.06);
    border: 1px solid rgba(99,102,241,0.15);
    border-radius: 12px;
    padding: 1rem 1.25rem;
    font-size: 0.85rem;
    color: var(--color-text-muted, #555);
    margin-bottom: 1.75rem;
}
.info-callout a { color: #6366f1; text-decoration: none; font-weight: 600; }
.info-callout a:hover { text-decoration: underline; }

/* ── Brand Grid ──────────────────────────────────────────────────────────── */
.brand-grid {
    display: grid;
    grid-template-columns: 300px 1fr;
    gap: 1.25rem;
    margin-bottom: 1.75rem;
}

@media (max-width: 700px) {
    .brand-grid { grid-template-columns: 1fr; }
}

/* ── Brand Cards ─────────────────────────────────────────────────────────── */
.brand-card {
    background: white;
    border: 1px solid rgba(0,0,0,0.07);
    border-radius: 16px;
    padding: 1.5rem;
    box-shadow: 0 2px 12px rgba(0,0,0,0.05);
    display: flex;
    flex-direction: column;
    gap: 1rem;
}

.brand-card-header {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
}

.card-icon {
    width: 36px;
    height: 36px;
    border-radius: 10px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
}
.logo-icon  { background: rgba(99,102,241,0.1);  color: #6366f1; }
.cover-icon { background: rgba(139,92,246,0.12); color: #8b5cf6; }

.card-title {
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--color-text-primary, #1a1a2e);
    margin: 0 0 0.15rem;
}
.card-desc {
    font-size: 0.78rem;
    color: var(--color-text-muted, #888);
    margin: 0;
}

/* Current image previews */
.current-image {
    position: relative;
    border-radius: 12px;
    overflow: hidden;
    border: 2px solid rgba(99,102,241,0.2);
}
.logo-preview  { max-width: 120px; aspect-ratio: 1; margin: 0 auto; }
.cover-preview { aspect-ratio: 3/1; }

.current-image img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
}

.delete-btn {
    position: absolute;
    top: 6px;
    inset-inline-end: 6px;
    width: 28px;
    height: 28px;
    border-radius: 8px;
    border: none;
    background: rgba(239,68,68,0.85);
    color: white;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.15s ease;
    padding: 0;
}
.delete-btn:hover { background: #ef4444; transform: scale(1.1); }

/* ── Drop Zone ───────────────────────────────────────────────────────────── */
.dropzone {
    border: 2px dashed rgba(99,102,241,0.3);
    border-radius: 14px;
    padding: 2rem 1.5rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.4rem;
    cursor: pointer;
    transition: all 0.25s ease;
    background: rgba(99,102,241,0.02);
    text-align: center;
}
.dropzone:hover, .dropzone.dragging {
    border-color: #6366f1;
    background: rgba(99,102,241,0.06);
}
.dropzone.uploading { cursor: not-allowed; opacity: 0.7; }
.dropzone.wide { padding: 2.5rem; }

.dz-spinner {
    width: 28px;
    height: 28px;
    border: 3px solid rgba(99,102,241,0.2);
    border-top-color: #6366f1;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
}

.dz-text { font-size: 0.875rem; font-weight: 500; color: var(--color-text-primary, #1a1a2e); }
.dz-sub  { font-size: 0.75rem;  color: var(--color-text-muted, #888); }

/* ── All brand images grid ───────────────────────────────────────────────── */
.all-brand-images { margin-top: 1rem; }

.section-title {
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-text-primary, #1a1a2e);
    margin: 0 0 1rem;
}

.image-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: 0.75rem;
}

.image-card {
    position: relative;
    border-radius: 12px;
    overflow: hidden;
    aspect-ratio: 1;
    border: 2px solid transparent;
    transition: all 0.2s ease;
}
.image-card:hover { transform: translateY(-2px); box-shadow: 0 8px 24px rgba(0,0,0,0.12); }
.image-card.is-cover { border-color: #8b5cf6; }
.image-card img { width: 100%; height: 100%; object-fit: cover; display: block; }

.cover-badge, .logo-badge {
    position: absolute;
    top: 6px;
    inset-inline-start: 6px;
    font-size: 0.65rem;
    font-weight: 700;
    text-transform: uppercase;
    padding: 2px 8px;
    border-radius: 20px;
    letter-spacing: 0.05em;
    z-index: 2;
}
.cover-badge { background: #8b5cf6; color: white; }
.logo-badge  { background: #6366f1; color: white; }

.delete-form { all: unset; }

.overlay-delete {
    position: absolute;
    bottom: 6px;
    inset-inline-end: 6px;
    opacity: 0;
    transition: opacity 0.2s ease;
}
.image-card:hover .overlay-delete { opacity: 1; }

@keyframes spin { to { transform: rotate(360deg); } }
</style>
