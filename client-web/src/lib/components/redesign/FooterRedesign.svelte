<script lang="ts">
  import { getLocale } from "$lib/paraglide/runtime.js";
  import Button from "./Button.svelte";

  const isRtl = $derived(getLocale() === "ar");

  let emailValue = $state("");
  let subscriptionStatus = $state<"idle" | "success" | "error">("idle");

  function handleNewsletterSubmit(e: SubmitEvent) {
    e.preventDefault();
    if (!emailValue) return;

    // TODO: Connect this hook to NewsletterService interface in a later phase.
    console.log(`[Newsletter TODO Hook] User email submitted: ${emailValue}`);
    
    // Simulate UI response state without fetching any real or fake endpoints
    subscriptionStatus = "success";
    emailValue = "";
  }

  // Footer navigation links map
  const columns = $derived([
    {
      id: "planning",
      title: isRtl ? "أدوات التخطيط" : "Planning Tools",
      links: [
        { label: isRtl ? "حاسبة الميزانية" : "Budget Calculator", href: "/planning/budget" },
        { label: isRtl ? "جدول المهام اليومية" : "Wedding Checklist", href: "/planning/checklist" },
        { label: isRtl ? "مستشار تخطيط زفاف" : "Planning Consultant", href: "/afrah" },
        { label: isRtl ? "ألبوم الأفكار والأنماط" : "Inspiration Gallery", href: "/discover" }
      ]
    },
    {
      id: "vendors",
      title: isRtl ? "مزودو الخدمات" : "Wedding Vendors",
      links: [
        { label: isRtl ? "قاعات الأفراح والقصور" : "Wedding Halls & Palaces", href: "/venues" },
        { label: isRtl ? "منظمو الحفلات والكوش" : "Wedding Planners & Stylists", href: "/vendors?category=wedding-planner" },
        { label: isRtl ? "مصورو الفوتو والفيديو" : "Photography & Videography", href: "/vendors?category=photography-video" },
        { label: isRtl ? "الضيافة والبوفيه المفتوح" : "Catering & Hospitality", href: "/vendors?category=catering" }
      ]
    },
    {
      id: "company",
      title: isRtl ? "الشركة" : "Company",
      links: [
        { label: isRtl ? "من نحن" : "About Us", href: "/about" },
        { label: isRtl ? "مجلة زفاف وورلد" : "ZafafWorld Magazine", href: "/discover" },
        { label: isRtl ? "تواصل معنا" : "Contact Us", href: "/contact" },
        { label: isRtl ? "مركز المساعدة والدعم" : "Help & Support Center", href: "/help" }
      ]
    },
    {
      id: "legal",
      title: isRtl ? "الشروط والسياسات" : "Legal & Privacy",
      links: [
        { label: isRtl ? "شروط الاستخدام" : "Terms of Service", href: "/terms" },
        { label: isRtl ? "سياسة الخصوصية" : "Privacy Policy", href: "/privacy" },
        { label: isRtl ? "سياسة ملفات الارتباط" : "Cookie Settings", href: "/cookies" },
        { label: isRtl ? "شروط الإلغاء والحجز" : "Cancellation Terms", href: "/cancellation" }
      ]
    }
  ]);

  let activeAccordion = $state<string | null>(null);

  function toggleAccordion(id: string) {
    activeAccordion = activeAccordion === id ? null : id;
  }
</script>

