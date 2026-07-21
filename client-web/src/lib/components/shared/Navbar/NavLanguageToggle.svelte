<script lang="ts">
  import { getLocale } from "$lib/paraglide/runtime.js";
  import * as m from "$lib/paraglide/messages.js";
  import { i18n } from "$lib/i18n.js";
  import type { NavbarState } from "$lib/stores/navbarState.svelte.js";

  let { isGlass, state, isMobile = false } = $props<{ isGlass: boolean; state: NavbarState; isMobile?: boolean }>();

  function toggleLanguage() {
    const current = getLocale();
    const next = current === "ar" ? "en" : "ar";
    const canonicalPath = i18n.route(location.pathname);
    const targetPath = i18n.resolveRoute(canonicalPath, next);
    const url = new URL(location.href);
    url.pathname = targetPath;

    // Set cookie explicitly for both paraglide_lang and PARAGLIDE_LOCALE
    document.cookie = `paraglide_lang=${next}; path=/; max-age=31536000; SameSite=Lax`;
    document.cookie = `PARAGLIDE_LOCALE=${next}; path=/; max-age=31536000; SameSite=Lax`;

    window.location.href = url.href;
    if (isMobile) {
      state.closeMenu();
    }
  }
</script>

{#if isMobile}
  <button
    onclick={toggleLanguage}
    class="flex items-center gap-3 px-4 py-3 rounded-xl text-xs font-semibold text-[var(--color-secondary)] bg-[var(--color-surface-alt)] hover:bg-[var(--color-border)] border border-[var(--color-border)] transition-all w-full text-start cursor-pointer select-none"
  >
    <span class="text-base">🌐</span>
    <div class="flex-1 flex justify-between items-center">
      <span>{m.auto_key_466()}</span>
      <span class="text-[10px] text-[var(--color-muted)] font-bold">{m.auto_key_87018()}</span>
    </div>
  </button>
{:else}
  <button
    onclick={toggleLanguage}
    class="
      flex items-center gap-1.5 px-3 py-2 rounded-xl text-xs font-semibold
      transition-all duration-300 border cursor-pointer select-none
      {isGlass
      ? 'text-[#C9A96E] border-[#C9A96E]/30 bg-[#1A1612]/30 hover:border-[#C9A96E]/50 backdrop-blur-sm'
      : getLocale() === 'en'
        ? 'text-[#065F46] border-[#34D399]/40 bg-[#A7F3D0]/20 hover:bg-[#A7F3D0]/40'
        : 'text-[var(--color-text)] border-[var(--color-border)] hover:border-[var(--color-primary)] hover:bg-[var(--color-surface-alt)]'}
    "
    aria-label={m.a11y_toggleLanguage()}
  >
    <span class="text-base" aria-hidden="true">{m.auto_key_75459()}</span>
    <span>{m.auto_key_98860()}</span>
  </button>
{/if}
