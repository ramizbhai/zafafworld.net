<script lang="ts">
  import { t, lang } from '$lib/i18n/index.js';
  import { Newspaper, Plus, Edit, Trash2, Globe, AlertCircle, Search, X, CheckCircle2 } from 'lucide-svelte';
  import { goto, invalidateAll } from '$app/navigation';
  import { enhance } from '$app/forms';

  interface Props {
    data: {
      articles: any[];
      total: number;
      page: number;
      totalPages: number;
      search: string;
      published: string;
      error?: string;
    };
    form?: { success?: boolean; error?: string; message?: string };
  }

  let { data, form }: Props = $props();

  // Local state synced from page data
  let searchInput = $state('');
  let publishFilter = $state('all');

  // Modal State
  let showModal = $state(false);
  let isEditing = $state(false);
  let selectedArticle = $state<any>(null);

  // Form inputs for create/edit
  let formSlug = $state('');
  let formCategory = $state('wedding-tips');
  let formTitleAr = $state('');
  let formTitleEn = $state('');
  let formSummaryAr = $state('');
  let formSummaryEn = $state('');
  let formBodyAr = $state('');
  let formBodyEn = $state('');
  let formPublished = $state(true);

  function openCreateModal() {
    isEditing = false;
    selectedArticle = null;
    formSlug = '';
    formCategory = 'wedding-tips';
    formTitleAr = '';
    formTitleEn = '';
    formSummaryAr = '';
    formSummaryEn = '';
    formBodyAr = '';
    formBodyEn = '';
    formPublished = true;
    showModal = true;
  }

  function openEditModal(art: any) {
    isEditing = true;
    selectedArticle = art;
    formSlug = art.slug || '';
    formCategory = art.category || 'wedding-tips';
    formTitleAr = art.titleAr || '';
    formTitleEn = art.titleEn || '';
    formSummaryAr = art.summaryAr || '';
    formSummaryEn = art.summaryEn || '';
    formBodyAr = art.bodyAr || '';
    formBodyEn = art.bodyEn || '';
    formPublished = art.published ?? true;
    showModal = true;
  }

  function closeModal() {
    showModal = false;
  }

  // Sync state if url changes externally
  $effect(() => {
    searchInput = data.search || '';
    publishFilter = data.published || 'all';
  });

  function applyFilters(page: number = 1) {
    const params = new URLSearchParams();
    if (searchInput.trim()) {
      params.set('search', searchInput.trim());
    }
    if (publishFilter && publishFilter !== 'all') {
      params.set('published', publishFilter);
    }
    params.set('page', String(page));
    goto(`/dashboard/cms?${params.toString()}`, { keepFocus: true, noScroll: true });
  }

  function handleFilterSubmit(e: Event) {
    e.preventDefault();
    applyFilters(1);
  }

  function handlePageChange(newPage: number) {
    applyFilters(newPage);
  }

  function statusCls(s: boolean) {
    if (s) return 'badge badge-dot badge-success';
    return 'badge badge-dot badge-warning';
  }

  function statusLbl(s: boolean) {
    return s 
      ? ($lang === 'ar' ? 'منشور' : 'Published') 
      : ($lang === 'ar' ? 'مسودة' : 'Draft');
  }

  function typeCls(t: string) {
    if (t === 'article' || t === 'CMS' || t === 'wedding-tips') return 'badge badge-purple';
    if (t === 'legal') return 'badge badge-muted';
    return 'badge badge-info';
  }

  function typeLabel(t: string) {
    const m: Record<string, string> = {
      page: $lang === 'ar' ? 'صفحة' : 'Page',
      article: $lang === 'ar' ? 'مقال' : 'Article',
      'wedding-tips': $lang === 'ar' ? 'نصائح زفاف' : 'Wedding Tips',
      legal: $lang === 'ar' ? 'قانوني' : 'Legal'
    };
    return m[t] ?? t;
  }

  function formatDate(d: string) {
    if (!d) return '-';
    return new Date(d).toLocaleDateString($lang === 'ar' ? 'ar-SA' : 'en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric'
    });
  }
</script>

