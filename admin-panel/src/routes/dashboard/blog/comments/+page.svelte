<script lang="ts">
  import { enhance } from '$app/forms';
  import { invalidateAll } from '$app/navigation';
  import { CheckCircle, XCircle, MessageSquare, AlertCircle, Calendar, ExternalLink } from 'lucide-svelte';
  import { lang } from '$lib/i18n';

  let { data, form } = $props<{ data: any; form: any }>();

  let isProcessing = $state<string | null>(null);

  function formatDate(d: string) {
    if (!d) return '-';
    return new Date(d).toLocaleDateString(lang === 'ar' ? 'ar-SA' : 'en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }
</script>

<div class="fade-in max-w-6xl mx-auto">
  <div class="page-header mb-6">
    <div class="page-header-left">
      <h1 class="page-title" id="comments-heading">{lang === 'ar' ? 'الإشراف على التعليقات' : 'Comment Moderation'}</h1>
      <p class="page-subtitle">{lang === 'ar' ? 'مراجعة وقبول أو رفض التعليقات المكتوبة على مقالات المدونة' : 'Moderate comments posted on your blog posts'}</p>
    </div>
  </div>

  {#if form?.error}
    <div class="notice-banner error mb-6">
      <AlertCircle size={18} class="notice-icon" />
      <div class="notice-text">{form.error}</div>
    </div>
  {/if}

  <div class="form-card">
    <h2 class="section-title text-lg font-bold mb-4 flex items-center gap-2">
      <MessageSquare size={18} class="text-[var(--color-primary)]" />
      {lang === 'ar' ? 'قائمة التعليقات' : 'Comments Queue'}
    </h2>

    <div class="table-container">
      <table aria-describedby="comments-heading">
        <thead>
          <tr>
            <th>{lang === 'ar' ? 'الكاتب' : 'Author'}</th>
            <th>{lang === 'ar' ? 'التعليق' : 'Comment'}</th>
            <th>{lang === 'ar' ? 'المقال' : 'Blog Post'}</th>
            <th>{lang === 'ar' ? 'التاريخ' : 'Date'}</th>
            <th>{lang === 'ar' ? 'الحالة' : 'Status'}</th>
            <th class="text-right">{lang === 'ar' ? 'الإجراءات' : 'Actions'}</th>
          </tr>
        </thead>
        <tbody>
          {#each data.comments as comment (comment.id)}
            <tr>
              <td>
                <div class="font-semibold text-[var(--text-primary)]">{comment.name}</div>
                <div class="text-xs text-[var(--text-ghost)]">{comment.email}</div>
              </td>
              <td class="max-w-xs md:max-w-md">
                <p class="text-sm whitespace-pre-line text-slate-800 font-sans">{comment.comment}</p>
              </td>
              <td>
                <div class="text-sm font-semibold">{comment.blog_title}</div>
                <div class="text-xs text-[var(--text-ghost)] font-mono">{comment.blog_slug}</div>
              </td>
              <td class="text-xs text-[var(--text-secondary)] whitespace-nowrap">
                <span class="flex items-center gap-1">
                  <Calendar size={12} />
                  {formatDate(comment.created_at)}
                </span>
              </td>
              <td>
                {#if comment.is_approved}
                  <span class="badge badge-dot badge-success text-xs font-semibold px-2 py-1 rounded bg-green-50 border border-green-200 text-green-700">
                    {lang === 'ar' ? 'مقبول' : 'Approved'}
                  </span>
                {:else}
                  <span class="badge badge-dot badge-warning text-xs font-semibold px-2 py-1 rounded bg-yellow-50 border border-yellow-200 text-yellow-700">
                    {lang === 'ar' ? 'قيد الانتظار' : 'Pending'}
                  </span>
                {/if}
              </td>
              <td class="text-right">
                <div class="flex items-center justify-end gap-2">
                  {#if !comment.is_approved}
                    <form
                      method="POST"
                      action="?/approve"
                      use:enhance={() => {
                        isProcessing = comment.id;
                        return async ({ update }) => {
                          isProcessing = null;
                          await update();
                          await invalidateAll();
                        };
                      }}
                      style="display: inline;"
                    >
                      <input type="hidden" name="id" value={comment.id} />
                      <button
                        type="submit"
                        class="btn btn-sm btn-success flex items-center gap-1"
                        disabled={isProcessing === comment.id}
                        aria-label="Approve comment from {comment.name}"
                      >
                        <CheckCircle size={14} />
                        {lang === 'ar' ? 'قبول' : 'Approve'}
                      </button>
                    </form>
                  {/if}

                  <form
                    method="POST"
                    action="?/reject"
                    use:enhance={() => {
                      isProcessing = comment.id;
                      return async ({ update }) => {
                        isProcessing = null;
                        await update();
                        await invalidateAll();
                      };
                    }}
                    style="display: inline;"
                  >
                    <input type="hidden" name="id" value={comment.id} />
                    <button
                      type="submit"
                      class="btn btn-sm btn-danger flex items-center gap-1"
                      disabled={isProcessing === comment.id}
                      aria-label="Delete comment from {comment.name}"
                    >
                      <XCircle size={14} />
                      {lang === 'ar' ? 'رفض / حذف' : 'Reject'}
                    </button>
                  </form>
                </div>
              </td>
            </tr>
          {:else}
            <tr>
              <td colspan="6" class="text-center py-12 text-[var(--text-ghost)]">
                <div class="flex flex-col items-center justify-center gap-2">
                  <MessageSquare size={32} class="text-slate-300" />
                  <p>{lang === 'ar' ? 'لا توجد تعليقات للمراجعة حالياً.' : 'No comments found in moderation queue.'}</p>
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
</div>
