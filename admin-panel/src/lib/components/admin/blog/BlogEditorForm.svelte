<script lang="ts">
  import { enhance } from '$app/forms';
  import { page } from '$app/stores';
  import { ArrowLeft, Save, Globe, Type, Image as ImageIcon, CheckCircle, AlertCircle, Eye, EyeOff, Plus, Folder, Tag, Sparkles } from 'lucide-svelte';
  import { resolveMediaUrl } from '$lib/shared/utils/media';
  import { env } from '$env/dynamic/public';
  import WysiwygEditor from './WysiwygEditor.svelte';

  let {
    blog = $bindable(),
    allCategories = $bindable([]),
    allTags = $bindable([]),
    action = '?/save',
    isNew = false
  } = $props<{
    blog: any;
    allCategories: any[];
    allTags: any[];
    action?: string;
    isNew?: boolean;
  }>();

  // Active translation tab ('en' or 'ar')
  let activeLangTab = $state<'en' | 'ar'>('en');

  // Preview Mode ('edit' or 'preview')
  let currentMode = $state<'edit' | 'preview'>('edit');

  // Image Upload state
  let isDragging = $state(false);
  let isUploadingImage = $state(false);
  let uploadError = $state('');

  // Inline Category / Tag Creation State
  let newCatName = $state('');
  let isCreatingCat = $state(false);
  let newTagName = $state('');
  let isCreatingTag = $state(false);

  // Read Time Default Override flag
  let userOverrodeReadTime = $state(false);

  // Autosave status
  let lastAutosavedStr = $state('');
  let autosaveTimer = $state<any>(null);

  // Inline validation errors
  let validationErrors = $state<Record<string, string>>({});

  // Format published_at for datetime-local
  let localPublishedAt = $state('');
  if (blog.published_at) {
    const d = new Date(blog.published_at);
    localPublishedAt = new Date(d.getTime() - d.getTimezoneOffset() * 60000)
      .toISOString()
      .slice(0, 16);
  }

  // Auto-generate slug from English title
  function handleTitleInput() {
    if (isNew && !blog.slug && blog.title_en) {
      blog.slug = blog.title_en
        .toLowerCase()
        .replace(/[^a-z0-9]+/g, '-')
        .replace(/(^-|-$)+/g, '');
    }
  }

  // HTML to Markdown converter
  function convertHtmlToMarkdown(html: string): string {
    if (!html) return '';
    let md = html;
    md = md.replace(/<strong[^>]*>(.*?)<\/strong>/gi, '**$1**');
    md = md.replace(/<b[^>]*>(.*?)<\/b>/gi, '**$1**');
    md = md.replace(/<em[^>]*>(.*?)<\/em>/gi, '*$1*');
    md = md.replace(/<i[^>]*>(.*?)<\/i>/gi, '*$1*');
    md = md.replace(/<u[^>]*>(.*?)<\/u>/gi, '<u>$1</u>');
    md = md.replace(/<h1[^>]*>(.*?)<\/h1>/gi, '# $1\n\n');
    md = md.replace(/<h2[^>]*>(.*?)<\/h2>/gi, '## $1\n\n');
    md = md.replace(/<h3[^>]*>(.*?)<\/h3>/gi, '### $1\n\n');
    md = md.replace(/<a\s+(?:[^>]*?\s+)?href="([^"]*)"[^>]*>(.*?)<\/a>/gi, '[$2]($1)');
    md = md.replace(/<ul[^>]*>(.*?)<\/ul>/gi, '$1\n');
    md = md.replace(/<ol[^>]*>(.*?)<\/ol>/gi, '$1\n');
    md = md.replace(/<li[^>]*>(.*?)<\/li>/gi, '- $1\n');
    md = md.replace(/<p[^>]*>(.*?)<\/p>/gi, '$1\n\n');
    md = md.replace(/<div[^>]*>(.*?)<\/div>/gi, '$1\n');
    md = md.replace(/<br\s*\/?>/gi, '\n');
    md = md.replace(/<[^>]+>/g, '');
    
    // Decode basic entities
    md = md.replace(/&nbsp;/g, ' ')
           .replace(/&amp;/g, '&')
           .replace(/&lt;/g, '<')
           .replace(/&gt;/g, '>')
           .replace(/&quot;/g, '"');
           
    return md.replace(/\n{3,}/g, '\n\n').trim();
  }

  // Dynamic Word Count and Read Time calculation
  $effect(() => {
    const rawContent = (blog.content_html || '');
    // Strip HTML to count words
    const strippedText = rawContent.replace(/<[^>]+>/g, ' ').trim();
    const wordCount = strippedText ? strippedText.split(/\s+/).filter(Boolean).length : 0;
    
    if (!userOverrodeReadTime) {
      blog.read_time_minutes = Math.max(1, Math.round(wordCount / 200));
    }
    
    // Auto-update markdown
    blog.content_markdown = convertHtmlToMarkdown(rawContent);
  });

  // Drag and Drop Upload Handlers
  function onDragOver(e: DragEvent) {
    e.preventDefault();
    isDragging = true;
  }

  function onDragLeave() {
    isDragging = false;
  }

  async function uploadFile(file: File) {
    if (!file.type.startsWith('image/')) {
      uploadError = 'Please upload a valid image file.';
      return;
    }
    
    const formData = new FormData();
    formData.append('file', file);

    isUploadingImage = true;
    uploadError = '';
    
    try {
      const apiBase = env.PUBLIC_API_URL && env.PUBLIC_API_URL.includes("localhost") ? "" : env.PUBLIC_API_URL || "";
      const response = await fetch(`${apiBase}/api/v1/admin/blogs/upload`, {
        method: 'POST',
        body: formData
      });
      const result = await response.json();
      if (result.status === 'success') {
        blog.cover_image_url = result.url;
      } else {
        uploadError = result.message || 'Image upload failed.';
      }
    } catch (e) {
      console.error(e);
      uploadError = 'Connection failed during image upload.';
    } finally {
      isUploadingImage = false;
      isDragging = false;
    }
  }

  function onDrop(e: DragEvent) {
    e.preventDefault();
    isDragging = false;
    if (e.dataTransfer?.files && e.dataTransfer.files.length > 0) {
      uploadFile(e.dataTransfer.files[0]);
    }
  }

  function onFileInput(e: Event) {
    const input = e.target as HTMLInputElement;
    if (input.files && input.files.length > 0) {
      uploadFile(input.files[0]);
    }
  }

  // Inline Category / Tag creation
  async function createCategoryInline() {
    if (!newCatName.trim()) return;
    isCreatingCat = true;
    try {
      const apiBase = env.PUBLIC_API_URL && env.PUBLIC_API_URL.includes("localhost") ? "" : env.PUBLIC_API_URL || "";
      const slug = newCatName.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)+/g, '');
      const sessionToken = document.cookie.match(/zafaf_admin_session=([^;]+)/)?.[1] || '';

      const res = await fetch(`${apiBase}/api/v1/admin/categories`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${sessionToken}`
        },
        body: JSON.stringify({ name: newCatName.trim(), slug })
      });
      
      if (res.ok) {
        // Refetch or append locally
        const categoriesRes = await fetch(`${apiBase}/api/v1/admin/categories`, {
          headers: { 'Authorization': `Bearer ${sessionToken}` }
        });
        const categoriesData = await categoriesRes.json();
        if (categoriesData.status === 'success') {
          allCategories = categoriesData.data || [];
          // Automatically check the newly created category
          const newCat = allCategories.find(c => c.name === newCatName.trim() || c.slug === slug);
          if (newCat && !blog.categories.includes(newCat.id)) {
            blog.categories = [...blog.categories, newCat.id];
          }
        }
        newCatName = '';
      }
    } catch (e) {
      console.error(e);
    } finally {
      isCreatingCat = false;
    }
  }

  async function createTagInline() {
    if (!newTagName.trim()) return;
    isCreatingTag = true;
    try {
      const apiBase = env.PUBLIC_API_URL && env.PUBLIC_API_URL.includes("localhost") ? "" : env.PUBLIC_API_URL || "";
      const slug = newTagName.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)+/g, '');
      const sessionToken = document.cookie.match(/zafaf_admin_session=([^;]+)/)?.[1] || '';

      const res = await fetch(`${apiBase}/api/v1/admin/tags`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${sessionToken}`
        },
        body: JSON.stringify({ name: newTagName.trim(), slug })
      });
      
      if (res.ok) {
        const tagsRes = await fetch(`${apiBase}/api/v1/admin/tags`, {
          headers: { 'Authorization': `Bearer ${sessionToken}` }
        });
        const tagsData = await tagsRes.json();
        if (tagsData.status === 'success') {
          allTags = tagsData.data || [];
          const newTag = allTags.find(t => t.name === newTagName.trim() || t.slug === slug);
          if (newTag && !blog.tags.includes(newTag.id)) {
            blog.tags = [...blog.tags, newTag.id];
          }
        }
        newTagName = '';
      }
    } catch (e) {
      console.error(e);
    } finally {
      isCreatingTag = false;
    }
  }

  // Handle Category/Tag checkbox toggling
  function toggleCategory(catId: string) {
    if (blog.categories.includes(catId)) {
      blog.categories = blog.categories.filter((id: string) => id !== catId);
    } else {
      blog.categories = [...blog.categories, catId];
    }
  }

  function toggleTag(tagId: string) {
    if (blog.tags.includes(tagId)) {
      blog.tags = blog.tags.filter((id: string) => id !== tagId);
    } else {
      blog.tags = [...blog.tags, tagId];
    }
  }

  // Plain validation
  function validateForm(): boolean {
    const errors: Record<string, string> = {};
    if (!blog.title_en?.trim()) errors.title_en = 'English title is required';
    if (!blog.title_ar?.trim()) errors.title_ar = 'Arabic title is required';
    if (!blog.slug?.trim()) errors.slug = 'Slug URL path is required';
    if (!blog.content_html?.trim() || blog.content_html === '<br>') errors.content = 'Blog post content is required';

    validationErrors = errors;
    return Object.keys(errors).length === 0;
  }

  // Autosave triggers every 30s
  function startAutosave() {
    autosaveTimer = setInterval(async () => {
      // Validate minimal fields before autosaving to backend
      if (!blog.title_en?.trim() && !blog.title_ar?.trim()) return;

      if (!blog.id) {
        // If new, save locally
        localStorage.setItem('zafaf_blog_draft_new', JSON.stringify({
          ...blog,
          localPublishedAt
        }));
        const now = new Date();
        lastAutosavedStr = `Draft saved locally at ${now.toLocaleTimeString()}`;
      } else {
        // If existing post, PUT to backend
        try {
          const apiBase = env.PUBLIC_API_URL && env.PUBLIC_API_URL.includes("localhost") ? "" : env.PUBLIC_API_URL || "";
          const sessionToken = document.cookie.match(/zafaf_admin_session=([^;]+)/)?.[1] || '';
          
          const payload = {
            slug: blog.slug,
            title: blog.title_en || blog.title_ar || 'Untitled',
            title_ar: blog.title_ar,
            title_en: blog.title_en,
            excerpt: blog.excerpt,
            content_html: blog.content_html,
            content_markdown: blog.content_markdown,
            cover_image_url: blog.cover_image_url,
            meta_title: blog.meta_title_en || blog.meta_title_ar,
            meta_title_ar: blog.meta_title_ar,
            meta_title_en: blog.meta_title_en,
            meta_description: blog.meta_description_en || blog.meta_description_ar,
            meta_description_ar: blog.meta_description_ar,
            meta_description_en: blog.meta_description_en,
            focus_keywords: blog.focus_keywords,
            read_time_minutes: blog.read_time_minutes,
            is_published: blog.is_published,
            published_at: localPublishedAt ? new Date(localPublishedAt).toISOString() : null,
            categories: blog.categories,
            tags: blog.tags,
            canonical_url: blog.canonical_url
          };

          const res = await fetch(`${apiBase}/api/v1/admin/blogs/${blog.id}`, {
            method: 'PUT',
            headers: {
              'Content-Type': 'application/json',
              'Authorization': `Bearer ${sessionToken}`
            },
            body: JSON.stringify(payload)
          });
          if (res.ok) {
            const now = new Date();
            lastAutosavedStr = `Draft autosaved at ${now.toLocaleTimeString()}`;
          }
        } catch (e) {
          console.error('Autosave failed:', e);
        }
      }
    }, 30000);
  }

  // Restore draft if exists on mount
  onMount(() => {
    if (isNew) {
      const saved = localStorage.getItem('zafaf_blog_draft_new');
      if (saved) {
        try {
          const parsed = JSON.parse(saved);
          if (confirm('A newer draft was found in your browser. Would you like to restore it?')) {
            Object.assign(blog, parsed);
            if (parsed.localPublishedAt) localPublishedAt = parsed.localPublishedAt;
          }
        } catch (e) {
          console.error('Failed to parse saved draft:', e);
        }
      }
    }
    startAutosave();
    return () => {
      if (autosaveTimer) clearInterval(autosaveTimer);
    };
  });

  // Clear local draft upon successful submit
  function handleFormSubmit(e: Event) {
    if (!validateForm()) {
      e.preventDefault();
      // Scroll to first error
      setTimeout(() => {
        const firstErr = document.querySelector('.input-error');
        firstErr?.scrollIntoView({ behavior: 'smooth', block: 'center' });
      }, 50);
    } else {
      if (isNew) localStorage.removeItem('zafaf_blog_draft_new');
    }
  }
