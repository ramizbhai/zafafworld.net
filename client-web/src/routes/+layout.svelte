<script lang="ts">
  import "../app.css";
  import { ParaglideJS } from "@inlang/paraglide-sveltekit";
  import { i18n } from "$lib/i18n.js";
  import * as m from "$lib/paraglide/messages.js";
  import Navbar from "$lib/components/shared/Navbar.svelte";
  import Footer from "$lib/components/shared/Footer.svelte";
  import ToastContainer from "$lib/components/ui/ToastContainer.svelte";
  import { onMount, untrack } from "svelte";
  import { env } from "$env/dynamic/public";
  import type { Snippet } from "svelte";
  import { page, navigating } from "$app/stores";
  import { uiStore } from "$lib/stores/ui.svelte.js";
  import Loading from "$lib/components/Loading.svelte";

  import { authStore } from "$lib/stores/auth.svelte.js";
  import { countryStore } from "$lib/stores/country.svelte.js";
  import { browser } from "$app/environment";
  import { initAnalytics, trackPageView, GTAG_ID } from "$lib/services/analytics.service";

  // Derive language and direction from the URL — used for the #app div's dir
  // attribute (required for Tailwind's rtl:/ltr: variants) and for Open Graph
  // locale meta tags. The <html lang> and <html dir> are injected server-side
  // via Paraglide's %paraglide.lang% / %paraglide.textDirection% placeholders
  // in app.html — we must NOT override them client-side or we cause a flicker.
  const currentLang = $derived(i18n.getLanguageFromUrl($page.url));
  const currentDir = $derived(currentLang === "ar" ? "rtl" : "ltr");

  $effect(() => {
    const countryParam = $page.url.searchParams.get("country");
    if (countryParam) {
      countryStore.setCountryFromUrl(countryParam);
    }
  });

  $effect(() => {
    // Track page views on SPA navigation
    trackPageView($page.url.pathname + $page.url.search);
  });

  // ── SEO Routing ────────────────────────────────────────────────────────────
  // Constants moved to {@const} block in HTML to ensure SSR hydration

  interface Props {
    data: any;
    children: Snippet;
  }
  let { data, children }: Props = $props();

  // Instant hydration of authStore on client startup before mounting
  const initialData = untrack(() => data);
  if (browser && initialData && "user" in initialData) {
    if (initialData.user) {
      authStore.setUser({
        id: initialData.user.id,
        name: initialData.user.first_name
          ? `${initialData.user.first_name} ${initialData.user.last_name}`
          : initialData.user.email,
        avatar: initialData.user.avatar,
        isVip: initialData.user.isVip,
      });
    } else {
      authStore.setUser(null);
    }
  }

  $effect(() => {
    if (data && "user" in data) {
      if (data.user) {
        authStore.setUser({
          id: data.user.id,
          name: data.user.first_name
            ? `${data.user.first_name} ${data.user.last_name}`
            : data.user.email,
          avatar: data.user.avatar,
          isVip: data.user.isVip,
        });
      } else {
        authStore.setUser(null);
      }
    }
  });

  let sseMessage = $state("");

  onMount(() => {
    initAnalytics();

    // UTM Campaign tracking for Blog Funnel
    if (browser) {
      const url = new URL(window.location.href);
      const utmSource = url.searchParams.get("utm_source");
      const utmCampaign = url.searchParams.get("utm_campaign");
      if (utmSource === "blog" && utmCampaign) {
        sessionStorage.setItem("zafaf_blog_attribution", utmCampaign);
      }
    }

    // Only open SSE stream for authenticated users to avoid wasting server resources
    if (!data?.user) return;

    const API_BASE = env.PUBLIC_API_URL || "http://localhost:8080";
    let eventSource: EventSource | null = null;
    let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
    let attempt = 0;
    const MAX_ATTEMPTS = 10;
    const BASE_DELAY_MS = 1000;
    let destroyed = false;

    function connect() {
      if (destroyed) return;
      eventSource = new EventSource(`${API_BASE}/api/v1/events/stream`, {
        withCredentials: true,
      });

      eventSource.onopen = () => {
        attempt = 0; // Reset backoff on successful connection
      };

      eventSource.onmessage = (event) => {
        try {
          const payload = JSON.parse(event.data);
          if (
            payload.type === "INQ-" ||
            payload.message?.includes("INQ-") ||
            payload.type === "booking" ||
            payload.message
          ) {
            sseMessage = payload.message || "New update received!";
            setTimeout(() => {
              sseMessage = "";
            }, 5000);
          }
        } catch (e) {
          // Handle non-JSON or other errors silently
        }
      };

      eventSource.onerror = () => {
        eventSource?.close();
        eventSource = null;
        if (destroyed || attempt >= MAX_ATTEMPTS) return;
        // Exponential backoff: 1s, 2s, 4s, 8s, 16s … max 30s
        const delay = Math.min(BASE_DELAY_MS * Math.pow(2, attempt), 30_000);
        attempt++;
        reconnectTimer = setTimeout(connect, delay);
      };
    }

    connect();

    return () => {
      destroyed = true;
      if (reconnectTimer) clearTimeout(reconnectTimer);
      eventSource?.close();
    };
  });
