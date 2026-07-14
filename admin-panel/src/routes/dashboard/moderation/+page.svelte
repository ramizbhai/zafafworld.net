<script lang="ts">
  import { t, lang } from '$lib/i18n/index.js';
  import { ShieldCheck, Search, Eye, Trash2, CheckCircle2, XCircle, Star, AlertCircle, MessageSquare } from 'lucide-svelte';
  import { enhance } from '$app/forms';

  interface Props {
    data: {
      reviews: any[];
      error?: string;
    };
  }

  let { data }: Props = $props();

  let search = $state('');

  let filtered = $derived(data.reviews.filter(item => {
    const q = search.toLowerCase();
    return !search || 
      item.id.toLowerCase().includes(q) || 
      item.review_text.toLowerCase().includes(q) || 
      item.client_name.toLowerCase().includes(q) || 
      (item.vendor_name_en && item.vendor_name_en.toLowerCase().includes(q)) ||
      (item.vendor_name_ar && item.vendor_name_ar.toLowerCase().includes(q));
  }));

  function formatDate(d: string): string {
    if (!d) return '-';
    return new Date(d).toLocaleDateString($lang === 'ar' ? 'ar-SA' : 'en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric'
    });
  }

  let highRatingCount = $derived(data.reviews.filter(r => r.rating >= 4).length);
  let lowRatingCount = $derived(data.reviews.filter(r => r.rating <= 2).length);
</script>

<div class="fade-in">
  <div class="page-header">
    <div class="page-header-left">
      <h1 class="page-title">{$t('nav.moderation')}</h1>
      <p class="page-subtitle">{$lang === 'ar' ? 'مراجعة وإشراف على تقييمات العملاء للموردين' : 'Review and moderate client reviews for vendors'}</p>
    </div>
  </div>

  {#if data.error}
    <div class="notice-banner error">
      <AlertCircle size={18} class="notice-icon" />
      <div class="notice-text">{data.error}</div>
    </div>
  {/if}

  <div class="mod-stats">
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'تقييمات معلقة' : 'Pending Reviews'}</span>
      <span class="mini-stat-value" style="color:var(--warning)">{data.reviews.length}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'تقييمات إيجابية معلقة' : 'Pending High Ratings'}</span>
      <span class="mini-stat-value" style="color:var(--success)">{highRatingCount}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'تقييمات سلبية معلقة' : 'Pending Low Ratings'}</span>
      <span class="mini-stat-value" style="color:var(--danger)">{lowRatingCount}</span>
    </div>
    <div class="mini-stat card">
      <span class="mini-stat-label">{$lang === 'ar' ? 'إجمالي المراجعات المعلقة' : 'Total Pending Queue'}</span>
      <span class="mini-stat-value">{data.reviews.length}</span>
    </div>
  </div>

  <div class="table-container">
    <div class="table-head-bar" style="display: flex; justify-content: space-between; align-items: center; gap: 16px;">
      <div class="search-box" style="flex: 1; max-width: 320px;">
        <Search size={15} />
        <input type="search" placeholder={$t('common.search')} bind:value={search} />
      </div>
      <span class="table-title">{$lang === 'ar' ? 'قائمة انتظار التقييمات' : 'Reviews Moderation Queue'} ({filtered.length})</span>
    </div>
    <div class="table-scroll">
      <table>
        <thead>
          <tr>
            <th>ID</th>
            <th>{$lang === 'ar' ? 'العميل' : 'Client'}</th>
            <th>{$lang === 'ar' ? 'المورد' : 'Vendor'}</th>
            <th>{$lang === 'ar' ? 'التقييم' : 'Rating'}</th>
            <th>{$lang === 'ar' ? 'التعليق' : 'Comment'}</th>
            <th>{$lang === 'ar' ? 'التاريخ' : 'Date'}</th>
            <th>{$t('common.actions')}</th>
          </tr>
        </thead>
        <tbody>
          {#each filtered as item}
            <tr>
              <td><span class="mono text-gold" style="font-size:11px; font-weight:700">{item.id.slice(0, 8)}...</span></td>
              <td class="text-muted">{item.client_name}</td>
              <td class="text-muted">
                {$lang === 'ar' ? (item.vendor_name_ar || item.vendor_name_en) : item.vendor_name_en}
              </td>
              <td>
                <div style="display: flex; gap: 2px; color: var(--gold); align-items: center;">
                  {#each Array(item.rating) as _}
                    <Star size={12} fill="currentColor" />
                  {/each}
                  {#each Array(5 - item.rating) as _}
                    <Star size={12} />
                  {/each}
                </div>
              </td>
              <td style="max-width: 300px; white-space: normal; word-break: break-word; font-size: 12.5px;">
                {item.review_text}
              </td>
              <td class="text-muted" style="font-size:12px">{formatDate(item.created_at)}</td>
              <td>
                <div style="display:flex; gap:6px">
                  <form method="POST" action="?/moderate" use:enhance>
                    <input type="hidden" name="id" value={item.id} />
                    <input type="hidden" name="approve" value="true" />
                    <button class="btn btn-gold btn-xs" style="background: var(--success); border-color: var(--success-border); color: #fff;" type="submit" aria-label="Approve">
                      <CheckCircle2 size={12} />
                      {$lang === 'ar' ? 'قبول' : 'Approve'}
                    </button>
                  </form>
                  <form method="POST" action="?/moderate" use:enhance>
                    <input type="hidden" name="id" value={item.id} />
                    <input type="hidden" name="approve" value="false" />
                    <button class="btn btn-outline btn-xs" style="color: var(--danger); border-color: var(--danger-border);" type="submit" aria-label="Reject">
                      <XCircle size={12} />
                      {$lang === 'ar' ? 'رفض' : 'Reject'}
                    </button>
                  </form>
                </div>
              </td>
            </tr>
          {/each}
          {#if filtered.length === 0}
            <tr>
              <td colspan="7">
                <div class="empty-state" style="padding: 40px 0; text-align: center; color: var(--text-ghost);">
                  <div class="empty-icon" style="margin-bottom: 12px;"><ShieldCheck size={28} /></div>
                  <h3>{$lang === 'ar' ? 'قائمة الانتظار فارغة' : 'Moderation queue is empty'}</h3>
                </div>
              </td>
            </tr>
          {/if}
        </tbody>
      </table>
    </div>
  </div>
</div>

<style>
  .mod-stats { display: grid; grid-template-columns: repeat(4, 1fr); gap: 12px; margin-bottom: 20px; }
  .mini-stat { padding: 16px 18px; display: flex; flex-direction: column; gap: 6px; }
  .mini-stat-label { font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.4px; color: var(--text-ghost); }
  .mini-stat-value { font-size: 22px; font-weight: 800; letter-spacing: -0.4px; }
  @media (max-width: 900px) { .mod-stats { grid-template-columns: repeat(2, 1fr); } }
</style>
