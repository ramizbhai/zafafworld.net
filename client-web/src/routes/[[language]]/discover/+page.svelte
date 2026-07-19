<script lang="ts">
  import * as m from '$lib/paraglide/messages.js';
  import { getLocale } from '$lib/paraglide/runtime.js';
  import { formatDate } from '$lib/utils/localize.js';
  import Badge from '$lib/components/ui/Badge.svelte';
  import { i18n } from '$lib/i18n.js';
  import { resolveMediaUrl, getOptimizedImage } from '$lib/shared/utils/media.js';

  import { untrack } from 'svelte';

  let { data }: { data: any } = $props();

  let currentPosts = $state(untrack(() => data.posts ?? []));
  let currentPage = $state(1);
  let hasMore = $state(untrack(() => (data.posts ?? []).length === 12));
  let isLoading = $state(false);

  $effect(() => {
    const posts = data.posts ?? [];
    currentPosts = posts;
    currentPage = 1;
    hasMore = posts.length === 12;
  });

  const featuredPost = $derived(currentPosts.length > 0 ? currentPosts[0] : null);
  const restPosts = $derived(currentPosts.length > 1 ? currentPosts.slice(1) : []);

  async function loadMore() {
    if (isLoading || !hasMore) return;
    isLoading = true;
    try {
      const nextPage = currentPage + 1;
      const lang = getLocale();
      const res = await fetch(`/api/v1/public/blogs?lang=${lang}&page=${nextPage}&limit=12`);
      if (res.ok) {
        const json = await res.json();
        if (json.status === 'success' && Array.isArray(json.data) && json.data.length > 0) {
          currentPosts = [...currentPosts, ...json.data];
          currentPage = nextPage;
          hasMore = json.data.length === 12;
        } else {
          hasMore = false;
        }
      } else {
        hasMore = false;
      }
    } catch (e) {
      console.error(e);
      hasMore = false;
    } finally {
      isLoading = false;
    }
  }

  function l(path: string) {
    return i18n.resolveRoute(path, getLocale());
  }
</script>

<svelte:head>
  <title>{m.blog_title()} - {m.meta_siteName()}</title>
  <meta name="description" content={m.blog_subtitle()} />
</svelte:head>

<!-- Header -->
<div class="bg-[var(--color-surface-alt)] border-b border-[var(--color-border)]">
  <div class="container-page py-12">
    <span class="divider-gold"></span>
    <h1 class="font-display text-3xl sm:text-4xl font-bold text-[var(--color-secondary)] mt-4 mb-2">
      {m.blog_title()}
    </h1>
    <p class="text-[var(--color-muted)]">{m.blog_subtitle()}</p>
  </div>
</div>

