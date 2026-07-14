<script lang="ts">
  import { t, lang } from '$lib/i18n';
  import { goto } from '$app/navigation';
  import { enhance } from '$app/forms';
  import { AlertCircle, Edit, Info, Globe, Plus, Trash2, MessageSquare, Check, X, Send } from 'lucide-svelte';

  export let data: any;
  export let form: any;

  let activeTab = 'posts'; // 'posts' or 'comments'

  // Reply dialog state
  let replyModalOpen = false;
  let replyCommentId = '';
  let replyCommentText = '';
  let replyCommentAuthor = '';
  let replyBlogSlug = '';
  let replyText = '';

  function handlePageChange(newPage: number) {
    goto(`?page=${newPage}`);
  }

  function statusCls(p: any) {
    if (!p.is_published) return 'badge badge-dot badge-warning';
    if (p.published_at && new Date(p.published_at).getTime() > Date.now()) return 'badge badge-dot badge-purple';
    return 'badge badge-dot badge-success';
  }

  function statusLbl(p: any) {
    if (!p.is_published) return $lang === 'ar' ? 'مسودة' : 'Draft';
    if (p.published_at && new Date(p.published_at).getTime() > Date.now()) return $lang === 'ar' ? 'مجدول' : 'Scheduled';
    return $lang === 'ar' ? 'منشور' : 'Published';
  }

  function formatDate(d: string) {
    if (!d) return '-';
    return new Date(d).toLocaleDateString($lang === 'ar' ? 'ar-SA' : 'en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric'
    });
  }

  function openReplyModal(comment: any) {
    replyCommentId = comment.id;
    replyCommentText = comment.comment;
    replyCommentAuthor = comment.name;
    replyBlogSlug = comment.blog_slug;
    replyText = '';
    replyModalOpen = true;
  }

  function getParentAuthor(parentId: string) {
    const parent = data.comments?.find((c: any) => c.id === parentId);
    return parent ? parent.name : 'Unknown';
  }
</script>

