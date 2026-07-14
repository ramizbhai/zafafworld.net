<script lang="ts">
  import { page } from "$app/stores";
  import * as m from "$lib/paraglide/messages.js";
  import { getLocale } from "$lib/paraglide/runtime.js";
  import { getLocalizedField } from "$lib/utils/localize.js";
  import { buildFilteredRoute } from "$lib/utils/navigation";
  import { getCategoryImageUrl } from "$lib/constants/categoryImages";
  import { i18n } from "$lib/i18n.js";
  import type { NavbarState } from "$lib/stores/navbarState.svelte.js";

  let { state, isGlass } = $props<{ state: NavbarState; isGlass: boolean }>();

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
  class="transition-all duration-300 py-2.5 relative
  {isGlass
    ? 'bg-transparent border-b border-transparent text-white'
    : 'backdrop-blur-md bg-[var(--color-surface-alt)]/60 border-b border-[var(--color-border)]/60 text-[var(--color-text)]'}"
>
  <div class="container-page flex items-center justify-center">
    <!-- Connected Ribbon Links of equal styling weight -->
    <div
      class="hidden lg:flex items-center gap-6 text-xs font-semibold select-none w-full justify-center
      {isGlass ? 'text-white/95' : 'text-[var(--color-text)]'}"
    >
      <!-- Link 1: Wedding Halls -->
      <a
        href={l("/listings")}
        onmouseenter={() => state.setDropdown("wedding-halls")}
        onmouseleave={state.handleMouseLeave}
        class="flex items-center gap-2 transition-all duration-200 pb-1.5 group
          {isGlass
          ? 'hover:text-[var(--color-primary)]'
          : 'hover:text-[var(--color-primary-contrast)]'} 
          {state.activeDropdown === 'wedding-halls'
          ? isGlass
            ? 'text-[var(--color-primary)] border-b-2 border-[var(--color-primary)]'
            : 'text-[var(--color-primary-contrast)] border-b-2 border-[var(--color-primary-contrast)]'
          : ''}"
      >
        <svg
          viewBox="0 0 24 24"
          class="w-4 h-4 transition-colors
            {isGlass
            ? 'text-[var(--color-primary)] group-hover:text-[var(--color-primary-light)]'
            : 'text-[var(--color-primary-contrast)] group-hover:text-[var(--color-primary-contrast)]'}"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M2 22h20" />
          <path d="M4 22V11h16v11" />
          <path d="M12 2 2 7v4h20V7Z" />
          <path d="M8 11v7" />
          <path d="M12 11v7" />
          <path d="M16 11v7" />
        </svg>
        <span>{m.auto_wedding_halls()}</span>
      </a>

      <span
        class="font-normal select-none {isGlass
          ? 'text-white/30'
          : 'text-[var(--color-muted)]/50'}">•</span
      >

      <!-- Link 2: Wedding Planning Companies -->
      <a
        href={l("/vendors")}
        onmouseenter={() => state.setDropdown("wedding-companies")}
        onmouseleave={state.handleMouseLeave}
        class="flex items-center gap-2 transition-all duration-200 pb-1.5 group
          {isGlass
          ? 'hover:text-[var(--color-primary)]'
          : 'hover:text-[var(--color-primary-contrast)]'} 
          {state.activeDropdown === 'wedding-companies'
          ? isGlass
            ? 'text-[var(--color-primary)] border-b-2 border-[var(--color-primary)]'
            : 'text-[var(--color-primary-contrast)] border-b-2 border-[var(--color-primary-contrast)]'
          : ''}"
      >
        <svg
          viewBox="0 0 24 24"
          class="w-4 h-4 transition-colors
            {isGlass
            ? 'text-[var(--color-primary)] group-hover:text-[var(--color-primary-light)]'
            : 'text-[var(--color-primary-contrast)] group-hover:text-[var(--color-primary-contrast)]'}"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <rect x="2" y="7" width="20" height="14" rx="2" ry="2" />
          <path d="M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16" />
        </svg>
        <span>{m.auto_wedding_planning_com()}</span>
      </a>

      <span
        class="font-normal select-none {isGlass
          ? 'text-white/30'
          : 'text-[var(--color-muted)]/50'}">•</span
      >

      <!-- Link 3: Wedding Planning Department -->
      <a
        href={l("/listings")}
        class="flex items-center gap-2 transition-all duration-200 pb-1.5 group
          {isGlass
          ? 'hover:text-[var(--color-primary)]'
          : 'hover:text-[var(--color-primary-contrast)]'}"
      >
        <svg
          viewBox="0 0 24 24"
          class="w-4 h-4 transition-colors
            {isGlass
            ? 'text-[var(--color-primary)] group-hover:text-[var(--color-primary-light)]'
            : 'text-[var(--color-primary-contrast)] group-hover:text-[var(--color-primary-contrast)]'}"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <circle cx="12" cy="7" r="4" />
          <path d="M10 2.5l2-1.5 2 1.5" />
          <path
            d="M8 7c-1 2-2 5-2 11 0 2.5 1.5 4 4 4h4c2.5 0 4-1.5 4-4 0-6-1-9-2-11"
          />
          <path d="M9 15c.6-1.2 1.8-2 3-2s2.4.8 3 2v4H9v-4z" />
        </svg>
        <span>{m.auto_wedding_planning_dep()}</span>
      </a>

      <span
        class="font-normal select-none {isGlass
          ? 'text-white/30'
          : 'text-[var(--color-muted)]/50'}">•</span
      >

      <!-- Link 4: Discounts -->
      <a
        href={l("/offers")}
        class="flex items-center gap-2 transition-all duration-200 pb-1.5 group
          {isGlass
          ? 'hover:text-[var(--color-primary)]'
          : 'hover:text-[var(--color-primary-contrast)]'}"
      >
        <svg
          viewBox="0 0 24 24"
          class="w-4 h-4 transition-colors
            {isGlass
            ? 'text-[var(--color-primary)] group-hover:text-[var(--color-primary-light)]'
            : 'text-[var(--color-primary-contrast)] group-hover:text-[var(--color-primary-contrast)]'}"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path
            d="M12 2H2v10l9.29 9.29a1 1 0 0 0 1.41 0l7.29-7.29a1 1 0 0 0 0-1.42z"
          />
          <line x1="7" y1="7" x2="7.01" y2="7" />
        </svg>
        <span>{m.auto_discounts()}</span>
      </a>

      <span
        class="font-normal select-none {isGlass
          ? 'text-white/30'
          : 'text-[var(--color-muted)]/50'}">•</span
      >

      <!-- Link 5: Discover -->
      <a
        href={l("/discover")}
        class="flex items-center gap-2 transition-all duration-200 pb-1.5 group
          {isGlass
          ? 'hover:text-[var(--color-primary)]'
          : 'hover:text-[var(--color-primary-contrast)]'}"
      >
        <svg
          viewBox="0 0 24 24"
          class="w-4 h-4 transition-colors
            {isGlass
            ? 'text-[var(--color-primary)] group-hover:text-[var(--color-primary-light)]'
            : 'text-[var(--color-primary-contrast)] group-hover:text-[var(--color-primary-contrast)]'}"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path
            d="m12 3-1.912 5.813a2 2 0 0 1-1.275 1.275L3 12l5.813 1.912a2 2 0 0 1 1.275 1.275L12 21l1.912-5.813a2 2 0 0 1 1.275-1.275L21 12l-5.813-1.912a2 2 0 0 1-1.275-1.275Z"
          />
          <path d="m5 3 1 2.5L8.5 6 6 7 5 9.5 4 7 1.5 6 4 5Z" />
        </svg>
        <span>{m.auto_discover()}</span>
      </a>
    </div>
  </div>

  <!-- UNIFIED MEGA DROPDOWN PANEL -->
  {#if state.activeDropdown}
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      class="absolute top-full left-1/2 -translate-x-1/2 z-50 w-full max-w-5xl mt-2 rounded-2xl glass-dropdown overflow-hidden shadow-2xl p-6 border border-white/20 focus:outline-none transition-all duration-300 transform opacity-100 translate-y-0"
      onmouseenter={() => state.setDropdown(state.activeDropdown)}
      onmouseleave={state.handleMouseLeave}
      role="menu"
      tabindex="-1"
      aria-label="Sub-navigation Menu"
    >
      <div class="grid grid-cols-12 gap-8 items-stretch">
        <!-- LEFT LIST ITEMS -->
        <div class="col-span-5 flex flex-col justify-center">
          {#if state.activeDropdown === "wedding-halls"}
            <span class="text-[10px] font-bold tracking-wider text-[var(--color-muted)] uppercase mb-3 block">
              {m.auto_venue_categories()}
            </span>
            <div class="flex flex-col gap-3 font-semibold text-[#2D2620]">
              <a href={buildFilteredRoute("/listings", { category: "hotel-venue" })} onclick={() => (state.activeDropdown = null)} class="text-xs hover:text-[var(--color-primary-contrast)] transition-colors py-1 flex items-center gap-2">
                <span>🏨</span> <span>{m.auto_hotels()}</span>
              </a>
              <a href={buildFilteredRoute("/listings", { category: "wedding-palace" })} onclick={() => (state.activeDropdown = null)} class="text-xs hover:text-[var(--color-primary-contrast)] transition-colors py-1 flex items-center gap-2">
                <span>🏛️</span> <span>{m.auto_palaces_of_joy()}</span>
              </a>
              <a href={buildFilteredRoute("/listings", { category: "villa-resort" })} onclick={() => (state.activeDropdown = null)} class="text-xs hover:text-[var(--color-primary-contrast)] transition-colors py-1 flex items-center gap-2">
                <span>🏡</span> <span>{m.auto_breaks()}</span>
              </a>
              <a href={buildFilteredRoute("/listings", { category: "restaurant-event" })} onclick={() => (state.activeDropdown = null)} class="text-xs hover:text-[var(--color-primary-contrast)] transition-colors py-1 flex items-center gap-2">
                <span>🍽️</span> <span>{m.auto_restaurants()}</span>
              </a>
            </div>
          {:else if state.activeDropdown === "wedding-companies"}
            <span class="text-[10px] font-bold tracking-wider text-[var(--color-muted)] uppercase mb-3 block">
              {m.auto_preferred_categories()}
            </span>
            <div class="grid grid-cols-2 gap-2 text-xs font-semibold text-[#2D2620]">
              {#each $page.data?.metadata?.categories || [] as cat}
                <a href={buildFilteredRoute("/listings", { category: cat.key || cat.slug })} onclick={() => (state.activeDropdown = null)} class="hover:text-[var(--color-primary-contrast)] py-1 transition-colors">
                  {getLocale() === "ar" ? cat.labelAr || cat.ar || cat.key : cat.labelEn || cat.en || cat.key}
                </a>
              {/each}
            </div>
          {:else if state.activeDropdown === "wedding-planning-dept"}
            <span class="text-[10px] font-bold tracking-wider text-[var(--color-muted)] uppercase mb-3 block">
              {m.auto_planning_assistance_()}
            </span>
            <div class="flex flex-col gap-3 font-semibold text-[#2D2620]">
              <a href={l("/listings")} onclick={() => (state.activeDropdown = null)} class="text-xs hover:text-[var(--color-primary-contrast)] transition-colors py-1">{m.auto_interactive_budget_p()}</a>
              <a href={l("/listings")} onclick={() => (state.activeDropdown = null)} class="text-xs hover:text-[var(--color-primary-contrast)] transition-colors py-1">{m.auto_checklist_scheduler_()}</a>
              <a href={l("/vendors")} onclick={() => (state.activeDropdown = null)} class="text-xs hover:text-[var(--color-primary-contrast)] transition-colors py-1">{m.auto_supplier_recommendat()}</a>
              <a href={l("/listings")} onclick={() => (state.activeDropdown = null)} class="text-xs hover:text-[var(--color-primary-contrast)] transition-colors py-1">{m.auto_bridal_checklist__t()}</a>
            </div>
          {:else if state.activeDropdown === "discounts"}
            <span class="text-[10px] font-bold tracking-wider text-[var(--color-muted)] uppercase mb-3 block">
              {m.auto_current_active_offer()}
            </span>
            <div class="flex flex-col gap-3 font-semibold text-[#2D2620]">
              <a href={l("/offers")} onclick={() => (state.activeDropdown = null)} class="text-xs hover:text-[var(--color-primary-contrast)] transition-colors py-1">{m.auto_summer_ballroom_spec()}</a>
              <a href={l("/offers")} onclick={() => (state.activeDropdown = null)} class="text-xs hover:text-[var(--color-primary-contrast)] transition-colors py-1">{m.auto_catering__hospitali()}</a>
              <a href={l("/offers")} onclick={() => (state.activeDropdown = null)} class="text-xs hover:text-[var(--color-primary-contrast)] transition-colors py-1">{m.auto_exclusive_early_bird()}</a>
              <a href={l("/offers")} onclick={() => (state.activeDropdown = null)} class="text-xs hover:text-[var(--color-primary-contrast)] transition-colors py-1">{m.auto_complete_wedding_bun()}</a>
            </div>
          {:else if state.activeDropdown === "discover"}
            <span class="text-[10px] font-bold tracking-wider text-[var(--color-muted)] uppercase mb-3 block">
              {m.auto_inspiration__trends()}
            </span>
            <div class="flex flex-col gap-3 font-semibold text-[#2D2620]">
              <a href={l("/discover")} onclick={() => (state.activeDropdown = null)} class="text-xs hover:text-[var(--color-primary-contrast)] transition-colors py-1">{m.auto_modern_bridal_fashio()}</a>
              <a href={l("/discover")} onclick={() => (state.activeDropdown = null)} class="text-xs hover:text-[var(--color-primary-contrast)] transition-colors py-1">{m.auto_decor__chandelier_f()}</a>
              <a href={l("/discover")} onclick={() => (state.activeDropdown = null)} class="text-xs hover:text-[var(--color-primary-contrast)] transition-colors py-1">{m.auto_ballroom_coordinatio()}</a>
              <a href={l("/discover")} onclick={() => (state.activeDropdown = null)} class="text-xs hover:text-[var(--color-primary-contrast)] transition-colors py-1">{m.auto_real_wedding_covers_()}</a>
            </div>
          {/if}
        </div>

        <!-- RIGHT CONTENT DISPLAY -->
        <div class="col-span-7 border-s border-[var(--color-border)]/50 ps-8 flex flex-col justify-center">
          {#if state.activeDropdown === "wedding-halls"}
            <a href={buildFilteredRoute("/listings", { category: "wedding-palace" })} onclick={() => (state.activeDropdown = null)} class="relative rounded-xl overflow-hidden aspect-[21/9] shadow-md border border-[var(--color-border)]/50 group block">
              <img src={getCategoryImageUrl("wedding-palace")} alt="Wedding Banquet Ballroom" class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500" loading="eager" width="840" height="360" />
              <div class="absolute inset-0 bg-gradient-to-t from-black/75 via-black/20 to-transparent flex items-end p-4">
                <span class="text-white font-bold text-sm">{m.auto_luxury_wedding_halls()}</span>
              </div>
            </a>
          {:else}
            <div class="grid grid-cols-2 gap-4">
              <a href={buildFilteredRoute("/listings", { category: "photography-video" })} onclick={() => (state.activeDropdown = null)} class="relative rounded-xl overflow-hidden aspect-[16/9] border border-white/10 group block">
                <img src={getCategoryImageUrl("photography-video")} alt="Photographer" class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500" loading="eager" width="320" height="180" />
                <div class="absolute inset-0 bg-black/45 flex items-center justify-center p-3 text-center"><span class="text-white font-semibold text-xs tracking-wide">{getCategoryMetadata("photography-video").label}</span></div>
              </a>
              <a href={buildFilteredRoute("/listings", { category: "wedding-palace" })} onclick={() => (state.activeDropdown = null)} class="relative rounded-xl overflow-hidden aspect-[16/9] border border-white/10 group block">
                <img src={getCategoryImageUrl("wedding-palace")} alt="Halls" class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500" loading="eager" width="320" height="180" />
                <div class="absolute inset-0 bg-black/45 flex items-center justify-center p-3 text-center"><span class="text-white font-semibold text-xs tracking-wide">{getCategoryMetadata("wedding-palace").label}</span></div>
              </a>
              <a href={buildFilteredRoute("/listings", { category: "wedding-planner" })} onclick={() => (state.activeDropdown = null)} class="relative rounded-xl overflow-hidden aspect-[16/9] border border-white/10 group block">
                <img src={getCategoryImageUrl("wedding-planner")} alt="Wedding Planner" class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500" loading="eager" width="320" height="180" />
                <div class="absolute inset-0 bg-black/45 flex items-center justify-center p-3 text-center"><span class="text-white font-semibold text-xs tracking-wide">{getCategoryMetadata("wedding-planner").label}</span></div>
              </a>
              <a href={buildFilteredRoute("/listings", { category: "wedding-invitation" })} onclick={() => (state.activeDropdown = null)} class="relative rounded-xl overflow-hidden aspect-[16/9] border border-white/10 group block">
                <img src={getCategoryImageUrl("wedding-invitation")} alt="Wedding Invitations" class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500" loading="eager" width="320" height="180" />
                <div class="absolute inset-0 bg-black/45 flex items-center justify-center p-3 text-center"><span class="text-white font-semibold text-xs tracking-wide">{getCategoryMetadata("wedding-invitation").label}</span></div>
              </a>
            </div>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</div>
