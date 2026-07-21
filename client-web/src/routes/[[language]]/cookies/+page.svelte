<script lang="ts">
  import * as m from '$lib/paraglide/messages.js';
  import { getLocale } from '$lib/paraglide/runtime.js';
  import { getLocalizedField } from '$lib/utils/localize.js';

  const sections = $derived([
    {
      id: 'what-are-cookies',
      titleAr: '١. ما هي ملفات تعريف الارتباط؟',
      titleEn: '1. What Are Cookies?',
      contentAr: 'ملفات تعريف الارتباط هي ملفات نصية صغيرة يتم تخزينها على جهاز الكمبيوتر أو الهاتف المحمول الخاص بك عند زيارة موقعنا الإلكتروني. تساعد هذه الملفات المنصة في تذكر تفضيلاتك وتخصيص تجربتك وتحسين أداء الموقع بشكل عام.',
      contentEn: 'Cookies are small text files stored on your computer or mobile device when you visit our website. These files help the platform remember your preferences, customize your experience, and improve overall website performance.'
    },
    {
      id: 'how-we-use',
      titleAr: '٢. كيف نستخدم ملفات تعريف الارتباط؟',
      titleEn: '2. How We Use Cookies',
      contentAr: 'تستخدم منصة زفاف وورلد ملفات تعريف الارتباط للأغراض التالية:\n\n- ملفات تعريف الارتباط الأساسية: لتشغيل المنصة، بما في ذلك تذكر تسجيل الدخول وحفظ تفضيلات اللغة الخاصة بك (العربية/الإنجليزية).\n- ملفات الأداء والتحليلات: لمساعدتنا في فهم كيفية تفاعل الزوار مع المنصة ومعدل زيارة قاعات الزفاف، مما يمكننا من تحسين تجربة المستخدم باستمرار.\n- ملفات التفضيلات: لتذكر خياراتك السابقة، مثل مدينة البحث المفضلة لديك وسجل استعلاماتك لتسهيل زيارتك القادمة.',
      contentEn: 'Zafaf World uses cookies for the following purposes:\n\n- Essential Cookies: Necessary for platform operations, including remembering session states, logins, and language preferences (Arabic/English).\n- Performance & Analytics Cookies: Helps us understand how visitors interact with the marketplace, track which venue categories are popular, and continually optimize usability.\n- Preference Cookies: Used to save your previous settings, such as your preferred city and search history, making your next visit seamless.'
    },
    {
      id: 'control-cookies',
      titleAr: '٣. التحكم في ملفات تعريف الارتباط',
      titleEn: '3. Controlling Cookies',
      contentAr: 'يمكنك التحكم في ملفات تعريف الارتباط أو تعطيلها من خلال إعدادات متصفح الإنترنت الخاص بك. يرجى العلم بأن تعطيل بعض ملفات تعريف الارتباط قد يؤدي إلى عدم إمكانية استخدام بعض الميزات التفاعلية للمنصة بشكل صحيح.',
      contentEn: 'You have the right to control and disable cookies via your internet browser settings. Please note that disabling essential cookies may prevent some interactive platform features from functioning correctly.'
    }
  ]);
</script>

<svelte:head>
  <title>{getLocale() === 'ar' ? 'سياسة ملفات تعريف الارتباط' : 'Cookie Policy'} - {m.meta_siteName()}</title>
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
        {getLocale() === 'ar' ? 'سياسة ملفات تعريف الارتباط' : 'Cookie Policy'}
      </h1>
      <p class="text-sm sm:text-base text-white/70 max-w-2xl leading-relaxed">
        {getLocale() === 'ar' 
          ? 'نحن نستخدم ملفات تعريف الارتباط لضمان حصولك على أفضل تجربة على منصتنا.' 
          : 'We use cookies to ensure that you get the best experience on our platform.'}
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
