<script lang="ts">
  import type { Snippet } from 'svelte';

  type BadgeVariant = 'default' | 'primary' | 'success' | 'warning' | 'error' | 'muted';

  interface Props {
    variant?: BadgeVariant;
    size?: 'sm' | 'md';
    class?: string;
    children: Snippet;
  }

  let { variant = 'default', size = 'md', class: extraClass = '', children }: Props = $props();

  const variants: Record<BadgeVariant, string> = {
    default:  'bg-[var(--color-surface-alt)] text-[var(--color-text)]',
    primary:  'bg-[var(--color-primary)] text-[var(--color-secondary)]',
    success:  'bg-green-100 text-green-800',
    warning:  'bg-amber-100 text-amber-800',
    error:    'bg-red-100 text-red-800',
    muted:    'bg-[var(--color-border)] text-[var(--color-muted)]',
  };

  const sizes = {
    sm: 'text-xs px-2 py-0.5',
    md: 'text-sm px-3 py-1',
  };
</script>

<span class="inline-flex items-center font-medium rounded-full {variants[variant]} {sizes[size]} {extraClass}">
  {@render children()}
</span>
