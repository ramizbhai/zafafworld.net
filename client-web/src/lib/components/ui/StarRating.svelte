<script lang="ts">
  interface Props {
    rating?: number | null;
    max?: number;
    size?: 'sm' | 'md' | 'lg';
    showValue?: boolean;
    reviewCount?: number;
    label?: string;
  }

  let { rating = 0, max = 5, size = 'md', showValue = false, reviewCount, label }: Props = $props();
  const safeRating = $derived(rating ?? 0);


  const sizes = { sm: 'text-sm', md: 'text-base', lg: 'text-lg' };

  function getStarType(index: number): 'full' | 'half' | 'empty' {
    const val = safeRating - index;
    if (val >= 1) return 'full';
    if (val >= 0.5) return 'half';
    return 'empty';
  }
</script>

<div class="inline-flex items-center gap-1.5" aria-label={label ?? `${safeRating} out of ${max} stars`} role="img">
  <span class="flex items-center gap-0.5 {sizes[size]}">
    {#each Array(max) as _, i}
      {@const type = getStarType(i)}
      <svg
        viewBox="0 0 20 20"
        class="w-[1em] h-[1em] flex-shrink-0"
        aria-hidden="true"
      >
        {#if type === 'full'}
          <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" fill="var(--color-primary)"/>
        {:else if type === 'half'}
          <defs>
            <clipPath id="half-{i}">
              <rect x="0" y="0" width="10" height="20"/>
            </clipPath>
          </defs>
          <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" fill="var(--color-border)"/>
          <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" fill="var(--color-primary)" clip-path="url(#half-{i})"/>
        {:else}
          <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" fill="var(--color-border)"/>
        {/if}
      </svg>
    {/each}
  </span>

  {#if showValue}
    <span class="font-semibold text-[var(--color-text)] {sizes[size]}">{safeRating.toFixed(1)}</span>
  {/if}

  {#if reviewCount !== undefined}
    <span class="text-[var(--color-muted)] text-sm">({reviewCount})</span>
  {/if}
</div>
