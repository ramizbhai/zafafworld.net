<script lang="ts">
  import type { Snippet } from 'svelte';

  type ButtonVariant = 'primary' | 'secondary' | 'accent-rose' | 'accent-teal' | 'outline' | 'ghost';
  type ButtonSize = 'sm' | 'md' | 'lg';

  interface Props {
    variant?: ButtonVariant;
    size?: ButtonSize;
    type?: 'button' | 'submit' | 'reset';
    disabled?: boolean;
    loading?: boolean;
    fullWidth?: boolean;
    href?: string;
    target?: string;
    rel?: string;
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
    target,
    rel,
    class: extraClass = '',
    onclick,
    children,
  }: Props = $props();

  const baseClasses = 'inline-flex items-center justify-center gap-zw-2 font-medium rounded-zw-full transition-all duration-200 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 disabled:opacity-50 disabled:cursor-not-allowed select-none cursor-pointer';

  const variantClasses: Record<ButtonVariant, string> = {
    primary: 'bg-zw-primary text-zw-secondary hover:bg-zw-primary-dark focus-visible:outline-zw-primary shadow-gold',
    secondary: 'bg-zw-secondary text-zw-surface hover:bg-zw-surface-dark focus-visible:outline-zw-secondary',
    'accent-rose': 'bg-zw-interactive-rose text-zw-surface hover:bg-zw-interactive-rose-hover focus-visible:outline-zw-interactive-rose',
    'accent-teal': 'bg-zw-interactive-teal text-zw-surface hover:bg-zw-interactive-teal-hover focus-visible:outline-zw-interactive-teal',
    outline: 'bg-transparent border border-zw-primary-contrast text-zw-primary-contrast hover:bg-zw-primary-contrast hover:text-zw-surface focus-visible:outline-zw-primary-contrast',
    ghost: 'bg-transparent text-zw-secondary hover:bg-zw-surface-alt focus-visible:outline-zw-secondary',
  };

  const sizeClasses: Record<ButtonSize, string> = {
    sm: 'text-zw-xs px-zw-4 py-zw-2',
    md: 'text-zw-sm px-zw-6 py-zw-3',
    lg: 'text-zw-base px-zw-8 py-zw-4',
  };

  const mergedClasses = $derived([
    baseClasses,
    variantClasses[variant],
    sizeClasses[size],
    fullWidth ? 'w-full' : '',
    extraClass,
  ].filter(Boolean).join(' '));
</script>

{#if href}
  <a
    {href}
    {target}
    {rel}
    class={mergedClasses}
    aria-disabled={disabled || loading}
    tabindex={disabled || loading ? -1 : 0}
  >
    {@render children()}
  </a>
{:else}
  <button
    {type}
    disabled={disabled || loading}
    class={mergedClasses}
    {onclick}
    aria-busy={loading}
  >
    {@render children()}
  </button>
{/if}