<footer class="bg-zw-secondary text-zw-surface border-t border-zw-primary/10 pt-zw-16 pb-zw-8 relative z-10" aria-label="Site Footer">
  <div class="w-full max-w-[1400px] mx-auto px-zw-4 sm:px-zw-6 md:px-zw-8 lg:px-zw-12">
    
    <!-- Top Row: Newsletter + Brand Introduction -->
    <div class="grid grid-cols-1 lg:grid-cols-12 gap-zw-8 mb-zw-12">
      <!-- Brand Info -->
      <div class="lg:col-span-5 flex flex-col gap-zw-4 text-start">
        <div class="flex items-center gap-zw-2 select-none">
          <span class="text-zw-primary text-zw-3xl">★</span>
          <span class="font-display text-zw-2xl font-bold tracking-wide text-white">
            Zafaf<span class="text-zw-primary">World</span>
          </span>
        </div>
        <p class="text-zw-muted text-zw-xs leading-relaxed max-w-sm">
          {isRtl
            ? "المنصة العربية الرائدة والأكثر ثقة لحجز قاعات ومزودي خدمات الأفراح والمناسبات في منطقة الخليج العربي. نجمع لك الفخامة والسهولة في مكان واحد."
            : "The premier Arabic marketplace for booking luxury wedding venues and Suppliers in the Arabian Gulf. Luxury and simplicity in one place."}
        </p>
      </div>

      <!-- Newsletter Signup -->
      <div class="lg:col-span-7 flex flex-col gap-zw-3 lg:ps-zw-8 text-start">
        <h3 class="font-display text-zw-base font-semibold text-white">
          {isRtl ? "اشترك في نشرتنا الملهمة" : "Subscribe to Our Inspiration Letter"}
        </h3>
        <p class="text-zw-muted text-zw-xs">
          {isRtl
            ? "احصل على نصائح الخبراء الأسبوعية للتخطيط لزفافك، وأحدث باليتات الألوان، وعروض القاعات الحصرية."
            : "Receive weekly expert guides, color palette ideas, and exclusive seasonal venue offers."}
        </p>
        
        <form class="flex flex-col sm:flex-row gap-zw-2 max-w-xl mt-zw-2" onsubmit={handleNewsletterSubmit}>
          <label for="newsletter-email" class="sr-only">
            {isRtl ? "البريد الإلكتروني" : "Email Address"}
          </label>
          <input
            id="newsletter-email"
            type="email"
            placeholder={isRtl ? "أدخل بريدك الإلكتروني" : "e.g. bride@example.com"}
            bind:value={emailValue}
            class="flex-grow h-zw-12 bg-white/5 border border-white/10 focus:border-zw-primary text-white rounded-zw-full px-zw-5 text-zw-xs outline-none transition-all focus:ring-2 focus:ring-zw-primary/20"
            required
            aria-required="true"
          />
          <Button variant="primary" size="md" type="submit" class="h-zw-12">
            <span>{isRtl ? "اشترك الآن" : "Subscribe"}</span>
          </Button>
        </form>

        {#if subscriptionStatus === "success"}
          <p class="text-[10px] text-green-400 font-bold mt-zw-1 animate-fade-in" aria-live="polite">
            ✓ {isRtl ? "شكراً لاشتراكك! ترقب رسائلنا قريباً." : "Thank you for subscribing! Keep an eye on your inbox."}
          </p>
        {/if}
      </div>
    </div>

    <hr class="border-white/10 mb-zw-12" />

    <!-- Center Navigation Links: Desktop Grid / Mobile Accordions -->
    <div class="grid grid-cols-1 md:grid-cols-4 gap-zw-8 mb-zw-12">
      {#each columns as col}
        <!-- Desktop list view -->
        <nav class="hidden md:flex flex-col gap-zw-4 text-start" aria-label={`${col.title} Links`}>
          <h4 class="font-display text-zw-xs font-bold uppercase tracking-wider text-zw-primary select-none">
            {col.title}
          </h4>
          <ul class="flex flex-col gap-zw-3">
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
        </nav>

        <!-- Mobile collapsible accordion view -->
        <div class="md:hidden border-b border-white/5 last:border-0 pb-zw-3 last:pb-0">
          <button
            class="w-full flex justify-between items-center py-zw-2 text-start font-display text-zw-xs font-bold uppercase tracking-wider text-zw-primary cursor-pointer select-none focus-visible:outline focus-visible:outline-2 focus-visible:outline-zw-primary"
            onclick={() => toggleAccordion(col.id)}
            aria-expanded={activeAccordion === col.id}
            aria-controls={`accordion-panel-${col.id}`}
          >
            <span>{col.title}</span>
            <span class="text-zw-muted text-[10px] transform transition-transform duration-300 {activeAccordion === col.id ? 'rotate-180' : ''}" aria-hidden="true">
              ▼
            </span>
          </button>
          {#if activeAccordion === col.id}
            <ul id={`accordion-panel-${col.id}`} class="flex flex-col gap-zw-3 pt-zw-2 pb-zw-4 ps-zw-2 animate-fade-in">
              {#each col.links as link}
                <li>
                  <a
                    href={link.href}
                    class="text-zw-muted hover:text-zw-primary text-zw-xs transition-colors block text-start"
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

    <hr class="border-white/10 mb-zw-8" />

    <!-- Bottom copyright + social media links -->
    <div class="flex flex-col sm:flex-row items-center justify-between gap-zw-4 text-zw-muted text-zw-xs">
      <p class="select-none">
        © {new Date().getFullYear()} ZafafWorld. {isRtl ? "جميع الحقوق محفوظة." : "All rights reserved."}
      </p>

      <!-- Social Icons list -->
      <nav class="flex items-center gap-zw-4" aria-label="Social media networks">
        <a href="https://instagram.com/zafafworld" target="_blank" rel="noopener noreferrer" class="hover:text-zw-primary transition-colors font-semibold" aria-label="Instagram">
          Instagram
        </a>
        <a href="https://twitter.com/zafafworld" target="_blank" rel="noopener noreferrer" class="hover:text-zw-primary transition-colors font-semibold" aria-label="Twitter / X">
          Twitter / X
        </a>
        <a href="https://facebook.com/zafafworld" target="_blank" rel="noopener noreferrer" class="hover:text-zw-primary transition-colors font-semibold" aria-label="Facebook">
          Facebook
        </a>
      </nav>
    </div>

  </div>
</footer>
