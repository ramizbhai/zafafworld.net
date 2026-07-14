<script lang="ts">
  import { page } from "$app/stores";
  import * as m from "$lib/paraglide/messages.js";
  import { getLocale } from "$lib/paraglide/runtime.js";
  import { getLocalizedField } from "$lib/utils/localize.js";
  import { buildFilteredRoute } from "$lib/utils/navigation";
  import { i18n } from "$lib/i18n.js";
  import Button from "$lib/components/ui/Button.svelte";
  import NavSearch from "./NavSearch.svelte";
  import NavLanguageToggle from "./NavLanguageToggle.svelte";
  import type { NavbarState } from "$lib/stores/navbarState.svelte.js";

  let { state } = $props<{ state: NavbarState }>();

  function l(path: string) {
    return i18n.resolveRoute(path, getLocale());
  }

  function getCategoryMetadata(slug: string) {
    const backendCategories = $page.data?.metadata?.categories || [];
    const found = backendCategories.find(
      (c: any) => c.key === slug || c.slug === slug,
    );
    if (found) {
      return {
        slug: found.key || found.slug,
        icon: found.icon || "✨",
        label:
          getLocale() === "ar"
            ? found.labelAr || found.ar
            : found.labelEn || found.en,
      };
    }
    const fallbackMap: Record<
      string,
      { icon: string; ar: string; en: string }
    > = {
      "wedding-palace": { icon: "🏛️", ar: "قاعات الأفراح", en: "Wedding Palace" },
      "hotel-venue": { icon: "🏨", ar: "فنادق وقاعات", en: "Hotel Ballroom" },
      "villa-resort": { icon: "🏡", ar: "استراحات وفلل", en: "Villa & Resort" },
      "restaurant-event": { icon: "🍽️", ar: "مطاعم وقاعات خاصة", en: "Restaurant & Dining" },
      "outdoor-garden": { icon: "🌿", ar: "حدائق وأماكن مفتوحة", en: "Outdoor Garden" },
      chalet: { icon: "🏕️", ar: "شاليهات", en: "Chalet" },
      "photography-video": { icon: "📷", ar: "تصوير وفيديو", en: "Photography & Video" },
      "photo-studio": { icon: "📸", ar: "استوديو تصوير", en: "Photo Studio" },
      "wedding-planner": { icon: "📋", ar: "منظم حفلات", en: "Wedding Planner" },
      "hair-makeup": { icon: "💄", ar: "شعر ومكياج", en: "Hair & Makeup" },
      "henna-art": { icon: "🌿", ar: "فن الحناء", en: "Henna Art" },
      "male-grooming": { icon: "🪒", ar: "حلاقة ومزين رجالي", en: "Male Grooming" },
      "wedding-gown": { icon: "👗", ar: "فساتين الزفاف", en: "Wedding Gown" },
      "wedding-invitation": { icon: "✉️", ar: "دعوات زفاف", en: "Wedding Invitations" },
      "entertainment-dj": { icon: "🎵", ar: "دي جي وحفلات", en: "DJ & Entertainment" },
      "wedding-jewelry": { icon: "💍", ar: "مجوهرات وخواتم", en: "Bridal Jewelry" },
      "wedding-sweets": { icon: "🍬", ar: "حلويات عربية", en: "Arabic Sweets" },
      "wedding-gifts": { icon: "🎁", ar: "هدايا وتوزيعات", en: "Wedding Gifts" },
      catering: { icon: "🍱", ar: "ضيافة وطعام", en: "Wedding Catering" },
      "wedding-cake": { icon: "🎂", ar: "كيك الزفاف", en: "Wedding Cake" },
      "flowers-floral": { icon: "💐", ar: "ورد وزهور", en: "Flowers & Floral" },
    };
    const fb = fallbackMap[slug] || { icon: "✨", ar: slug, en: slug };
    return {
      slug,
      icon: fb.icon,
      label: getLocalizedField(fb, "", getLocale()),
    };
  }
</script>

