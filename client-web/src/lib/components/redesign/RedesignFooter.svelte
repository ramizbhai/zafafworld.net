<script lang="ts">
  import { getLocale } from "$lib/paraglide/runtime.js";
  import Button from "./Button.svelte";

  const isRtl = $derived(getLocale() === "ar");

  let activeAccordion = $state<string | null>(null);

  function toggle(section: string) {
    activeAccordion = activeAccordion === section ? null : section;
  }

  // Footer data structure
  const columns = $derived([
    {
      id: "services",
      title: isRtl ? "خدماتنا" : "Our Services",
      links: [
        { label: isRtl ? "قاعات الأفراح" : "Wedding Venues", href: "/venues" },
        { label: isRtl ? "منظمو الحفلات" : "Wedding Planners", href: "/vendors" },
        { label: isRtl ? "العروض والخصومات" : "Discounts & Offers", href: "/offers" },
        { label: isRtl ? "ألبوم الأفكار" : "Inspiration Hub", href: "/discover" }
      ]
    },
    {
      id: "destinations",
      title: isRtl ? "أهم المدن" : "Popular Cities",
      links: [
        { label: isRtl ? "الرياض" : "Riyadh", href: "/venues?city=riyadh" },
        { label: isRtl ? "جدة" : "Jeddah", href: "/venues?city=jeddah" },
        { label: isRtl ? "الدمام" : "Dammam", href: "/venues?city=dammam" },
        { label: isRtl ? "الخبر" : "Khobar", href: "/venues?city=khobar" }
      ]
    },
    {
      id: "company",
      title: isRtl ? "الشركة" : "Company",
      links: [
        { label: isRtl ? "من نحن" : "About Us", href: "/about" },
        { label: isRtl ? "تواصل معنا" : "Contact Us", href: "/contact" },
        { label: isRtl ? "الأسئلة الشائعة" : "FAQ", href: "/faq" },
        { label: isRtl ? "مركز المساعدة" : "Help Center", href: "/help" }
      ]
    },
    {
      id: "legal",
      title: isRtl ? "الشروط والسياسات" : "Legal",
      links: [
        { label: isRtl ? "شروط الاستخدام" : "Terms of Service", href: "/terms" },
        { label: isRtl ? "سياسة الخصوصية" : "Privacy Policy", href: "/privacy" },
        { label: isRtl ? "سياسة الإلغاء" : "Cancellation Policy", href: "/cancellation" },
        { label: isRtl ? "سياسة ملفات الارتباط" : "Cookie Settings", href: "/cookies" }
      ]
    }
  ]);
</script>

