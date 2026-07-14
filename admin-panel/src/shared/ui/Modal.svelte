<script lang="ts">
  import type { Snippet } from 'svelte';
  
  let {
    open = false,
    title = '',
    onClose,
    children
  }: {
    open: boolean;
    title?: string;
    onClose: () => void;
    children?: Snippet;
  } = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && open) {
      onClose();
    }
  }

  // Focus trap logic could be added using a svelte action, 
  // but for a pure UI component without external deps, we implement a simple version or just rely on a11y standards.
  let modalRef = $state<HTMLDivElement>();
  
  $effect(() => {
    if (open && modalRef) {
      modalRef.focus();
    }
  });
</script>

<svelte:window on:keydown={handleKeydown} />

{#if open}
  <!-- Backdrop -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div 
    class="fixed inset-0 bg-black/40 backdrop-blur-sm z-[9998] transition-opacity duration-[var(--dur-base)]"
    role="button"
    tabindex="-1"
    onclick={onClose}
    aria-label="Close modal backdrop"
  ></div>

  <!-- Modal Dialog -->
  <div 
    bind:this={modalRef}
    class="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-full max-w-lg bg-[var(--bg-base)] border border-solid border-[var(--glass-border)] rounded-[var(--radius-xl)] shadow-[var(--shadow-xl)] z-[9999] overflow-hidden outline-none flex flex-col max-h-[90vh]"
    role="dialog"
    aria-modal="true"
    aria-labelledby={title ? 'modal-title' : undefined}
    tabindex="-1"
  >
    <!-- Header -->
    <div class="px-[24px] py-[18px] border-b border-solid border-[var(--glass-border)] flex items-center justify-between gap-[12px] bg-[var(--bg-elevated)]">
      {#if title}
        <h2 id="modal-title" class="text-[18px] font-bold text-[var(--text-primary)] m-0 leading-tight">
          {title}
        </h2>
      {:else}
        <div></div> <!-- Spacer -->
      {/if}
      <button 
        type="button"
        class="w-[30px] h-[30px] rounded-[var(--radius-sm)] border-none bg-transparent text-[var(--text-secondary)] hover:bg-[var(--bg-hover)] hover:text-[var(--text-primary)] flex items-center justify-center cursor-pointer transition-colors"
        onclick={onClose}
        aria-label="Close modal"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="18" y1="6" x2="6" y2="18"></line>
          <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
      </button>
    </div>

    <!-- Body -->
    <div class="px-[24px] py-[20px] overflow-y-auto">
      {@render children?.()}
    </div>
  </div>
{/if}
