<script lang="ts">
  import type { Snippet } from 'svelte';
  
  type Variant = 'gold' | 'success' | 'warning' | 'danger' | 'info' | 'purple' | 'muted';
  type Shape = 'default' | 'pill';
  type Dot = boolean;

  let {
    variant = 'muted',
    shape = 'default',
    dot = false,
    class: className = '',
    children,
    ...rest
  }: {
    variant?: Variant;
    shape?: Shape;
    dot?: Dot;
    class?: string;
    children?: Snippet;
    [key: string]: any;
  } = $props();

  const baseClasses = "inline-flex items-center gap-[5px] whitespace-nowrap tracking-[0.1px]";
  
  const shapeClasses = {
    default: "px-[9px] py-[3px] rounded-[var(--radius-full)] text-[11.5px] font-semibold",
    pill: "px-[7px] py-[2px] rounded-[var(--radius-full)] text-[10px] font-[750] tracking-[0.6px] uppercase"
  };

  const variantClasses = {
    gold: "bg-[var(--gold-subtle)] text-[var(--gold-bright)] border border-solid border-[var(--gold-border)]",
    success: "bg-[var(--success-dim)] text-[hsl(142,69%,58%)] border border-solid border-[var(--success-border)]",
    warning: "bg-[var(--warning-dim)] text-[hsl(38,92%,65%)] border border-solid border-[var(--warning-border)]",
    danger: "bg-[var(--danger-dim)] text-[hsl(0,84%,72%)] border border-solid border-[var(--danger-border)]",
    info: "bg-[var(--info-dim)] text-[hsl(217,91%,72%)] border border-solid border-[var(--info-border)]",
    purple: "bg-[var(--purple-dim)] text-[hsl(258,90%,78%)] border border-solid border-[var(--purple-border)]",
    muted: "bg-[rgba(255,255,255,0.04)] text-[var(--text-tertiary)] border border-solid border-[var(--glass-border)]"
  };

  const dotColors = {
    gold: "bg-[var(--gold)]",
    success: "bg-[var(--success)]",
    warning: "bg-[var(--warning)]",
    danger: "bg-[var(--danger)]",
    info: "bg-[var(--info)]",
    purple: "bg-[var(--purple)]",
    muted: "bg-[var(--text-ghost)]"
  };

</script>

<span class="{baseClasses} {shapeClasses[shape]} {variantClasses[variant]} {className}" {...rest}>
  {#if dot}
    <span class="w-[5px] h-[5px] rounded-full shrink-0 {dotColors[variant]}" aria-hidden="true"></span>
  {/if}
  {@render children?.()}
</span>
