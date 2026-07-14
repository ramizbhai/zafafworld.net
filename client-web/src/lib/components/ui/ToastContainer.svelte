<script lang="ts">
  import { toasts } from '$lib/stores/toast.svelte.js';
  import { fly } from 'svelte/transition';
</script>

<div class="fixed bottom-4 end-4 z-50 flex flex-col gap-2 pointer-events-none">
  {#each toasts.messages as toast (toast.id)}
    <div
      transition:fly={{ y: 20, duration: 300 }}
      class="pointer-events-auto flex items-center gap-3 min-w-[300px] max-w-md p-4 rounded-xl shadow-[var(--shadow-xl)] border border-[var(--color-border)] text-sm font-medium
      {toast.type === 'success' ? 'bg-green-50 text-green-800 border-green-200' : 'bg-red-50 text-red-800 border-red-200'}"
      role="alert"
    >
      <div class="flex-shrink-0">
        {#if toast.type === 'success'}
          <svg class="w-5 h-5 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
          </svg>
        {:else}
          <svg class="w-5 h-5 text-red-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
          </svg>
        {/if}
      </div>
      <p class="flex-1">{toast.message}</p>
      <button
        type="button"
        class="opacity-50 hover:opacity-100 transition-opacity"
        onclick={() => { toasts.messages = toasts.messages.filter(m => m.id !== toast.id); }}
        aria-label="Close"
      >
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>
  {/each}
</div>
