<script lang="ts">
  import { onMount } from "svelte";
  import * as m from "$lib/paraglide/messages.js";
  import { getLocale } from "$lib/paraglide/runtime.js";
  import { getLocalizedField } from "$lib/utils/localize.js";
  import { countryStore } from "$lib/stores/country.svelte.js";
  import { toasts } from "$lib/stores/toast.svelte.js";
  import { goto } from "$app/navigation";
  import { env } from "$env/dynamic/public";
  import { trackBlogFunnelEvent } from "$lib/utils/analytics.js";
  import { page } from "$app/stores";
  import { buildListingsUrl } from "$lib/utils/navigation.js";
  import { getCategoryIcon } from "$lib/constants/categoryIcons.js";

  // Props: accept pre-loaded data from layout/page to avoid redundant onMount fetch
  interface Props {
    categories?: { key: string; icon: string; labelAr: string; labelEn: string; listingsCount?: number }[];
    cities?: { id: string; slug: string; name_ar: string; name_en: string; country_id: string }[];
  }
  let { categories: propCategories = [], cities: propCities = [] }: Props = $props();

  let category = $state("");
  let city = $state("");
  let expectedWeddingDate = $state("");
  let conciergeName = $state("");
  let conciergeMobile = $state("");
  let conciergeWhatsapp = $state(true);
  let isSubmitting = $state(false);
  let isSuccess = $state(false);
  let conciergeError = $state("");

  let localCategories = $state<any[]>([]);
  let localCities = $state<any[]>([]);

  $effect(() => {
    if (propCategories.length && !localCategories.length) localCategories = propCategories;
    if (propCities.length && !localCities.length) localCities = propCities;
  });

  onMount(async () => {
    // Only fetch if data wasn't provided via props (avoids double network request
    // when parent already has this data from the layout server load)
    const needCategories = !propCategories.length;
    const needCities = !propCities.length;
    if (!needCategories && !needCities) return;

    try {
      const API_BASE = env.PUBLIC_API_URL || 'http://localhost:8080';
      const requests: Promise<any>[] = [];
      if (needCategories) requests.push(fetch(`${API_BASE}/api/v1/public/categories`).then(r => r.json()));
      if (needCities) requests.push(fetch(`${API_BASE}/api/v1/public/cities`).then(r => r.json()));
      const results = await Promise.all(requests);

      let idx = 0;
      if (needCategories) {
        const catRes = results[idx++];
        if (catRes && catRes.status === 'success') {
          let flattened: any[] = [];
          if (Array.isArray(catRes.allCategories)) {
            flattened = catRes.allCategories;
          } else if (catRes.categories) {
            const venuesList = catRes.categories.venues || [];
            const servicesList = catRes.categories.services || [];
            flattened = [...venuesList, ...servicesList];
          }

          localCategories = flattened.map((c: any) => ({
            key: c.slug,
            icon: getCategoryIcon(c.slug),
            labelAr: c.ar,
            labelEn: c.en,
            listingsCount: c.listingsCount ?? 0
          }));
        }
      }

      if (needCities) {
        const cityRes = results[idx++];
        if (cityRes && cityRes.status === 'success') {
          const rawCities = cityRes.cities || cityRes.data || [];
          localCities = rawCities.map((c: any) => ({
            id: c.id,
            slug: c.slug,
            name_ar: c.name_ar || c.ar || '',
            name_en: c.name_en || c.en || '',
            country_id: c.country_id || ''
          }));
        }
      }
    } catch (err) {
      console.error('Failed to dynamically fetch categories/cities in HeroSection:', err);
    }
  });

  const categories = $derived(localCategories.length > 0 ? localCategories : ($page.data?.metadata?.categories || []));
  const cities = $derived(localCities.length > 0 ? localCities : ($page.data?.metadata?.cities || []));

  const filteredCities = $derived(
    cities.filter(
      (c: any) =>
        !countryStore.activeCode ||
        c.country_id?.toLowerCase() === countryStore.activeCode?.toLowerCase(),
    ),
  );

  // Generate next 12 months dynamically
  const nextMonths = $derived(
    Array.from({ length: 12 }).map((_, i) => {
      const d = new Date();
      d.setMonth(d.getMonth() + i + 1);
      const value = d.toISOString().substring(0, 7) + "-01"; // YYYY-MM-01 format
      const label = new Intl.DateTimeFormat(
        getLocale() === "ar" ? "ar-SA" : "en-US",
        { month: "long", year: "numeric" },
      ).format(d);
      return { value, label };
    }),
  );

  function handleSearch(e: SubmitEvent) {
    e.preventDefault();
    
    const targetCategory = category || undefined;
    const cleanPath = buildListingsUrl({ city: city || undefined, category: targetCategory });
    goto(cleanPath);
  }

  const isAr = $derived(getLocale() === 'ar');
  const PHONE_REGEX = /^\+[1-9]\d{6,14}$/;

  async function startConcierge() {
    if (!expectedWeddingDate) return;
    conciergeError = '';

    // Validate name
    if (!conciergeName.trim() || conciergeName.trim().length < 3) {
      conciergeError = isAr ? 'الاسم مطلوب (3 أحرف على الأقل)' : 'Name is required (min 3 characters)';
      return;
    }
    // Validate mobile
    if (!conciergeMobile.trim()) {
      conciergeError = isAr ? 'رقم الهاتف مطلوب' : 'Phone number is required';
      return;
    }
    if (!PHONE_REGEX.test(conciergeMobile.trim())) {
      conciergeError = isAr ? 'صيغة غير صحيحة. مثال: 966512345678+' : 'Invalid format. Example: +966512345678';
      return;
    }

    isSubmitting = true;
    try {
      const res = await fetch('/bff/v1/public/afrah', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          name: conciergeName.trim(),
          phone: conciergeMobile.trim(),
          isWhatsapp: conciergeWhatsapp,
          eventDate: expectedWeddingDate,
          message: isAr ? 'طلب تخطيط زفاف عبر أفراح' : 'Wedding planning request via Afrah concierge',
        }),
      });

      const body = await res.json();
      if (res.ok && body.status === 'success') {
        isSuccess = true;
        await trackBlogFunnelEvent('afrah_start');
        toasts.push('success', isAr ? 'تم إنشاء جلسة التخطيط بنجاح!' : 'Your planning session has been created!');
      } else if (res.status === 429) {
        conciergeError = isAr ? 'عدد كبير من الطلبات. يرجى المحاولة لاحقاً.' : 'Too many requests. Please try again later.';
        toasts.push('error', conciergeError);
      } else {
        conciergeError = body.message || (isAr ? 'حدث خطأ. يرجى المحاولة مرة أخرى.' : 'An error occurred. Please try again.');
        toasts.push('error', conciergeError);
      }
    } catch (e) {
      console.error(e);
      conciergeError = isAr ? 'فشل الاتصال بالخادم.' : 'Connection to server failed.';
      toasts.push('error', conciergeError);
    } finally {
      if (!isSuccess) {
        isSubmitting = false;
      }
    }
  }
