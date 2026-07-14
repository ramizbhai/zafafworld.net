<script lang="ts">
  import * as m from '$lib/paraglide/messages.js';
  import { getLocale } from '$lib/paraglide/runtime.js';
  import { toasts } from '$lib/stores/toast.svelte.js';
  import { trackBlogFunnelEvent } from '$lib/utils/analytics.js';

  let { class: className = '' }: { class?: string } = $props();

  let expectedWeddingDate = $state('');
  let conciergeName = $state('');
  let conciergeMobile = $state('');
  let conciergeWhatsapp = $state(true);
  let isSubmitting = $state(false);
  let isSuccess = $state(false);
  let conciergeError = $state('');

  const isAr = $derived(getLocale() === 'ar');
  const PHONE_REGEX = /^\+[1-9]\d{6,14}$/;

  const nextMonths = $derived(
    Array.from({ length: 12 }, (_, i) => {
      const d = new Date();
      d.setMonth(d.getMonth() + i + 1);
      d.setDate(1);
      const value = d.toISOString().split('T')[0];
      const label = new Intl.DateTimeFormat(getLocale() === 'ar' ? 'ar-SA' : 'en-US', {
        year: 'numeric', month: 'long'
      }).format(d);
      return { value, label };
    })
  );

  async function startConcierge(e: SubmitEvent) {
    e.preventDefault();
    if (!expectedWeddingDate) {
      toasts.push('error', isAr ? 'يرجى اختيار تاريخ الزفاف المتوقع' : 'Please select your expected wedding date');
      return;
    }

    conciergeError = '';

    // Validate name
    if (!conciergeName.trim() || conciergeName.trim().length < 3) {
      conciergeError = isAr ? 'الاسم مطلوب (3 أحرف على الأقل)' : 'Name is required (min 3 characters)';
      return;
    }
    // Validate mobile
    if (!conciergeMobile.trim()) {
      conciergeError = isAr ? 'رقم الهاتف مطلوب' : 'Phone number is required';
      return;
    }
    if (!PHONE_REGEX.test(conciergeMobile.trim())) {
      conciergeError = isAr ? 'صيغة غير صحيحة. مثال: 966512345678+' : 'Invalid format. Example: +966512345678';
      return;
    }

    isSubmitting = true;
    try {
      const res = await fetch('/bff/v1/public/afrah', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          name: conciergeName.trim(),
          phone: conciergeMobile.trim(),
          isWhatsapp: conciergeWhatsapp,
          eventDate: expectedWeddingDate,
          message: isAr ? 'طلب تخطيط زفاف عبر أفراح' : 'Wedding planning request via Afrah concierge',
        }),
      });

      const body = await res.json();
      if (res.ok && body.status === 'success') {
        isSuccess = true;
        await trackBlogFunnelEvent('afrah_start');
        toasts.push('success', isAr ? 'تم إنشاء جلسة التخطيط بنجاح مع أفراح' : 'Afrah has created your personal planning session');
      } else if (res.status === 429) {
        conciergeError = isAr ? 'عدد كبير من الطلبات. يرجى المحاولة لاحقاً.' : 'Too many requests. Please try again later.';
        toasts.push('error', conciergeError);
      } else {
        conciergeError = body.message || (isAr ? 'حدث خطأ. يرجى المحاولة مرة أخرى.' : 'An error occurred. Please try again.');
        toasts.push('error', conciergeError);
      }
    } catch (err) {
      console.error(err);
      conciergeError = isAr ? 'فشل الاتصال بالخادم.' : 'Connection to the server failed.';
      toasts.push('error', conciergeError);
    } finally {
      if (!isSuccess) {
        isSubmitting = false;
      }
    }
  }
</script>