</script>

<!-- Live Preview Styles -->
<style>
  .font-arabic {
    font-family: 'Noto Sans Arabic', sans-serif !important;
  }
  .editor-layout {
    display: grid;
    grid-template-columns: 2.2fr 1fr;
    gap: 24px;
    align-items: start;
  }
  .input-error {
    border-color: var(--danger) !important;
    box-shadow: 0 0 0 3px var(--danger-glow) !important;
  }
  .error-message {
    color: var(--danger);
    font-size: 0.8rem;
    margin-top: 4px;
    font-weight: 500;
  }
  .dropzone {
    border: 2px dashed #cbd5e1;
    transition: all 0.2s;
  }
  .dropzone.dragging {
    border-color: var(--color-primary);
    background-color: var(--purple-dim);
  }
  
  @media (max-width: 900px) {
    .editor-layout {
      grid-template-columns: 1fr;
    }
  }
</style>

<div class="editor-shell max-w-6xl mx-auto px-4 pb-12">
  <!-- Top Bar with Actions & Title -->
  <div class="flex flex-col md:flex-row justify-between items-start md:items-center gap-4 mb-8">
    <div class="flex items-center gap-2">
      <a href="/dashboard/blog" class="btn btn-outline btn-sm">
        <ArrowLeft size={16} />
      </a>
      <div>
        <h1 class="page-title">{isNew ? 'Create Bilingual Blog Post' : 'Edit Blog Post'}</h1>
        {#if lastAutosavedStr}
          <p class="text-xs text-[var(--color-primary)] font-semibold mt-1 flex items-center gap-1">
            <Sparkles size={12} />
            {lastAutosavedStr}
          </p>
        {/if}
      </div>
    </div>

    <!-- Toggle Live Preview -->
    <div class="flex items-center gap-3 w-full md:w-auto">
      <button 
        type="button" 
        class="btn btn-outline flex-1 md:flex-initial flex items-center justify-center gap-2"
        onclick={() => currentMode = currentMode === 'edit' ? 'preview' : 'edit'}
      >
        {#if currentMode === 'edit'}
          <Eye size={16} /> Live Preview
        {:else}
          <EyeOff size={16} /> Back to Editor
        {/if}
      </button>
    </div>
  </div>

  {#if currentMode === 'preview'}
    <!-- Live Blog Post Preview Mode -->
    <div class="preview-mode bg-white border border-slate-200 rounded-2xl p-6 md:p-10 shadow-sm">
      <div class="flex justify-between items-center border-b pb-4 mb-6">
        <h2 class="text-lg font-bold text-slate-800">Preview Site Post View</h2>
        <div class="flex gap-2">
          <button 
            type="button" 
            class="px-3 py-1 text-xs font-semibold rounded-lg {activeLangTab === 'en' ? 'bg-slate-800 text-white' : 'bg-slate-100 text-slate-700'}"
            onclick={() => activeLangTab = 'en'}
          >
            English
          </button>
          <button 
            type="button" 
            class="px-3 py-1 text-xs font-semibold rounded-lg {activeLangTab === 'ar' ? 'bg-slate-800 text-white' : 'bg-slate-100 text-slate-700'}"
            onclick={() => activeLangTab = 'ar'}
          >
            العربية
          </button>
        </div>
      </div>

      {#if activeLangTab === 'en'}
        <!-- English View -->
        <article class="prose prose-slate max-w-none">
          {#if blog.cover_image_url}
            <img src={resolveMediaUrl(blog.cover_image_url)} alt="Cover" class="w-full h-80 object-cover rounded-xl mb-6" />
          {/if}
          <h1 class="text-3xl font-extrabold text-slate-900 mb-2">{blog.title_en || 'Untitled Post'}</h1>
          <div class="flex items-center gap-4 text-sm text-slate-500 mb-6">
            <span>Read Time: {blog.read_time_minutes} min</span>
            {#if blog.is_published}
              <span class="text-green-600 font-semibold">Published</span>
            {:else}
              <span class="text-amber-600 font-semibold">Draft</span>
            {/if}
          </div>
          <div class="text-slate-800 leading-relaxed font-serif">
            {@html blog.content_html || '<p class="text-slate-400">No content written yet.</p>'}
          </div>
        </article>
      {:else}
        <!-- Arabic View -->
        <article class="prose prose-slate max-w-none text-right font-arabic" dir="rtl">
          {#if blog.cover_image_url}
            <img src={resolveMediaUrl(blog.cover_image_url)} alt="Cover" class="w-full h-80 object-cover rounded-xl mb-6" />
          {/if}
          <h1 class="text-3xl font-extrabold text-slate-900 mb-2">{blog.title_ar || 'مقال بدون عنوان'}</h1>
          <div class="flex items-center gap-4 text-sm text-slate-500 mb-6 justify-start">
            <span>وقت القراءة: {blog.read_time_minutes} دقيقة</span>
            {#if blog.is_published}
              <span class="text-green-600 font-semibold">منشور</span>
            {:else}
              <span class="text-amber-600 font-semibold">مسودة</span>
            {/if}
          </div>
          <div class="text-slate-800 leading-relaxed font-arabic">
            {@html blog.content_html || '<p class="text-slate-400">لا يوجد محتوى مكتوب بعد.</p>'}
          </div>
        </article>
      {/if}
    </div>
  {:else}
    <!-- Edit Form View -->
    <form
      method="POST"
      {action}
      onsubmit={handleFormSubmit}
      use:enhance={() => {
        return async ({ update }) => {
          await update();
        };
      }}
      class="editor-layout"
    >
      <!-- Hidden inputs matching payload -->
      <input type="hidden" name="content_html" bind:value={blog.content_html} />
      <input type="hidden" name="content_markdown" bind:value={blog.content_markdown} />

      <div class="flex flex-col gap-6">
        <!-- Bilingual tab switcher -->
        <div class="form-card">
          <div class="flex justify-between items-center border-b pb-3 mb-4">
            <h2 class="section-title text-sm font-bold flex items-center gap-2 m-0 border-b-0 pb-0">
              <Type size={16} /> Content Fields
            </h2>
            <div class="flex gap-2">
              <button
                type="button"
                class="px-4 py-1.5 text-xs font-semibold rounded-lg transition-colors {activeLangTab === 'en' ? 'bg-[var(--color-primary)] text-white' : 'bg-slate-100 text-slate-700'}"
                onclick={() => activeLangTab = 'en'}
              >
                🇬🇧 English Input
              </button>
              <button
                type="button"
                class="px-4 py-1.5 text-xs font-semibold rounded-lg transition-colors font-arabic {activeLangTab === 'ar' ? 'bg-[var(--color-primary)] text-white' : 'bg-slate-100 text-slate-700'}"
                onclick={() => activeLangTab = 'ar'}
              >
                🇸🇦 المحتوى العربي
              </button>
            </div>
          </div>

          <!-- ENGLISH INPUT FIELDS -->
          <div class={activeLangTab === 'en' ? 'block' : 'hidden'}>
            <div class="form-group mb-4">
              <label for="title_en_input" class="font-bold block mb-1">Title (English)</label>
              <input
                type="text"
                id="title_en_input"
                name="title_en"
                bind:value={blog.title_en}
                oninput={handleTitleInput}
                class="base-input w-full {validationErrors.title_en ? 'input-error' : ''}"
                placeholder="Enter English Title..."
              />
              {#if validationErrors.title_en}
                <div class="error-message">{validationErrors.title_en}</div>
              {/if}
              <!-- Synced legacy title -->
              <input type="hidden" name="title" value={blog.title_en || blog.title_ar || ''} />
            </div>

            <div class="form-group mb-4">
              <label for="meta_title_en_input" class="font-bold block mb-1">Meta Title (English)</label>
              <input
                type="text"
                id="meta_title_en_input"
                name="meta_title_en"
                bind:value={blog.meta_title_en}
                class="base-input w-full"
                placeholder="Google search results title..."
              />
              <input type="hidden" name="meta_title" value={blog.meta_title_en || ''} />
            </div>

            <div class="form-group mb-4">
              <label for="meta_desc_en_input" class="font-bold block mb-1">Meta Description (English)</label>
              <textarea
                id="meta_desc_en_input"
                name="meta_description_en"
                bind:value={blog.meta_description_en}
                class="base-input w-full"
                rows="3"
                placeholder="Google search snippet description..."
              ></textarea>
              <input type="hidden" name="meta_description" value={blog.meta_description_en || ''} />
            </div>
          </div>

          <!-- ARABIC INPUT FIELDS -->
          <div class={activeLangTab === 'ar' ? 'block' : 'hidden'} dir="rtl">
            <div class="form-group mb-4 text-right">
              <label for="title_ar_input" class="font-bold font-arabic block mb-1 text-right">العنوان (العربية)</label>
              <input
                type="text"
                id="title_ar_input"
                name="title_ar"
                bind:value={blog.title_ar}
                class="base-input w-full font-arabic text-right {validationErrors.title_ar ? 'input-error' : ''}"
                placeholder="أدخل عنوان المقالة باللغة العربية..."
              />
              {#if validationErrors.title_ar}
                <div class="error-message text-right">{validationErrors.title_ar}</div>
              {/if}
            </div>

            <div class="form-group mb-4 text-right">
              <label for="meta_title_ar_input" class="font-bold font-arabic block mb-1 text-right">عنوان الميتا للبحث (Meta Title)</label>
              <input
                type="text"
                id="meta_title_ar_input"
                name="meta_title_ar"
                bind:value={blog.meta_title_ar}
                class="base-input w-full font-arabic text-right"
                placeholder="عنوان البحث في جوجل..."
              />
            </div>

            <div class="form-group mb-4 text-right">
              <label for="meta_desc_ar_input" class="font-bold font-arabic block mb-1 text-right">وصف الميتا للبحث (Meta Description)</label>
              <textarea
                id="meta_desc_ar_input"
                name="meta_description_ar"
                bind:value={blog.meta_description_ar}
                class="base-input w-full font-arabic text-right"
                rows="3"
                placeholder="وصف المقال في نتائج بحث جوجل..."
              ></textarea>
            </div>
          </div>
        </div>

        <!-- HTML Rich Content Editor Card -->
        <div class="form-card">
          <h2 class="section-title text-sm font-bold mb-4 flex items-center gap-2">
            <Type size={16} /> Rich Content Editor (Bilingual Content)
          </h2>
          
          {#if validationErrors.content}
            <div class="notice-banner error mb-4">
              <AlertCircle size={16} />
              <div class="notice-text">{validationErrors.content}</div>
            </div>
          {/if}

          <WysiwygEditor 
            bind:value={blog.content_html} 
            placeholder="Write your article rich text content here..."
            direction={activeLangTab}
          />
        </div>

        <!-- Google Snippet Preview Grid -->
        <div class="form-card">
          <h2 class="section-title text-sm font-bold mb-4 flex items-center gap-2">
            <Globe size={16} /> Google Search Preview
          </h2>
          <div class="google-snippet-card">
            <div class="snippet-lang-preview">
              <div class="snippet-meta-info">
                <img src="/favicon.webp" alt="" class="snippet-favicon" onerror={(e) => e.currentTarget.style.display="none"} />
                <span class="snippet-domain">https://zafafworld.net › blog › {blog.slug || 'new-post'}</span>
              </div>
              <h4 class="snippet-title">{blog.meta_title_en || blog.title_en || 'New Post'} | ZafafWorld</h4>
              <p class="snippet-description">{blog.meta_description_en || 'Add blog details to preview search results snippet...'}</p>
            </div>
            <div class="snippet-divider"></div>
            <div class="snippet-lang-preview ar-preview" dir="rtl">
              <div class="snippet-meta-info">
                <img src="/favicon.webp" alt="" class="snippet-favicon" onerror={(e) => e.currentTarget.style.display="none"} />
                <span class="snippet-domain">https://zafafworld.net › blog › {blog.slug || 'new-post'}</span>
              </div>
              <h4 class="snippet-title font-arabic">{blog.meta_title_ar || blog.title_ar || 'منشور جديد'} | زفاف وورلد</h4>
              <p class="snippet-description font-arabic">{blog.meta_description_ar || 'أضف تفاصيل المقال لعرض معاينة البحث...'}</p>
            </div>
          </div>
        </div>
      </div>

      <!-- Side Column: Organization & Publishing -->
      <div class="flex flex-col gap-6">
        <!-- Publishing Control -->
        <div class="form-card">
          <h2 class="section-title text-sm font-bold mb-4"><Globe size={16} /> Publish & Schedule</h2>
          
          <div class="toggle-switch-wrapper mb-4 flex items-center gap-3">
            <label class="toggle-switch relative inline-block w-11 h-6">
              <input
                type="checkbox"
                name="is_published"
                bind:checked={blog.is_published}
                value="true"
                class="opacity-0 w-0 h-0"
              />
              <span class="slider absolute cursor-pointer inset-0 bg-gray-300 transition-colors duration-300 rounded-full"></span>
            </label>
            <span class="toggle-label font-bold text-sm">
              {blog.is_published ? 'Published (Public)' : 'Draft (Hidden)'}
            </span>
          </div>

          {#if blog.is_published}
            <div class="form-group mb-4">
              <label for="published_at_input" class="font-bold block mb-1">Schedule Date (Optional)</label>
              <input
                type="datetime-local"
                id="published_at_input"
                name="published_at"
                bind:value={localPublishedAt}
                class="base-input w-full"
              />
              <small class="text-slate-400 text-xs mt-1 block">Leave empty to publish immediately, or set a future date to schedule.</small>
            </div>
          {/if}

          <div class="form-group mb-4">
            <label for="read_time_input" class="font-bold block mb-1">Read Time (Minutes)</label>
            <input
              type="number"
              id="read_time_input"
              name="read_time_minutes"
              bind:value={blog.read_time_minutes}
              oninput={() => userOverrodeReadTime = true}
              class="base-input w-full"
              min="1"
            />
            <small class="text-slate-400 text-xs mt-1 block">Default automatically calculates based on text length.</small>
          </div>

          <!-- Single obvious Publish Button -->
          <button
            type="submit"
            class="btn btn-gold w-full py-3 px-4 flex items-center justify-center gap-2 font-bold"
          >
            <Save size={18} />
            {isNew ? 'Publish Post' : 'Save Modifications'}
          </button>
        </div>

        <!-- Slug URL / Cover Image -->
        <div class="form-card">
          <h2 class="section-title text-sm font-bold mb-4"><ImageIcon size={16} /> Cover & URL Settings</h2>

          <div class="form-group mb-4">
            <label for="slug_input" class="font-bold block mb-1">Slug URL Path</label>
            <input
              type="text"
              id="slug_input"
              name="slug"
              bind:value={blog.slug}
              class="base-input w-full font-mono text-sm {validationErrors.slug ? 'input-error' : ''}"
              placeholder="post-slug-path"
            />
            {#if validationErrors.slug}
              <div class="error-message">{validationErrors.slug}</div>
            {/if}
          </div>
          
          <div class="form-group mb-4">
            <label for="canonical_url_input" class="font-bold block mb-1">Canonical URL</label>
            <input
              type="url"
              id="canonical_url_input"
              name="canonical_url"
              bind:value={blog.canonical_url}
              class="base-input w-full text-sm"
              placeholder="https://example.com/canonical-source-page"
            />
            <small class="text-slate-400 text-xs mt-1 block">The primary source URL of the article, to prevent duplicate content SEO issues.</small>
          </div>

          <div class="form-group mb-2">
            <label for="cover_image_input" class="font-bold block mb-2">Featured Cover Image</label>
            <input type="hidden" name="cover_image_url" bind:value={blog.cover_image_url} />
            
            <div
              class="dropzone flex flex-col items-center justify-center p-6 bg-slate-50 rounded-xl cursor-pointer text-center relative overflow-hidden min-h-[160px]"
              class:dragging={isDragging}
              ondragover={onDragOver}
              ondragleave={onDragLeave}
              ondrop={onDrop}
              onclick={() => document.getElementById('cover_image_file_input')?.click()}
            >
              <input
                type="file"
                id="cover_image_file_input"
                accept="image/*"
                class="hidden"
                onchange={onFileInput}
              />
              
              {#if isUploadingImage}
                <div class="text-sm font-semibold text-[var(--color-primary)]">Uploading Image...</div>
              {:else if blog.cover_image_url}
                <img 
                  src={resolveMediaUrl(blog.cover_image_url)} 
                  alt="Cover Preview" 
                  class="absolute inset-0 w-full h-full object-cover" 
                />
                <!-- Hover remove button -->
                <div class="absolute inset-0 bg-black/40 opacity-0 hover:opacity-100 transition-opacity flex items-center justify-center">
                  <span class="text-white text-xs font-bold bg-red-600 px-3 py-1.5 rounded-lg">Change Image</span>
                </div>
              {:else}
                <ImageIcon size={32} class="text-slate-400 mb-2" />
                <span class="text-sm font-semibold text-slate-600 block">Drag & drop cover image, or click to upload</span>
                <span class="text-xs text-slate-400 mt-1 block">JPG, PNG, WEBP — up to 10MB</span>
              {/if}
            </div>
            
            {#if uploadError}
              <div class="error-message mt-2">{uploadError}</div>
            {/if}
          </div>

          <div class="form-group">
            <label for="keywords_input" class="font-bold block mb-1">Focus Keywords</label>
            <input
              type="text"
              id="keywords_input"
              name="focus_keywords"
              bind:value={blog.focus_keywords}
              class="base-input w-full"
              placeholder="e.g. saudi weddings, venues"
            />
            <small class="text-slate-400 text-xs mt-1 block">Separate terms using commas.</small>
          </div>

          <div class="form-group mt-4">
            <label for="excerpt_input" class="font-bold block mb-1">Excerpt / Summary</label>
            <textarea
              id="excerpt_input"
              name="excerpt"
              bind:value={blog.excerpt}
              class="base-input w-full text-sm"
              rows="3"
              placeholder="Short summary of the blog post..."
            ></textarea>
          </div>
        </div>

        <!-- Organization (Categories & Tags) -->
        <div class="form-card">
          <h2 class="section-title text-sm font-bold mb-4 flex items-center gap-2">
            <Folder size={16} /> Organization
          </h2>

          <!-- Categories Select box with inline creation -->
          <div class="form-group mb-4">
            <span class="font-bold mb-2 block">Categories</span>
            <div class="border p-3 rounded-lg bg-slate-50 max-h-40 overflow-y-auto mb-2 flex flex-col gap-2">
              {#each allCategories as cat (cat.id)}
                <label class="flex items-center gap-2 cursor-pointer text-sm font-medium">
                  <input
                    type="checkbox"
                    name="categories"
                    value={cat.id}
                    checked={blog.categories.includes(cat.id)}
                    onchange={() => toggleCategory(cat.id)}
                    class="rounded text-[var(--color-primary)] focus:ring-[var(--color-primary)]"
                  />
                  {cat.name}
                </label>
              {:else}
                <span class="text-xs text-slate-400">No categories found.</span>
              {/each}
            </div>

            <!-- Inline add Category -->
            <div class="flex gap-2">
              <input
                type="text"
                bind:value={newCatName}
                placeholder="New category..."
                class="base-input text-xs py-1 px-2 flex-1"
                disabled={isCreatingCat}
              />
              <button
                type="button"
                class="btn btn-outline btn-xs flex items-center gap-1"
                onclick={createCategoryInline}
                disabled={isCreatingCat || !newCatName.trim()}
              >
                <Plus size={12} /> Add
              </button>
            </div>
          </div>

          <!-- Tags Select box with inline creation -->
          <div class="form-group">
            <span class="font-bold mb-2 block">Tags</span>
            <div class="border p-3 rounded-lg bg-slate-50 max-h-40 overflow-y-auto mb-2 flex flex-col gap-2">
              {#each allTags as tag (tag.id)}
                <label class="flex items-center gap-2 cursor-pointer text-sm font-medium">
                  <input
                    type="checkbox"
                    name="tags"
                    value={tag.id}
                    checked={blog.tags.includes(tag.id)}
                    onchange={() => toggleTag(tag.id)}
                    class="rounded text-[var(--color-primary)] focus:ring-[var(--color-primary)]"
                  />
                  {tag.name}
                </label>
              {:else}
                <span class="text-xs text-slate-400">No tags found.</span>
              {/each}
            </div>

            <!-- Inline add Tag -->
            <div class="flex gap-2">
              <input
                type="text"
                bind:value={newTagName}
                placeholder="New tag..."
                class="base-input text-xs py-1 px-2 flex-1"
                disabled={isCreatingTag}
              />
              <button
                type="button"
                class="btn btn-outline btn-xs flex items-center gap-1"
                onclick={createTagInline}
                disabled={isCreatingTag || !newTagName.trim()}
              >
                <Plus size={12} /> Add
              </button>
            </div>
          </div>
        </div>
      </div>
    </form>
  {/if}
</div>
