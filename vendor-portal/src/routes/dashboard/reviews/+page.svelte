<script lang="ts">
  import { getI18n } from '$lib/i18n/i18n.svelte';
  import { enhance } from '$app/forms';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import type { PageData } from './$types';

  const i18n = getI18n();

  let { data }: { data: PageData } = $props();

  // Filter state
  let filterStatus = $state<'all' | 'pending' | 'approved' | 'rejected'>($page.url.searchParams.get('status') as any || 'all');

  function handleFilterChange() {
    const url = new URL($page.url);
    if (filterStatus === 'all') {
      url.searchParams.delete('status');
    } else {
      url.searchParams.set('status', filterStatus);
    }
    goto(url.toString(), { keepFocus: true, noScroll: true, replaceState: true });
  }

  function getStatusClass(status: string) {
    if (status === 'approved') return 'badge-done';
    if (status === 'rejected') return 'badge-expired';
    return 'badge-negot';
  }

  function getStatusLabel(status: string) {
    if (status === 'approved') return i18n.t.reviews?.approved || 'Approved';
    if (status === 'rejected') return i18n.t.reviews?.rejected || 'Rejected';
    return i18n.t.reviews?.pendingApproval || 'Pending Approval';
  }

  function formatDate(dateStr: string) {
    if (!dateStr) return '';
    try {
      const d = new Date(dateStr);
      return d.toLocaleDateString(i18n.locale === 'ar' ? 'ar-EG' : 'en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric'
      });
    } catch {
      return dateStr;
    }
  }
</script>

<svelte:head>
  <title>{i18n.t.nav?.reviews || 'Reviews'} - {i18n.t.common?.appName || 'Zafaf Portal'}</title>
</svelte:head>

