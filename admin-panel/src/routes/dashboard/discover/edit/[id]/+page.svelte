<script lang="ts">
  import { enhance } from "$app/forms";
  import { resolveMediaUrl } from "$lib/shared/utils/media";
  import { page } from "$app/stores";
  import {
    ArrowLeft,
    Save,
    Globe,
    Type,
    Image as ImageIcon,
    CheckCircle,
    AlertCircle,
  } from "lucide-svelte";
  import UnifiedDescriptionBuilder from "$lib/components/UnifiedDescriptionBuilder.svelte";
  import "$lib/components/wizard/wizard.css";
  import { env } from "$env/dynamic/public";

  export let data: any;
  export let form: any;

  let blog = data.blog || {};
  let allCategories = data.allCategories || [];
  let allTags = data.allTags || [];

  let isSaving = false;
  let showSuccess = $page.url.searchParams.get("saved") === "true";

  // Format the existing published_at for the datetime-local input
  let localPublishedAt = "";
  if (blog.published_at) {
    const d = new Date(blog.published_at);
    // Adjust to local time format 'YYYY-MM-DDTHH:mm'
    localPublishedAt = new Date(d.getTime() - d.getTimezoneOffset() * 60000)
      .toISOString()
      .slice(0, 16);
  }

  // Ensure categories and tags are initialized
  if (!blog.categories) blog.categories = [];
  if (!blog.tags) blog.tags = [];

  // Make a simple auto-slug generation from title
  function handleTitleInput() {
    if (!blog.id && blog.title_en && !blog.slug) {
      blog.slug = blog.title_en
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, "-")
        .replace(/(^-|-$)+/g, "");
    }
  }
  
  let isUploadingImage = false;

  async function handleImageUpload(event: Event) {
    const input = event.target as HTMLInputElement;
    if (!input.files || input.files.length === 0) return;
    
    const file = input.files[0];
    const formData = new FormData();
    formData.append('file', file);
    
    isUploadingImage = true;
    try {
      const response = await fetch(`/dashboard/discover/edit/${blog.id || 'new'}/upload`, {
        method: 'POST',
        body: formData
      });
      const result = await response.json();
      if (result.status === 'success') {
        blog.cover_image_url = result.url;
      } else {
        alert('Upload failed: ' + (result.message || 'Unknown error'));
      }
    } catch (e) {
      console.error(e);
      alert('Upload failed');
    } finally {
      isUploadingImage = false;
    }
  }
</script>

