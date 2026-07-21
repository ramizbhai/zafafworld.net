<script lang="ts">
  import * as m from '$lib/paraglide/messages.js';
  import { getLocale } from '$lib/paraglide/runtime.js';
  import { getLocalizedField } from '$lib/utils/localize.js';

  interface FAQItem {
    id: string | number;
    qAr: string;
    qEn: string;
    aAr: string;
    aEn: string;
  }

  let { data }: { data: any } = $props();

  // Highly professional business-aligned fallback FAQs
  const fallbackFaqs: FAQItem[] = [
    {
      id: 'faq-1',
      qEn: 'What is Zafaf World?',
      qAr: 'ما هو زفاف وورلد؟',
      aEn: 'Zafaf World is the premier wedding venue and planning marketplace in Saudi Arabia. We connect engaged couples with the finest wedding palaces, hotel ballrooms, and event vendors, enabling them to browse, compare, and submit booking requests easily.',
      aAr: 'زفاف وورلد هو المنصة الأولى لحجز قاعات ومزودي خدمات الأفراح في المملكة العربية السعودية. نحن نربط المقبلين على الزواج بأفضل قصور الأفراح، قاعات الفنادق، ومنظمي الحفلات لتسهيل البحث والمقارنة وتقديم طلبات الحجز.'
    },
    {
      id: 'faq-2',
      qEn: 'Is using Zafaf World free for couples?',
      qAr: 'هل استخدام زفاف وورلد مجاني للمقبلين على الزواج؟',
      aEn: 'Yes! Zafaf World is completely free for couples. You can search, filter, and send booking inquiries to any venue or vendor without any platform fees.',
      aAr: 'نعم! استخدام منصة زفاف وورلد مجاني بالكامل للمقبلين على الزواج. يمكنك البحث والمقارنة وإرسال طلبات الاستفسار والحجز للقاعات والموردين مجاناً وبدون أي رسوم منصة.'
    },
    {
      id: 'faq-3',
      qEn: 'How can I register my venue or service on the platform?',
      qAr: 'كيف يمكنني تسجيل قاعتي أو خدمتي في المنصة؟',
      aEn: 'If you own a wedding hall, hotel ballroom, or event service, you can register by clicking "Own a Venue?" on our homepage or visiting our Vendor Portal at https://vendor.zafafworld.net/ to create your business account and start receiving bookings.',
      aAr: 'إذا كنت تمتلك قاعة أفراح، قاعة فندق، أو تقدم خدمات زفاف، يمكنك تسجيل قاعتك بالضغط على زر "هل تمتلك قاعة؟" في الصفحة الرئيسية أو زيارة بوابة الشركاء عبر الرابط https://vendor.zafafworld.net/ لإنشاء حسابك وبدء استقبال طلبات الحجز.'
    },
    {
      id: 'faq-4',
      qEn: 'How do I confirm my booking and pay the deposit?',
      qAr: 'كيف يمكنني تأكيد حجزي ودفع العربون؟',
      aEn: 'Once you submit a booking inquiry, the venue manager will contact you directly to confirm availability, event details, and discuss pricing. Deposits and payments are processed directly with the venue according to their cancellation and refund policies.',
      aAr: 'بمجرد إرسال طلب الحجز، سيتواصل معك مدير القاعة مباشرة لتأكيد التوافر وتنسيق الأسعار وتفاصيل الحفل. يتم دفع العربون وإتمام الدفعات مباشرة مع القاعة ووفقاً لسياسة الإلغاء والاسترجاع الخاصة بهم.'
    },
    {
      id: 'faq-5',
      qEn: 'How can I contact Zafaf World support?',
      qAr: 'كيف يمكنني التواصل مع الدعم الفني لزفاف وورلد؟',
      aEn: 'You can reach our dedicated customer care team by emailing contact@zafafworld.net or calling/WhatsApping us at +966592112517.',
      aAr: 'يمكنك التواصل مع فريق خدمة العملاء والدعم الفني عبر البريد الإلكتروني contact@zafafworld.net أو الاتصال/الواتساب على الرقم 966592112517+.'
    }
  ];

  // Combine database FAQs with our professional fallbacks
  const faqs = $derived(
    data.faqs && data.faqs.length > 0 
      ? data.faqs.map((f: any) => ({
          id: f.id,
          qEn: f.q?.en || getLocalizedField(f.q, 'name', 'en'),
          qAr: f.q?.ar || getLocalizedField(f.q, 'name', 'ar'),
          aEn: f.a?.en || getLocalizedField(f.a, 'name', 'en'),
          aAr: f.a?.ar || getLocalizedField(f.a, 'name', 'ar'),
        }))
      : fallbackFaqs
  );

  let activeId = $state<string | number | null>(null);

  function toggle(id: string | number) {
    activeId = activeId === id ? null : id;
  }
</script>

<svelte:head>
  <title>{m.footer_faq()} - {m.meta_siteName()}</title>
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
        {m.auto_frequently_asked_que()}
      </h1>
      <p class="text-sm sm:text-base text-white/70 max-w-2xl leading-relaxed">
        {m.auto_find_quick_precise_()}
      </p>
    </div>
  </div>

  <!-- FAQ List -->
  <div class="container-page py-12 sm:py-16 max-w-3xl">
    <div class="flex flex-col gap-4">
      {#each faqs as item}
        {@const isOpen = activeId === item.id}
        <div class="bg-white rounded-2xl border border-[var(--color-border)] overflow-hidden shadow-sm hover:shadow-md transition-shadow duration-300">
          <button
            type="button"
            class="w-full text-start px-6 py-5 flex items-center justify-between gap-4 font-semibold text-[var(--color-secondary)] hover:text-[var(--color-primary)] transition-colors focus:outline-none cursor-pointer"
            onclick={() => toggle(item.id)}
            aria-expanded={isOpen}
          >
            <span class="text-sm sm:text-base leading-snug">
              {getLocale() === 'ar' ? item.qAr : item.qEn}
            </span>
            <svg
              viewBox="0 0 20 20"
              class="w-5 h-5 flex-shrink-0 transform transition-transform duration-200 text-[var(--color-muted)] {isOpen ? 'rotate-180 text-[var(--color-primary)]' : ''}"
              fill="currentColor"
              aria-hidden="true"
            >
              <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clip-rule="evenodd"/>
            </svg>
          </button>
          
          {#if isOpen}
            <div class="px-6 pb-6 border-t border-[var(--color-surface-alt)] pt-4">
              <p class="text-sm sm:text-base text-[var(--color-muted)] leading-relaxed">
                {getLocale() === 'ar' ? item.aAr : item.aEn}
              </p>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </div>
</div>