<footer class="bg-zw-surface-dark text-zw-surface border-t border-zw-secondary pt-zw-16 pb-zw-8">
  <div class="w-full max-w-[1400px] mx-auto px-zw-4 sm:px-zw-6 md:px-zw-8 lg:px-zw-12">
    <!-- Top Block: Newsletter Subscription & Brand Introduction -->
    <div class="grid grid-cols-1 lg:grid-cols-12 gap-zw-8 mb-zw-12">
      <div class="lg:col-span-5 flex flex-col gap-zw-4">
        <!-- Logo -->
        <div class="flex items-center gap-zw-2">
          <span class="text-zw-primary text-zw-3xl">★</span>
          <span class="font-display text-zw-2xl font-bold tracking-wide text-zw-surface">
            Zafaf<span class="text-zw-primary">World</span>
          </span>
        </div>
        <p class="text-zw-muted text-zw-xs leading-relaxed max-w-sm">
          {isRtl
            ? "المنصة الأولى والأكثر ثقة لحجز قاعات ومزودي خدمات الأفراح والمناسبات في منطقة الخليج العربي."
            : "The premier and most trusted marketplace for booking wedding venues and professional event suppliers in the Arabian Gulf."}
        </p>
      </div>

      <!-- Newsletter subscription form -->
      <div class="lg:col-span-7 flex flex-col gap-zw-3 lg:ps-zw-8">
        <h3 class="font-display text-zw-base font-semibold text-zw-surface">
          {isRtl ? "اشترك في النشرة البريدية" : "Subscribe to Our Newsletter"}
        </h3>
        <p class="text-zw-muted text-zw-xs">
          {isRtl
            ? "احصل على أفضل العروض والنصائح الحصرية لتخطيط ليلة زفافك المثالية."
            : "Get exclusive discounts, planning checksheets, and seasonal vendor offers directly in your inbox."}
        </p>
        <form class="flex flex-col sm:flex-row gap-zw-2 max-w-xl mt-zw-2" onsubmit={(e) => e.preventDefault()}>
          <label for="newsletter-email" class="sr-only">
            {isRtl ? "البريد الإلكتروني" : "Email Address"}
          </label>
          <input
            id="newsletter-email"
            type="email"
            placeholder={isRtl ? "أدخل بريدك الإلكتروني" : "e.g. bride@example.com"}
            class="flex-grow h-zw-12 bg-zw-secondary/40 border border-zw-muted/30 focus:border-zw-primary text-zw-surface rounded-zw-full px-zw-5 text-zw-xs outline-none transition-all focus:ring-2 focus:ring-zw-primary/20"
            required
          />
          <Button variant="primary" size="md" type="submit" class="h-zw-12">
            <span>{isRtl ? "اشترك الآن" : "Subscribe Now"}</span>
          </Button>
        </form>
      </div>
    </div>

    <hr class="border-zw-border/10 mb-zw-12" />

    <!-- Center Block: Desktop Grid / Mobile Accordions -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-zw-8 mb-zw-12">
      {#each columns as col}
        <!-- Desktop columns -->
        <div class="hidden md:flex flex-col gap-zw-4">
          <h3 class="font-display text-zw-xs font-bold uppercase tracking-wider text-zw-primary animate-fade-in">
            {col.title}
          </h3>
          <ul class="flex flex-col gap-zw-2.5">
            {#each col.links as link}
              <li>
                <a
                  href={link.href}
                  class="text-zw-muted hover:text-zw-primary text-zw-xs transition-colors duration-200 block"
                >
                  {link.label}
                </a>
              </li>
            {/each}
          </ul>
        </div>

        <!-- Mobile accordions -->
        <div class="md:hidden border-b border-zw-border/10 last:border-0 pb-zw-3 last:pb-0">
          <button
            class="w-full flex justify-between items-center py-zw-2 text-start font-display text-zw-xs font-bold uppercase tracking-wider text-zw-primary cursor-pointer select-none"
            onclick={() => toggle(col.id)}
            aria-expanded={activeAccordion === col.id}
            aria-controls="accordion-{col.id}"
          >
            <span>{col.title}</span>
            <span class="text-zw-muted text-[10px] transform transition-transform duration-300 {activeAccordion === col.id ? 'rotate-180' : ''}">
              ▼
            </span>
          </button>
          {#if activeAccordion === col.id}
            <ul id="accordion-{col.id}" class="flex flex-col gap-zw-3 pt-zw-2 pb-zw-4 ps-zw-2">
              {#each col.links as link}
                <li>
                  <a
                    href={link.href}
                    class="text-zw-muted hover:text-zw-primary text-zw-xs transition-colors block"
                  >
                    {link.label}
                  </a>
                </li>
              {/each}
            </ul>
          {/if}
        </div>
      {/each}
    </div>

    <hr class="border-zw-border/10 mb-zw-8" />

    <!-- Bottom Block: Copyright & Social media -->
    <div class="flex flex-col sm:flex-row items-center justify-between gap-zw-4 text-zw-muted text-zw-xs">
      <p>
        © {new Date().getFullYear()} ZafafWorld. {isRtl ? "جميع الحقوق محفوظة." : "All rights reserved."}
      </p>

      <!-- Social Links -->
      <div class="flex items-center gap-zw-4">
        <a href="https://instagram.com/zafafworld" target="_blank" rel="noopener noreferrer" class="hover:text-zw-primary transition-colors" aria-label="Instagram">
          Instagram
        </a>
        <a href="https://twitter.com/zafafworld" target="_blank" rel="noopener noreferrer" class="hover:text-zw-primary transition-colors" aria-label="Twitter">
          Twitter / X
        </a>
        <a href="https://facebook.com/zafafworld" target="_blank" rel="noopener noreferrer" class="hover:text-zw-primary transition-colors" aria-label="Facebook">
          Facebook
        </a>
      </div>
    </div>
  </div>
</footer>
