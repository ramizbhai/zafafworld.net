<script lang="ts">
  import { getResponsiveSources } from '$lib/shared/utils/media.js';

  // Svelte 5 props definition
  let {
    src = '',
    alt = '',
    className = '',
    sizes = '100vw',
    loading = 'lazy' as 'lazy' | 'eager' | undefined | null,
    decoding = 'async' as 'async' | 'sync' | 'auto' | undefined | null,
    fetchpriority = 'auto' as 'high' | 'low' | 'auto' | undefined | null,
    width = undefined as number | string | undefined,
    height = undefined as number | string | undefined,
    aspectRatio = undefined as string | undefined,
    style = ''
  } = $props();

  let loaded = $state(false);
  let error = $state(false);

  let sources = $derived(getResponsiveSources(src));

  // Determine preload attributes if fetchpriority is high
  let preloadHref = $derived(sources?.avif?.original || sources?.fallback || src);
  let preloadSrcset = $derived(
    sources
      ? `${sources.avif.thumb} 150w, ${sources.avif.card} 400w, ${sources.avif.medium} 800w, ${sources.avif.large} 1200w, ${sources.avif.original} 1920w`
      : undefined
  );

  let webpSrcset = $derived(
    sources
      ? `${sources.webp.thumb} 150w, ${sources.webp.card} 400w, ${sources.webp.medium} 800w, ${sources.webp.large} 1200w, ${sources.webp.original} 1920w`
      : undefined
  );

  let avifSrcset = $derived(
    sources
      ? `${sources.avif.thumb} 150w, ${sources.avif.card} 400w, ${sources.avif.medium} 800w, ${sources.avif.large} 1200w, ${sources.avif.original} 1920w`
      : undefined
  );

  let fallbackSrc = $derived(sources?.fallback || src);

  // Style helper for preventing CLS and reserving space
  let computedStyle = $derived(
    [
      aspectRatio ? `aspect-ratio: ${aspectRatio};` : '',
      width ? `width: ${typeof width === 'number' ? width + 'px' : width};` : '',
      height ? `height: ${typeof height === 'number' ? height + 'px' : height};` : '',
      style
    ]
      .filter(Boolean)
      .join(' ')
  );
</script>

<svelte:head>
  {#if fetchpriority === 'high' && src}
    <link
      rel="preload"
      as="image"
      href={preloadHref}
      imagesrcset={preloadSrcset}
      imagesizes={sizes}
      {fetchpriority}
    />
  {/if}
</svelte:head>

<div
  class="relative overflow-hidden {className}"
  style={computedStyle}
>
  {#if error}
    <!-- Error Fallback State -->
    <div class="absolute inset-0 flex flex-col items-center justify-center bg-slate-100 dark:bg-slate-800 text-slate-400">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        fill="none"
        viewBox="0 0 24 24"
        stroke-width="1.5"
        stroke="currentColor"
        class="w-8 h-8 mb-2 opacity-50"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          d="m2.25 15.75 5.159-5.159a2.25 2.25 0 0 1 3.182 0l5.159 5.159m-1.5-1.5 1.409-1.409a2.25 2.25 0 0 1 3.182 0l2.909 2.909m-18 3.75h16.5a1.5 1.5 0 0 0 1.5-1.5V6a1.5 1.5 0 0 0-1.5-1.5H3.75A1.5 1.5 0 0 0 2.25 6v12a1.5 1.5 0 0 0 1.5 1.5Zm10.5-11.25h.008v.008h-.008V8.25Zm.375 0a.375 0 1 1-.75 0 .375 0 0 1 .75 0Z"
        />
      </svg>
      <span class="text-xs font-medium">Image unavailable</span>
    </div>
  {:else}
    <!-- Blur Placeholder / Loading Pulse Skeleton -->
    {#if !loaded}
      <div class="absolute inset-0 bg-slate-100 dark:bg-slate-800 animate-pulse flex items-center justify-center">
        {#if sources?.webp?.thumb}
          <img
            src={sources.webp.thumb}
            alt=""
            class="w-full h-full object-cover blur-2xl scale-110 opacity-70 pointer-events-none"
          />
        {/if}
      </div>
    {/if}

    <!-- High-Performance Picture Tag -->
    {#if sources}
      <picture class="w-full h-full">
        <!-- AVIF responsive sources -->
        <source
          type="image/avif"
          srcset={avifSrcset}
          {sizes}
        />
        <!-- WebP fallback sources -->
        <source
          type="image/webp"
          srcset={webpSrcset}
          {sizes}
        />
        <!-- Fallback Raw Image -->
        <img
          src={fallbackSrc}
          {alt}
          class="w-full h-full object-cover transition-opacity duration-500 {loaded ? 'opacity-100' : 'opacity-0'}"
          {loading}
          {decoding}
          fetchpriority={fetchpriority !== 'auto' ? fetchpriority : undefined}
          {width}
          {height}
          onload={() => loaded = true}
          onerror={() => error = true}
        />
      </picture>
    {:else}
      <!-- Inline direct image tag fallback -->
      <img
        src={src}
        {alt}
        class="w-full h-full object-cover transition-opacity duration-500 {loaded ? 'opacity-100' : 'opacity-0'}"
        {loading}
        {decoding}
        {width}
        {height}
        onload={() => loaded = true}
        onerror={() => error = true}
      />
    {/if}
  {/if}
</div>
