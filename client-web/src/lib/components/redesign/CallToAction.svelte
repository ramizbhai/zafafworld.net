<script lang="ts">
  import { getLocale } from "$lib/paraglide/runtime.js";
  import Button from "./Button.svelte";

  const isRtl = $derived(getLocale() === "ar");
</script>

<section
  class="relative py-zw-20 md:py-zw-28 bg-zw-secondary text-zw-surface overflow-hidden border-t border-zw-primary/20"
  aria-labelledby="cta-heading"
>
  <!-- Subtle radial gold glow gradient background -->
  <div class="absolute inset-0 bg-[radial-gradient(circle_at_center,rgba(201,169,110,0.12)_0%,transparent_70%)] pointer-events-none" aria-hidden="true"></div>

  <div class="relative z-10 w-full max-w-[1400px] mx-auto px-zw-4 sm:px-zw-6 md:px-zw-8 lg:px-zw-12 text-center flex flex-col items-center">
    <!-- Icon/Badge -->
    <div class="text-zw-primary text-zw-3xl mb-zw-4 select-none animate-fade-in" aria-hidden="true">
      ★
    </div>

    <!-- Headings -->
    <h2 id="cta-heading" class="font-display text-zw-4xl md:text-zw-5xl font-bold max-w-3xl mb-zw-4 leading-tight">
      {isRtl ? "دعنا نساعدك في التخطيط لليلة العمر المثالية" : "Let Us Coordinate Your Perfect Wedding Night"}
    </h2>
    <p class="text-zw-muted text-zw-xs md:text-zw-sm max-w-xl mb-zw-10 leading-relaxed">
      {isRtl
        ? "سواء كنت تبحث عن قاعة فخمة أو تفضل تجهيز الكوشة والزهور والضيافة، نحن هنا لنجمع لك أرقى الخيارات بلمسة واحدة وبأفضل الأسعار."
        : "Whether you need a royal palace or boutique flower arrangements and expert designers, we assemble the best Gulf region packages for you."}
    </p>

    <!-- CTAs Row -->
    <div class="flex flex-col sm:flex-row gap-zw-4 mb-zw-12 w-full sm:w-auto">
      <Button href="/venues" variant="primary" size="lg" class="font-bold">
        {isRtl ? "ابدأ التخطيط الآن" : "Start Planning Now"}
      </Button>
      <Button href="/contact" variant="outline" size="lg" class="font-bold border-white/20 text-white hover:bg-white/10 hover:text-white">
        {isRtl ? "تواصل مع مستشار زفاف" : "Talk to a Consultant"}
      </Button>
    </div>

    <!-- Trust Text -->
    <p class="text-[10px] text-zw-muted uppercase tracking-widest mb-zw-12 font-bold select-none">
      🛡️ {isRtl 
        ? "أكثر من ٥,٠٠٠ كوبل حجزوا قاعاتهم وخططوا لزفافهم بثقة وأمان" 
        : "Over 5,000 couples successfully planned and secured their bookings"}
    </p>

    <hr class="w-full max-w-md border-white/10 mb-zw-12" />

    <!-- Small Statistics Grid -->
    <div class="grid grid-cols-3 gap-zw-6 md:gap-zw-12 max-w-xl w-full">
      <div class="flex flex-col gap-zw-1">
        <span class="font-display text-zw-2xl md:text-zw-3xl font-bold text-zw-primary leading-none">500+</span>
        <span class="text-[9px] text-zw-muted font-bold uppercase tracking-wider">
          {isRtl ? "قاعة ومزود خدمة" : "Venues & Vendors"}
        </span>
      </div>
      <div class="flex flex-col gap-zw-1 border-x border-white/10 px-zw-2">
        <span class="font-display text-zw-2xl md:text-zw-3xl font-bold text-zw-primary leading-none">10k+</span>
        <span class="text-[9px] text-zw-muted font-bold uppercase tracking-wider">
          {isRtl ? "كوبل سعيد" : "Happy Couples"}
        </span>
      </div>
      <div class="flex flex-col gap-zw-1">
        <span class="font-display text-zw-2xl md:text-zw-3xl font-bold text-zw-primary leading-none">99.8%</span>
        <span class="text-[9px] text-zw-muted font-bold uppercase tracking-wider">
          {isRtl ? "نسبة نجاح الحجز" : "Booking Success"}
        </span>
      </div>
    </div>
  </div>
</section>
