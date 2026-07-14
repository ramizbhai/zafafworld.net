<script lang="ts">
  import type { Snippet } from 'svelte';

  type CardVariant = 'default' | 'gold';

  let {
    variant = 'default',
    hoverEffect = false,
    class: className = '',
    header,
    footer,
    children,
    ...rest
  }: {
    variant?: CardVariant;
    hoverEffect?: boolean;
    class?: string;
    header?: Snippet;
    footer?: Snippet;
    children?: Snippet;
    [key: string]: any;
  } = $props();

  const baseClasses = "rounded-[var(--radius-lg)] backdrop-blur-[20px] transition-all duration-[var(--dur-base)] ease-[var(--ease-smooth)] overflow-hidden";
  
  const variantClasses = {
    default: "bg-[var(--glass-sm)] border border-solid border-[var(--glass-border)] shadow-[var(--shadow-sm)]",
    gold: "border border-solid border-[var(--gold-border)] bg-gradient-to-br from-[var(--bg-elevated)] to-[rgba(40,30,10,0.5)]"
  };

  const hoverClasses = {
    default: "hover:border-[var(--glass-border-hover)] hover:shadow-[var(--shadow-md)]",
    gold: "hover:shadow-[var(--shadow-gold)]",
    lift: "hover:-translate-y-[2px] hover:shadow-[var(--shadow-lg)]"
  };

</script>

<div 
  class="{baseClasses} {variantClasses[variant]} {hoverClasses[variant]} {hoverEffect ? hoverClasses.lift : ''} {className}" 
  {...rest}
>
  {#if header}
    <div class="px-[24px] py-[16px] border-b border-solid border-[var(--glass-border)] flex items-center justify-between gap-[12px]">
      {@render header()}
    </div>
  {/if}
  
  <div class="px-[24px] py-[20px]">
    {@render children?.()}
  </div>

  {#if footer}
    <div class="px-[24px] py-[14px] border-t border-solid border-[var(--glass-border)]">
      {@render footer()}
    </div>
  {/if}
</div>
