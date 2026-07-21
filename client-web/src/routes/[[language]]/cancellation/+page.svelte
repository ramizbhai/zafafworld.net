<script lang="ts">
  import * as m from '$lib/paraglide/messages.js';
  import { getLocale } from '$lib/paraglide/runtime.js';
  import { getLocalizedField } from '$lib/utils/localize.js';

  const sections = $derived([
    {
      id: 'deposits',
      titleAr: '١. عربون الحجز والدفعات المباشرة',
      titleEn: '1. Booking Deposits (Arboun)',
      contentAr: 'لتأمين وحجز قاعات الأفراح، قاعات الفنادق، أو خدمات الزفاف المدرجة عبر منصة زفاف وورلد، يلتزم العميل عادةً بدفع عربون حجز ("عربون") مباشرة إلى المورد المعني. يتم تحديد قيمة أو نسبة العربون وشروط الدفع المسبق بالكامل من قبل المورد نفسه وتوضيحها قبل توقيع العقد النهائي.',
      contentEn: 'To secure a booking for a wedding hall, hotel ballroom, or planning service listed on Zafaf World, couples are typically required to pay a booking deposit ("Arboun") directly to the respective vendor. The exact value or percentage of the deposit is established solely by the vendor and disclosed before final contract execution.'
    },
    {
      id: 'cancellation',
      titleAr: '٢. سياسة إلغاء الحجز',
      titleEn: '2. Cancellation Policy',
      contentAr: 'تخضع جميع طلبات إلغاء الحجوزات للعقود والاتفاقيات المبرمة مباشرة بين المقبلين على الزواج والموردين (القاعات والخدمات). تعتمد الفترة الزمنية المسموحة للإلغاء (على سبيل المثال: قبل ٣٠ أو ٦٠ يوماً من تاريخ الحفل) ومدى قابلية استرداد العربون على الشروط والأحكام الخاصة بالمورد. لا تتدخل منصة زفاف وورلد في تحديد أو فرض سياسات الإلغاء.',
      contentEn: 'All cancellation requests are governed exclusively by the direct contracts signed between the couple and the respective vendors. The permissible cancellation notice period (e.g. 30 or 60 days prior to the event) and the refundability of the booking deposit depend entirely on the vendor\'s specific policy. Zafaf World does not establish or enforce cancellation rules.'
    },
    {
      id: 'refunds',
      titleAr: '٣. المبالغ المستردة والنزاعات',
      titleEn: '3. Refunds & Disputes',
      contentAr: 'في حال استحقاق استرداد العربة أو أي مبالغ مالية بموجب شروط إلغاء المورد، يتم تسوية وصرف المبالغ المستردة مباشرة من قبل المورد إلى العميل. لا تقوم منصة زفاف وورلد بتحصيل أو معالجة أو إدارة أي دفعات مالية بالنيابة عن الموردين، وبالتالي لا تتحمل أي مسؤولية قانونية أو تجارية في حال نشوء أي نزاع مالي بين الطرفين.',
      contentEn: 'Where refunds are applicable under the vendor\'s cancellation terms, the refund will be processed and paid directly by the vendor to the client. Zafaf World does not collect, process, or hold any customer payments or deposits, and bears no responsibility or liability for financial disputes between the parties.'
    }
  ]);
</script>

<svelte:head>
  <title>{getLocale() === 'ar' ? 'سياسة الإلغاء والاسترجاع' : 'Cancellation & Refund Policy'} - {m.meta_siteName()}</title>
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
        {getLocale() === 'ar' ? 'سياسة الإلغاء والاسترجاع' : 'Cancellation & Refund Policy'}
      </h1>
      <p class="text-sm sm:text-base text-white/70 max-w-2xl leading-relaxed">
        {getLocale() === 'ar' 
          ? 'دليلك لفهم شروط حجز قاعات الأفراح وسياسات دفع العربون وإلغاء الحجز مع الموردين.' 
          : 'Your guide to understanding wedding hall reservations, deposits, and cancellation agreements.'}
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
            {getLocale() === 'ar' ? 'أقسام السياسة' : 'Policy Sections'}
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

      <!-- Details -->
      <main class="lg:col-span-8 bg-white rounded-2xl border border-[var(--color-border)] p-6 sm:p-8 flex flex-col gap-8">
        {#each sections as sec}
          <section id={sec.id} class="scroll-mt-24">
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
