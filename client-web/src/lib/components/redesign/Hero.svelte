<script lang="ts">
  import { onMount } from "svelte";
  import { getLocale } from "$lib/paraglide/runtime.js";
  import * as m from "$lib/paraglide/messages.js";
  import { getLocalizedField } from "$lib/utils/localize.js";
  import { countryStore } from "$lib/stores/country.svelte.js";
  import { toasts } from "$lib/stores/toast.svelte.js";
  import { goto } from "$app/navigation";
  import { env } from "$env/dynamic/public";
  import { trackBlogFunnelEvent } from "$lib/utils/analytics.js";
  import { page } from "$app/stores";
  import { buildListingsUrl } from "$lib/utils/navigation.js";
  import { i18n } from "$lib/i18n.js";
  import Button from "./Button.svelte";
  import Card from "./Card.svelte";

  interface Props {
    categories?: { key: string; icon: string; labelAr: string; labelEn: string; listingsCount?: number }[];
    cities?: { id: string; slug: string; name_ar: string; name_en: string; country_id: string }[];
  }

  let { categories = [], cities = [] }: Props = $props();

  let category = $state("");
  let city = $state("");
  let expectedWeddingDate = $state("");
  let conciergeName = $state("");
  let conciergeMobile = $state("");
  let conciergeWhatsapp = $state(true);
  let isSubmitting = $state(false);
  let isSuccess = $state(false);
  let conciergeError = $state("");

  const isAr = $derived(getLocale() === "ar");
  const PHONE_REGEX = /^\+[1-9]\d{6,14}$/;

  const filteredCities = $derived(
    cities.filter(
      (c: any) =>
        !countryStore.activeCode ||
        c.country_id?.toLowerCase() === countryStore.activeCode?.toLowerCase()
    )
  );

  const nextMonths = $derived(
    Array.from({ length: 12 }).map((_, i) => {
      const d = new Date();
      d.setMonth(d.getMonth() + i + 1);
      const value = d.toISOString().substring(0, 7) + "-01";
      const label = new Intl.DateTimeFormat(
        getLocale() === "ar" ? "ar-SA" : "en-US",
        { month: "long", year: "numeric" }
      ).format(d);
      return { value, label };
    })
  );

  function handleSearch(e: SubmitEvent) {
    e.preventDefault();
    const targetCategory = category || undefined;
    const cleanPath = buildListingsUrl({ city: city || undefined, category: targetCategory });
    goto(cleanPath);
  }

  async function startConcierge() {
    if (!expectedWeddingDate) return;
    conciergeError = "";

    if (!conciergeName.trim() || conciergeName.trim().length < 3) {
      conciergeError = isAr ? "الاسم مطلوب (3 أحرف على الأقل)" : "Name is required (min 3 characters)";
      return;
    }
    if (!conciergeMobile.trim()) {
      conciergeError = isAr ? "رقم الهاتف مطلوب" : "Phone number is required";
      return;
    }
    if (!PHONE_REGEX.test(conciergeMobile.trim())) {
      conciergeError = isAr ? "صيغة غير صحيحة. مثال: 966512345678+" : "Invalid format. Example: +966512345678";
      return;
    }

    isSubmitting = true;
    try {
      const res = await fetch("/bff/v1/public/afrah", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          name: conciergeName.trim(),
          phone: conciergeMobile.trim(),
          isWhatsapp: conciergeWhatsapp,
          eventDate: expectedWeddingDate,
          message: isAr ? "طلب تخطيط زفاف عبر أفراح" : "Wedding planning request via Afrah concierge"
        })
      });

      const body = await res.json();
      if (res.ok && body.status === "success") {
        isSuccess = true;
        await trackBlogFunnelEvent("afrah_start");
        toasts.push("success", isAr ? "تم إنشاء جلسة التخطيط بنجاح!" : "Your planning session has been created!");
      } else if (res.status === 429) {
        conciergeError = isAr ? "عدد كبير من الطلبات. يرجى المحاولة لاحقاً." : "Too many requests. Please try again later.";
        toasts.push("error", conciergeError);
      } else {
        conciergeError = body.message || (isAr ? "حدث خطأ. يرجى المحاولة مرة أخرى." : "An error occurred. Please try again.");
        toasts.push("error", conciergeError);
      }
    } catch (e) {
      console.error(e);
      conciergeError = isAr ? "فشل الاتصال بالخادر." : "Connection to server failed.";
      toasts.push("error", conciergeError);
    } finally {
      if (!isSuccess) {
        isSubmitting = false;
      }
    }
  }

  function l(path: string) {
    try {
      if (!i18n) {
        console.error("[Hero l()] i18n is falsy");
        return path;
      }
      return i18n.resolveRoute(path, getLocale());
    } catch (e: any) {
      console.error("[Hero l()] Exception caught:", e.message, e.stack);
      return path;
    }
  }