<div class="reviews-page">
  <div class="toolbar">
    <div style="position: relative;">
      <select
        class="form-select btn-sm"
        style="height: 40px; padding: 0 32px 0 16px; border: 1.5px solid var(--border); font-size: 13px; font-weight: 600; width: 180px;"
        bind:value={filterStatus}
        onchange={handleFilterChange}
        aria-label={i18n.t.reviews?.status || 'Filter by Status'}
      >
        <option value="all">{i18n.t.nav?.reviews || 'Reviews'}</option>
        <option value="pending">{i18n.t.reviews?.pendingApproval || 'Pending Approval'}</option>
        <option value="approved">{i18n.t.reviews?.approved || 'Approved'}</option>
        <option value="rejected">{i18n.t.reviews?.rejected || 'Rejected'}</option>
      </select>
      <span style="position: absolute; left: 12px; top: 50%; transform: translateY(-50%) var(--arrow-rotation); pointer-events: none; color: var(--text-sec); font-size: 9px;">
        ▼
      </span>
    </div>
  </div>

  {#await data.heavyReviews}
    <!-- Skeleton layout during background resolution -->
    <div class="reviews-grid">
      {#each Array(3) as _}
        <div class="review-card skeleton-card">
          <div class="review-header">
            <div class="reviewer-block">
              <div class="reviewer-avatar skeleton-pulse"></div>
              <div>
                <div class="skeleton-pulse skeleton-text-title"></div>
                <div class="skeleton-pulse skeleton-text-subtitle"></div>
              </div>
            </div>
            <div class="skeleton-pulse skeleton-badge"></div>
          </div>
          <div class="rating-stars skeleton-stars">
            {#each Array(5) as _}
              <span class="star">★</span>
            {/each}
          </div>
          <div class="review-comment skeleton-pulse skeleton-comment"></div>
        </div>
      {/each}
    </div>
  {:then reviews}
    {@const filteredReviews = reviews}
    {#if filteredReviews.length === 0}
      <div class="empty-state">
        <div class="empty-icon">⭐</div>
        <h3>{i18n.t.nav?.reviews || 'Reviews'}</h3>
        <p>{i18n.t.reviews?.empty || 'No reviews found.'}</p>
      </div>
    {:else}
      <div class="reviews-grid">
        {#each filteredReviews as item (item.id)}
        <div class="review-card">
            <div class="review-header">
              <div class="reviewer-block">
                <div class="reviewer-avatar">{(item.couple_name ?? '?')[0].toUpperCase()}</div>
                <div>
                  <h3 class="reviewer-name">{item.couple_name}</h3>
                  <p class="review-dates">📅 {formatDate(item.created_at ?? '')}</p>
                </div>
              </div>
              <span class="badge {getStatusClass(item.status)}">
                {getStatusLabel(item.status)}
              </span>
            </div>

            <div class="rating-stars" aria-label="Rating: {item.rating} stars">
              {#each Array(5) as _, i}
                <span class="star {i < item.rating ? 'star-filled' : 'star-empty'}">★</span>
              {/each}
            </div>

            <p class="review-comment">"{item.comment || ''}"</p>

            {#if item.status === 'pending'}
              <div class="review-actions">
                <form method="POST" action="?/updateStatus" use:enhance>
                  <input type="hidden" name="id" value={item.id} />
                  <button type="submit" name="status" value="approved" class="btn btn-primary btn-sm">
                    ✓ {i18n.t.reviews?.approveBtn || 'Approve'}
                  </button>
                  <button type="submit" name="status" value="rejected" class="btn btn-outline btn-sm reject-btn-custom">
                    ✗ {i18n.t.reviews?.rejectBtn || 'Reject'}
                  </button>
                </form>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  {:catch error}
    <div class="empty-state">
      <div class="empty-icon">⚠️</div>
      <h3>{i18n.t.nav?.reviews || 'Reviews'}</h3>
      <p class="text-danger">Failed to load reviews data: {error.message}</p>
    </div>
  {/await}
</div>

<style>
  .reviews-page {
    display: flex;
    flex-direction: column;
    gap: 20px;
    animation: fade-in 0.4s ease-out;
  }

  @keyframes fade-in {
    from { opacity: 0; transform: translateY(8px); }
    to   { opacity: 1; transform: translateY(0); }
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 12px;
    background: var(--white);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 12px 16px;
    box-shadow: var(--shadow-xs);
  }

  .reviews-grid {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .review-card {
    background: var(--white);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 20px 24px;
    box-shadow: var(--shadow-xs);
    transition: transform 0.2s ease, box-shadow 0.2s ease;
    position: relative;
    overflow: hidden;
  }

  .review-card:hover {
    transform: translateY(-2px);
    box-shadow: var(--shadow-md);
  }

  .review-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 14px;
    flex-wrap: wrap;
    gap: 12px;
  }

  .reviewer-block {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .reviewer-avatar {
    width: 40px;
    height: 40px;
    border-radius: 12px;
    background: linear-gradient(135deg, hsl(162, 72%, 50%) 0%, hsl(162, 72%, 34%) 100%);
    display: flex;
    align-items: center;
    justify-content: center;
    color: #fff;
    font-size: 15px;
    font-weight: 700;
    flex-shrink: 0;
  }

  .reviewer-name {
    font-size: 15px;
    font-weight: 700;
    color: var(--text);
    margin: 0;
    line-height: 1.2;
  }

  .review-dates {
    font-size: 12px;
    color: var(--text-sec);
    margin-top: 3px;
  }

  .rating-stars {
    margin-bottom: 12px;
    display: flex;
    gap: 3px;
    align-items: center;
  }

  .star { font-size: 18px; line-height: 1; }
  .star-filled { color: hsl(40, 96%, 55%); }
  .star-empty  { color: #cbd5e1; }

  .review-comment {
    font-size: 13.5px;
    color: var(--text-sec);
    line-height: 1.65;
    margin: 0 0 4px;
    font-style: italic;
    text-align: var(--text-align);
    padding: 12px 16px;
    background: var(--bg);
    border-radius: var(--radius-sm);
    border-inline-start: 3px solid var(--border);
  }

  .review-actions {
    display: flex;
    gap: 10px;
    border-top: 1px solid var(--border-light);
    padding-top: 14px;
    justify-content: flex-end;
    margin-top: 14px;
  }

  .reject-btn-custom {
    color: var(--red) !important;
    border-color: var(--red) !important;
  }
  .reject-btn-custom:hover {
    background: var(--red-light) !important;
  }

  /* Shimmer skeleton styling */
  .skeleton-card {
    background: var(--white);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 20px 24px;
    box-shadow: var(--shadow-xs);
    pointer-events: none;
  }

  .skeleton-pulse {
    background: linear-gradient(90deg, #f1f5f9 25%, #e2e8f0 50%, #f1f5f9 75%);
    background-size: 200% 100%;
    animation: shimmer-animation 1.5s infinite linear;
  }

  @keyframes shimmer-animation {
    0% { background-position: -200% 0; }
    100% { background-position: 200% 0; }
  }

  .reviewer-avatar.skeleton-pulse {
    width: 40px;
    height: 40px;
    border-radius: 12px;
  }

  .skeleton-text-title {
    width: 120px;
    height: 14px;
    border-radius: 4px;
    margin-bottom: 6px;
  }

  .skeleton-text-subtitle {
    width: 80px;
    height: 12px;
    border-radius: 4px;
  }

  .skeleton-badge {
    width: 90px;
    height: 22px;
    border-radius: 12px;
  }

  .skeleton-stars {
    opacity: 0.15;
    margin-bottom: 12px;
  }

  .skeleton-comment {
    height: 52px;
    border-radius: var(--radius-sm);
    width: 100%;
  }
</style>
