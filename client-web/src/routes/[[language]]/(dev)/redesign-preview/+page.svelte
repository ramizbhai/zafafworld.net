<!--
  TEMPORARY COMPONENT PREVIEW PAGE
  Path: src/routes/(dev)/redesign-preview/+page.svelte
  Safe to delete in a future phase. Used to visually check new Svelte 5 redesign components.
-->
<script lang="ts">
  import RedesignNavbar from "$lib/components/redesign/RedesignNavbar.svelte";
  import FooterRedesign from "$lib/components/redesign/FooterRedesign.svelte";
  import Section from "$lib/components/redesign/Section.svelte";
  import Card from "$lib/components/redesign/Card.svelte";
  import Button from "$lib/components/redesign/Button.svelte";
  import Hero from "$lib/components/redesign/Hero.svelte";
  import TrustIndicators from "$lib/components/redesign/TrustIndicators.svelte";
  import FeaturedServices from "$lib/components/redesign/FeaturedServices.svelte";
  import CategoryShowcase from "$lib/components/redesign/CategoryShowcase.svelte";
  import VenueGallery from "$lib/components/redesign/VenueGallery.svelte";
  import Testimonials from "$lib/components/redesign/Testimonials.svelte";
  import BlogPreview from "$lib/components/redesign/BlogPreview.svelte";
  import CallToAction from "$lib/components/redesign/CallToAction.svelte";
  import { getLocale } from "$lib/paraglide/runtime.js";

  let { data }: { data: any } = $props();

  const isRtl = $derived(getLocale() === "ar");

  let clickCount = $state(0);
  function handleCardClick() {
    clickCount += 1;
  }
</script>

<svelte:head>
  <title>{isRtl ? "معاينة تصميم زفاف وورلد الجديد" : "ZafafWorld Redesign - Component Preview"}</title>
  <meta name="description" content={isRtl ? "الصفحة التعريفية الجديدة لـ زفاف وورلد - حجز صالات وقاعات الأفراح والمناسبات في الرياض، جدة، والخليج." : "Preview of the new ZafafWorld redesign platform - booking grand halls and wedding venues in Riyadh, Jeddah, and the Gulf."} />
  <link rel="canonical" href="https://zafafworld.net/redesign-preview" />
  
  <!-- Open Graph -->
  <meta property="og:title" content={isRtl ? "معاينة تصميم زفاف وورلد الجديد" : "ZafafWorld Redesign Preview"} />
  <meta property="og:description" content={isRtl ? "الصفحة التعريفية الجديدة لـ زفاف وورلد - حجز صالات وقاعات الأفراح والمناسبات في الرياض، جدة، والخليج." : "Preview of the new ZafafWorld redesign platform."} />
  <meta property="og:type" content="website" />
  <meta property="og:url" content="https://zafafworld.net/redesign-preview" />
  <meta property="og:image" content="https://zafafworld.net/hero.webp" />
  <meta property="og:site_name" content="ZafafWorld" />
  
  <!-- Twitter Card -->
  <meta name="twitter:card" content="summary_large_image" />
  <meta name="twitter:title" content={isRtl ? "معاينة تصميم زفاف وورلد الجديد" : "ZafafWorld Redesign Preview"} />
  <meta name="twitter:description" content={isRtl ? "الصفحة التعريفية الجديدة لـ زفاف وورلد." : "Preview of the new ZafafWorld redesign platform."} />
  <meta name="twitter:image" content="https://zafafworld.net/hero.webp" />

  <!-- JSON-LD Structured Data Schema -->
  <script type="application/ld+json">
    {
      "@context": "https://schema.org",
      "@type": "WebPage",
      "name": "ZafafWorld Redesign Component Hub",
      "description": "Bilingual wedding and event venue marketplace preview directory",
      "url": "https://zafafworld.net/redesign-preview",
      "publisher": {
        "@type": "Organization",
        "name": "ZafafWorld",
        "logo": {
          "@type": "ImageObject",
          "url": "https://zafafworld.net/logo.png"
        }
      }
    }
  </script>
</svelte:head>

<!-- Redesign Navbar (renders at the top, fixed sticky) -->
<RedesignNavbar />