<div
  id="mobile-menu"
  class="lg:hidden bg-white/98 backdrop-blur-md border-t border-[var(--color-border)] shadow-[var(--shadow-lg)] max-h-[calc(100vh-80px)] overflow-y-auto"
  role="dialog"
  aria-modal="true"
  aria-label={m.nav_menu()}
>
  <div class="container-page py-6 flex flex-col gap-6">
    <!-- Mobile Search -->
    <NavSearch {state} isGlass={false} isMobile={true} />

    <!-- Quick Links -->
    <div class="px-2">
      <span class="text-[10px] font-bold tracking-wider text-[var(--color-muted)] uppercase mb-3 block">
        {m.auto_planning_assistance_() || "Quick Links"}
      </span>
      <div class="grid grid-cols-3 gap-2">
        <a href={l("/listings")} onclick={state.closeMenu} class="flex flex-col items-center justify-center p-3 rounded-xl bg-[var(--color-surface-alt)] border border-[var(--color-border)] text-center transition-all hover:bg-[var(--color-border)]">
          <span class="text-xl mb-1">🏛️</span>
          <span class="text-[10px] font-bold text-[var(--color-secondary)] leading-tight">{m.auto_wedding_planning_dep()}</span>
        </a>
        <a href={l("/offers")} onclick={state.closeMenu} class="flex flex-col items-center justify-center p-3 rounded-xl bg-[var(--color-surface-alt)] border border-[var(--color-border)] text-center transition-all hover:bg-[var(--color-border)]">
          <span class="text-xl mb-1">🏷️</span>
          <span class="text-[10px] font-bold text-[var(--color-secondary)] leading-tight">{m.auto_discounts()}</span>
        </a>
        <a href={l("/discover")} onclick={state.closeMenu} class="flex flex-col items-center justify-center p-3 rounded-xl bg-[var(--color-surface-alt)] border border-[var(--color-border)] text-center transition-all hover:bg-[var(--color-border)]">
          <span class="text-xl mb-1">✨</span>
          <span class="text-[10px] font-bold text-[var(--color-secondary)] leading-tight">{m.auto_discover()}</span>
        </a>
      </div>
    </div>

    <!-- Categories Grid -->
    <div class="px-2">
      <span class="text-[10px] font-bold tracking-wider text-[var(--color-muted)] uppercase mb-3 block">
        {m.auto_venue_categories() || "Categories"}
      </span>
      <div class="grid grid-cols-2 gap-2">
        {#each $page.data?.metadata?.categories || [] as cat}
          {@const meta = getCategoryMetadata(cat.key || cat.slug)}
          <a href={buildFilteredRoute("/listings", { category: meta.slug })} onclick={state.closeMenu} class="flex items-center gap-2.5 p-3 rounded-xl border border-[var(--color-border)] hover:border-[var(--color-primary)] hover:bg-[var(--color-surface-alt)]/50 transition-all text-xs font-semibold text-[var(--color-secondary)]">
            <span class="text-lg flex-shrink-0">{meta.icon}</span>
            <span class="truncate">{meta.label}</span>
          </a>
        {/each}
      </div>
    </div>

    <!-- System & Auth Actions -->
    <div class="pt-4 border-t border-[var(--color-border)] flex flex-col gap-4 px-2">
      <NavLanguageToggle {state} isGlass={false} isMobile={true} />

      <div class="flex flex-col gap-2 mt-2">
        <Button href="https://vendor.zafafworld.net" variant="primary" fullWidth onclick={state.closeMenu} class="py-3 text-xs font-bold bg-[#C9A96E] hover:bg-[#B5965C] text-[#2D2620] border-none shadow-[var(--shadow-gold)] flex items-center justify-center gap-2 transition-transform hover:scale-[1.02]">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="20" height="14" x="2" y="7" rx="2" ry="2" /><path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16" /></svg>
          <span>{getLocale() === "ar" ? "منصة شركاء الأعمال" : "Join as a Business Partner"}</span>
        </Button>
      </div>
    </div>
  </div>
</div>
