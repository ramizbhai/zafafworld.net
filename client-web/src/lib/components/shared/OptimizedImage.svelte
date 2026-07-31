<script lang="ts">
  import { getResponsiveSources } from '$lib/shared/utils/media.js';

  let {
    src = '',
    alt = '',
    className = '',
    loading = 'lazy',
    decoding = 'async',
    fetchpriority = 'auto',
    size = 'original'
  } = $props();

  let sources = $derived(getResponsiveSources(src));
</script>

{#if sources}
  <picture class={className}>
    <!-- AVIF Source -->
    <source srcset={sources.avif[size]} type="image/avif" />
    <!-- WebP Fallback Source -->
    <source srcset={sources.webp[size]} type="image/webp" />
    <!-- Default Fallback Image -->
    <img
      src={sources.fallback}
      {alt}
      class={className}
      {loading}
      {decoding}
      fetchpriority={fetchpriority !== 'auto' ? fetchpriority : undefined}
    />
  </picture>
{:else}
  <!-- Fallback if url is empty -->
  <img {src} {alt} class={className} {loading} {decoding} />
{/if}
