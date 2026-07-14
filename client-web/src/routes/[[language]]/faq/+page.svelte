<script lang="ts">
  import * as m from '$lib/paraglide/messages.js';
import { getLocale } from '$lib/paraglide/runtime.js';
import { getLocalizedField } from '$lib/utils/localize.js';

  interface FAQItem {
    id: number;
    qAr: string;
    qEn: string;
    aAr: string;
    aEn: string;
  }

  let { data }: { data: any } = $props();

  const faqs = $derived(data.faqs || []);

  let activeId = $state<number | null>(null);

  function toggle(id: number) {
    activeId = activeId === id ? null : id;
  }
</script>

<svelte:head>
  <title>{m.footer_faq()} — {m.meta_siteName()}</title>
</svelte:head>

<div class="bg-[var(--color-surface-alt)] min-h-screen">
  <!-- Hero -->
  <div class="bg-[var(--color-secondary)] text-white py-16 sm:py-20 relative overflow-hidden">
    <div class="absolute inset-0 opacity-10" aria-hidden="true">
      <div class="absolute top-0 start-0 w-80 h-80 rounded-full bg-[var(--color-primary)] -translate-x-1/4 -translate-y-1/4"></div>
    </div>
    <div class="container-page relative z-10">
      <span class="divider-gold"></span>
      <h1 class="font-display text-3xl sm:text-4xl font-bold mt-6 mb-4">
        {m.auto_frequently_asked_que()}
      </h1>
      <p class="text-sm sm:text-base text-white/70 max-w-2xl leading-relaxed">
        {m.auto_find_quick_precise_()}
      </p>
    </div>
  </div>

  <!-- FAQ List -->
  <div class="container-page py-12 sm:py-16 max-w-3xl">
    <div class="flex flex-col gap-4">
      {#each faqs as item}
        {@const isOpen = activeId === item.id}
        <div class="bg-white rounded-2xl border border-[var(--color-border)] overflow-hidden shadow-sm hover:shadow-md transition-shadow duration-300">
          <button
            type="button"
            class="w-full text-start px-6 py-5 flex items-center justify-between gap-4 font-semibold text-[var(--color-secondary)] hover:text-[var(--color-primary)] transition-colors focus:outline-none"
            onclick={() => toggle(item.id)}
            aria-expanded={isOpen}
          >
            <span class="text-sm sm:text-base leading-snug">
              {getLocalizedField(item.q, 'name', getLocale())}
            </span>
            <svg
              viewBox="0 0 20 20"
              class="w-5 h-5 flex-shrink-0 transform transition-transform duration-200 text-[var(--color-muted)] {isOpen ? 'rotate-180 text-[var(--color-primary)]' : ''}"
              fill="currentColor"
              aria-hidden="true"
            >
              <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clip-rule="evenodd"/>
            </svg>
          </button>
          
          {#if isOpen}
            <div class="px-6 pb-6 border-t border-[var(--color-surface-alt)] pt-4">
              <p class="text-sm sm:text-base text-[var(--color-muted)] leading-relaxed">
                {getLocalizedField(item.a, 'name', getLocale())}
              </p>
            </div>
          {/if}
        </div>
      {:else}
        <div class="text-center py-12 text-[var(--color-muted)]">
          {m.auto_faqs_are_currently_u()}
        </div>
      {/each}
    </div>
  </div>
</div>
