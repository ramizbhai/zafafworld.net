<script lang="ts">
  import type { Snippet } from 'svelte';

  type SectionBg = 'surface' | 'surface-alt' | 'surface-dark';

  interface Props {
    bg?: SectionBg;
    class?: string;
    children: Snippet;
    id?: string;
    ariaLabelledby?: string;
  }

  let {
    bg = 'surface',
    class: extraClass = '',
    children,
    id,
    ariaLabelledby,
  }: Props = $props();

  const bgClasses: Record<SectionBg, string> = {
    surface: 'bg-zw-surface text-zw-secondary',
    'surface-alt': 'bg-zw-surface-alt text-zw-secondary',
    'surface-dark': 'bg-zw-surface-dark text-zw-surface border-t border-zw-secondary',
  };

  const baseClasses = 'py-zw-16 md:py-zw-24 relative overflow-hidden';
  const containerClasses = 'w-full max-w-[1400px] mx-auto px-zw-4 sm:px-zw-6 md:px-zw-8 lg:px-zw-12';

  const mergedClasses = $derived([
    baseClasses,
    bgClasses[bg],
    extraClass,
  ].filter(Boolean).join(' '));
</script>

<section {id} aria-labelledby={ariaLabelledby} class={mergedClasses}>
  <div class={containerClasses}>
    {@render children()}
  </div>
</section>
