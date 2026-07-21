<script lang="ts">
  import * as m from '$lib/paraglide/messages.js';
  import { getLocale } from '$lib/paraglide/runtime.js';
  import { getLocalizedField } from '$lib/utils/localize.js';

  const sections = $derived([
    {
      id: 'collection',
      titleAr: '١. جمع البيانات الشخصية',
      titleEn: '1. Collection of Personal Data',
      contentAr: 'نقوم بجمع البيانات الشخصية اللازمة لتمكين وتسهيل خدمات تخطيط حفلات الزفاف وتوصيلك بمقدمي الخدمات وقاعات الأفراح. تشمل هذه البيانات: الاسم، عنوان البريد الإلكتروني، رقم الهاتف، الموقع الجغرافي، تاريخ الزفاف المتوقع، الميزانية التقديرية، عدد الضيوف، وأي تفاصيل ترسلها في طلب الاستفسار. بالنسبة للموردين، نقوم أيضًا بجمع تفاصيل التسجيل التجاري، والأسعار، وبيانات الفواتير والتحصيل.',
      contentEn: 'We collect personal data necessary to facilitate wedding planning services and connect you with venues and vendors. This includes your name, email address, phone number, location, expected wedding date, budget preferences, guest count, and any details sent via inquiry forms. For vendors, we also collect business registration details, pricing, and bank account details for billing.'
    },
    {
      id: 'processing',
      titleAr: '٢. معالجة البيانات واستخدامها',
      titleEn: '2. Data Processing & Usage',
      contentAr: 'تتم معالجة بياناتك للأغراض التالية: مطابقتك مع قاعات الزفاف والموردين المناسبين لتفضيلاتك، إرسال طلبات الحجز والاستفسارات، إدارة حسابك الشخصي، توفير تنبيهات المتابعة عبر الرسائل النصية القصيرة، وتحسين كفاءة استخدام المنصة. قد نستخدم بريدك الإلكتروني أو رقم هاتفك لإرسال إشعارات المعاملات، والنشرات التسويقية الدورية (والتي يمكنك إلغاء الاشتراك فيها في أي وقت).',
      contentEn: 'Your data is processed for the following purposes: matching you with appropriate wedding venues and service vendors, transmitting booking inquiries, managing your user account, providing follow-up SMS alerts, and improving platform usability. We may use your email or phone number to send transactional notifications, updates, and marketing newsletters (which you can opt-out of at any time).'
    },
    {
      id: 'cookies',
      titleAr: '٣. ملفات تعريف الارتباط والتخزين',
      titleEn: '3. Cookies & Local Storage',
      contentAr: 'تستخدم منصة زفاف وورلد ملفات تعريف الارتباط وتخزين البيانات المحلي لتحسين تجربة التصفح وحفظ تفضيلاتك. نستخدم هذه التقنيات لحفظ لغتك المفضلة (العربية أو الإنجليزية)، تتبع الجلسة النشطة، والاحتفاظ ببيانات تسجيل الدخول المؤقتة لتسريع الاستخدام، وتحليل حركة الزوار الإحصائية. يمكنك التحكم بملفات تعريف الارتباط أو تعطيلها من إعدادات متصفحك في أي وقت.',
      contentEn: 'Zafaf World uses cookies and local storage configurations to enhance your browsing experience and save preferences. These technologies help us remember your language choice (Arabic/English), manage active sessions, preserve temporary authentication tokens, and analyze anonymized site traffic statistics. You can configure your browser to reject cookies, though some interactive platform features may become limited.'
    },
    {
      id: 'sharing',
      titleAr: '٤. مشاركة البيانات والإفصاح عنها',
      titleEn: '4. Data Sharing & Disclosures',
      contentAr: 'نحن لا نبيع ولا نؤجر بياناتك الشخصية لأي جهات تسويقية خارجية. عندما تقوم بإرسال استفسار أو طلب حجز لقاعة أو خدمة زفاف، فإننا نقوم بمشاركة بيانات الاتصال وتفاصيل الحجز المدخلة مع ذلك المورد المحدد فقط ليتمكن من معالجة طلبك والرد عليك. قد نقوم بالإفصاح عن البيانات لمزودي الخدمات المساعدين (كخدمات الاستضافة والرسائل النصية)، أو في الحالات التي يتطلبها القانون والأنظمة المعتمدة بالمملكة العربية السعودية.',
      contentEn: 'We do not sell or lease your personal data to external marketing agencies. When you submit a booking or inquiry request for a venue or service, we share your input contact details and wedding requirements with that specific vendor to allow them to process and respond to your request. We may disclose data to auxiliary service providers (e.g. cloud hosting, SMS gateways) or where required to comply with Saudi laws and security regulations.'
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