<div class="container-page py-12">
  {#if featuredPost}

      <!-- Featured post -->
      <article class="relative rounded-3xl overflow-hidden mb-12 border border-[var(--color-border)] bg-white/95 backdrop-blur-sm shadow-md hover:shadow-2xl hover:-translate-y-1.5 transition-all duration-300 group" aria-label="Featured post">
        <div class="grid grid-cols-1 lg:grid-cols-12 gap-0">
          <div class="lg:col-span-7 relative h-80 sm:h-96 lg:h-[420px] overflow-hidden bg-[var(--color-surface-alt)]">
            <img
              src={featuredPost.cover_image_url ? resolveMediaUrl(getOptimizedImage(featuredPost.cover_image_url, 'medium')) : '/images/fallbacks/default-cover.svg'}
              alt={featuredPost.title}
              class="w-full h-full object-cover transition-transform duration-700 group-hover:scale-103"
              fetchpriority="high"
              onerror={(e) => { (e.currentTarget as HTMLImageElement).src = '/images/fallbacks/default-cover.svg'; }}
            />
            <div class="absolute inset-0 bg-gradient-to-t from-black/50 via-transparent to-transparent lg:hidden"></div>
          </div>
          
          <div class="lg:col-span-5 flex flex-col justify-center p-8 lg:p-10 gap-4">
            <div class="flex items-center gap-3">
              <Badge variant="primary" size="sm">Featured Article</Badge>
              <span class="text-xs text-[var(--color-muted)]">📅 {formatDate(featuredPost.published_at || featuredPost.created_at)}</span>
            </div>
            
            <h2 class="font-display text-2xl sm:text-3xl font-bold text-[var(--color-secondary)] group-hover:text-[var(--color-primary-contrast)] transition-colors duration-200 leading-tight">
              <a href={l("/discover/" + featuredPost.slug)}>
                {featuredPost.title}
              </a>
            </h2>
            
            <p class="text-[var(--color-muted)] text-sm leading-relaxed line-clamp-3">
              {featuredPost.excerpt}
            </p>
            
            <div class="flex items-center gap-4 text-xs text-[var(--color-muted)]">
              <span>⏱ {featuredPost.read_time_minutes || 5} {m.blog_minRead()}</span>
              <span>•</span>
              <span>By {featuredPost.author || 'Zafaf World Team'}</span>
            </div>
            
            <div class="pt-2">
              <a href={l("/discover/" + featuredPost.slug)} class="inline-flex items-center gap-2 px-5 py-2.5 rounded-full bg-[var(--color-primary)] text-[var(--color-secondary)] font-bold text-xs hover:bg-[var(--color-primary-dark)] transition-colors shadow-sm">
                {m.blog_readMore()}
                <span>→</span>
              </a>
            </div>
          </div>
        </div>
      </article>
    {/if}

    {#if restPosts.length > 0}
      <!-- Post grid -->
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-8">
        {#each restPosts as post (post.id)}
          <article class="group flex flex-col rounded-2xl overflow-hidden border border-[var(--color-border)] bg-white/95 backdrop-blur-sm shadow-sm hover:shadow-2xl hover:-translate-y-1.5 transition-all duration-300 card-hover">
            <div class="relative h-56 overflow-hidden bg-[var(--color-surface-alt)]">
              <img
                src={post.cover_image_url ? resolveMediaUrl(getOptimizedImage(post.cover_image_url, 'card')) : '/images/fallbacks/default-cover.svg'}
                alt={post.title}
                class="w-full h-full object-cover transition-transform duration-700 group-hover:scale-105"
                loading="lazy"
                onerror={(e) => { (e.currentTarget as HTMLImageElement).src = '/images/fallbacks/default-cover.svg'; }}
              />
              <div class="absolute top-4 start-4">
                <Badge variant="muted" size="sm">Article</Badge>
              </div>
            </div>
            <div class="flex flex-col flex-1 p-6 gap-3">
              <div class="flex items-center gap-2 text-xs text-[var(--color-muted)]">
                <span>📅 {formatDate(post.published_at || post.created_at)}</span>
                <span>•</span>
                <span>⏱ {post.read_time_minutes || 5} {m.blog_minRead()}</span>
              </div>
              
              <h3 class="font-display text-xl font-bold text-[var(--color-secondary)] leading-snug group-hover:text-[var(--color-primary-contrast)] transition-colors duration-200">
                <a href={l("/discover/" + post.slug)}>
                  {post.title}
                </a>
              </h3>
              
              <p class="text-sm text-[var(--color-muted)] leading-relaxed line-clamp-3 flex-grow">
                {post.excerpt}
              </p>
              
              <div class="pt-4 border-t border-[var(--color-border)]/60 flex items-center justify-between mt-auto">
                <span class="text-xs text-[var(--color-muted)]">By {post.author || 'Zafaf World Team'}</span>
                <a href={l("/discover/" + post.slug)} class="text-xs font-semibold text-[var(--color-primary-contrast)] hover:text-[var(--color-primary-dark)] flex items-center gap-1 group/link transition-colors duration-200">
                  {m.blog_readMore()}
                  <span class="inline-block transition-transform duration-200 group-hover/link:translate-x-1 rtl:group-hover/link:-translate-x-1">→</span>
                </a>
              </div>
            </div>
          </article>
        {/each}
      </div>
    {/if}

    {#if hasMore}
      <div class="mt-12 text-center">
        <button
          class="px-6 py-3 rounded-full bg-[var(--color-primary)] text-[var(--color-secondary)] font-bold text-sm hover:bg-[var(--color-primary-dark)] transition-colors shadow-sm disabled:opacity-70 disabled:cursor-not-allowed inline-flex items-center justify-center min-w-[200px]"
          onclick={loadMore}
          disabled={isLoading}
        >
          {#if isLoading}
            <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-current" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path></svg>
            Loading...
          {:else}
            Load More Articles
          {/if}
        </button>
      </div>
    {/if}

    {#if !featuredPost && restPosts.length === 0}
      <div class="text-center py-24 bg-white border border-[var(--color-border)] rounded-3xl">
        <div class="text-5xl mb-4">✍️</div>
        <h3 class="font-display text-xl font-bold text-[var(--color-secondary)] mb-2">
          {m.auto_no_blog_posts_availa()}
        </h3>
        <p class="text-sm text-[var(--color-muted)]">Check back later for new articles and wedding inspiration.</p>
      </div>
    {/if}
</div>

