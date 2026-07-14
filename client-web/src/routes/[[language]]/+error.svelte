<script lang="ts">
  import { page } from '$app/stores';
  import * as m from '$lib/paraglide/messages.js';
import { getLocale } from '$lib/paraglide/runtime.js';
import { getLocalizedField } from '$lib/utils/localize.js';
  import Button from '$lib/components/ui/Button.svelte';

  const status = $derived($page.status || 500);
  const error = $derived($page.error);
  
  // Safe bilingually resolved messages
  const title = $derived(
    status === 404
      ? (getLocale() === 'ar' ? 'الصفحة غير موجودة' : 'Page Not Found')
      : (getLocale() === 'ar' ? 'خطأ في الخادم' : 'Server Error')
  );
  
  const description = $derived(
    status === 404
      ? (getLocale() === 'ar' ? 'عذراً، الصفحة التي تبحث عنها غير موجودة.' : 'Sorry, the page you are looking for does not exist.')
      : (getLocale() === 'ar' ? 'حدث خطأ غير متوقع. يرجى المحاولة لاحقاً.' : 'An unexpected error occurred. Please try again later.')
  );

  const errorId = $derived((error as any)?.errorId || '');

  let copied = $state(false);

  function copyErrorId() {
    if (!errorId) return;
    navigator.clipboard.writeText(errorId).then(() => {
      copied = true;
      setTimeout(() => {
        copied = false;
      }, 2000);
    });
  }
</script>

<svelte:head>
  <title>{title} — {m.meta_siteName()}</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center bg-[var(--color-surface)] px-4 py-12" dir={getLocale() === 'ar' ? 'rtl' : 'ltr'}>
  <div class="w-full max-w-xl bg-white rounded-2xl shadow-[var(--shadow-md)] border border-[var(--color-border)] p-8 text-center relative overflow-hidden">
    <!-- Premium luxury layout borders -->
    <div class="absolute top-0 inset-x-0 h-1.5 bg-gradient-to-r from-[var(--color-primary-light)] via-[var(--color-primary)] to-[var(--color-primary-dark)]"></div>

    <!-- Error status numeric display -->
    <div class="font-display text-7xl font-light text-[var(--color-primary)] opacity-50 tracking-widest mb-4">
      {status}
    </div>

    <!-- Heart decoration -->
    <div class="inline-flex items-center justify-center w-16 h-16 rounded-full bg-[var(--color-surface-alt)] text-[var(--color-primary-dark)] mb-6">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
        <path d="M19 14c1.49-1.46 3-3.21 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.76 0-3 .5-4.5 2-1.5-1.5-2.74-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.3 1.5 4.05 3 5.5l7 7Z"/>
      </svg>
    </div>

    <!-- Message -->
    <h1 class="font-display text-2xl md:text-3xl font-semibold text-[var(--color-secondary)] mb-3">
      {title}
    </h1>
    <p class="text-sm text-[var(--color-muted)] leading-relaxed max-w-md mx-auto mb-8">
      {description}
    </p>

    <!-- Support Card -->
    {#if errorId}
      <div class="bg-[var(--color-surface-alt)]/55 border border-[var(--color-border)] rounded-xl p-4 mb-8 text-start max-w-md mx-auto flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 transition-smooth">
        <div>
          <span class="text-[10px] font-bold text-[var(--color-muted)] block uppercase tracking-wider mb-1">
            {m.auto_technical_support_id()}
          </span>
          <code class="font-mono text-sm font-bold text-[var(--color-secondary)] select-all">
            {errorId}
          </code>
        </div>
        <button
          type="button"
          onclick={copyErrorId}
          class="text-xs px-3.5 py-2 font-medium bg-white border border-[var(--color-border)] rounded-lg text-[var(--color-secondary)] hover:border-[var(--color-primary)] hover:text-[var(--color-primary-dark)] transition-colors select-none shrink-0 cursor-pointer shadow-sm"
        >
          {copied 
            ? (m.auto_copied()) 
            : (m.auto_copy_id())}
        </button>
      </div>
    {/if}

    <!-- Action -->
    <div class="flex items-center justify-center gap-4">
      <Button href="/" variant="primary" size="md">
        {m.errors_goHome()}
      </Button>
    </div>
  </div>
</div>
