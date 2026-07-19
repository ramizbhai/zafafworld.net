<script lang="ts">
  import * as m from '$lib/paraglide/messages.js';
import { getLocale } from '$lib/paraglide/runtime.js';
import { getLocalizedField } from '$lib/utils/localize.js';

  const sections = $derived([
    {
      id: 'agreement',
      titleAr: '١. الاتفاقية العامة',
      titleEn: '1. General Agreement',
      contentAr: 'هذا القسم مخصص للبنود العامة الحاكمة للعلاقة التعاقدية بين مستخدمي المنصة وإدارتها. يرجى مراجعة وتحديث هذا النص بما يتوافق مع السياسات التجارية الخاصة بكم.\n\n[Business-Approved Policy Segment: General Platform Agreement]',
      contentEn: 'This section contains the general terms governing the relationship between platform users and management. Please review and update this text to align with your approved commercial policies.\n\n[Business-Approved Policy Segment: General Platform Agreement]'
    },
    {
      id: 'roles',
      titleAr: '٢. أدوار المنصة ومسؤولياتها',
      titleEn: '2. Platform Roles & Responsibilities',
      contentAr: 'تحدد هذه الفقرة مسؤوليات وحقوق العملاء (المقبلين على الزواج) والموردين (مقدمي الخدمات وقاعات الأفراح) عبر المنصة.\n\n[Business-Approved Policy Segment: Platform Roles and Responsibilities]',
      contentEn: 'This clause outlines the respective roles and responsibilities of couples (clients) and vendors (service and hall providers) on the marketplace.\n\n[Business-Approved Policy Segment: Platform Roles and Responsibilities]'
    },
    {
      id: 'accounts',
      titleAr: '٣. تسجيل الحسابات والأمان',
      titleEn: '3. Account Registration & Security',
      contentAr: 'مخصص للالتزامات المتعلقة بأمان كلمات المرور وحسابات المستخدمين وسرية بيانات الدخول الخاصة بهم.\n\n[Business-Approved Policy Segment: Account Registration and Security]',
      contentEn: 'Dedicated to user responsibilities regarding password safety, account security, and data access confidentiality.\n\n[Business-Approved Policy Segment: Account Registration and Security]'
    },
    {
      id: 'commercial',
      titleAr: '٤. الشروط التجارية والمالية',
      titleEn: '4. Commercial & Financial Terms',
      contentAr: 'يشمل هذا القسم سياسات الرسوم والدفعات والعروض المدرجة من قبل الشركاء في المنصة.\n\n[Business-Approved Policy Segment: Commercial and Financial Terms]',
      contentEn: 'Includes payment guidelines, fee structures, and commercial offerings published by marketplace partners.\n\n[Business-Approved Policy Segment: Commercial and Financial Terms]'
    }
  ]);
</script>

<svelte:head>
  <title>{m.footer_terms()} - {m.meta_siteName()}</title>
</svelte:head>

<div class="bg-[var(--color-surface-alt)]">
  <!-- Hero -->
  <div class="bg-[var(--color-secondary)] text-white py-16 sm:py-20 relative overflow-hidden">
    <div class="absolute inset-0 opacity-10" aria-hidden="true">
      <div class="absolute top-0 start-0 w-80 h-80 rounded-full bg-[var(--color-primary)] -translate-x-1/4 -translate-y-1/4"></div>
    </div>
    <div class="container-page relative z-10">
      <span class="divider-gold"></span>
      <h1 class="font-display text-3xl sm:text-4xl font-bold mt-6 mb-4">
        {m.auto_terms__conditions()}
      </h1>
      <p class="text-sm sm:text-base text-white/70 max-w-2xl leading-relaxed">
        {m.auto_please_read_these_te()}
      </p>
    </div>
  </div>

  <!-- Content section -->
  <div class="container-page py-12 sm:py-16">
    <div class="grid grid-cols-1 lg:grid-cols-12 gap-8">
      <!-- Sidebar index -->
      <aside class="lg:col-span-4" aria-label="Table of contents">
        <div class="bg-white rounded-2xl border border-[var(--color-border)] p-6 sticky top-24">
          <h2 class="font-display text-lg font-bold text-[var(--color-secondary)] mb-4 pb-3 border-b border-[var(--color-border)]">
            {m.auto_terms_directory()}
          </h2>
          <nav class="flex flex-col gap-2">
            {#each sections as sec}
              <a
                href="#{sec.id}"
                class="block px-3 py-2 rounded-xl text-sm font-medium text-[var(--color-text)] hover:bg-[var(--color-surface-alt)] hover:text-[var(--color-primary)] transition-all"
              >
                {getLocalizedField(sec, 'title', getLocale())}
              </a>
            {/each}
          </nav>
        </div>
      </aside>

      <!-- Terms details -->
      <main class="lg:col-span-8 bg-white rounded-2xl border border-[var(--color-border)] p-6 sm:p-8 flex flex-col gap-8">
        {#each sections as sec}
          <section id={sec.id} class="scroll-">
            <h2 class="font-display text-xl font-bold text-[var(--color-secondary)] mb-4 flex items-center gap-2">
              <span class="text-[var(--color-primary)]">•</span>
              {getLocalizedField(sec, 'title', getLocale())}
            </h2>
            <div class="text-sm sm:text-base text-[var(--color-muted)] leading-relaxed whitespace-pre-wrap bg-[var(--color-surface-alt)] p-5 rounded-xl border border-[var(--color-border)]">
              {getLocalizedField(sec, 'content', getLocale())}
            </div>
          </section>
        {/each}
      </main>
    </div>
  </div>
</div>
