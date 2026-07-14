<script lang="ts">
  import type { Snippet } from 'svelte';
  
  type Variant = 'primary' | 'secondary' | 'danger' | 'ghost' | 'gold';
  type Size = 'sm' | 'md' | 'lg' | 'icon';

  let {
    variant = 'primary',
    size = 'md',
    loading = false,
    disabled = false,
    class: className = '',
    onclick,
    children,
    type = 'button',
    ...rest
  }: {
    variant?: Variant;
    size?: Size;
    loading?: boolean;
    disabled?: boolean;
    class?: string;
    onclick?: (e: MouseEvent) => void;
    children?: Snippet;
    type?: 'button' | 'submit' | 'reset';
    [key: string]: any;
  } = $props();

  const baseClasses = "inline-flex items-center justify-center gap-[7px] border-none font-semibold font-inherit leading-none whitespace-nowrap cursor-pointer relative overflow-hidden select-none transition-all duration-[var(--dur-fast)] ease-[var(--ease-smooth)] focus-visible:outline-2 focus-visible:outline-[var(--gold)] focus-visible:outline-offset-2 disabled:opacity-45 disabled:cursor-not-allowed disabled:pointer-events-none";

  const sizeClasses = {
    sm: "px-[13px] py-[6px] text-[12.5px] rounded-[7px]",
    md: "px-[18px] py-[9px] text-[13.5px] rounded-[var(--radius-sm)]",
    lg: "px-[24px] py-[11px] text-[14.5px] rounded-[var(--radius-md)]",
    icon: "w-[34px] h-[34px] p-0 rounded-[var(--radius-sm)] border border-[var(--glass-border)]"
  };

  const variantClasses = {
    primary: "bg-gradient-to-br from-[var(--color-primary)] to-[var(--color-primary-hover)] text-white shadow-[0_4px_14px_-4px_rgba(91,33,182,0.30)] hover:-translate-y-[1px] hover:shadow-[0_6px_20px_-6px_rgba(91,33,182,0.45)] active:translate-y-0",
    secondary: "bg-transparent border-[1.5px] border-solid border-[var(--glass-border-hover)] text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:border-[rgba(255,255,255,0.12)] hover:text-[var(--text-primary)]",
    danger: "bg-gradient-to-br from-[hsl(0,80%,58%)] to-[hsl(0,84%,48%)] text-white shadow-[0_4px_14px_-4px_hsl(0,84%,60%,0.30)] hover:-translate-y-[1px]",
    ghost: "bg-transparent border-[1.5px] border-solid border-transparent text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)]",
    gold: "bg-gradient-to-br from-[hsl(40,96%,58%)] to-[hsl(36,90%,46%)] text-[#0b0f1a] font-bold shadow-[0_4px_14px_-4px_hsl(40,96%,58%,0.35)] hover:-translate-y-[1px] hover:shadow-[0_6px_20px_-6px_hsl(40,96%,58%,0.50)]"
  };

  const finalVariant = size === 'icon' && variant === 'primary' ? 'ghost' : variant; // fallback for icon default

</script>

<button
  {type}
  class="{baseClasses} {sizeClasses[size]} {size === 'icon' ? (variant === 'primary' ? 'bg-[var(--glass-sm)] text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:border-[var(--glass-border-hover)] hover:text-[var(--text-primary)]' : variantClasses[finalVariant]) : variantClasses[finalVariant]} {className}"
  disabled={disabled || loading}
  onclick={onclick}
  {...rest}
>
  {#if loading}
    <svg class="animate-spin h-4 w-4 mr-1 text-current" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
    </svg>
  {/if}
  {@render children?.()}
</button>