<div class="bg-[#FAF6F0]/95 border border-[#EAE0D0] rounded-[2.5rem] p-8 md:p-10 shadow-[0_12px_40px_rgba(45,38,32,0.06)] flex flex-col md:flex-row items-center gap-6 md:gap-8 max-w-5xl mx-auto w-full relative transition-all duration-300 {className}">
  <!-- Left: Avatar -->
  <div class="relative shrink-0 flex items-center justify-center">
    <div class="w-24 h-24 sm:w-28 sm:h-28 md:w-32 md:h-32 rounded-full p-1 bg-gradient-to-tr from-[#EE7E97] via-[#C9A96E] to-[#5EBEB2] flex items-center justify-center shadow-md">
      <img
        src="/afrah_avatar.webp"
        alt="Afrah Avatar"
        class="w-full h-full object-cover rounded-full bg-white"
      />
    </div>
  </div>

  <!-- Right: Content -->
  <div class="flex-1 flex flex-col gap-3 text-center md:text-start min-w-0">
    <span class="text-[10px] tracking-wider text-[#008080] font-black uppercase">
      {m.auto_afrah_is_here_to_pla()}
    </span>
    <h3 class="font-display text-2xl font-bold text-[#2D2620] leading-tight">
      {m.auto_meet_your_wedding_pl()}
    </h3>
    <p class="text-xs md:text-sm text-[#9E8E7A] font-medium leading-relaxed max-w-2xl">
      {m.auto_tell_us_about_your_d()}
    </p>

    <form onsubmit={startConcierge} class="flex flex-col gap-3 w-full max-w-xl mt-2">
      <!-- Row 1: Date + Name -->
      <div class="flex flex-col sm:flex-row gap-3 items-stretch sm:items-end w-full">
        <!-- Date Select dropdown -->
        <div class="relative flex-1 text-start">
          <label for="afrah-expected-date" class="text-[9px] text-[#9E8E7A] font-bold uppercase tracking-wider mb-1 block">
            {m.auto_expected_wedding_dat()}
          </label>
          <div class="relative">
            <select
              id="afrah-expected-date"
              bind:value={expectedWeddingDate}
              class="w-full h-11 bg-white border border-[#EAE0D0] hover:border-[#C9A96E] rounded-full pl-5 pr-10 rtl:pl-10 rtl:pr-5 text-xs font-extrabold text-[#2D2620] focus:outline-none appearance-none cursor-pointer shadow-sm transition-all"
            >
              <option value="">{m.auto_select_a_date()}</option>
              {#each nextMonths as mItem}
                <option value={mItem.value}>{mItem.label}</option>
              {/each}
            </select>
            <div class="absolute inset-y-0 right-4 rtl:left-4 rtl:right-auto flex items-center pointer-events-none text-[#9E8E7A]">
              <svg viewBox="0 0 20 20" class="w-4 h-4" fill="currentColor">
                <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clip-rule="evenodd" />
              </svg>
            </div>
          </div>
        </div>

        <!-- Name field -->
        <div class="flex-1 text-start">
          <label for="afrah-name" class="text-[9px] text-[#9E8E7A] font-bold uppercase tracking-wider mb-1 block">
            {isAr ? 'الاسم' : 'Your Name'}
          </label>
          <input
            id="afrah-name"
            type="text"
            bind:value={conciergeName}
            placeholder={isAr ? 'مثال: أحمد محمد' : 'e.g. Ahmed Mohammed'}
            class="w-full h-11 bg-white border border-[#EAE0D0] hover:border-[#C9A96E] rounded-full px-5 text-xs font-extrabold text-[#2D2620] focus:outline-none focus:ring-2 focus:ring-[#C9A96E] shadow-sm"
            required
          />
        </div>
      </div>

      <!-- Row 2: Mobile + Submit -->
      <div class="flex flex-col sm:flex-row gap-3 items-stretch sm:items-end w-full">
        <!-- Mobile field -->
        <div class="flex-1 text-start">
          <label for="afrah-mobile" class="text-[9px] text-[#9E8E7A] font-bold uppercase tracking-wider mb-1 block">
            {isAr ? 'رقم الهاتف' : 'Phone'}
          </label>
          <input
            id="afrah-mobile"
            type="tel"
            bind:value={conciergeMobile}
            placeholder="+966512345678"
            class="w-full h-11 bg-white border border-[#EAE0D0] hover:border-[#C9A96E] rounded-full px-5 text-xs font-extrabold text-[#2D2620] focus:outline-none focus:ring-2 focus:ring-[#C9A96E] shadow-sm ltr"
            dir="ltr"
            required
          />
          <label class="flex items-center gap-1.5 mt-1.5 ps-2 cursor-pointer select-none">
            <input type="checkbox" bind:checked={conciergeWhatsapp} class="w-3.5 h-3.5 rounded accent-green-600" />
            <span class="text-[9px] text-[#9E8E7A] font-medium">{isAr ? 'واتساب' : 'WhatsApp'}</span>
          </label>
        </div>

        <!-- Action Button -->
        <button
          type="submit"
          disabled={isSubmitting || isSuccess}
          class="px-8 h-11 rounded-full font-bold text-xs transition-all duration-300 shadow-md hover:shadow-lg cursor-pointer disabled:cursor-not-allowed whitespace-nowrap flex items-center justify-center shrink-0
            {isSuccess ? 'bg-green-500 text-white' : 'bg-[#5EBEB2] hover:bg-[#4EA89D] disabled:bg-gray-300 text-white'}"
        >
          {#if isSubmitting}
            <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
          {/if}
          {isSuccess ? (isAr ? '✅ تم الإرسال!' : '✅ Sent!') : m.auto_start_quickly()}
        </button>
      </div>

      <!-- Error message -->
      {#if conciergeError}
        <p class="text-[10px] text-red-500 font-medium text-start">{conciergeError}</p>
      {/if}
    </form>

    <!-- Specific date click here link -->
    <a href="/afrah" class="text-[10px] text-[#9E8E7A] hover:text-[#2D2620] mt-1 font-medium transition-colors duration-200 self-start md:ms-2">
      {m.auto_if_you_have_a_specif()}
    </a>
  </div>
</div>
