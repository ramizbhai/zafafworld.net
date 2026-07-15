<script lang="ts">
  import { t, lang } from '$lib/i18n';
  import { goto, invalidateAll } from '$app/navigation';
  import { enhance } from '$app/forms';
  import { AlertCircle, Edit, Globe, Plus, Trash2, MessageSquare, Check, Calendar, Tag, Folder, Eye } from 'lucide-svelte';

  let { data, form } = $props<{ data: any; form: any }>();

  function handlePageChange(newPage: number) {
    goto(`?page=${newPage}`);
  }

  function statusCls(p: any) {
    if (!p.is_published) return 'badge badge-dot badge-warning';
    if (p.published_at && new Date(p.published_at).getTime() > Date.now()) return 'badge badge-dot badge-purple';
    return 'badge badge-dot badge-success';
  }

  function statusLbl(p: any) {
    if (!p.is_published) return lang === 'ar' ? 'مسودة' : 'Draft';
    if (p.published_at && new Date(p.published_at).getTime() > Date.now()) return lang === 'ar' ? 'مجدول' : 'Scheduled';
    return lang === 'ar' ? 'منشور' : 'Published';
  }

  function formatDate(d: string) {
    if (!d) return '-';
    return new Date(d).toLocaleDateString(lang === 'ar' ? 'ar-SA' : 'en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric'
    });
  }
</script>

