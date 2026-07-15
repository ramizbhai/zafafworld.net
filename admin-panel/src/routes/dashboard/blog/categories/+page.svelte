<script lang="ts">
  import { enhance } from '$app/forms';
  import { invalidateAll } from '$app/navigation';
  import { Plus, Trash2, Folder, AlertCircle } from 'lucide-svelte';
  import { lang } from '$lib/i18n';

  let { data, form } = $props<{ data: any; form: any }>();

  let name = $state('');
  let slug = $state('');
  let isSubmitting = $state(false);

  // Auto-generate slug from name on input
  function handleNameInput() {
    slug = name
      .toLowerCase()
      .replace(/[^a-z0-9\s-]/g, '')
      .trim()
      .replace(/\s+/g, '-')
      .replace(/-+/g, '-');
  }
</script>

<div class="fade-in max-w-6xl mx-auto">
  <div class="page-header mb-6">
    <div class="page-header-left">
      <h1 class="page-title" id="categories-heading">{lang === 'ar' ? 'إدارة تصنيفات المدونة' : 'Blog Categories'}</h1>
      <p class="page-subtitle">{lang === 'ar' ? 'إنشاء وتعديل وحذف فئات مواضيع المدونة' : 'Organize and structure your blog topics'}</p>
    </div>
  </div>

  {#if form?.error}
    <div class="notice-banner error mb-6">
      <AlertCircle size={18} class="notice-icon" />
      <div class="notice-text">{form.error}</div>
    </div>
  {/if}

  <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
    <!-- Categories List -->
    <div class="md:col-span-2">
      <div class="form-card">
        <h2 class="section-title text-lg font-bold mb-4 flex items-center gap-2">
          <Folder size={18} class="text-[var(--color-primary)]" />
          {lang === 'ar' ? 'التصنيفات الحالية' : 'Current Categories'}
        </h2>

        <div class="table-container">
          <table aria-describedby="categories-heading">
            <thead>
              <tr>
                <th>{lang === 'ar' ? 'الاسم' : 'Name'}</th>
                <th>{lang === 'ar' ? 'الرابط البديل (Slug)' : 'Slug'}</th>
                <th class="text-right">{lang === 'ar' ? 'الإجراءات' : 'Actions'}</th>
              </tr>
            </thead>
            <tbody>
              {#each data.categories as cat (cat.id)}
                <tr>
                  <td class="font-semibold text-[var(--text-primary)]">{cat.name}</td>
                  <td><code class="text-xs bg-slate-100 px-2 py-1 rounded text-slate-700 font-mono">{cat.slug}</code></td>
                  <td class="text-right">
                    <form
                      method="POST"
                      action="?/delete"
                      use:enhance={() => {
                        return async ({ update }) => {
                          await update();
                          await invalidateAll();
                        };
                      }}
                      style="display: inline;"
                    >
                      <input type="hidden" name="id" value={cat.id} />
                      <button
                        type="submit"
                        class="btn-icon text-red-600 hover:bg-red-50 p-2 rounded-lg transition-colors"
                        aria-label="Delete category {cat.name}"
                        title={lang === 'ar' ? 'حذف' : 'Delete'}
                      >
                        <Trash2 size={16} />
                      </button>
                    </form>
                  </td>
                </tr>
              {:else}
                <tr>
                  <td colspan="3" class="text-center py-8 text-[var(--text-ghost)]">
                    {lang === 'ar' ? 'لا توجد تصنيفات مضافة بعد.' : 'No categories found.'}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      </div>
    </div>

    <!-- Create New Form -->
    <div>
      <div class="form-card">
        <h2 class="section-title text-lg font-bold mb-4">
          {lang === 'ar' ? 'إضافة تصنيف جديد' : 'Create Category'}
        </h2>

        <form
          method="POST"
          action="?/create"
          use:enhance={() => {
            isSubmitting = true;
            return async ({ update }) => {
              isSubmitting = false;
              name = '';
              slug = '';
              await update();
              await invalidateAll();
            };
          }}
          class="flex flex-col gap-4"
        >
          <div class="form-group">
            <label for="cat-name-input" class="font-bold block mb-1">
              {lang === 'ar' ? 'اسم التصنيف' : 'Category Name'}
            </label>
            <input
              type="text"
              id="cat-name-input"
              name="name"
              bind:value={name}
              oninput={handleNameInput}
              class="base-input w-full"
              placeholder={lang === 'ar' ? 'مثال: نصائح الزفاف' : 'e.g. Wedding Tips'}
              required
            />
          </div>

          <div class="form-group">
            <label for="cat-slug-input" class="font-bold block mb-1">
              {lang === 'ar' ? 'الرابط البديل (Slug)' : 'Slug'}
            </label>
            <input
              type="text"
              id="cat-slug-input"
              name="slug"
              bind:value={slug}
              class="base-input w-full font-mono text-sm"
              placeholder="wedding-tips"
              required
            />
          </div>

          <button
            type="submit"
            class="btn btn-gold w-full mt-2 flex items-center justify-center gap-2"
            disabled={isSubmitting}
          >
            <Plus size={16} />
            {isSubmitting ? (lang === 'ar' ? 'جاري الحفظ...' : 'Creating...') : (lang === 'ar' ? 'إضافة التصنيف' : 'Create Category')}
          </button>
        </form>
      </div>
    </div>
  </div>
</div>