<div class="fade-in">
  <div class="page-header">
    <div class="page-header-left">
      <h1 class="page-title">Discover & Blog</h1>
      <p class="page-subtitle">Manage SEO-optimized blog posts, comments, and analytics</p>
    </div>
    <a href="/dashboard/discover/edit/new" class="btn btn-gold btn-sm">
      <Plus size={14} />
      New Post
    </a>
  </div>

  {#if form?.error || data.error}
    <div class="notice-banner error">
      <AlertCircle size={18} class="notice-icon" />
      <div class="notice-text">{form?.error || data.error}</div>
    </div>
  {/if}

  {#if form?.success}
    <div class="notice-banner success">
      <Check size={18} class="notice-icon" />
      <div class="notice-text">{form.message}</div>
    </div>
  {/if}

  <div class="cms-stats">
    <div class="mini-stat card">
      <span class="mini-stat-label">Total Posts</span>
      <span class="mini-stat-value">{data.total}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">Published</span>
      <span class="mini-stat-value text-success">
        {data.blogs.filter((p: any) => p.is_published).length}
      </span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">Pending Comments</span>
      <span class="mini-stat-value text-warning">
        {data.analytics?.pending_comments || 0}
      </span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">Total Views</span>
      <span class="mini-stat-value text-gold">{data.analytics?.total_views || 0}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">CTA Clicks</span>
      <span class="mini-stat-value">{(data.analytics?.afrah_cta_clicks || 0) + (data.analytics?.vendor_cta_clicks || 0)}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">Afrah Starts</span>
      <span class="mini-stat-value">{data.analytics?.afrah_starts || 0}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">Inquiry & Bookings</span>
      <span class="mini-stat-value">{(data.analytics?.inquiry_starts || 0) + (data.analytics?.booking_conversions || 0)}</span>
    </div>
  </div>

  <!-- Tabs Navigation -->
  <div class="tabs-container">
    <button class="tab-btn" class:active={activeTab === 'posts'} onclick={() => activeTab = 'posts'}>
      <Globe size={14} />
      Blog Posts
      <span class="tab-count">{data.total}</span>
    </button>
    <button class="tab-btn" class:active={activeTab === 'comments'} onclick={() => activeTab = 'comments'}>
      <MessageSquare size={14} />
      Comments & Moderation
      <span class="tab-count">{data.comments?.length || 0}</span>
    </button>
  </div>

  {#if activeTab === 'posts'}
    <!-- Blog posts list tab -->
    <div class="table-container">
      <div class="table-head-bar">
        <span class="table-title">{data.total} Entries</span>
      </div>

      <div class="table-scroll">
        <table aria-label="Blogs table">
          <thead>
            <tr>
              <th>Slug</th>
              <th>Title</th>
              <th>Views</th>
              <th>Status</th>
              <th>Created At</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each data.blogs as p (p.id)}
              <tr>
                <td>
                  <span class="mono" style="font-size:11.5px; color:var(--text-tertiary)">
                    {p.slug}
                  </span>
                </td>
                <td style="font-weight:600; font-size:13.5px">
                  {p.title}
                </td>
                <td><span class="badge badge-dot">{p.views_count || 0}</span></td>
                <td><span class={statusCls(p)}>{statusLbl(p)}</span></td>
                <td class="text-muted" style="font-size:12.5px">{formatDate(p.created_at)}</td>
                <td>
                  <div style="display:flex; gap:6px; align-items:center;">
                    <a href={`/dashboard/discover/edit/${p.id}`} class="btn-icon" aria-label="Edit" title="Edit">
                      <Edit size={14} />
                    </a>
                    <a href={`${data.publicClientUrl}/discover/${p.slug}`} target="_blank" class="btn-icon" aria-label="Preview" title="Preview">
                      <Info size={14} />
                    </a>
                    <!-- Form for Toggling Publish Status -->
                    <form method="POST" action="?/togglePublish" use:enhance style="display:inline">
                      <input type="hidden" name="id" value={p.id} />
                      {#if p.is_published}
                        <button type="submit" class="btn btn-outline btn-xs text-warning" style="padding: 2px 6px; font-size: 11px; font-weight: 600; border-color: var(--warning);" title="Revert to Draft">
                          Set Draft
                        </button>
                      {:else}
                        <button type="submit" class="btn btn-gold btn-xs" style="padding: 2px 6px; font-size: 11px; font-weight: 600;" title="Publish Post">
                          Publish
                        </button>
                      {/if}
                    </form>
                    <!-- Form for Deleting a Post -->
                    <form method="POST" action="?/deletePost" use:enhance style="display:inline">
                      <input type="hidden" name="id" value={p.id} />
                      <button type="submit" class="btn-icon text-danger" style="border:none; background:none; cursor:pointer;" aria-label="Delete" title="Delete" onclick={(e) => { if (!confirm('Are you sure you want to delete this blog post?')) e.preventDefault(); }}>
                        <Trash2 size={14} />
                      </button>
                    </form>
                  </div>
                </td>
              </tr>
            {/each}
            {#if data.blogs.length === 0}
              <tr>
                <td colspan="6">
                  <div class="empty-state">
                    <div class="empty-icon"><Globe size={28} /></div>
                    <h3>No blog posts found</h3>
                    <p>Get started by writing your first SEO article.</p>
                  </div>
                </td>
              </tr>
            {/if}
          </tbody>
        </table>
      </div>

      {#if data.totalPages > 1}
        <div class="pagination">
          <span class="pagination-info">Showing page {data.page} of {data.totalPages}</span>
          <div class="pagination-controls">
            <button class="page-btn" onclick={() => handlePageChange(Math.max(1, data.page - 1))} disabled={data.page === 1}>‹</button>
            {#each Array.from({length: data.totalPages}, (_, i) => i + 1) as pageNum}
              <button class="page-btn" class:active={pageNum === data.page} onclick={() => handlePageChange(pageNum)}>{pageNum}</button>
            {/each}
            <button class="page-btn" onclick={() => handlePageChange(Math.min(data.totalPages, data.page + 1))} disabled={data.page === data.totalPages}>›</button>
          </div>
        </div>
      {/if}
    </div>
  {:else}
    <!-- Comments Moderation Tab -->
    <div class="table-container">
      <div class="table-head-bar">
        <span class="table-title">{data.comments?.length || 0} Comments Found</span>
      </div>

      <div class="table-scroll">
        <table aria-label="Comments table">
          <thead>
            <tr>
              <th>Blog Post</th>
              <th>Author</th>
              <th>Comment</th>
              <th>Status</th>
              <th>Created At</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each data.comments as c (c.id)}
              <tr>
                <td style="font-size:12.5px; font-weight:600; max-width: 150px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
                  {c.blog_title}
                </td>
                <td style="font-size:12.5px">
                  <div style="font-weight:600">{c.name}</div>
                  <div class="text-muted text-xs">{c.email || '-'}</div>
                </td>
                <td style="font-size:13px; max-width: 250px; white-space: normal; word-break: break-word;">
                  {#if c.parent_id}
                    <span class="badge badge-purple" style="font-size: 10px; margin-bottom: 4px; display:inline-block;">
                      Reply to {getParentAuthor(c.parent_id)}
                    </span>
                  {/if}
                  <p style="margin:0;">{c.comment}</p>
                </td>
                <td>
                  {#if c.is_approved}
                    <span class="badge badge-dot badge-success">Approved</span>
                  {:else}
                    <span class="badge badge-dot badge-warning">Pending</span>
                  {/if}
                </td>
                <td class="text-muted" style="font-size:12px">{formatDate(c.created_at)}</td>
                <td>
                  <div style="display:flex; gap:6px; align-items:center;">
                    <!-- Approve button -->
                    {#if !c.is_approved}
                      <form method="POST" action="?/approveComment" use:enhance style="display:inline">
                        <input type="hidden" name="id" value={c.id} />
                        <button type="submit" class="btn btn-gold btn-xs" style="background:var(--success); border-color:var(--success); color:#fff; display:inline-flex; align-items:center; gap:2px; padding: 4px 8px;" aria-label="Approve">
                          <Check size={12} /> Approve
                        </button>
                      </form>
                    {/if}
                    <!-- Reply button -->
                    <button type="button" class="btn btn-outline btn-xs" style="display:inline-flex; align-items:center; gap:2px; padding: 4px 8px;" onclick={() => openReplyModal(c)} aria-label="Reply">
                      <Send size={12} /> Reply
                    </button>
                    <!-- Reject/Delete button -->
                    <form method="POST" action="?/deleteComment" use:enhance style="display:inline">
                      <input type="hidden" name="id" value={c.id} />
                      <button type="submit" class="btn btn-outline btn-xs text-danger" style="border-color: rgba(239, 68, 68, 0.2); display:inline-flex; align-items:center; gap:2px; padding: 4px 8px;" aria-label="Delete" onclick={(e) => { if (!confirm('Are you sure you want to delete this comment?')) e.preventDefault(); }}>
                        <Trash2 size={12} /> Delete
                      </button>
                    </form>
                  </div>
                </td>
              </tr>
            {/each}
            {#if !data.comments || data.comments.length === 0}
              <tr>
                <td colspan="6">
                  <div class="empty-state">
                    <div class="empty-icon"><MessageSquare size={28} /></div>
                    <h3>No comments found</h3>
                    <p>When users post comments on your blogs, they will appear here for moderation.</p>
                  </div>
                </td>
              </tr>
            {/if}
          </tbody>
        </table>
      </div>
    </div>
  {/if}
</div>

<!-- Reply Dialog Modal -->
{#if replyModalOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="dialog-backdrop" role="button" tabindex="-1" onclick={() => replyModalOpen = false}></div>
  <div class="dialog-box card p-4">
    <div class="dialog-header">
      <h4>Post Reply</h4>
      <button type="button" class="close-btn" onclick={() => replyModalOpen = false}>
        <X size={16} />
      </button>
    </div>
    <div class="dialog-body">
      <p>Replying to <strong>{replyCommentAuthor}</strong>'s comment: <span style="font-style:italic; color:var(--text-secondary)">"{replyCommentText}"</span></p>
      <form method="POST" action="?/replyComment" use:enhance={() => {
        return async ({ update }) => {
          replyModalOpen = false;
          replyText = '';
          await update();
        };
      }}>
        <input type="hidden" name="parentId" value={replyCommentId} />
        <input type="hidden" name="slug" value={replyBlogSlug} />
        
        <div class="form-group mb-3">
          <label for="reply-text">Admin Reply Content</label>
          <textarea id="reply-text" name="comment" bind:value={replyText} class="form-control" rows="4" placeholder="Write reply..." required></textarea>
        </div>

        <div class="dialog-actions">
          <button type="button" class="btn btn-outline btn-sm" onclick={() => replyModalOpen = false}>Cancel</button>
          <button type="submit" class="btn btn-gold btn-sm">Submit Reply</button>
        </div>
      </form>
    </div>
  </div>
{/if}

<style>
  .cms-stats { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; margin-bottom: 20px; }
  .mini-stat { padding: 16px 18px; display: flex; flex-direction: column; gap: 6px; }
  .mini-stat-label { font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.4px; color: var(--text-ghost); }
  .mini-stat-value { font-size: 20px; font-weight: 800; letter-spacing: -0.4px; }
  @media (max-width: 900px) { .cms-stats { grid-template-columns: repeat(2, 1fr); } }
  @media (max-width: 560px) { .cms-stats { grid-template-columns: 1fr; } }

  .tabs-container {
    display: flex;
    gap: 8px;
    margin-bottom: 24px;
    border-bottom: 1px solid var(--border);
    padding-bottom: 8px;
  }
  .tab-btn {
    background: none;
    border: none;
    padding: 10px 20px;
    cursor: pointer;
    font-size: 14px;
    font-weight: 600;
    color: var(--text-muted);
    border-radius: 8px;
    display: flex;
    align-items: center;
    gap: 8px;
    transition: all 0.2s;
  }
  .tab-btn:hover {
    background: var(--bg-secondary);
    color: var(--text-primary);
  }
  .tab-btn.active {
    background: var(--bg-hover);
    color: var(--gold);
    border: 1px solid var(--border);
  }
  .tab-count {
    background: rgba(255, 255, 255, 0.05);
    padding: 2px 6px;
    border-radius: 12px;
    font-size: 11px;
    color: var(--text-tertiary);
  }
  .tab-btn.active .tab-count {
    background: var(--gold-subtle);
    color: var(--gold);
  }

  /* dialog styles */
  .dialog-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.65);
    backdrop-filter: blur(4px);
    z-index: 999;
  }
  .dialog-box {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    background: var(--bg-elevated);
    border: 1px solid var(--glass-border-hover);
    border-radius: 12px;
    padding: 1.5rem;
    width: 90%;
    max-width: 460px;
    z-index: 1000;
    box-shadow: 0 20px 45px rgba(0, 0, 0, 0.55);
  }
  .dialog-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
    border-bottom: 1px solid var(--border);
    padding-bottom: 0.5rem;
  }
  .dialog-header h4 {
    margin: 0;
    font-size: 1.15rem;
    font-weight: 750;
    color: #ffffff;
  }
  .close-btn {
    background: transparent;
    border: none;
    color: var(--text-tertiary);
    cursor: pointer;
    padding: 0.3rem;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .close-btn:hover {
    background: var(--bg-hover);
    color: #ffffff;
  }
  .dialog-body p {
    margin: 0 0 1rem 0;
    font-size: 0.85rem;
    color: var(--text-tertiary);
    line-height: 1.4;
  }
  .dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 1.2rem;
  }
</style>
