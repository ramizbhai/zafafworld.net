<script lang="ts">
  import type { Snippet } from 'svelte';

  type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'outline' | 'danger';
  type ButtonSize = 'sm' | 'md' | 'lg';

  interface Props {
    variant?: ButtonVariant;
    size?: ButtonSize;
    type?: 'button' | 'submit' | 'reset';
    disabled?: boolean;
    loading?: boolean;
    fullWidth?: boolean;
    href?: string;
    class?: string;
    onclick?: (e: MouseEvent) => void;
    children: Snippet;
  }

  let {
    variant = 'primary',
    size = 'md',
    type = 'button',
    disabled = false,
    loading = false,
    fullWidth = false,
    href,
    class: extraClass = '',
    onclick,
    children,
  }: Props = $props();

  const base = 'inline-flex items-center justify-center gap-2 font-medium rounded-lg transition-all duration-200 focus-visible:outline-2 focus-visible:outline-offset-2 disabled:opacity-50 disabled:cursor-not-allowed select-none cursor-pointer';

  const variants: Record<ButtonVariant, string> = {
    primary: 'bg-primary text-secondary hover:bg-primary-dark focus-visible:outline-primary shadow-gold',
    secondary: 'bg-secondary text-inverse hover:bg-[#1a0e0a] focus-visible:outline-secondary',
    ghost: 'bg-transparent text-text hover:bg-surface-alt focus-visible:outline-primary',
    outline: 'bg-transparent border border-primary-contrast text-primary-contrast hover:bg-primary-contrast hover:text-inverse focus-visible:outline-primary-contrast',
    danger: 'bg-error text-white hover:opacity-90 focus-visible:outline-error',
  };

  const sizes: Record<ButtonSize, string> = {
    sm: 'text-sm px-4 py-2',
    md: 'text-base px-6 py-3',
    lg: 'text-lg px-8 py-4',
  };

  const classes = $derived([
    base,
    variants[variant],
    sizes[size],
    fullWidth ? 'w-full' : '',
    extraClass,
  ].filter(Boolean).join(' '));
</script>

{#if href}
  <a {href} class={classes} aria-disabled={disabled}>
    {@render children()}
  </a>
{:else}
  <button {type} disabled={disabled || loading} class={classes} {onclick}>
    {@render children()}
  </button>
{/if}
