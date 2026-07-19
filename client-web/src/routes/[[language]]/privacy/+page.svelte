<script lang="ts">
  import * as m from '$lib/paraglide/messages.js';
import { getLocale } from '$lib/paraglide/runtime.js';
import { getLocalizedField } from '$lib/utils/localize.js';

  const sections = $derived([
    {
      id: 'collection',
      titleAr: '١. جمع البيانات الشخصية',
      titleEn: '1. Collection of Personal Data',
      contentAr: 'هذا القسم مخصص لتوضيح فئات البيانات الشخصية التي تجمعها المنصة من العملاء والموردين. يرجى مراجعة وتحديث هذا النص بما يتوافق مع السياسات التجارية الخاصة بكم.\n\n[Business-Approved Policy Segment: Personal Data Collection]',
      contentEn: 'This section details the categories of personal data collected from couples and vendors on the platform. Please review and update this text to align with your approved data handling policies.\n\n[Business-Approved Policy Segment: Personal Data Collection]'
    },
    {
      id: 'processing',
      titleAr: '٢. معالجة البيانات واستخدامها',
      titleEn: '2. Data Processing & Usage',
      contentAr: 'هنا يتم بيان الأغراض التي لأجلها تتم معالجة البيانات، كمعالجة الحجوزات أو تفعيل الخدمات المساعدة للعملاء.\n\n[Business-Approved Policy Segment: Data Processing and Usage]',
      contentEn: 'Outlines the purposes for which user data is processed, such as completing venue bookings or empowering helper services.\n\n[Business-Approved Policy Segment: Data Processing and Usage]'
    },
    {
      id: 'cookies',
      titleAr: '٣. ملفات تعريف الارتباط والتخزين',
      titleEn: '3. Cookies & Local Storage',
      contentAr: 'توضيح كيفية استخدام ملفات تعريف الارتباط والتقنيات المشابهة لتحسين أداء المنصة وتخصيص تجربة المستخدم.\n\n[Business-Approved Policy Segment: Cookies and Local Storage]',
      contentEn: 'Explains how cookies and similar local storage mechanisms are utilized to enhance site performance and save preferences.\n\n[Business-Approved Policy Segment: Cookies and Local Storage]'
    },
    {
      id: 'sharing',
      titleAr: '٤. مشاركة البيانات والإفصاح عنها',
      titleEn: '4. Data Sharing & Disclosures',
      contentAr: 'تفصيل الحالات المسموح فيها بمشاركة البيانات مع الموردين لإتمام الحجوزات أو الإجراءات القانونية المخولة.\n\n[Business-Approved Policy Segment: Data Sharing and Disclosures]',
      contentEn: 'Details the occurrences under which user details may be shared with partners to fulfill bookings or legal obligations.\n\n[Business-Approved Policy Segment: Data Sharing and Disclosures]'
    }
  ]);
</script>

<svelte:head>
  <title>{m.footer_privacy()} - {m.meta_siteName()}</title>
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
        {m.auto_privacy_policy()}
      </h1>
      <p class="text-sm sm:text-base text-white/70 max-w-2xl leading-relaxed">
        {m.auto_we_care_about_your_p()}
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
            {m.auto_privacy_directory()}
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

      <!-- Privacy details -->
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
