<script lang="ts">
  import { page } from "$app/stores";
  import * as m from "$lib/paraglide/messages.js";
  import { getLocale } from "$lib/paraglide/runtime.js";
  import Button from "$lib/components/ui/Button.svelte";
  
  import { createNavbarState } from "$lib/stores/navbarState.svelte.js";
  
  import NavLogo from "./Navbar/NavLogo.svelte";
  import NavSearch from "./Navbar/NavSearch.svelte";
  import NavLanguageToggle from "./Navbar/NavLanguageToggle.svelte";
  import NavLinks from "./Navbar/NavLinks.svelte";
  import NavMobileDrawer from "./Navbar/NavMobileDrawer.svelte";

  let { user = null } = $props();

  let state = createNavbarState();

  $effect(() => {
    if (typeof window === "undefined") return;
    const handler = () => {
      const heroEl = document.getElementById("home-hero-section");
      if (heroEl) {
        state.isScrolled = window.scrollY > heroEl.offsetHeight - 120;
      } else {
        state.isScrolled = window.scrollY > 20;
      }
    };
    window.addEventListener("scroll", handler, { passive: true });
    handler();
    return () => window.removeEventListener("scroll", handler);
  });

  const isGlass = $derived(
    !state.isScrolled &&
      ($page.route.id === "/[[language]]" ||
        $page.url.pathname === "/" ||
        $page.url.pathname === "/ar" ||
        $page.url.pathname === "/en"),
  );
</script>

<a
  href="#main-content"
  class="sr-only focus:not-sr-only focus:absolute focus:top-4 focus:start-4 z-50 bg-[var(--color-primary)] text-[var(--color-secondary)] px-4 py-2 rounded-lg font-medium"
>
  {m.a11y_skipToContent()}
</a>

<header
  class="
    fixed top-0 inset-x-0 z-[var(--z-overlay)] transition-all duration-300
    {isGlass
    ? 'bg-[#1A1612]/15 backdrop-blur-[6px] border-b border-white/10 shadow-sm'
    : 'glass-header border-b border-[var(--color-border)]/50 shadow-[var(--shadow-md)]'}
  "
>
  <!-- ROW 1: PRIMARY HEADER BAR -->
  <div class="container-page border-b {isGlass ? 'border-transparent' : 'border-white/10'}">
    <div class="flex items-center justify-between h-20 gap-4">
      
      <!-- LOGO -->
      <NavLogo {isGlass} />

      <!-- CENTERED SEARCH WIDGET (Desktop) -->
      <NavSearch {state} {isGlass} isMobile={false} />

      <!-- SIGN IN & ACTIONS (Desktop) -->
      <div class="hidden lg:flex items-center gap-3 shrink-0">
        
        <!-- Language Switcher -->
        <NavLanguageToggle {isGlass} {state} isMobile={false} />

        <!-- Business Portal CTA -->
        <Button
          href="https://vendor.zafafworld.net"
          variant="primary"
          size="sm"
          class="px-4 py-2 text-xs font-semibold bg-[#C9A96E] hover:bg-[#B5965C] text-[#2D2620] border-none shadow-[var(--shadow-gold)] flex items-center gap-1.5 transition-transform hover:scale-105"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="20" height="14" x="2" y="7" rx="2" ry="2" /><path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16" /></svg>
          <span>{getLocale() === "ar" ? "منصة شركاء الأعمال" : "Business Portal"}</span>
        </Button>
      </div>

      <!-- Mobile Menu Trigger -->
      <button
        class="lg:hidden p-2 rounded-lg transition-colors {isGlass ? 'hover:bg-white/10' : 'hover:bg-[var(--color-surface-alt)]'} cursor-pointer shrink-0"
        onclick={state.toggleMenu}
        aria-label={state.isMenuOpen ? m.a11y_closeMenu() : m.a11y_openMenu()}
        aria-expanded={state.isMenuOpen}
        aria-controls="mobile-menu"
      >
        <svg viewBox="0 0 24 24" class="w-6 h-6 {isGlass ? 'text-white' : 'text-[var(--color-text)]'}" fill="none" stroke="currentColor" stroke-width="2">
          {#if state.isMenuOpen}
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
          {:else}
            <path stroke-linecap="round" stroke-linejoin="round" d="M4 6h16M4 12h16M4 18h16" />
          {/if}
        </svg>
      </button>
    </div>
  </div>

  <!-- ROW 2: UNIFIED NAVIGATION RIBBON -->
  <NavLinks {state} {isGlass} />

  <!-- MOBILE DRAWER CONTAINER -->
  {#if state.isMenuOpen}
    <NavMobileDrawer {state} />
  {/if}
</header>

<style>
  :global(.glass-header) {
    background: rgba(253, 250, 246, 0.97) !important;
    backdrop-filter: blur(20px) !important;
    -webkit-backdrop-filter: blur(20px) !important;
    border-bottom: 1px solid rgba(201, 169, 110, 0.25) !important;
  }

  :global(.glass-dropdown) {
    background: rgba(253, 250, 246, 0.98) !important;
    backdrop-filter: blur(24px) !important;
    -webkit-backdrop-filter: blur(24px) !important;
    border: 1px solid rgba(201, 169, 110, 0.25) !important;
    box-shadow: 0 16px 48px rgba(45, 38, 32, 0.2) !important;
  }
</style>