<div class="fade-in max-w-6xl mx-auto">
  <div class="page-header mb-6">
    <div class="page-header-left">
      <h1 class="page-title">{lang === 'ar' ? 'إدارة المدونة' : 'Blog Management'}</h1>
      <p class="page-subtitle">{lang === 'ar' ? 'إدارة المقالات ثنائية اللغة والتصنيفات والوسوم والتعليقات' : 'Manage your bilingual blog posts, categories, tags, and comments'}</p>
    </div>
    <div class="flex items-center gap-3">
      <a href="/dashboard/blog/new" class="btn btn-gold btn-sm flex items-center gap-1">
        <Plus size={16} />
        {lang === 'ar' ? 'مقالة جديدة' : 'New Post'}
      </a>
    </div>
  </div>

  <!-- Blog Sections Quick Nav Bar -->
  <div class="flex flex-wrap gap-2 border-b border-slate-200 pb-4 mb-6">
    <a href="/dashboard/blog" class="px-4 py-2 text-sm font-semibold rounded-lg bg-[var(--color-primary)] text-white flex items-center gap-2">
      <Globe size={16} />
      {lang === 'ar' ? 'المقالات' : 'Blog Posts'}
    </a>
    <a href="/dashboard/blog/categories" class="px-4 py-2 text-sm font-semibold rounded-lg bg-white border text-slate-700 hover:bg-slate-50 flex items-center gap-2">
      <Folder size={16} />
      {lang === 'ar' ? 'التصنيفات' : 'Categories'}
    </a>
    <a href="/dashboard/blog/tags" class="px-4 py-2 text-sm font-semibold rounded-lg bg-white border text-slate-700 hover:bg-slate-50 flex items-center gap-2">
      <Tag size={16} />
      {lang === 'ar' ? 'الوسوم' : 'Tags'}
    </a>
    <a href="/dashboard/blog/comments" class="px-4 py-2 text-sm font-semibold rounded-lg bg-white border text-slate-700 hover:bg-slate-50 flex items-center gap-2">
      <MessageSquare size={16} />
      {lang === 'ar' ? 'التعليقات' : 'Comments'}
    </a>
  </div>

  {#if form?.error || data.error}
    <div class="notice-banner error mb-6">
      <AlertCircle size={18} class="notice-icon" />
      <div class="notice-text">{form?.error || data.error}</div>
    </div>
  {/if}

  <!-- Posts Table -->
  <div class="form-card">
    <div class="table-container">
      <table>
        <thead>
          <tr>
            <th>{lang === 'ar' ? 'العنوان' : 'Title'}</th>
            <th>{lang === 'ar' ? 'الرابط البديل (Slug)' : 'Slug'}</th>
            <th>{lang === 'ar' ? 'تاريخ النشر' : 'Publish Date'}</th>
            <th>{lang === 'ar' ? 'الحالة' : 'Status'}</th>
            <th class="text-right">{lang === 'ar' ? 'الإجراءات' : 'Actions'}</th>
          </tr>
        </thead>
        <tbody>
          {#each data.blogs as blog (blog.id)}
            <tr>
              <td class="font-semibold text-[var(--text-primary)]">
                <div class="flex flex-col">
                  <span>{blog.title}</span>
                </div>
              </td>
              <td>
                <span class="mono text-xs bg-slate-100 px-2 py-1 rounded text-slate-700">{blog.slug}</span>
              </td>
              <td class="text-sm text-slate-600">
                {formatDate(blog.published_at || blog.created_at)}
              </td>
              <td>
                <span class={statusCls(blog)}>
                  {statusLbl(blog)}
                </span>
              </td>
              <td class="text-right">
                <div class="flex items-center justify-end gap-2">
                  <a
                    href="/dashboard/blog/{blog.id}"
                    class="btn-icon hover:bg-slate-100 p-2 rounded-lg transition-colors text-slate-700"
                    title={lang === 'ar' ? 'تعديل' : 'Edit'}
                  >
                    <Edit size={16} />
                  </a>

                  <!-- Public Preview Link -->
                  <a
                    href="{data.publicClientUrl}/blog/{blog.slug}"
                    target="_blank"
                    rel="noopener noreferrer"
                    class="btn-icon hover:bg-slate-100 p-2 rounded-lg transition-colors text-slate-500"
                    title={lang === 'ar' ? 'معاينة' : 'Preview'}
                  >
                    <Eye size={16} />
                  </a>

                  <form
                    method="POST"
                    action="?/deletePost"
                    use:enhance={() => {
                      return async ({ update }) => {
                        await update();
                        await invalidateAll();
                      };
                    }}
                    style="display: inline;"
                  >
                    <input type="hidden" name="id" value={blog.id} />
                    <button
                      type="submit"
                      class="btn-icon text-red-600 hover:bg-red-50 p-2 rounded-lg transition-colors"
                      title={lang === 'ar' ? 'حذف' : 'Delete'}
                      onclick={(e) => {
                        if (!confirm(lang === 'ar' ? 'هل أنت متأكد من حذف هذه المقالة؟' : 'Are you sure you want to delete this post?')) {
                          e.preventDefault();
                        }
                      }}
                    >
                      <Trash2 size={16} />
                    </button>
                  </form>
                </div>
              </td>
            </tr>
          {:else}
            <tr>
              <td colspan="5" class="text-center py-16 text-[var(--text-ghost)]">
                <div class="flex flex-col items-center justify-center gap-3">
                  <Globe size={40} class="text-slate-300" />
                  <p>{lang === 'ar' ? 'لا توجد مقالات منشورة بعد.' : 'No blog posts found.'}</p>
                  <a href="/dashboard/blog/new" class="btn btn-gold btn-sm mt-2">
                    {lang === 'ar' ? 'إنشاء مقال أول' : 'Create First Post'}
                  </a>
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <!-- Pagination -->
    {#if data.totalPages > 1}
      <div class="pagination mt-6 flex justify-between items-center">
        <span class="text-sm text-slate-500">
          {lang === 'ar' ? `صفحة ${data.page} من ${data.totalPages}` : `Page ${data.page} of ${data.totalPages}`}
        </span>
        <div class="flex gap-2">
          <button
            class="page-btn btn btn-outline btn-xs"
            disabled={data.page === 1}
            onclick={() => handlePageChange(data.page - 1)}
          >
            {lang === 'ar' ? 'السابق' : 'Previous'}
          </button>
          <button
            class="page-btn btn btn-outline btn-xs"
            disabled={data.page === data.totalPages}
            onclick={() => handlePageChange(data.page + 1)}
          >
            {lang === 'ar' ? 'التالي' : 'Next'}
          </button>
        </div>
      </div>
    {/if}
  </div>
</div>