</script>

<section
  id="home-hero-section"
  class="relative min-h-[750px] lg:h-screen lg:min-h-[850px] flex flex-col justify-between overflow-hidden"
  aria-label={m.auto_hero_section()}
>
  <!-- Background responsive picture element (Critical LCP optimization) -->
  <div class="absolute inset-0 z-0">
    <picture>
      <!-- AVIF sources -->
      <source srcset="/hero-desktop.avif" media="(min-width: 1025px)" type="image/avif" />
      <source srcset="/hero-tablet.avif" media="(min-width: 641px) and (max-width: 1024px)" type="image/avif" />
      <source srcset="/hero-mobile.avif" media="(max-width: 640px)" type="image/avif" />
      
      <!-- WebP sources -->
      <source srcset="/hero-desktop.webp" media="(min-width: 1025px)" type="image/webp" />
      <source srcset="/hero-tablet.webp" media="(min-width: 641px) and (max-width: 1024px)" type="image/webp" />
      <source srcset="/hero-mobile.webp" media="(max-width: 640px)" type="image/webp" />
      
      <!-- Fallback image (Eager LCP only) -->
      <img
        src="/hero.webp"
        alt="ZafafWorld luxury wedding venue background"
        fetchpriority="high"
        loading="eager"
        class="absolute inset-0 w-full h-full object-cover object-center"
        width="1920"
        height="1080"
      />
    </picture>
    
    <!-- Darkened gradient overlays for text readability -->
    <div class="absolute inset-x-0 top-0 h-[280px] bg-gradient-to-b from-[#1A1410]/85 via-[#1A1410]/45 to-transparent z-10 pointer-events-none" aria-hidden="true"></div>
    <div class="absolute inset-0 bg-[#1A1410]/20 z-10 pointer-events-none" aria-hidden="true"></div>
  </div>

  <!-- Hero Content Grid -->
  <div class="relative z-20 w-full flex-grow flex flex-col justify-between container-page pt-28 pb-10">
    <!-- Top Eyebrow Gold Badge -->
    <div class="w-full flex flex-col items-center text-center gap-zw-4 pt-zw-4 md:pt-zw-6">
      <div class="inline-flex items-center gap-zw-2 bg-zw-secondary/60 backdrop-blur-md border border-white/10 rounded-zw-full px-zw-5 py-zw-2 shadow-zw-lg select-none">
        <span class="text-zw-primary text-zw-sm">★</span>
        <span class="text-zw-xs text-zw-surface font-bold tracking-wide uppercase font-body-en">
          {getLocalizedField(countryStore.active, "eyebrow", getLocale())}
        </span>
      </div>
    </div>

    <!-- Redesigned Search Widget + Concierge Row -->
    <div class="w-full flex flex-col items-center mt-zw-6 lg:mt-zw-8">
      
      <!-- Unified Card Container -->
      <div class="bg-zw-surface/95 backdrop-blur-md border border-zw-border rounded-zw-3xl pt-zw-6 pb-zw-10 px-zw-6 md:pt-zw-8 md:pb-zw-12 md:px-zw-8 lg:pt-zw-10 lg:pb-zw-14 lg:px-zw-10 shadow-zw-xl w-full max-w-[1320px] mx-auto flex flex-col lg:flex-row gap-zw-8 lg:gap-zw-10 text-start relative transition-all duration-300">
        
        <!-- LEFT PANEL: Search form -->
        <div class="w-full lg:w-[50%] flex flex-col justify-between">
          <div>
            <h2 class="font-display text-zw-2xl md:text-zw-3xl font-bold text-zw-secondary mb-zw-2 leading-tight">
              {m.auto_from_venue_to_weddin()}
            </h2>
            <p class="text-zw-xs md:text-zw-sm text-zw-muted mb-zw-6 font-semibold">
              {m.auto_two_out_of_three_cou()}
            </p>
          </div>

          <form onsubmit={handleSearch} class="flex flex-col sm:flex-row gap-zw-3 items-stretch sm:items-end w-full">
            
            <!-- Category dropdown -->
            <div class="relative flex-1">
              <label for="search-category" class="text-[9px] text-zw-muted font-bold uppercase tracking-wider mb-zw-1 block">
                {m.auto_what_are_you_looking()}
              </label>
              <div class="relative">
                <select
                  id="search-category"
                  bind:value={category}
                  class="w-full h-zw-12 bg-zw-surface border border-zw-border hover:border-zw-primary-contrast rounded-zw-full pl-zw-5 pr-zw-10 rtl:pl-zw-10 rtl:pr-zw-5 text-zw-xs font-bold text-zw-secondary focus:outline-none appearance-none cursor-pointer shadow-sm transition-all focus:ring-2 focus:ring-zw-primary/20"
                >
                  <option value="">{m.auto_all_services()}</option>
                  {#each categories as cat}
                    <option value={cat.key}>
                      {isAr ? cat.labelAr || cat.key : cat.labelEn || cat.key}
                    </option>
                  {/each}
                </select>
                <div class="absolute inset-y-0 right-zw-4 rtl:left-zw-4 rtl:right-auto flex items-center pointer-events-none text-zw-muted" aria-hidden="true">
                  <svg viewBox="0 0 20 20" class="w-4 h-4" fill="currentColor">
                    <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clip-rule="evenodd"/>
                  </svg>
                </div>
              </div>
            </div>

            <!-- City dropdown -->
            <div class="relative flex-1">
              <label for="search-city" class="text-[9px] text-zw-muted font-bold uppercase tracking-wider mb-zw-1 block">
                {m.auto_city()}
              </label>
              <div class="relative">
                <select
                  id="search-city"
                  bind:value={city}
                  class="w-full h-zw-12 bg-zw-surface border border-zw-border hover:border-zw-primary-contrast rounded-zw-full pl-zw-5 pr-zw-10 rtl:pl-zw-10 rtl:pr-zw-5 text-zw-xs font-bold text-zw-secondary focus:outline-none appearance-none cursor-pointer shadow-sm transition-all focus:ring-2 focus:ring-zw-primary/20"
                >
                  <option value="">{m.auto_all_cities()}</option>
                  {#each filteredCities as c}
                    <option value={c.slug}>{isAr ? c.name_ar : c.name_en}</option>
                  {/each}
                </select>
                <div class="absolute inset-y-0 right-zw-4 rtl:left-zw-4 rtl:right-auto flex items-center pointer-events-none text-zw-muted" aria-hidden="true">
                  <svg viewBox="0 0 20 20" class="w-4 h-4" fill="currentColor">
                    <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clip-rule="evenodd"/>
                  </svg>
                </div>
              </div>
            </div>

            <!-- Search submit button -->
            <Button type="submit" variant="accent-rose" size="md" class="h-zw-12 font-bold shrink-0">
              {m.auto_service_providers_li()}
            </Button>
          </form>
        </div>

        <!-- Separation line -->
        <div class="hidden lg:block w-[1px] bg-zw-border self-stretch my-zw-2"></div>

        <!-- RIGHT PANEL: Afrah Concierge -->
        <div class="w-full lg:w-[48%] flex flex-col justify-between">
          <div class="flex items-start gap-zw-4">
            <!-- Planner Avatar -->
            <div class="w-16 h-16 md:w-20 md:h-20 rounded-zw-full border-4 border-zw-interactive-rose/25 overflow-hidden shrink-0 shadow-zw-sm">
              <img
                src="/afrah_avatar.webp"
                alt="Afrah Concierge Assistant"
                class="w-full h-full object-cover"
                width="80"
                height="80"
                decoding="async"
              />
            </div>
            
            <div class="flex-1 min-w-0">
              <span class="text-[9px] tracking-wider text-zw-interactive-teal font-extrabold uppercase mb-zw-1 block">
                {isAr ? "مساعدك الشخصي" : "Your Personal Assistant"}
              </span>
              <h3 class="font-display text-zw-xl md:text-zw-2xl font-bold text-zw-secondary mb-zw-1 leading-tight">
                {isAr ? "تحتاج مساعدة في اتخاذ القرار؟" : "Need Help Deciding?"}
              </h3>
              <p class="text-[11px] text-zw-muted font-medium leading-relaxed">
                {isAr ? "أخبرنا بما تبحث عنه وسنوصلك بأفضل المزودين." : "Tell us what you are looking for and we will match you with the perfect vendor."}
              </p>
            </div>
          </div>

          <div class="mt-zw-6">
            <div class="flex flex-col sm:flex-row gap-zw-3 items-stretch sm:items-end w-full">
              <!-- Date Selector -->
              <div class="relative flex-1">
                <label for="concierge-date" class="text-[9px] text-zw-muted font-bold uppercase tracking-wider mb-zw-1 block">
                  {m.auto_expected_wedding_dat()}
                </label>
                <div class="relative">
                  <select
                    id="concierge-date"
                    bind:value={expectedWeddingDate}
                    class="w-full h-zw-12 bg-zw-surface border border-zw-border hover:border-zw-primary-contrast rounded-zw-full pl-zw-5 pr-zw-10 rtl:pl-zw-10 rtl:pr-zw-5 text-zw-xs font-bold text-zw-secondary focus:outline-none appearance-none cursor-pointer shadow-sm focus:ring-2 focus:ring-zw-primary/20"
                  >
                    <option value="">{m.auto_select_a_date()}</option>
                    {#each nextMonths as mItem}
                      <option value={mItem.value}>{mItem.label}</option>
                    {/each}
                  </select>
                  <div class="absolute inset-y-0 right-zw-4 rtl:left-zw-4 rtl:right-auto flex items-center pointer-events-none text-zw-muted" aria-hidden="true">
                    <svg viewBox="0 0 20 20" class="w-4 h-4" fill="currentColor">
                      <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clip-rule="evenodd"/>
                    </svg>
                  </div>
                </div>
              </div>

              <!-- Name input -->
              <div class="flex-1">
                <label for="concierge-name" class="text-[9px] text-zw-muted font-bold uppercase tracking-wider mb-zw-1 block">
                  {isAr ? "الاسم" : "Your Name"}
                </label>
                <input
                  id="concierge-name"
                  type="text"
                  bind:value={conciergeName}
                  placeholder={isAr ? "مثال: أحمد محمد" : "e.g. Ahmed Mohammed"}
                  class="w-full h-zw-12 bg-zw-surface border border-zw-border hover:border-zw-primary-contrast rounded-zw-full px-zw-5 text-zw-xs font-bold text-zw-secondary focus:outline-none focus:ring-2 focus:ring-zw-primary/20 shadow-sm"
                  required
                />
              </div>

              <!-- Mobile input -->
              <div class="flex-1 relative">
                <label for="concierge-mobile" class="text-[9px] text-zw-muted font-bold uppercase tracking-wider mb-zw-1 block">
                  {isAr ? "رقم الهاتف" : "Phone"}
                </label>
                <input
                  id="concierge-mobile"
                  type="tel"
                  bind:value={conciergeMobile}
                  placeholder="+966512345678"
                  class="w-full h-zw-12 bg-zw-surface border border-zw-border hover:border-zw-primary-contrast rounded-zw-full px-zw-5 text-zw-xs font-bold text-zw-secondary focus:outline-none focus:ring-2 focus:ring-zw-primary/20 shadow-sm ltr"
                  dir="ltr"
                  required
                />
                <label class="absolute left-0 right-0 top-full flex items-center gap-zw-1.5 mt-zw-2 ps-zw-3 cursor-pointer select-none">
                  <input type="checkbox" bind:checked={conciergeWhatsapp} class="w-3.5 h-3.5 rounded accent-zw-interactive-teal" />
                  <span class="text-[9px] text-zw-muted font-medium">{isAr ? "واتساب" : "WhatsApp"}</span>
                </label>
              </div>

              <!-- Concierge Submit Button -->
              <button
                onclick={startConcierge}
                disabled={isSubmitting || isSuccess}
                class="px-zw-6 h-zw-12 rounded-zw-full font-bold text-zw-xs transition-all duration-300 shadow-zw-sm hover:shadow-zw-md cursor-pointer disabled:cursor-not-allowed whitespace-nowrap select-none flex items-center justify-center shrink-0
                  {isSuccess ? 'bg-green-500 text-white' : 'bg-zw-interactive-teal hover:bg-zw-interactive-teal-hover disabled:bg-zw-muted/40 text-white'}"
              >
                {#if isSubmitting}
                  <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                {/if}
                {isSuccess ? (isAr ? "✅ تم الإرسال!" : "✅ Sent!") : m.auto_start_quickly()}
              </button>
            </div>
            
            {#if conciergeError}
              <p class="text-[10px] text-red-500 font-bold mt-zw-8 text-start">{conciergeError}</p>
            {/if}
          </div>
        </div>

        <!-- Specific date details link -->
        <a
          href={l("/afrah")}
          class="absolute bottom-3 md:bottom-4 lg:bottom-5 end-6 md:end-8 lg:end-10 text-[10px] text-zw-muted hover:text-zw-secondary block font-medium transition-colors duration-200"
        >
          {m.auto_if_you_have_a_specif()}
        </a>
      </div>

      <!-- Trending tags (repairs broken redirect link) -->
      <div class="flex flex-wrap items-center justify-center gap-zw-2 mt-zw-6">
        <span class="text-zw-xs text-white/80 font-medium select-none">
          {m.auto_trending()}
        </span>
        {#each isAr ? [
          { name: "الرياض", href: l("/search?city=riyadh") },
          { name: "جدة", href: l("/search?city=jeddah") },
          { name: "الدمام", href: l("/search?city=dammam") },
          { name: "قاعات زفاف", href: l("/search?category=wedding-palace") },
          { name: "حفلات خطوبة", href: l("/search?category=villa-resort") }
        ] : [
          { name: "Riyadh", href: l("/search?city=riyadh") },
          { name: "Jeddah", href: l("/search?city=jeddah") },
          { name: "Weddings", href: l("/search?category=wedding-palace") },
          { name: "Engagement", href: l("/search?category=villa-resort") },
          { name: "Corporate", href: l("/search?category=hotel-venue") }
        ] as tag}
          <a
            href={tag.href}
            class="text-zw-xs bg-white/10 hover:bg-zw-primary/30 border border-white/20 text-white/90 hover:text-white rounded-zw-full px-zw-4 py-zw-1.5 transition-all duration-200 shadow-zw-sm"
          >
            {tag.name}
          </a>
        {/each}
      </div>
    </div>
  </div>
</section>
