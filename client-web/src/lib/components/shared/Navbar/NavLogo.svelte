<script lang="ts">
  import * as m from "$lib/paraglide/messages.js";
  import { getLocale } from "$lib/paraglide/runtime.js";
  import { i18n } from "$lib/i18n.js";

  let { isGlass } = $props<{ isGlass: boolean }>();

  function l(path: string) {
    return i18n.resolveRoute(path, getLocale());
  }
</script>

<a
  href={l("/")}
  class="flex items-center gap-3 group select-none shrink-0"
  aria-label={m.meta_siteName()}
>
  <img
    src="/logo.webp"
    alt={m.meta_siteName()}
    class="h-12 md:h-[4.5rem] lg:h-[4.5rem] w-auto object-contain transition-transform duration-300 group-hover:scale-105"
    decoding="async"
    loading="eager"
  />
  <div class="leading-none flex flex-col justify-center">
    <span
      class="block font-display text-lg md:text-xl font-bold transition-colors duration-300 {isGlass
        ? 'text-white'
        : 'text-[var(--color-secondary)]'}"
    >
      {m.auto_zafaf()}
    </span>
    <span
      class="block text-[10px] md:text-[11px] text-[var(--color-primary)] font-bold uppercase tracking-[0.15em] mt-0.5"
    >
      {m.auto_world()}
    </span>
  </div>
</a>