<div class="fade-in">
  <div class="page-header">
    <div class="page-header-left">
      <h1 class="page-title">{$t('nav.cms')}</h1>
      <p class="page-subtitle">{$lang === 'ar' ? 'إدارة محتوى الموقع والمقالات والصفحات' : 'Manage website content, articles, and pages'}</p>
    </div>
    <button class="btn btn-gold btn-sm" onclick={openCreateModal}>
      <Plus size={14} /> 
      {$lang === 'ar' ? 'محتوى جديد' : 'New Content'}
    </button>
  </div>

  {#if data.error || form?.error}
    <div class="notice-banner error">
      <AlertCircle size={18} class="notice-icon" />
      <div class="notice-text">{data.error || form?.error}</div>
    </div>
  {/if}

  {#if form?.success}
    <div class="notice-banner success" style="background:var(--success-dim); color:var(--success); padding:12px; border-radius:8px; margin-bottom:16px;">
      <CheckCircle2 size={18} />
      <span>{form.message || ($lang === 'ar' ? 'تمت العملية بنجاح' : 'Operation completed successfully')}</span>
    </div>
  {/if}

  <div class="cms-stats">
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'إجمالي المقالات المطابقة' : 'Total Articles'}</span>
      <span class="mini-stat-value">{data.total}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'منشورة' : 'Published'}</span>
      <span class="mini-stat-value text-success">
        {data.articles.filter(p => p.published).length}
      </span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'مسودات' : 'Drafts'}</span>
      <span class="mini-stat-value" style="color:var(--warning)">
        {data.articles.filter(p => !p.published).length}
      </span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'الصفحة الحالية' : 'Current Page'}</span>
      <span class="mini-stat-value text-gold">{data.page} / {data.totalPages}</span>
    </div>
  </div>

  <div class="table-container">
    <div class="table-head-bar">
      <form class="toolbar" onsubmit={handleFilterSubmit} style="margin:0; flex:1; display:flex; gap:12px; align-items:center;">
        <div class="search-box" style="flex:1; max-width:300px;">
          <Search size={15} />
          <input 
            type="search" 
            placeholder={$t('common.search')} 
            bind:value={searchInput} 
            aria-label="Search articles"
          />
        </div>
        <select 
          class="form-select" 
          style="height:36px; width:140px;" 
          bind:value={publishFilter}
          onchange={handleFilterSubmit}
          aria-label="Filter by published status"
        >
          <option value="all">{$t('common.all')}</option>
          <option value="true">{$lang === 'ar' ? 'منشور' : 'Published'}</option>
          <option value="false">{$lang === 'ar' ? 'مسودة' : 'Draft'}</option>
        </select>
        <button type="submit" class="btn btn-outline btn-sm">
          {$lang === 'ar' ? 'تطبيق' : 'Apply'}
        </button>
      </form>
      <span class="table-title">
        {data.total} {$lang === 'ar' ? 'سجل' : 'Entries'}
      </span>
    </div>

    <div class="table-scroll">
      <table aria-label="CMS content table">
        <thead>
          <tr>
            <th>Slug</th>
            <th>{$lang === 'ar' ? 'العنوان' : 'Title'}</th>
            <th>{$lang === 'ar' ? 'الفئة' : 'Category'}</th>
            <th>{$lang === 'ar' ? 'آخر تحديث' : 'Last Updated'}</th>
            <th>{$t('common.status')}</th>
            <th>{$t('common.actions')}</th>
          </tr>
        </thead>
        <tbody>
          {#each data.articles as p (p.id)}
            <tr>
              <td>
                <span class="mono" style="font-size:11.5px; color:var(--text-tertiary)">
                  {p.slug}
                </span>
              </td>
              <td style="font-weight:600; font-size:13.5px">
                {$lang === 'ar' && p.titleAr ? p.titleAr : p.titleEn}
              </td>
              <td><span class={typeCls(p.category)}>{typeLabel(p.category)}</span></td>
              <td class="text-muted" style="font-size:12.5px">{formatDate(p.updatedAt)}</td>
              <td><span class={statusCls(p.published)}>{statusLbl(p.published)}</span></td>
              <td>
                <div style="display:flex; gap:6px">
                  <button class="btn-icon" aria-label="Edit" title="Edit" onclick={() => openEditModal(p)}>
                    <Edit size={14} />
                  </button>
                  <form method="POST" action="?/deleteArticle" use:enhance={() => {
                    return async ({ update }) => {
                      await invalidateAll();
                      update();
                    };
                  }} style="display:inline;">
                    <input type="hidden" name="id" value={p.id} />
                    <button type="submit" class="btn-icon danger" aria-label="Delete" title="Delete" onclick={(e) => {
                      if (!confirm($lang === 'ar' ? 'هل أنت تأكد من حذف هذا المقال؟' : 'Are you sure you want to delete this article?')) {
                        e.preventDefault();
                      }
                    }}>
                      <Trash2 size={14} />
                    </button>
                  </form>
                </div>
              </td>
            </tr>
          {/each}
          {#if data.articles.length === 0}
            <tr>
              <td colspan="6">
                <div class="empty-state">
                  <div class="empty-icon"><Newspaper size={28} /></div>
                  <h3>{$t('common.no_data')}</h3>
                  <p>{$lang === 'ar' ? 'لا يوجد محتوى أو مقالات في قاعدة البيانات' : 'No CMS articles found in the database'}</p>
                </div>
              </td>
            </tr>
          {/if}
        </tbody>
      </table>
    </div>
  </div>
</div>

{#if showModal}
  <div class="modal-backdrop" onclick={closeModal}>
    <div class="modal-card" onclick={(e) => e.stopPropagation()}>
      <div class="modal-header">
        <h2>{isEditing ? ($lang === 'ar' ? 'تعديل المحتوى' : 'Edit Article') : ($lang === 'ar' ? 'إضافة محتوى جديد' : 'New Article')}</h2>
        <button class="btn-icon" onclick={closeModal}><X size={18} /></button>
      </div>
      <form method="POST" action={isEditing ? '?/updateArticle' : '?/createArticle'} use:enhance={() => {
        return async ({ update }) => {
          closeModal();
          await invalidateAll();
          update();
        };
      }}>
        {#if isEditing}
          <input type="hidden" name="id" value={selectedArticle?.id} />
        {/if}
        <div class="form-grid">
          <div class="form-group">
            <label for="cms_slug">Slug *</label>
            <input id="cms_slug" type="text" name="slug" required bind:value={formSlug} placeholder="e.g. wedding-guide-2026" class="form-input" />
          </div>
          <div class="form-group">
            <label for="cms_category">Category</label>
            <select id="cms_category" name="category" bind:value={formCategory} class="form-select">
              <option value="wedding-tips">Wedding Tips</option>
              <option value="article">Article</option>
              <option value="page">Page</option>
              <option value="legal">Legal</option>
            </select>
          </div>
          <div class="form-group">
            <label for="cms_title_ar">Title (Arabic) *</label>
            <input id="cms_title_ar" type="text" name="title_ar" required bind:value={formTitleAr} placeholder="عنوان المقال بالعربية" class="form-input" dir="rtl" />
          </div>
          <div class="form-group">
            <label for="cms_title_en">Title (English) *</label>
            <input id="cms_title_en" type="text" name="title_en" required bind:value={formTitleEn} placeholder="Article English Title" class="form-input" />
          </div>
          <div class="form-group full-width">
            <label for="cms_summary_ar">Summary (Arabic)</label>
            <textarea id="cms_summary_ar" name="summary_ar" bind:value={formSummaryAr} rows="2" class="form-textarea" dir="rtl"></textarea>
          </div>
          <div class="form-group full-width">
            <label for="cms_summary_en">Summary (English)</label>
            <textarea id="cms_summary_en" name="summary_en" bind:value={formSummaryEn} rows="2" class="form-textarea"></textarea>
          </div>
          <div class="form-group full-width">
            <label for="cms_body_ar">Body Content (Arabic)</label>
            <textarea id="cms_body_ar" name="body_ar" bind:value={formBodyAr} rows="4" class="form-textarea" dir="rtl"></textarea>
          </div>
          <div class="form-group full-width">
            <label for="cms_body_en">Body Content (English)</label>
            <textarea id="cms_body_en" name="body_en" bind:value={formBodyEn} rows="4" class="form-textarea"></textarea>
          </div>
          <div class="form-group">
            <label for="cms_published">Published Status</label>
            <select id="cms_published" name="published" value={formPublished ? 'true' : 'false'} class="form-select">
              <option value="true">Published</option>
              <option value="false">Draft</option>
            </select>
          </div>
        </div>
        <div class="modal-actions">
          <button type="button" class="btn btn-outline" onclick={closeModal}>Cancel</button>
          <button type="submit" class="btn btn-gold">Save Changes</button>
        </div>
      </form>
    </div>
  </div>
{/if}
