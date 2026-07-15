<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import * as m from '$lib/paraglide/messages.js';
  import { getLocale } from '$lib/paraglide/runtime.js';
  import { formatDate } from '$lib/utils/localize.js';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Afrah from '$lib/components/ui/Afrah.svelte';
  import { env } from '$env/dynamic/public';
  import { i18n } from '$lib/i18n.js';
  import { resolveMediaUrl, getOptimizedImage } from '$lib/shared/utils/media.js';

  let { data } = $props();

  const post = $derived(data.post);
  const related = $derived(data.related || []);

  const lang = $derived(getLocale());

  const activeTitle = $derived(lang === 'ar' ? (post.title_ar || post.title) : (post.title_en || post.title));
  const activeMetaTitle = $derived(lang === 'ar' ? (post.meta_title_ar || post.meta_title || post.title_ar || post.title) : (post.meta_title_en || post.meta_title || post.title_en || post.title));
  const activeMetaDesc = $derived(lang === 'ar' ? (post.meta_description_ar || post.meta_description || post.excerpt) : (post.meta_description_en || post.meta_description || post.excerpt));

  let parsedBlocks = $derived.by(() => {
    if (!post.content_html) return [];
    try {
      const parsed = JSON.parse(post.content_html);
      if (Array.isArray(parsed)) return parsed;
      return [];
    } catch (e) {
      return []; // Return empty array if parsing fails (e.g. WordPress raw HTML) to fallback to {@html}
    }
  });

  function l(path: string) {
    return i18n.resolveRoute(path, getLocale());
  }

  const comments = $derived(post.comments || []);
  const rootComments = $derived(comments.filter((c: any) => !c.parent_id));
  const getReplies = $derived((parentId: string) => comments.filter((c: any) => c.parent_id === parentId));

  let commentContent = $state('');
  let commentStatus = $state('');
  let isSubmitting = $state(false);

  let activeReplyId = $state('');
  let replyContent = $state('');
  let replyStatus = $state('');
  let isReplying = $state(false);

  onMount(() => {
    // Record view
    fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/public/blogs/${post.slug}/view`, {
      method: 'POST'
    }).catch(console.error);
  });

  async function submitComment(e: Event, parentId: string | null = null) {
    e.preventDefault();
    const content = parentId ? replyContent : commentContent;
    if (!content.trim()) return;
    
    if (parentId) {
      isReplying = true;
    } else {
      isSubmitting = true;
    }

    try {
      const res = await fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/public/blogs/${post.slug}/comments`, {
        method: 'POST',
        headers: { 
          'Content-Type': 'application/json',
          ...(data.user?.session?.access_token ? { 'Authorization': `Bearer ${data.user.session.access_token}` } : {})
        },
        body: JSON.stringify({ 
          comment: content,
          parent_id: parentId
        })
      });
      
      if (res.ok) {
        if (parentId) {
          replyStatus = 'Reply submitted successfully and is awaiting moderation.';
          replyContent = '';
          activeReplyId = '';
        } else {
          commentStatus = 'Comment submitted successfully and is awaiting moderation.';
          commentContent = '';
        }
      } else {
        const errData = await res.json().catch(() => ({}));
        const msg = errData.message || 'You must be logged in to comment.';
        if (parentId) {
          replyStatus = msg;
        } else {
          commentStatus = msg;
        }
      }
    } catch (err) {
      if (parentId) {
        replyStatus = 'Failed to submit reply.';
      } else {
        commentStatus = 'Failed to submit comment.';
      }
    } finally {
      isSubmitting = false;
      isReplying = false;
    }
  }
</script>

