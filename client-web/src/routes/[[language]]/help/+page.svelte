<script lang="ts">
  import * as m from '$lib/paraglide/messages.js';
import { getLocale } from '$lib/paraglide/runtime.js';
import { getLocalizedField } from '$lib/utils/localize.js';

  const helpTopics = $derived([
    {
      icon: '🏛',
      titleAr: 'دليل القاعات والبحث',
      titleEn: 'Venue Catalog & Search',
      descAr: 'تعلم كيفية استخدام تصفية القاعات المتقدمة لفلترة المواقع حسب منطقتك الجغرافية واحتياجات التفريد الخاصة بك.',
      descEn: 'Learn how to leverage our advanced filters to find and sort wedding halls by geography, capacity, and partition setups.'
    },
    {
      icon: '✉️',
      titleAr: 'التواصل والدردشة المباشرة',
      titleEn: 'Direct Chat & Inquiries',
      descAr: 'طريقة إرسال الاستفسارات والمتابعة مع الموردين ومشاركة الملحقات ومخططات الحفلات في محادثات حية.',
      descEn: 'How to dispatch inquiries, chat dynamically with suppliers, exchange design templates, and coordinate wedding plans.'
    },
    {
      icon: '✨',
      titleAr: 'المساعد التفاعلي',
      titleEn: 'Interactive Planner',
      descAr: 'اكتشف إمكانيات مساعدة التخطيط التفاعلية للحصول على مقترحات وإجابات فورية لأسئلتك التخطيطية.',
      descEn: 'Explore how our interactive planner suggests venue options and answers questions to ease your organization journey.'
    }
  ]);
</script>

<svelte:head>
  <title>{m.footer_helpCenter()} - {m.meta_siteName()}</title>
</svelte:head>

<div class="bg-[var(--color-surface-alt)] min-h-screen">
  <!-- Hero -->
  <div class="bg-[var(--color-secondary)] text-white py-16 sm:py-20 relative overflow-hidden">
    <div class="absolute inset-0 opacity-10" aria-hidden="true">
      <div class="absolute top-0 start-0 w-80 h-80 rounded-full bg-[var(--color-primary)] -translate-x-1/4 -translate-y-1/4"></div>
    </div>
    <div class="container-page relative z-10">
      <span class="divider-gold"></span>
      <h1 class="font-display text-3xl sm:text-4xl font-bold mt-6 mb-4">
        {m.footer_helpCenter()}
      </h1>
      <p class="text-sm sm:text-base text-white/70 max-w-2xl leading-relaxed">
        {m.auto_we_are_here_to_suppo()}
      </p>
    </div>
  </div>

  <!-- Help topics and direct support link -->
  <div class="container-page py-12 sm:py-16">
    <div class="max-w-4xl mx-auto flex flex-col gap-12">
      <!-- Grid of topics -->
      <section aria-labelledby="topics-title">
        <h2 id="topics-title" class="font-display text-xl sm:text-2xl font-bold text-[var(--color-secondary)] mb-6 text-center">
          {m.auto_featured_help_catego()}
        </h2>
        <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
          {#each helpTopics as topic}
            <div class="bg-white rounded-2xl p-6 border border-[var(--color-border)] hover:shadow-md transition-shadow flex flex-col gap-3">
              <span class="text-4xl mb-2" aria-hidden="true">{topic.icon}</span>
              <h3 class="font-display text-base sm:text-lg font-semibold text-[var(--color-secondary)]">
                {getLocalizedField(topic, 'title', getLocale())}
              </h3>
              <p class="text-xs sm:text-sm text-[var(--color-muted)] leading-relaxed">
                {getLocalizedField(topic, 'desc', getLocale())}
              </p>
            </div>
          {/each}
        </div>
      </section>

      <!-- Support Flow CTA -->
      <section class="bg-white border border-[var(--color-border)] rounded-3xl p-6 sm:p-10 flex flex-col sm:flex-row items-center justify-between gap-6 shadow-sm">
        <div class="flex-1 flex flex-col gap-2 text-center sm:text-start">
          <h2 class="font-display text-lg sm:text-xl font-bold text-[var(--color-secondary)]">
            {m.auto_need_direct_assistan()}
          </h2>
          <p class="text-xs sm:text-sm text-[var(--color-muted)] leading-relaxed max-w-lg">
            {m.auto_if_you_could_not_fin()}
          </p>
        </div>
        
        <a
          href="/contact"
          class="inline-flex items-center justify-center px-6 py-3.5 rounded-xl text-xs sm:text-sm font-semibold bg-[var(--color-primary)] text-[var(--color-secondary)] hover:bg-[var(--color-primary-dark)] transition-colors shadow-sm select-none shrink-0"
        >
          {m.footer_contactUs()} ↗
        </a>
      </section>
    </div>
  </div>
</div>
