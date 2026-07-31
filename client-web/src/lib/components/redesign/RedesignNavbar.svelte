<script lang="ts">
  import { onMount } from "svelte";
  import { getLocale } from "$lib/paraglide/runtime.js";
  import * as m from "$lib/paraglide/messages.js";
  import { i18n } from "$lib/i18n.js";
  import Button from "./Button.svelte";

  interface Props {
    user?: any;
    class?: string;
  }

  let { user = null, class: extraClass = "" }: Props = $props();

  let isScrolled = $state(false);
  let isMenuOpen = $state(false);
  let scrollY = $state(0);

  onMount(() => {
    const handleScroll = () => {
      isScrolled = window.scrollY > 20;
      scrollY = window.scrollY;
    };
    window.addEventListener("scroll", handleScroll, { passive: true });
    handleScroll();
    return () => window.removeEventListener("scroll", handleScroll);
  });

  const isRtl = $derived(getLocale() === "ar");

  function l(path: string) {
    return i18n.resolveRoute(path, getLocale());
  }

  function toggleLanguage() {
    const current = getLocale();
    const next = current === "ar" ? "en" : "ar";
    const canonicalPath = i18n.route(window.location.pathname);
    const targetPath = i18n.resolveRoute(canonicalPath, next);
    const url = new URL(window.location.href);
    url.pathname = targetPath;

    document.cookie = `paraglide_lang=${next}; path=/; max-age=31536000; SameSite=Lax`;
    document.cookie = `PARAGLIDE_LOCALE=${next}; path=/; max-age=31536000; SameSite=Lax`;

    window.location.href = url.href;
  }

  const links = $derived([
    { href: l("/"), label: isRtl ? "الرئيسية" : "Home" },
    { href: l("/venues"), label: isRtl ? "قاعات الأفراح" : "Venues" },
    { href: l("/discover"), label: isRtl ? "اكتشف" : "Discover" },
    { href: l("/offers"), label: isRtl ? "العروض" : "Offers" },
    { href: l("/about"), label: isRtl ? "من نحن" : "About" },
    { href: l("/contact"), label: isRtl ? "اتصل بنا" : "Contact" }
  ]);

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && isMenuOpen) {
      isMenuOpen = false;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<header
  class="fixed top-0 inset-x-0 z-[100] transition-all duration-300 border-b
  {isScrolled
    ? 'bg-zw-surface/90 backdrop-blur-md border-zw-border/50 shadow-zw-md py-zw-3'
    : 'bg-[#1A1612]/15 backdrop-blur-[6px] border-white/10 py-zw-4'} {extraClass}"
>
  <div class="w-full max-w-[1400px] mx-auto px-zw-4 sm:px-zw-6 md:px-zw-8 lg:px-zw-12 flex items-center justify-between">
    <!-- Brand Logo -->
    <a href={l("/")} class="flex items-center gap-zw-2 group" aria-label="ZafafWorld Home">
      <span class="text-zw-primary text-zw-2xl select-none group-hover:scale-110 transition-transform">★</span>
      <span class="font-display text-zw-xl font-bold tracking-wide transition-colors
        {isScrolled ? 'text-zw-secondary' : 'text-zw-surface'}"
      >
        Zafaf<span class="text-zw-primary">World</span>
      </span>
    </a>

    <!-- Desktop Navigation Menu -->
    <nav class="hidden lg:flex items-center gap-zw-6" aria-label="Main Navigation">
      <ul class="flex items-center gap-zw-6">
        {#each links as link}
          <li>
            <a
              href={link.href}
              class="font-body text-zw-xs font-semibold tracking-wide transition-colors pb-zw-1 border-b-2 border-transparent hover:border-zw-primary
              {isScrolled ? 'text-zw-secondary' : 'text-zw-surface/90 hover:text-zw-surface'}"
            >
              {link.label}
            </a>
          </li>
        {/each}
      </ul>
    </nav>

    <!-- Header Actions -->
    <div class="hidden lg:flex items-center gap-zw-4">
      <!-- Language Toggle -->
      <button
        onclick={toggleLanguage}
        class="flex items-center gap-zw-1.5 px-zw-3 py-zw-2 rounded-zw-xl text-zw-xs font-semibold transition-all duration-300 border cursor-pointer select-none
        {isScrolled
          ? 'text-zw-secondary border-zw-border hover:bg-zw-surface-alt'
          : 'text-zw-primary border-zw-primary/35 bg-white/10 hover:bg-white/20 backdrop-blur-sm'}"
        aria-label={isRtl ? "تغيير اللغة إلى الإنجليزية" : "Switch language to Arabic"}
      >
        <span aria-hidden="true">🌐</span>
        <span>{isRtl ? "English" : "العربية"}</span>
      </button>

      <!-- Business Portal CTA Button -->
      <Button
        href="https://vendor.zafafworld.net"
        target="_blank"
        rel="noopener noreferrer"
        variant="primary"
        size="sm"
        class="text-zw-xs font-bold hover:scale-105 transition-transform"
      >
        <span>{isRtl ? "منصة شركاء الأعمال" : "Business Portal"}</span>
      </Button>
    </div>

    <!-- Mobile Navigation Toggle -->
    <button
      class="lg:hidden p-zw-2 rounded-zw-md transition-colors cursor-pointer select-none
      {isScrolled ? 'hover:bg-zw-surface-alt text-zw-secondary' : 'hover:bg-white/10 text-zw-surface'}"
      onclick={() => isMenuOpen = !isMenuOpen}
      aria-label={isMenuOpen ? "Close main navigation menu" : "Open main navigation menu"}
      aria-expanded={isMenuOpen}
      aria-controls="mobile-nav-menu"
    >
      <svg viewBox="0 0 24 24" class="w-6 h-6" fill="none" stroke="currentColor" stroke-width="2">
        {#if isMenuOpen}
          <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
        {:else}
          <path stroke-linecap="round" stroke-linejoin="round" d="M4 6h16M4 12h16M4 18h16" />
        {/if}
      </svg>
    </button>
  </div>

  <!-- Mobile Menu Drawer (Slide-down overlay) -->
  {#if isMenuOpen}
    <div
      id="mobile-nav-menu"
      class="lg:hidden absolute top-full left-0 right-0 bg-zw-surface border-b border-zw-border shadow-zw-xl p-zw-6 flex flex-col gap-zw-6 z-50 animate-fade-in"
      role="dialog"
      aria-modal="true"
      aria-label="Mobile Navigation"
    >
      <nav aria-label="Mobile Navigation Links">
        <ul class="flex flex-col gap-zw-4">
          {#each links as link}
            <li>
              <a
                href={link.href}
                onclick={() => isMenuOpen = false}
                class="font-body text-zw-sm font-semibold text-zw-secondary hover:text-zw-primary transition-colors block py-zw-1"
              >
                {link.label}
              </a>
            </li>
          {/each}
        </ul>
      </nav>

      <hr class="border-zw-border/50" />

      <div class="flex flex-col gap-zw-3">
        <!-- Mobile Language Selector -->
        <button
          onclick={toggleLanguage}
          class="flex items-center gap-zw-3 px-zw-4 py-zw-3 rounded-zw-xl text-zw-xs font-semibold text-zw-secondary bg-zw-surface-alt hover:bg-zw-border border border-zw-border transition-all w-full text-start cursor-pointer select-none"
        >
          <span class="text-base" aria-hidden="true">🌐</span>
          <div class="flex-grow flex justify-between items-center">
            <span>{isRtl ? "English" : "العربية"}</span>
            <span class="text-zw-xs text-zw-muted font-bold">{isRtl ? "Switch to English" : "تغيير للعربية"}</span>
          </div>
        </button>

        <!-- Mobile Partner Portal Callout -->
        <Button
          href="https://vendor.zafafworld.net"
          target="_blank"
          rel="noopener noreferrer"
          variant="primary"
          size="md"
          class="w-full text-center"
          onclick={() => isMenuOpen = false}
        >
          <span>{isRtl ? "منصة شركاء الأعمال" : "Business Portal"}</span>
        </Button>
      </div>
    </div>
  {/if}
</header>
