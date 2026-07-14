<script lang="ts">
  import { t, lang } from '$lib/i18n/index.js';
  import { Newspaper, Plus, Edit, Eye, Globe, AlertCircle, Search } from 'lucide-svelte';
  import { goto } from '$app/navigation';

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
  }

  let { data }: Props = $props();

  // Local state synced from page data
  let searchInput = $state('');
  let publishFilter = $state('all');

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
    if (t === 'article' || t === 'CMS') return 'badge badge-purple';
    if (t === 'legal') return 'badge badge-muted';
    return 'badge badge-info';
  }

  function typeLabel(t: string) {
    const m: Record<string, string> = {
      page: $lang === 'ar' ? 'صفحة' : 'Page',
      article: $lang === 'ar' ? 'مقال' : 'Article',
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
    <button class="btn btn-gold btn-sm" disabled>
      <Plus size={14} /> 
      {$lang === 'ar' ? 'محتوى جديد' : 'New Content'}
    </button>
  </div>

  {#if data.error}
    <div class="notice-banner error">
      <AlertCircle size={18} class="notice-icon" />
      <div class="notice-text">{data.error}</div>
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
      <span class="mini-stat-label">{$lang === 'ar' ? 'الصحفة الحالية' : 'Current Page'}</span>
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
                  <button class="btn-icon" aria-label="Edit" title="Edit" disabled>
                    <Edit size={14} />
                  </button>
                  <button class="btn-icon" aria-label="Preview" title="Preview" disabled>
                    <Eye size={14} />
                  </button>
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

    {#if data.totalPages > 1}
      <div class="pagination">
        <span class="pagination-info">
          {$lang === 'ar'
            ? `عرض الصفحة ${data.page} من ${data.totalPages}`
            : `Showing page ${data.page} of ${data.totalPages}`}
        </span>
        <div class="pagination-controls">
          <button 
            class="page-btn" 
            onclick={() => handlePageChange(Math.max(1, data.page - 1))} 
            disabled={data.page === 1} 
            aria-label="Previous page"
          >
            ‹
          </button>
          {#each Array.from({length: data.totalPages}, (_, i) => i + 1) as pageNum}
            <button 
              class="page-btn" 
              class:active={pageNum === data.page} 
              onclick={() => handlePageChange(pageNum)} 
              aria-label="Page {pageNum}" 
              aria-current={pageNum === data.page ? 'page' : undefined}
            >
              {pageNum}
            </button>
          {/each}
          <button 
            class="page-btn" 
            onclick={() => handlePageChange(Math.min(data.totalPages, data.page + 1))} 
            disabled={data.page === data.totalPages} 
            aria-label="Next page"
          >
            ›
          </button>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .cms-stats { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; margin-bottom: 20px; }
  .mini-stat { padding: 16px 18px; display: flex; flex-direction: column; gap: 6px; }
  .mini-stat-label { font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.4px; color: var(--text-ghost); }
  .mini-stat-value { font-size: 20px; font-weight: 800; letter-spacing: -0.4px; }
  @media (max-width: 900px) { .cms-stats { grid-template-columns: repeat(2, 1fr); } }
  @media (max-width: 560px) { .cms-stats { grid-template-columns: 1fr; } }
</style>