</script>

<svelte:head>
  <title>{m.meta_siteName()} — {m.meta_tagline()}</title>
  <meta name="description" content={m.meta_description()} />
  <meta property="og:site_name" content={m.meta_siteName()} />
  <meta property="og:type" content="website" />
  <meta name="twitter:card" content="summary_large_image" />

  <!-- Google Analytics -->
  <script async src={`https://www.googletagmanager.com/gtag/js?id=${GTAG_ID}`}></script>

  <!-- Preload Critical Above-The-Fold Dropdown WebP Assets -->
  <link rel="preload" href="/categories/wedding-palace.webp" as="image" type="image/webp" />
  <link rel="preload" href="/categories/photography-video.webp" as="image" type="image/webp" />
  <link rel="preload" href="/categories/wedding-planner.webp" as="image" type="image/webp" />
  <link rel="preload" href="/categories/wedding-invitation.webp" as="image" type="image/webp" />

  <meta property="og:locale" content={currentLang === 'ar' ? 'ar_SA' : 'en_US'} />
  <meta property="og:locale:alternate" content={currentLang === 'ar' ? 'en_US' : 'ar_SA'} />

  {#if !$page.data.post}
    <link rel="canonical" href={`https://zafafworld.net${i18n.resolveRoute(i18n.route($page.url.pathname), currentLang as any)}`} data-seo="canonical" />
  {/if}
  <link rel="alternate" hreflang="ar" href={`https://zafafworld.net${i18n.resolveRoute(i18n.route($page.url.pathname), "ar")}`} />
  <link rel="alternate" hreflang="en" href={`https://zafafworld.net${i18n.resolveRoute(i18n.route($page.url.pathname), "en")}`} />
  <link rel="alternate" hreflang="x-default" href={`https://zafafworld.net${i18n.resolveRoute(i18n.route($page.url.pathname), "ar")}`} data-seo="x-default" />
</svelte:head>

<ParaglideJS {i18n}>
  <Loading show={!!$navigating || uiStore.globalLoading} />
  <div id="app" dir={currentDir} class="flex flex-col min-h-screen overflow-x-hidden">
    <Navbar user={data.user} />
    {#if $page.route.id !== '/[[language]]' && $page.route.id !== '/'}
      <!-- Global Layout Spacing to prevent content hiding under the fixed Navbar -->
      <div class="h-20 lg:h-[120px] shrink-0" aria-hidden="true"></div>
    {/if}
    <main id="main-content" tabindex="-1" class="flex-grow flex flex-col">
      {@render children()}
    </main>
    <Footer />
    {#if sseMessage}
      <div
        class="fixed top-24 left-1/2 -translate-x-1/2 z-[var(--z-modal)] bg-white border border-[var(--color-border)] border-b-4 border-b-[var(--color-primary)] shadow-[var(--shadow-lg)] px-6 py-4 rounded-xl flex items-center gap-3 transition-all"
      >
        <span class="text-2xl">✨</span>
        <p class="text-sm font-medium text-[var(--color-secondary)]">
          {sseMessage}
        </p>
      </div>
    {/if}
    <ToastContainer />
  </div>
</ParaglideJS>