<main class="w-full">
  <!-- Hero Section -->
  <Hero categories={data.categories} cities={data.cities} />

  <!-- Trust Indicators / Logobar -->
  <TrustIndicators />

  <!-- Featured Services (Diamond Showcase replacement) -->
  <FeaturedServices listings={data.listings} />

  <!-- Curated Category Showcase (Phase 5a) -->
  <CategoryShowcase categories={data.categories} />

  <!-- Featured Venues Gallery (Phase 5a) -->
  <VenueGallery />

  <!-- Customer Success Stories (Phase 5b) -->
  <Testimonials testimonials={data.testimonials} />

  <!-- Latest Wedding Inspiration (Phase 5b) -->
  <BlogPreview posts={data.blogPosts} />

  <!-- Final CTA Section (Phase 5b) -->
  <CallToAction />

  <!-- Title / Introduction Section -->
  <Section bg="surface">
    <div class="text-center max-w-3xl mx-auto mb-zw-12">
      <span class="text-zw-primary text-zw-sm font-bold tracking-widest uppercase mb-zw-3 block">
        {isRtl ? "مرحلة تصميم المكونات الأساسية" : "Design Token & Component Stage"}
      </span>
      <h2 class="font-display text-zw-4xl md:text-zw-5xl font-bold mb-zw-4">
        {isRtl ? "مستودع مكونات زفاف وورلد الجديد" : "ZafafWorld Redesign Component Hub"}
      </h2>
      <p class="text-zw-muted text-zw-base leading-relaxed">
        {isRtl
          ? "تستعرض هذه الصفحة المكونات المعزولة والمبنية باستخدام الرموز التصميمية الجديدة (zw-). تم التحقق من تباين الألوان ودعم اللغتين والعربية باتجاه اليمين."
          : "This isolated page displays the newly designed core primitives utilizing the `zw-` design tokens. All items verify color contrast, Svelte 5 runes integration, and RTL mirroring."}
      </p>
    </div>
  </Section>

  <!-- Buttons Showcase Section -->
  <Section bg="surface-alt" id="buttons-showcase">
    <div class="mb-zw-8">
      <h3 class="font-display text-zw-3xl font-bold mb-zw-2">{isRtl ? "أزرار الإجراءات" : "Redesign Button Family"}</h3>
      <p class="text-zw-muted text-zw-sm">{isRtl ? "أزرار بمختلف المتغيرات والمقاسات والأحجام." : "Showcasing variant actions, responsive sizing tiers, and disabled behaviors."}</p>
    </div>

    <div class="flex flex-col gap-zw-6">
      <!-- Variants Row -->
      <div class="bg-zw-surface border border-zw-border rounded-zw-2xl p-zw-6 flex flex-wrap gap-zw-4 items-center">
        <Button variant="primary">Primary Gold</Button>
        <Button variant="secondary">Secondary Espresso</Button>
        <Button variant="accent-rose">Accent Rose</Button>
        <Button variant="accent-teal">Accent Teal</Button>
        <Button variant="outline">Outline Button</Button>
        <Button variant="ghost">Ghost Button</Button>
      </div>

      <!-- Sizing Row -->
      <div class="bg-zw-surface border border-zw-border rounded-zw-2xl p-zw-6 flex flex-wrap gap-zw-4 items-end">
        <div class="flex flex-col gap-zw-1">
          <span class="text-zw-xs text-zw-muted">Size: sm</span>
          <Button size="sm" variant="primary">Small Action</Button>
        </div>
        <div class="flex flex-col gap-zw-1">
          <span class="text-zw-xs text-zw-muted">Size: md</span>
          <Button size="md" variant="accent-rose">Medium Action</Button>
        </div>
        <div class="flex flex-col gap-zw-1">
          <span class="text-zw-xs text-zw-muted">Size: lg</span>
          <Button size="lg" variant="accent-teal">Large Action</Button>
        </div>
        <div class="flex flex-col gap-zw-1">
          <span class="text-zw-xs text-zw-muted">Disabled state</span>
          <Button variant="primary" disabled>Disabled Action</Button>
        </div>
      </div>
    </div>
  </Section>

  <!-- Cards Showcase Section -->
  <Section bg="surface" id="cards-showcase">
    <div class="mb-zw-8">
      <h3 class="font-display text-zw-3xl font-bold mb-zw-2">{isRtl ? "البطاقات وحاويات المحتوى" : "Redesign Content Cards"}</h3>
      <p class="text-zw-muted text-zw-sm">{isRtl ? "بطاقات عادية وبطاقات تفاعلية ترتفع للأعلى عند التمرير." : "Generic container elements verifying flexbox alignments and radius scaling."}</p>
    </div>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-zw-6">
      <!-- Standard Card -->
      <Card>
        <h4 class="font-display text-zw-xl font-semibold mb-zw-2">Standard Static Card</h4>
        <p class="text-zw-muted text-zw-sm leading-relaxed mb-zw-4">
          This card is static. It renders content inside a rounded container with standard borders. Ideal for static descriptions, details, or listing reviews.
        </p>
        <div class="flex gap-zw-2">
          <span class="bg-zw-surface-alt text-zw-primary-contrast text-zw-xs font-bold px-zw-3 py-zw-1.5 rounded-zw-md">Tag 1</span>
          <span class="bg-zw-surface-alt text-zw-primary-contrast text-zw-xs font-bold px-zw-3 py-zw-1.5 rounded-zw-md">Tag 2</span>
        </div>
      </Card>

      <!-- Elevated Card -->
      <Card elevated onclick={handleCardClick}>
        <div class="flex justify-between items-start mb-zw-2">
          <h4 class="font-display text-zw-xl font-semibold">Elevated Interactive Card</h4>
          <span class="bg-zw-interactive-rose text-zw-surface text-zw-xs px-zw-2 py-zw-1 rounded-zw-md font-bold">New</span>
        </div>
        <p class="text-zw-muted text-zw-sm leading-relaxed mb-zw-4">
          This card responds to hover states (lifts slightly, deepens shadow, changes border). Click here to verify the interactive buttons state loop.
        </p>
        <div class="flex justify-between items-center">
          <span class="text-zw-xs text-zw-muted">Clicked count: {clickCount}</span>
          <span class="text-zw-primary text-zw-sm font-semibold hover:underline cursor-pointer">Explore →</span>
        </div>
      </Card>
    </div>
  </Section>

  <!-- Typography Scale Showcase -->
  <Section bg="surface-alt" id="typography-showcase">
    <div class="mb-zw-8">
      <h3 class="font-display text-zw-3xl font-bold mb-zw-2">{isRtl ? "تدرج الخطوط وأحجامها" : "Bilingual Typography Hierarchy"}</h3>
      <p class="text-zw-muted text-zw-sm">{isRtl ? "يستعرض هذا القسم الخطوط الجديدة ومقاساتها المعتمدة." : "Testing fonts load states, weights pairings, and line heights variables."}</p>
    </div>

    <div class="bg-zw-surface border border-zw-border rounded-zw-2xl p-zw-6 md:p-zw-8 flex flex-col gap-zw-6">
      <div class="border-b border-zw-border/50 pb-zw-4">
        <h4 class="text-zw-xs text-zw-primary font-bold uppercase tracking-wider mb-zw-2">Font Families (Bilingual Load)</h4>
        <p class="font-display-en text-zw-2xl mb-zw-2">English Display: Playfair Display (The Golden Night of Joy)</p>
        <p class="font-body-en text-zw-base mb-zw-4">English Body: Outfit (Geometric clean paragraphs for descriptions)</p>
        <p class="font-arabic text-zw-2xl mb-zw-2">العربية للعناوين: خط نوتو سانز العربي بوزن خفيف (ليالي الزفاف السعيدة)</p>
        <p class="font-arabic text-zw-base text-zw-muted">العربية للنصوص: خط نوتو سانز العربي بوزن عادي للمواصفات والتفاصيل</p>
      </div>

      <div>
        <h4 class="text-zw-xs text-zw-primary font-bold uppercase tracking-wider mb-zw-4">Typography Scales</h4>
        <div class="flex flex-col gap-zw-4 text-start">
          <div class="flex items-center border-b border-zw-border/30 pb-zw-3"><span class="text-zw-xs text-zw-muted w-24">text-zw-7xl</span><span class="text-zw-7xl font-bold">Luxury</span></div>
          <div class="flex items-center border-b border-zw-border/30 pb-zw-3"><span class="text-zw-xs text-zw-muted w-24">text-zw-6xl</span><span class="text-zw-6xl font-bold">Display Title</span></div>
          <div class="flex items-center border-b border-zw-border/30 pb-zw-3"><span class="text-zw-xs text-zw-muted w-24">text-zw-5xl</span><span class="text-zw-5xl font-semibold">Section Hero</span></div>
          <div class="flex items-center border-b border-zw-border/30 pb-zw-3"><span class="text-zw-xs text-zw-muted w-24">text-zw-4xl</span><span class="text-zw-4xl font-semibold">Card Banner</span></div>
          <div class="flex items-center border-b border-zw-border/30 pb-zw-3"><span class="text-zw-xs text-zw-muted w-24">text-zw-3xl</span><span class="text-zw-3xl font-semibold">Heading H3</span></div>
          <div class="flex items-center border-b border-zw-border/30 pb-zw-3"><span class="text-zw-xs text-zw-muted w-24">text-zw-2xl</span><span class="text-zw-2xl font-semibold">Heading H4</span></div>
          <div class="flex items-center border-b border-zw-border/30 pb-zw-3"><span class="text-zw-xs text-zw-muted w-24">text-zw-xl</span><span class="text-zw-xl font-medium">Sub-Header</span></div>
          <div class="flex items-center border-b border-zw-border/30 pb-zw-3"><span class="text-zw-xs text-zw-muted w-24">text-zw-lg</span><span class="text-zw-lg">Large Text Body</span></div>
          <div class="flex items-center border-b border-zw-border/30 pb-zw-3"><span class="text-zw-xs text-zw-muted w-24">text-zw-base</span><span class="text-zw-base">Base Paragraph Copy</span></div>
          <div class="flex items-center border-b border-zw-border/30 pb-zw-3"><span class="text-zw-xs text-zw-muted w-24">text-zw-sm</span><span class="text-zw-sm">Small Caption Text</span></div>
          <div class="flex items-center"><span class="text-zw-xs text-zw-muted w-24">text-zw-xs</span><span class="text-zw-xs uppercase tracking-wider text-zw-muted">Metadata Label</span></div>
        </div>
      </div>
    </div>
  </Section>

  <!-- Swatch Preview Section for Contrast verification -->
  <Section bg="surface" id="colors-showcase">
    <div class="mb-zw-8">
      <h3 class="font-display text-zw-3xl font-bold mb-zw-2">{isRtl ? "لوحة الألوان وتباين النصوص" : "Color Swatches & Contrast"}</h3>
      <p class="text-zw-muted text-zw-sm">{isRtl ? "مربعات الألوان الجديدة للتحقق البصري من نسب التباين." : "Visual validation of new hex tones and accessibility contrast check."}</p>
    </div>

    <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 gap-zw-4">
      <div class="bg-zw-primary text-zw-secondary p-zw-4 rounded-zw-xl border border-zw-border font-bold flex flex-col gap-zw-6 h-32 justify-between">
        <span class="text-zw-xs">zw-primary</span>
        <span class="text-zw-xs">#C9A96E</span>
      </div>
      <div class="bg-zw-secondary text-zw-surface p-zw-4 rounded-zw-xl border border-zw-border font-bold flex flex-col gap-zw-6 h-32 justify-between">
        <span class="text-zw-xs">zw-secondary</span>
        <span class="text-zw-xs">#2D2620</span>
      </div>
      <div class="bg-zw-primary-contrast text-zw-surface p-zw-4 rounded-zw-xl border border-zw-border font-bold flex flex-col gap-zw-6 h-32 justify-between">
        <span class="text-zw-xs">zw-primary-contrast</span>
        <span class="text-zw-xs">#916F3E</span>
      </div>
      <div class="bg-zw-interactive-rose text-zw-surface p-zw-4 rounded-zw-xl border border-zw-border font-bold flex flex-col gap-zw-6 h-32 justify-between">
        <span class="text-zw-xs">zw-interactive-rose</span>
        <span class="text-zw-xs">#EE7E97</span>
      </div>
      <div class="bg-zw-interactive-teal text-zw-surface p-zw-4 rounded-zw-xl border border-zw-border font-bold flex flex-col gap-zw-6 h-32 justify-between">
        <span class="text-zw-xs">zw-interactive-teal</span>
        <span class="text-zw-xs">#5EBEB2</span>
      </div>
      <div class="bg-zw-surface-alt text-zw-secondary p-zw-4 rounded-zw-xl border border-zw-border font-bold flex flex-col gap-zw-6 h-32 justify-between">
        <span class="text-zw-xs">zw-surface-alt</span>
        <span class="text-zw-xs">#F5EFE6</span>
      </div>
    </div>
  </Section>

  <!-- Spacing testing height block to test navbar scroll state transition -->
  <Section bg="surface-alt">
    <div class="text-center py-zw-12">
      <span class="text-zw-2xl">↓</span>
      <p class="text-zw-sm text-zw-muted mt-zw-2">{isRtl ? "مرر الصفحة لأسفل لمعاينة تأثير الزجاج الشفاف على الشريط العلوي" : "Scroll down to preview the sticky glass transition of the Navbar"}</p>
    </div>
  </Section>
</main>

<!-- Redesign Footer -->
<FooterRedesign />
