<script lang="ts">
  import * as m from '$lib/paraglide/messages.js';
  import { getLocale } from '$lib/paraglide/runtime.js';
  import { getLocalizedField } from '$lib/utils/localize.js';

  const sections = $derived([
    {
      id: 'agreement',
      titleAr: '١. الاتفاقية العامة',
      titleEn: '1. General Agreement',
      contentAr: 'مرحبًا بكم في زفاف وورلد (zafafworld.net). باستخدامك للمنصة أو وصولك إليها، فإنك توافق على الالتزام بشروط وأحكام الاستخدام هذه بالكامل. زفاف وورلد هو سوق إلكتروني رائد ومخصص لخدمات الزفاف يربط بين المقبلين على الزواج وبين قاعات الأفراح، الفنادق، ومنظمي الحفلات، ومزودي الخدمات في المملكة العربية السعودية. تحكم هذه الشروط استخدامك للموقع الإلكتروني، إصدار الهاتف المحمول، وأي خدمات مساعدة تقدمها المنصة.',
      contentEn: 'Welcome to Zafaf World (zafafworld.net). By accessing or using our platform, you agree to comply with and be bound by these Terms & Conditions. Zafaf World is a premier wedding marketplace connecting engaged couples with wedding venues, hotel ballrooms, planners, and service vendors across the Kingdom of Saudi Arabia. These terms govern your use of the website, mobile interfaces, and any helper services.'
    },
    {
      id: 'roles',
      titleAr: '٢. أدوار المنصة ومسؤولياتها',
      titleEn: '2. Platform Roles & Responsibilities',
      contentAr: 'تعمل منصة زفاف وورلد كمنصة وسيطة لتسهيل اكتشاف مزودي الخدمات والمقارنة بينهم وتقديم طلبات الحجز والاستفسارات.\n\n- للمقبلين على الزواج: تصفح المنصة وتقديم الاستفسارات مجاني بالكامل. يجب عليك تقديم معلومات حقيقية ودقيقة عند إرسال طلبات التواصل أو الحجز.\n- للموردين وقاعات الأفراح: الموردون وقاعات الأفراح المسجلون هم جهات مستقلة تمامًا. يتحمل المورد المسؤولية الكاملة عن صحة ودقة الصور، والأسعار، والتفاصيل، والتوافر المعروض لخدماته. يعتبر عقد تقديم الخدمة النهائي اتفاقًا مباشرًا وملزمًا بين المقبلين على الزواج والمورد دون أدنى مسؤولية على المنصة.',
      contentEn: 'Zafaf World operates as an intermediary matchmaking marketplace designed to facilitate service discovery, comparisons, and booking inquiries.\n\n- For Couples: Browsing the platform and submitting inquiries is completely free. You agree to submit authentic, accurate, and current information when creating booking requests.\n- For Vendors & Venues: Registered vendors are independent business entities. Each vendor bears sole responsibility for the accuracy of their listings, images, pricing, availability, and services. The actual contract of service is executed directly between the couple and the vendor, without any platform involvement.'
    },
    {
      id: 'accounts',
      titleAr: '٣. تسجيل الحسابات والأمان',
      titleEn: '3. Account Registration & Security',
      contentAr: 'للاستفادة من ميزات التخطيط المتقدمة وحفظ المفضلات وإرسال طلبات الحجز، يتوجب على المستخدمين والموردين إنشاء حساب. أنت مسؤول مسؤولية كاملة عن الحفاظ على سرية بيانات حسابك وكلمة المرور الخاصة بك. أي نشاط يتم تحت حسابك يعتبر مسؤوليتك. تحتفظ منصة زفاف وورلد بالحق في تعليق أو إغلاق أي حسابات تقدم معلومات غير صحيحة أو تنتهك معايير أمان مجتمعنا.',
      contentEn: 'To utilize advanced planning tools, save favorites, and submit booking inquiries, users and vendors must create an account. You are solely responsible for maintaining the confidentiality of your account credentials and passwords. All activities under your account are your responsibility. Zafaf World reserves the right to suspend or terminate accounts that provide fraudulent information or violate our security guidelines.'
    },
    {
      id: 'commercial',
      titleAr: '٤. الشروط التجارية والمالية',
      titleEn: '4. Commercial & Financial Terms',
      contentAr: 'الأسعار المعروضة على المنصة يتم توفيرها مباشرة من الموردين وتخضع للتحديث المستمر. شروط دفع العربون، والضمان المالي، والإلغاء، واسترداد الأموال تخضع تمامًا للسياسات التجارية الخاصة بكل مورد ومضمون عقده المباشر مع العميل. لا تعتبر منصة زفاف وورلد طرفًا ماليًا في هذه المعاملات ولا تتحمل أي مسؤولية عن أي نزاع يتعلق بالعربون أو جودة تقديم الخدمات أو عدم الالتزام بالاتفاقيات.',
      contentEn: 'Pricing shown on the platform is provided directly by vendors and is subject to change. Deposit requirements, cancellation windows, and refunds are strictly governed by the specific vendor\'s commercial policies and their direct agreement with the client. Zafaf World is not a party to these transactions and bears no liability for deposit disputes, service failures, or contract breaches.'
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