<svelte:head>
  <title>{activeMetaTitle}</title>
  <meta name="description" content={activeMetaDesc} />
  {#if post.focus_keywords}
    <meta name="keywords" content={post.focus_keywords} />
  {/if}
  
  <!-- Canonical tag -->
  <link rel="canonical" href={post.canonical_url || `${$page.url.origin}${$page.url.pathname}`} />

  <!-- Open Graph / Facebook -->
  <meta property="og:type" content="article" />
  <meta property="og:url" content={`${$page.url.origin}${$page.url.pathname}`} />
  <meta property="og:title" content={activeMetaTitle} />
  <meta property="og:description" content={activeMetaDesc} />
  {#if post.cover_image_url}
    <meta property="og:image" content={resolveMediaUrl(post.cover_image_url)} />
  {/if}

  <!-- Twitter -->
  <meta property="twitter:card" content="summary_large_image" />
  <meta property="twitter:url" content={`${$page.url.origin}${$page.url.pathname}`} />
  <meta property="twitter:title" content={activeMetaTitle} />
  <meta property="twitter:description" content={activeMetaDesc} />
  {#if post.cover_image_url}
    <meta property="twitter:image" content={resolveMediaUrl(post.cover_image_url)} />
  {/if}

  <!-- JSON-LD for SEO -->
  <script type="application/ld+json">
    {JSON.stringify({
      "@context": "https://schema.org",
      "@type": "BlogPosting",
      "headline": activeTitle,
      "image": post.cover_image_url ? resolveMediaUrl(post.cover_image_url) : undefined,
      "datePublished": post.created_at,
      "description": post.excerpt,
      "author": {
        "@type": "Organization",
        "name": "Zafaf World"
      }
    })}
  </script>
</svelte:head>

<article class="container-page pt-8 pb-12" dir={post.lang === 'ar' ? 'rtl' : 'ltr'}>
  <div class="mb-6">
    <a href={l('/discover')} class="inline-flex items-center gap-2 text-[var(--color-muted)] hover:text-[var(--color-primary)] transition-colors font-medium">
      <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" class="w-5 h-5 rtl:rotate-180">
        <path stroke-linecap="round" stroke-linejoin="round" d="M10.5 19.5 3 12m0 0 7.5-7.5M3 12h18" />
      </svg>
      {getLocale() === 'ar' ? 'عودة' : 'Back'}
    </a>
  </div>

  <div class="mb-10 rounded-[2rem] overflow-hidden aspect-video shadow-lg bg-[var(--color-surface-alt)]">
    <img
      src={post.cover_image_url ? resolveMediaUrl(getOptimizedImage(post.cover_image_url, 'large')) : '/images/fallbacks/default-cover.svg'}
      alt={activeTitle}
      class="w-full h-full object-cover"
      onerror={(e) => { (e.currentTarget as HTMLImageElement).src = '/images/fallbacks/default-cover.svg'; }}
    />
  </div>

  <header class="mb-12 text-center">
    <Badge variant="primary" class="mb-6">Article</Badge>
    <h1 class="text-4xl sm:text-5xl lg:text-6xl font-display font-bold text-[var(--color-secondary)] mb-6 leading-tight">
      {activeTitle}
    </h1>
    <div class="flex items-center justify-center gap-6 text-[var(--color-muted)] text-sm font-medium">
      <span class="flex items-center gap-2">📅 {formatDate(post.created_at)}</span>
      <span class="flex items-center gap-2">⏱ {post.read_time_minutes || 5} min read</span>
      <span class="flex items-center gap-2">👁 {post.views_count} views</span>
    </div>
  </header>

  <div class="blog-content-blocks">
    {#each parsedBlocks as block}
      {#if block.type === 'heading' && block.content}
        <h2 class="text-3xl font-display font-bold text-[var(--color-secondary)] mt-12 mb-6 leading-tight">{block.content}</h2>
      {:else if block.type === 'subheading' && block.content}
        <h3 class="text-2xl font-display font-bold text-[var(--color-secondary)] mt-8 mb-4 leading-snug">{block.content}</h3>
      {:else if block.type === 'text' && block.content}
        <p class="text-lg text-[var(--color-text)] mb-6 leading-relaxed whitespace-pre-wrap">{block.content}</p>
      {:else if block.type === 'list' && block.content}
        <ul class="list-disc list-inside text-lg text-[var(--color-text)] space-y-3 mb-8 pl-4">
          {#each block.content.split('\n') as item}
            {#if item.trim()}
              <li>{item}</li>
            {/if}
          {/each}
        </ul>
      {:else if block.type === 'image' && block.url}
        <figure class="my-10">
          <img src={resolveMediaUrl(getOptimizedImage(block.url, 'medium'))} alt="" loading="lazy" class="w-full rounded-2xl shadow-md border border-gray-100 object-cover" />
        </figure>
      {:else if block.type === 'gallery' && block.url}
        <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-4 my-10">
          {#each block.url.split(/[\n,]/).map((u: string) => u.trim()).filter(Boolean) as imageUrl}
            <div class="aspect-video rounded-xl overflow-hidden shadow-sm border border-gray-100">
              <img src={resolveMediaUrl(getOptimizedImage(imageUrl, 'card'))} alt="" loading="lazy" class="w-full h-full object-cover hover:scale-105 transition-transform duration-500" />
            </div>
          {/each}
        </div>
      {:else if block.type === 'image_text' && block.url}
        <div class="flex flex-col md:flex-row gap-8 items-center my-12 bg-gray-50/50 p-6 rounded-3xl border border-gray-100">
          {#if block.layout === 'right'}
            <div class="flex-1 text-lg text-[var(--color-text)] leading-relaxed whitespace-pre-wrap order-2 md:order-1">{block.content || ''}</div>
            <div class="w-full md:w-1/2 rounded-2xl overflow-hidden shadow-md order-1 md:order-2 shrink-0">
              <img src={resolveMediaUrl(getOptimizedImage(block.url, 'medium'))} alt="" loading="lazy" class="w-full h-full object-cover aspect-[4/3]" />
            </div>
          {:else}
            <div class="w-full md:w-1/2 rounded-2xl overflow-hidden shadow-md shrink-0">
              <img src={resolveMediaUrl(getOptimizedImage(block.url, 'medium'))} alt="" loading="lazy" class="w-full h-full object-cover aspect-[4/3]" />
            </div>
            <div class="flex-1 text-lg text-[var(--color-text)] leading-relaxed whitespace-pre-wrap">{block.content || ''}</div>
          {/if}
        </div>
      {:else if block.type === 'divider'}
        <hr class="my-12 border-t-2 border-dashed border-gray-200 w-2/3 mx-auto" />
      {/if}
    {/each}
    {#if parsedBlocks.length === 0 && post.content_html}
      <div class="prose prose-lg prose-gold max-w-none">
        {@html post.content_html}
      </div>
    {/if}
  </div>

  <Afrah class="my-16" />

  <hr class="my-12 border-[var(--color-border)]" />

  <!-- Comments Section -->
  <section class="mb-16">
    <h3 class="text-2xl font-display font-bold mb-6">Comments</h3>
    
    {#if data.user}
      <form onsubmit={(e) => submitComment(e, null)} class="mb-8">
        <div class="mb-4">
          <textarea bind:value={commentContent} placeholder="Leave a comment..." class="w-full p-4 border border-[var(--color-border)] rounded-xl focus:ring-2 focus:ring-[var(--color-primary)] outline-none resize-y min-h-[100px]" required></textarea>
        </div>
        <div class="flex items-center justify-between">
          <button type="submit" class="bg-[var(--color-primary)] text-white px-6 py-2 rounded-full font-semibold hover:bg-[var(--color-primary-dark)] transition-colors" disabled={isSubmitting}>
            {isSubmitting ? 'Posting...' : 'Post Comment'}
          </button>
          {#if commentStatus}
            <span class="text-sm font-medium {commentStatus.includes('successfully') ? 'text-green-600' : 'text-red-600'}">{commentStatus}</span>
          {/if}
        </div>
      </form>
    {:else}
      <div class="p-6 bg-[var(--color-surface-alt)] border border-[var(--color-border)] rounded-2xl text-center mb-8">
        <p class="text-[var(--color-muted)] mb-3">You must be logged in to leave comments or replies.</p>
        <a href="/auth/login" class="inline-block bg-[var(--color-primary)] text-white px-6 py-2 rounded-full font-semibold hover:bg-[var(--color-primary-dark)] transition-colors">
          Log In
        </a>
      </div>
    {/if}

    <div class="space-y-6">
      {#if rootComments.length > 0}
        {#each rootComments as comment (comment.id)}
          {@const replies = getReplies(comment.id)}
          <div class="p-5 bg-[var(--color-surface-alt)] rounded-xl border border-[var(--color-border)]">
            <div class="flex justify-between items-start mb-2">
              <span class="font-bold text-[var(--color-secondary)]">{comment.name}</span>
              <span class="text-xs text-[var(--color-muted)]">{formatDate(comment.created_at)}</span>
            </div>
            <p class="text-sm mb-3">{comment.comment}</p>
            
            <!-- Replies List -->
            {#if replies.length > 0}
              <div class="mt-4 pl-4 border-l-2 border-[var(--color-primary)] space-y-4">
                {#each replies as reply (reply.id)}
                  <div class="p-3 bg-white/70 rounded-lg border border-[var(--color-border)] shadow-sm">
                    <div class="flex justify-between items-start mb-1">
                      <span class="font-bold text-xs text-[var(--color-secondary)]">{reply.name}</span>
                      <span class="text-xs text-[var(--color-muted)]">{formatDate(reply.created_at)}</span>
                    </div>
                    <p class="text-xs text-[var(--color-text)]">{reply.comment}</p>
                  </div>
                {/each}
              </div>
            {/if}

            <!-- Reply Form Trigger & Form -->
            {#if data.user}
              <div class="mt-3">
                {#if activeReplyId === comment.id}
                  <form onsubmit={(e) => submitComment(e, comment.id)} class="mt-3">
                    <textarea bind:value={replyContent} placeholder="Write a reply..." class="w-full p-3 text-sm border border-[var(--color-border)] rounded-lg focus:ring-2 focus:ring-[var(--color-primary)] outline-none resize-y min-h-[70px]" required></textarea>
                    <div class="flex items-center justify-between mt-2">
                      <div class="flex gap-2">
                        <button type="submit" class="bg-[var(--color-primary)] text-white text-xs px-4 py-1.5 rounded-full font-semibold hover:bg-[var(--color-primary-dark)] transition-colors" disabled={isReplying}>
                          {isReplying ? 'Posting...' : 'Post Reply'}
                        </button>
                        <button type="button" class="bg-transparent text-[var(--color-muted)] text-xs px-3 py-1.5 rounded-full hover:bg-white/50 transition-colors" onclick={() => { activeReplyId = ''; replyContent = ''; }}>
                          Cancel
                        </button>
                      </div>
                      {#if replyStatus && activeReplyId === comment.id}
                        <span class="text-xs font-medium {replyStatus.includes('successfully') ? 'text-green-600' : 'text-red-600'}">{replyStatus}</span>
                      {/if}
                    </div>
                  </form>
                {:else}
                  <button type="button" class="text-xs text-[var(--color-primary)] font-semibold hover:underline" onclick={() => { activeReplyId = comment.id; replyContent = ''; replyStatus = ''; }}>
                    Reply
                  </button>
                {/if}
              </div>
            {/if}
          </div>
        {/each}
      {:else}
        <p class="text-[var(--color-muted)]">No comments yet. Be the first to comment!</p>
      {/if}
    </div>
  </section>

  <!-- Related Content -->
  {#if related.length > 0}
    <section>
      <h3 class="text-2xl font-display font-bold mb-6">Related Articles</h3>
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-6">
        {#each related as rel}
          <a href={l("/discover/" + rel.slug)} class="group block border border-[var(--color-border)] rounded-2xl overflow-hidden hover:shadow-lg transition-shadow">
            <div class="aspect-video bg-[var(--color-surface-alt)] overflow-hidden">
              <img
                src={rel.cover_image_url ? resolveMediaUrl(getOptimizedImage(rel.cover_image_url, 'card')) : '/images/fallbacks/default-cover.svg'}
                alt={rel.title}
                class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500"
                onerror={(e) => { (e.currentTarget as HTMLImageElement).src = '/images/fallbacks/default-cover.svg'; }}
              />
            </div>
            <div class="p-4">
              <h4 class="font-bold text-lg mb-2 group-hover:text-[var(--color-primary)] transition-colors">{rel.title}</h4>
              <p class="text-sm text-[var(--color-muted)] line-clamp-2">{rel.excerpt}</p>
            </div>
          </a>
        {/each}
      </div>
    </section>
  {/if}
</article>

<!-- Mobile Sticky CTA -->
<div class="fixed bottom-0 left-0 right-0 z-50 p-4 bg-white/90 backdrop-blur-md border-t border-[var(--color-border)] shadow-[0_-4px_10px_rgba(0,0,0,0.05)] sm:hidden transform transition-transform duration-300">
  <div class="max-w-md mx-auto flex items-center justify-between gap-4">
    <div class="flex-1 text-sm font-bold text-[var(--color-secondary)]">
      Ready to start planning?
    </div>
    <a 
      href="/afrah?utm_source=blog&utm_campaign={post.slug}"
      onclick={() => {
        fetch(`${env.PUBLIC_API_URL || 'http://localhost:8080'}/api/v1/public/blogs/${post.slug}/track`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ event_type: 'cta_click_afrah' })
        }).catch(console.error);
      }}
      class="bg-[var(--color-primary)] text-white px-5 py-2.5 rounded-full font-bold whitespace-nowrap shadow-md active:scale-95 transition-transform"
    >
      Chat with Afrah
    </a>
  </div>
</div>

<style>
  /* Base prose styling */
  :global(.prose-gold h2) {
    font-family: var(--font-display);
    font-size: 1.8rem;
    font-weight: 700;
    margin-top: 2rem;
    margin-bottom: 1rem;
    color: var(--color-secondary);
  }
  :global(.prose-gold p) {
    margin-bottom: 1.5rem;
    line-height: 1.8;
    color: var(--color-text);
  }
  :global(.prose-gold blockquote) {
    border-left: 4px solid var(--color-primary);
    padding-left: 1rem;
    font-style: italic;
    color: var(--color-muted);
  }
  :global(.prose-gold img) {
    border-radius: 1rem;
    margin: 2rem 0;
  }
</style>