<div class="fade-in wizard-shell">
  <div class="page-header px-6 pt-4 pb-0 mb-4 border-b-0">
    <div class="page-header-left">
      <a
        href="/dashboard/discover"
        class="btn btn-outline btn-sm"
        style="margin-right: 12px; border:none; padding: 0 8px;"
      >
        <ArrowLeft size={16} />
      </a>
      <h1 class="page-title">{blog.id ? "Edit Blog Post" : "New Blog Post"}</h1>
    </div>
  </div>

  {#if form?.error}
    <div class="notice-banner error mx-6">
      <AlertCircle size={18} class="notice-icon" />
      <div class="notice-text">{form.error}</div>
    </div>
  {/if}

  {#if form?.success || showSuccess}
    <div class="notice-banner success mx-6">
      <CheckCircle size={18} class="notice-icon" />
      <div class="notice-text">Blog post saved successfully!</div>
    </div>
  {/if}

  <form
    method="POST"
    action="?/save"
    class="editor-layout wizard-content"
    use:enhance={() => {
      isSaving = true;
      return async ({ update }) => {
        isSaving = false;
        await update();
      };
    }}
  >
    <div class="main-column">
      <!-- Google-style Review/SEO Preview -->
      <div class="form-card mb-6">
        <h3 class="section-title"><Globe size={16} /> Search Engine Preview</h3>
        <div class="google-snippet-card">
          <div class="snippet-lang-preview">
            <div class="snippet-meta-info">
              <img
                src="/favicon.webp"
                alt=""
                class="snippet-favicon"
                onerror={(e) => {
                  (e.currentTarget as HTMLImageElement).style.display = "none";
                }}
              />
              <span class="snippet-domain"
                >https://zafafworld.net › blog › {blog.slug || "new-post"}</span
              >
            </div>
            <h4 class="snippet-title">
              {blog.meta_title_en ||
                `${blog.title_en || "New Post"} | ZafafWorld`}
            </h4>
            <p class="snippet-description">
              {blog.meta_description_en ||
                "Add blog content to see a live preview of how this post will appear on search engine results..."}
            </p>
          </div>

          <div class="snippet-divider"></div>

          <div class="snippet-lang-preview ar-preview">
            <div class="snippet-meta-info">
              <img
                src="/favicon.webp"
                alt=""
                class="snippet-favicon"
                onerror={(e) => {
                  (e.currentTarget as HTMLImageElement).style.display = "none";
                }}
              />
              <span class="snippet-domain"
                >https://zafafworld.net › blog › {blog.slug || "new-post"}</span
              >
            </div>
            <h4 class="snippet-title">
              {blog.meta_title_ar ||
                `${blog.title_ar || "منشور جديد"} | زفاف وورلد`}
            </h4>
            <p class="snippet-description">
              {blog.meta_description_ar ||
                "أضف محتوى المنشور لمشاهدة معاينة حية لكيفية ظهور هذه الصفحة على محركات البحث وجوجل..."}
            </p>
          </div>
        </div>
      </div>

      <!-- Title & Slug -->
      <div class="form-card mb-6">
        <h3 class="section-title"><Type size={16} /> Post Details</h3>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
          <div class="form-group">
            <label for="title_en" class="font-bold flex justify-between"
              >Title (English) <span
                class="text-xs text-gray-500 font-normal self-end"
                >Auto-generates Slug</span
              ></label
            >
            <input
              type="text"
              id="title_en"
              name="title_en"
              bind:value={blog.title_en}
              oninput={handleTitleInput}
              class="form-control form-control-lg"
              placeholder="Enter English Title..."
              required
            />
            <input type="hidden" name="title" value={blog.title_en} />
          </div>
          <div class="form-group" dir="rtl">
            <label for="title_ar" class="font-bold font-arabic mb-1 block"
              >العنوان (Arabic)</label
            >
            <input
              type="text"
              id="title_ar"
              name="title_ar"
              bind:value={blog.title_ar}
              class="form-control form-control-lg text-right font-arabic"
              placeholder="أدخل العنوان بالعربية..."
              required
            />
          </div>
        </div>

        <div class="form-group">
          <label for="slug" class="font-bold">Slug / URL</label>
          <div
            class="input-with-prefix flex items-center bg-gray-50 border rounded-md"
          >
            <span
              class="prefix px-3 py-2 text-gray-500 font-mono text-sm border-r"
              >zafafworld.net/blog/</span
            >
            <input
              type="text"
              id="slug"
              name="slug"
              bind:value={blog.slug}
              class="form-control flex-1 border-0 rounded-none shadow-none focus:ring-0 font-mono"
              required
            />
          </div>
        </div>
      </div>

      <!-- UnifiedDescriptionBuilder (Dynamic Blocks) -->
      <div class="form-card mb-6">
        <h3 class="section-title"><Type size={16} /> Content Editor</h3>
        <UnifiedDescriptionBuilder
          bind:descriptionAr={blog.content_html}
          bind:descriptionEn={blog.content_markdown}
          hideSeo={true}
        />
        <!-- We use content_html field to store the JSON blocks for simplicity, keeping the backend unchanged -->
        <input
          type="hidden"
          name="content_html"
          bind:value={blog.content_html}
        />
        <!-- Set a default markdown content to pass backend validation -->
        <input
          type="hidden"
          name="content_markdown"
          bind:value={blog.content_markdown}
        />
      </div>
    </div>

    <div class="side-column">
      <!-- Featured Image -->
      <div class="form-card mb-6">
        <h3 class="section-title"><ImageIcon size={16} /> Featured Image</h3>
        <div class="form-group">
          <label for="cover_image_upload">Cover Image</label>
          <input
            type="hidden"
            name="cover_image_url"
            bind:value={blog.cover_image_url}
          />
          <input
            type="file"
            id="cover_image_upload"
            accept="image/*"
            class="form-control mb-3"
            onchange={handleImageUpload}
            disabled={isUploadingImage}
          />
          {#if isUploadingImage}
            <div class="mb-3 text-sm text-[var(--color-primary)] font-medium">Uploading image...</div>
          {/if}
          {#if blog.cover_image_url}
            <div
              class="preview-image rounded-lg overflow-hidden border border-gray-200 shadow-sm"
              style="background-image: url('{resolveMediaUrl(blog.cover_image_url)}'); height: 160px; background-size: cover; background-position: center;"
            ></div>
          {:else}
            <div
              class="preview-image-placeholder bg-gray-50 border-2 border-dashed border-gray-300 rounded-lg h-40 flex items-center justify-center text-gray-400 text-sm"
            >
              No image uploaded
            </div>
          {/if}
        </div>
      </div>

      <div class="form-card mb-6">
        <h3 class="section-title"><Globe size={16} /> Publish Status</h3>

        <div class="toggle-switch-wrapper mb-3 flex items-center gap-3">
          <label class="toggle-switch relative inline-block w-11 h-6">
            <input
              type="checkbox"
              name="is_published"
              bind:checked={blog.is_published}
              value="true"
              class="opacity-0 w-0 h-0"
            />
            <span
              class="slider absolute cursor-pointer inset-0 bg-gray-300 transition-colors duration-300 rounded-full"
            ></span>
          </label>
          <span class="toggle-label font-medium"
            >{blog.is_published ? "Published" : "Draft (Hidden)"}</span
          >
        </div>

        {#if blog.is_published}
          <div class="form-group mb-3 fade-in mt-4">
            <label for="published_at" class="font-bold"
              >Publish Date (Schedule)</label
            >
            <input
              type="datetime-local"
              id="published_at"
              name="published_at"
              bind:value={localPublishedAt}
              class="form-control mt-1"
            />
            <small class="text-muted text-xs block mt-1"
              >If set in the future, post will be hidden until then.</small
            >
          </div>
        {/if}

        <button
          type="submit"
          class="submit-btn mt-6 w-full py-3 px-4 bg-purple-700 hover:bg-purple-800 text-white font-bold rounded-lg transition-colors flex items-center justify-center gap-2"
          disabled={isSaving}
        >
          <Save size={16} />
          {isSaving ? "Saving..." : "Save Post"}
        </button>
      </div>

      <div class="form-card mb-6">
        <h3 class="section-title"><Globe size={16} /> SEO Settings</h3>
        <div class="form-group mb-4">
          <label for="meta_title_en" class="font-bold"
            >Meta Title (English)</label
          >
          <input
            type="text"
            id="meta_title_en"
            name="meta_title_en"
            bind:value={blog.meta_title_en}
            class="form-control"
          />
          <small class="text-muted text-xs"
            >If empty, English post title is used.</small
          >
        </div>

        <div class="form-group mb-4" dir="rtl">
          <label for="meta_title_ar" class="font-bold font-arabic block mb-1"
            >عنوان الميتا (Arabic)</label
          >
          <input
            type="text"
            id="meta_title_ar"
            name="meta_title_ar"
            bind:value={blog.meta_title_ar}
            class="form-control text-right font-arabic"
          />
          <small class="text-muted text-xs text-left block" dir="ltr"
            >If empty, Arabic post title is used.</small
          >
        </div>

        <div class="form-group mb-4">
          <label for="meta_description_en" class="font-bold"
            >Meta Description (English)</label
          >
          <textarea
            id="meta_description_en"
            name="meta_description_en"
            bind:value={blog.meta_description_en}
            class="form-control"
            rows="3"
          ></textarea>
        </div>

        <div class="form-group mb-4" dir="rtl">
          <label
            for="meta_description_ar"
            class="font-bold font-arabic block mb-1">وصف الميتا (Arabic)</label
          >
          <textarea
            id="meta_description_ar"
            name="meta_description_ar"
            bind:value={blog.meta_description_ar}
            class="form-control text-right font-arabic"
            rows="3"
          ></textarea>
        </div>

        <div class="form-group mb-4">
          <label for="focus_keywords" class="font-bold">Focus Keywords</label>
          <input
            type="text"
            id="focus_keywords"
            name="focus_keywords"
            bind:value={blog.focus_keywords}
            class="form-control"
            placeholder="saudi weddings, riyadh venues..."
          />
        </div>

        <div class="form-group">
          <label for="read_time_minutes" class="font-bold"
            >Read Time (Minutes)</label
          >
          <input
            type="number"
            id="read_time_minutes"
            name="read_time_minutes"
            bind:value={blog.read_time_minutes}
            class="form-control"
            min="1"
          />
        </div>
      </div>

      <div class="form-card mb-6">
        <h3 class="section-title"><Globe size={16} /> Organization</h3>

        <div class="form-group mb-4">
          <span class="font-bold mb-2 block">Categories</span>
          <div
            class="checkbox-scroll-box border p-3 rounded bg-gray-50 max-h-40 overflow-y-auto"
          >
            {#each allCategories as cat}
              <label
                class="checkbox-label flex items-center gap-2 mb-2 cursor-pointer text-sm"
              >
                <input
                  type="checkbox"
                  name="categories"
                  value={cat.id}
                  checked={blog.categories.includes(cat.id)}
                  class="text-purple-600 rounded"
                />
                {cat.name}
              </label>
            {/each}
            {#if allCategories.length === 0}
              <small class="text-muted">No categories found.</small>
            {/if}
          </div>
        </div>

        <div class="form-group">
          <span class="font-bold mb-2 block">Tags</span>
          <div
            class="checkbox-scroll-box border p-3 rounded bg-gray-50 max-h-40 overflow-y-auto"
          >
            {#each allTags as tag}
              <label
                class="checkbox-label flex items-center gap-2 mb-2 cursor-pointer text-sm"
              >
                <input
                  type="checkbox"
                  name="tags"
                  value={tag.id}
                  checked={blog.tags.includes(tag.id)}
                  class="text-purple-600 rounded"
                />
                {tag.name}
              </label>
            {/each}
            {#if allTags.length === 0}
              <small class="text-muted">No tags found.</small>
            {/if}
          </div>
        </div>
      </div>
    </div>
  </form>
</div>

<style>
  .editor-layout {
    display: grid;
    grid-template-columns: 2fr 1fr;
    gap: 24px;
  }
  .section-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    font-weight: 700;
    margin-bottom: 16px;
    border-bottom: 1px solid var(--border);
    padding-bottom: 8px;
  }
  .form-control-lg {
    font-size: 18px;
    padding: 12px;
    font-weight: 600;
  }
  .input-with-prefix {
    display: flex;
    align-items: center;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 6px;
    overflow: hidden;
  }
  .prefix {
    padding: 0 12px;
    color: var(--text-tertiary);
    font-family: monospace;
    font-size: 13px;
    background: var(--bg-tertiary);
    border-right: 1px solid var(--border);
  }
  .input-with-prefix .form-control {
    border: none;
    border-radius: 0;
    box-shadow: none;
    font-family: monospace;
  }

  .toggle-switch-wrapper {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .toggle-switch {
    position: relative;
    display: inline-block;
    width: 44px;
    height: 24px;
  }
  .toggle-switch input {
    opacity: 0;
    width: 0;
    height: 0;
  }
  .slider {
    position: absolute;
    cursor: pointer;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: var(--border);
    transition: 0.3s;
    border-radius: 24px;
  }
  .slider:before {
    position: absolute;
    content: "";
    height: 18px;
    width: 18px;
    left: 3px;
    bottom: 3px;
    background-color: white;
    transition: 0.3s;
    border-radius: 50%;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  }
  input:checked + .slider {
    background-color: var(--success);
  }
  input:checked + .slider:before {
    transform: translateX(20px);
  }
  .toggle-label {
    font-size: 14px;
    font-weight: 600;
  }

  /* Google Snippet Card styling */
  .google-snippet-card {
    border: 1px solid #dadce0;
    border-radius: 12px;
    background: #ffffff;
    padding: 16px 20px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
    display: flex;
    flex-direction: column;
    gap: 16px;
    margin-top: 10px;
  }
  .snippet-lang-preview {
    display: flex;
    flex-direction: column;
    text-align: left;
  }
  .snippet-lang-preview.ar-preview {
    direction: rtl;
    text-align: right;
  }
  .snippet-meta-info {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 4px;
  }
  .snippet-favicon {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #f1f5f9;
    object-fit: contain;
  }
  .snippet-domain {
    font-size: 0.75rem;
    color: #202124;
    font-family: Arial, sans-serif;
  }
  .snippet-title {
    font-size: 1.1rem;
    font-weight: 500;
    color: #1a0dab;
    margin: 0 0 4px 0;
    font-family: Arial, sans-serif;
    line-height: 1.3;
  }
  .snippet-title:hover {
    text-decoration: underline;
    cursor: pointer;
  }
  .snippet-description {
    font-size: 0.85rem;
    color: #4d5156;
    margin: 0;
    line-height: 1.5;
    font-family: Arial, sans-serif;
    word-break: break-word;
  }
  .snippet-divider {
    height: 1px;
    background: #dadce0;
  }

  @media (max-width: 900px) {
    .editor-layout {
      grid-template-columns: 1fr;
    }
  }
</style>
