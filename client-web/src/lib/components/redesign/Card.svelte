<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    elevated?: boolean;
    class?: string;
    onclick?: (e: MouseEvent) => void;
    children: Snippet;
  }

  let {
    elevated = false,
    class: extraClass = '',
    onclick,
    children,
  }: Props = $props();

  const baseClasses = 'bg-zw-surface border border-zw-border rounded-zw-2xl p-zw-6 transition-all duration-300';
  const hoverClasses = 'hover:translate-y-[-4px] hover:shadow-zw-xl hover:border-zw-border-hover';

  const mergedClasses = $derived([
    baseClasses,
    elevated ? hoverClasses : '',
    extraClass,
  ].filter(Boolean).join(' '));
</script>

{#if onclick}
  <button
    class="{mergedClasses} text-start w-full cursor-pointer focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-zw-primary"
    {onclick}
  >
    {@render children()}
  </button>
{:else}
  <div class={mergedClasses}>
    {@render children()}
  </div>
{/if}