</script>

<svelte:head>
  <link rel="preload" as="image" href="/hero.webp" fetchpriority="high" />
</svelte:head>

<section
  id="home-hero-section"
  class="relative min-h-[750px] lg:h-screen lg:min-h-[850px] flex flex-col justify-between overflow-hidden"
  aria-label={m.auto_hero_section()}
>
  <!-- Background image and overlay -->
  <div class="absolute inset-0 z-0">
    <img
      src="/hero.webp"
      alt="Hero background"
      fetchpriority="high"
      class="absolute inset-0 w-full h-full object-cover object-center"
    />
    <!-- Subtle top overlay for navbar legibility and general slight dimming to keep text contrast high -->
    <div
      class="absolute inset-x-0 top-0 h-[280px] bg-gradient-to-b from-black/85 via-black/45 to-transparent z-10 pointer-events-none"
    ></div>
    <div
      class="absolute inset-0 bg-black/15 z-10 pointer-events-none"
    ></div>
  </div>

  <!-- Main content: Top portion (Badge + Ribbon) + Card + Trending (in flex col structure) -->
  <div
    class="relative z-20 w-full flex-grow flex flex-col justify-between container-page pt-28 pb-10"
  >
    <!-- Top Area: Badge -->
    <div class="w-full flex flex-col items-center text-center gap-4 pt-4 md:pt-6">
      <!-- Badge -->
      <div
        class="inline-flex items-center gap-2 bg-[#1A1612]/50 backdrop-blur-sm border border-white/15 rounded-full px-5 py-2.5 shadow-lg select-none"
      >
        <span class="text-amber-400 text-sm">★</span>
        <span class="text-xs text-white/95 font-medium tracking-wide">
          {getLocalizedField(countryStore.active, "eyebrow", getLocale())}
        </span>
      </div>
    </div>

    <!-- Center/Bottom Area: Search + Concierge Card & Trending -->
    <div class="w-full flex flex-col items-center mt-6 lg:mt-8">
      <!-- Unified Card Container -->
      <div
        class="bg-[#FAF6F0]/95 backdrop-blur-md border border-[#EAE0D0] rounded-[32px] pt-6 pb-10 px-6 md:pt-8 md:pb-12 md:px-8 lg:pt-10 lg:pb-14 lg:px-10 shadow-[0_20px_50px_rgba(45,38,32,0.15)] w-full max-w-[1320px] mx-auto flex flex-col lg:flex-row gap-8 lg:gap-10 text-start relative transition-all duration-300"
      >
        <!-- LEFT PANEL: Search -->
        <div class="w-full lg:w-[50%] flex flex-col justify-between">
          <div>
            <h2
              class="font-display text-2xl md:text-[28px] font-bold text-[#2D2620] mb-2 leading-tight"
            >
              {m.auto_from_venue_to_weddin()}
            </h2>
            <p class="text-xs md:text-sm text-[#9E8E7A] mb-6 font-medium">
              {m.auto_two_out_of_three_cou()}
            </p>
          </div>

          <form
            onsubmit={handleSearch}
            class="flex flex-col sm:flex-row gap-3 items-stretch sm:items-end w-full"
          >
            <!-- Dropdown 1: Category -->
            <div class="relative flex-1">
              <label
                for="search-category"
                class="text-[9px] text-[#9E8E7A] font-bold uppercase tracking-wider mb-1 block"
              >
                {m.auto_what_are_you_looking()}
              </label>
              <div class="relative">
                <select
                  id="search-category"
                  bind:value={category}
                  class="w-full h-11 bg-[#FAF6F0] border border-[#EAE0D0] hover:border-[#C9A96E] rounded-full pl-5 pr-10 rtl:pl-10 rtl:pr-5 text-xs font-extrabold text-[#2D2620] focus:outline-none appearance-none cursor-pointer"
                >
                  <option value="">{m.auto_all_services()}</option>
                  {#each categories as cat}
                    <option value={cat.slug || cat.key}
                      >{getLocale() === "ar"
                        ? cat.labelAr || cat.ar || cat.key
                        : cat.labelEn || cat.en || cat.key}</option
                    >
                  {/each}
                </select>
                <div
                  class="absolute inset-y-0 right-4 rtl:left-4 rtl:right-auto flex items-center pointer-events-none text-[#9E8E7A]"
                >
                  <svg viewBox="0 0 20 20" class="w-4 h-4" fill="currentColor">
                    <path
                      fill-rule="evenodd"
                      d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z"
                      clip-rule="evenodd"
                    />
                  </svg>
                </div>
              </div>
            </div>

            <!-- Dropdown 2: City -->
            <div class="relative flex-1">
              <label
                for="search-city"
                class="text-[9px] text-[#9E8E7A] font-bold uppercase tracking-wider mb-1 block"
              >
                {m.auto_city()}
              </label>
              <div class="relative">
                <select
                  id="search-city"
                  bind:value={city}
                  class="w-full h-11 bg-[#FAF6F0] border border-[#EAE0D0] hover:border-[#C9A96E] rounded-full pl-5 pr-10 rtl:pl-10 rtl:pr-5 text-xs font-extrabold text-[#2D2620] focus:outline-none appearance-none cursor-pointer"
                >
                  <option value="">{m.auto_all_cities()}</option>
                  {#each filteredCities as c}
                    <option value={c.slug}
                      >{getLocale() === "ar" ? c.name_ar : c.name_en}</option
                    >
                  {/each}
                </select>
                <div
                  class="absolute inset-y-0 right-4 rtl:left-4 rtl:right-auto flex items-center pointer-events-none text-[#9E8E7A]"
                >
                  <svg viewBox="0 0 20 20" class="w-4 h-4" fill="currentColor">
                    <path
                      fill-rule="evenodd"
                      d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z"
                      clip-rule="evenodd"
                    />
                  </svg>
                </div>
              </div>
            </div>

            <!-- Pink Button -->
            <button
              type="submit"
              class="px-6 h-11 rounded-full font-bold text-xs bg-[#EE7E97] hover:bg-[#E56A85] text-white transition-all duration-300 shadow-md hover:shadow-lg cursor-pointer whitespace-nowrap select-none flex items-center justify-center shrink-0"
            >
              {m.auto_service_providers_li()}
            </button>
          </form>
        </div>

        <!-- VERTICAL DIVIDER -->
        <div class="hidden lg:block w-[1px] bg-[#EAE0D0] self-stretch my-2"></div>

        <!-- RIGHT PANEL: Afrah Concierge -->
        <div class="w-full lg:w-[48%] flex flex-col justify-between">
          <div class="flex items-start gap-4">
            <!-- Planner Avatar -->
            <div
              class="w-16 h-16 md:w-20 md:h-20 rounded-full border-4 border-[#EE7E97]/20 overflow-hidden shrink-0 shadow-sm"
            >
              <img
                src="/afrah_avatar.webp"
                alt="Afrah"
                class="w-full h-full object-cover"
              />
            </div>
            <!-- Planner Intro -->
            <div class="flex-1 min-w-0">
              <span
                class="text-[9px] tracking-wider text-[#008080] font-extrabold uppercase mb-1 block"
              >
                {isAr ? 'مساعدك الشخصي' : 'Your Personal Assistant'}
              </span>
              <h3
                class="font-display text-xl md:text-2xl font-bold text-[#2D2620] mb-1.5 leading-tight"
              >
                {isAr ? 'تحتاج مساعدة في اتخاذ القرار؟' : 'Need Help Deciding?'}
              </h3>
              <p class="text-[11px] text-[#9E8E7A] font-medium leading-relaxed">
                {isAr ? 'أخبرنا بما تبحث عنه وسنوصلك بأفضل المزودين.' : 'Tell us what you are looking for and we will match you with the perfect vendor.'}
              </p>
            </div>
          </div>

          <div class="mt-6">
            <div
              class="flex flex-col sm:flex-row gap-3 items-stretch sm:items-end w-full"
            >
              <!-- Expected Wedding Date Selector -->
              <div class="relative flex-1">
                <label
                  for="concierge-date"
                  class="text-[9px] text-[#9E8E7A] font-bold uppercase tracking-wider mb-1 block"
                >
                  {m.auto_expected_wedding_dat()}
                </label>
                <div class="relative">
                  <select
                    id="concierge-date"
                    bind:value={expectedWeddingDate}
                    class="w-full h-11 bg-[#FAF6F0] border border-[#EAE0D0] hover:border-[#C9A96E] rounded-full pl-5 pr-10 rtl:pl-10 rtl:pr-5 text-xs font-extrabold text-[#2D2620] focus:outline-none appearance-none cursor-pointer"
                  >
                    <option value="">{m.auto_select_a_date()}</option>
                    {#each nextMonths as mItem}
                      <option value={mItem.value}>{mItem.label}</option>
                    {/each}
                  </select>
                  <div
                    class="absolute inset-y-0 right-4 rtl:left-4 rtl:right-auto flex items-center pointer-events-none text-[#9E8E7A]"
                  >
                    <svg viewBox="0 0 20 20" class="w-4 h-4" fill="currentColor">
                      <path
                        fill-rule="evenodd"
                        d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z"
                        clip-rule="evenodd"
                      />
                    </svg>
                  </div>
                </div>
              </div>

              <!-- Name field -->
              <div class="flex-1">
                <label
                  for="concierge-name"
                  class="text-[9px] text-[#9E8E7A] font-bold uppercase tracking-wider mb-1 block"
                >
                  {isAr ? 'الاسم' : 'Your Name'}
                </label>
                <input
                  id="concierge-name"
                  type="text"
                  bind:value={conciergeName}
                  placeholder={isAr ? 'مثال: أحمد محمد' : 'e.g. Ahmed Mohammed'}
                  class="w-full h-11 bg-[#FAF6F0] border border-[#EAE0D0] hover:border-[#C9A96E] rounded-full px-5 text-xs font-extrabold text-[#2D2620] focus:outline-none focus:ring-2 focus:ring-[#C9A96E]"
                  required
                />
              </div>

              <!-- Mobile field -->
              <div class="flex-1 relative">
                <label
                  for="concierge-mobile"
                  class="text-[9px] text-[#9E8E7A] font-bold uppercase tracking-wider mb-1 block"
                >
                  {isAr ? 'رقم الهاتف' : 'Phone'}
                </label>
                <input
                  id="concierge-mobile"
                  type="tel"
                  bind:value={conciergeMobile}
                  placeholder="+966512345678"
                  class="w-full h-11 bg-[#FAF6F0] border border-[#EAE0D0] hover:border-[#C9A96E] rounded-full px-5 text-xs font-extrabold text-[#2D2620] focus:outline-none focus:ring-2 focus:ring-[#C9A96E] ltr"
                  dir="ltr"
                  required
                />
                <label class="absolute left-0 right-0 top-full flex items-center gap-1.5 mt-2 ps-3 cursor-pointer select-none">
                  <input type="checkbox" bind:checked={conciergeWhatsapp} class="w-3.5 h-3.5 rounded accent-green-600" />
                  <span class="text-[9px] text-[#9E8E7A] font-medium">{isAr ? 'واتساب' : 'WhatsApp'}</span>
                </label>
              </div>

              <!-- Error message -->
              {#if conciergeError}
                <p class="text-[10px] text-red-500 font-medium flex-shrink-0">{conciergeError}</p>
              {/if}

              <!-- Start Quickly Button -->
              <button
                onclick={startConcierge}
                disabled={isSubmitting || isSuccess}
                class="px-6 h-11 rounded-full font-bold text-xs transition-all duration-300 shadow-md hover:shadow-lg cursor-pointer disabled:cursor-not-allowed whitespace-nowrap select-none flex items-center justify-center shrink-0
                  {isSuccess ? 'bg-green-500 text-white' : 'bg-[#5EBEB2] hover:bg-[#4EA89D] disabled:bg-gray-300 text-white'}"
              >
                {#if isSubmitting}
                  <svg
                    class="animate-spin -ml-1 mr-2 h-4 w-4 text-white"
                    fill="none"
                    viewBox="0 0 24 24"
                  >
                    <circle
                      class="opacity-25"
                      cx="12"
                      cy="12"
                      r="10"
                      stroke="currentColor"
                      stroke-width="4"
                    ></circle>
                    <path
                      class="opacity-75"
                      fill="currentColor"
                      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                    ></path>
                  </svg>
                {/if}
                {isSuccess ? (isAr ? '✅ تم الإرسال!' : '✅ Sent!') : m.auto_start_quickly()}
              </button>
            </div>
          </div>
        </div>

        <!-- Specific date click here (Absolutely positioned at the bottom-end of the card) -->
        <a
          href="/afrah"
          class="absolute bottom-3 md:bottom-4 lg:bottom-5 end-6 md:end-8 lg:end-10 text-[10px] text-[#9E8E7A] hover:text-[#2D2620] mt-0 block font-medium transition-colors duration-200"
        >
          {m.auto_if_you_have_a_specif()}
        </a>
      </div>

      <!-- Trending tags -->
      <div class="flex flex-wrap items-center justify-center gap-2 mt-6">
        <span class="text-xs text-white/80 font-medium select-none">
          {m.auto_trending()}
        </span>
        {#each getLocale() === "ar" ? ["الرياض", "جدة", "الدمام", "قاعات زفاف", "حفلات خطوبة"] : ["Riyadh", "Jeddah", "Weddings", "Engagement", "Corporate"] as tag}
          <a
            href="/"
            class="text-xs bg-white/10 hover:bg-[#C9A96E]/30 border border-white/20 text-white/90 hover:text-white rounded-full px-4 py-1.5 transition-all duration-200 shadow-sm"
          >
            {tag}
          </a>
        {/each}
      </div>
    </div>
  </div>
</section>
