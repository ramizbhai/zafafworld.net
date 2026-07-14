<script lang="ts">
  /**
   * InquiryForm.svelte — Enterprise-grade Zero-Auth Public Inquiry Form
   *
   * Features:
   * - No auth requirement — works for all visitors
   * - Per-instance state isolation via $state() runes
   * - Client-side validation (invalid data never hits server)
   * - Honeypot field for bot spam prevention
   * - Success state with tracking ID display
   * - Fully bilingual (AR/EN) via Paraglide
   * - Accessible: ARIA labels, error announcements, focus management
   */

  import * as m from '$lib/paraglide/messages.js';
  import { getLocale } from '$lib/paraglide/runtime.js';
  import Input from '$lib/components/ui/Input.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import { toasts } from '$lib/stores/toast.svelte.js';
  import { trackBlogFunnelEvent } from '$lib/utils/analytics.js';

  // ── Props ────────────────────────────────────────────────────────────────────
  interface Props {
    listingId?: string;
    listingName?: string;
    vendorName?: string;
    variant?: 'sidebar' | 'modal' | 'inline';
    inquiryType?: 'listing' | 'concierge';
    onSuccess?: (trackingId: string) => void;
    onCancel?: () => void;
  }

  let {
    listingId = '',
    listingName = '',
    vendorName = '',
    variant = 'sidebar',
    inquiryType = 'listing',
    onSuccess,
    onCancel,
  }: Props = $props();

  // ── Per-instance Form State ──────────────────────────────────────────────────
  let name = $state('');
  let countryCode = $state('+966');
  let mobileNumber = $state('');
  let email = $state('');
  let message = $state('');
  let isWhatsapp = $state(false);
  let eventDate = $state('');
  let honeypot = $state('');  // Hidden field — bots fill this

  let errors = $state<Record<string, string>>({});
  let formState = $state<'idle' | 'validating' | 'submitting' | 'success' | 'error'>('idle');
  let trackingId = $state('');
  let serverError = $state('');

  const isAr = $derived(getLocale() === 'ar');

  // ── Icons (Gold/Primary UI Match) ────────────────────────────────────────────
  const UserIcon = `<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-[var(--color-primary)]"><path d="M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>`;
  const MailIcon = `<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-[var(--color-primary)]"><rect width="20" height="16" x="2" y="4" rx="2"/><path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7"/></svg>`;
  const CalendarIcon = `<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-[var(--color-primary)]"><rect width="18" height="18" x="3" y="4" rx="2" ry="2"/><line x1="16" x2="16" y1="2" y2="6"/><line x1="8" x2="8" y1="2" y2="6"/><line x1="3" x2="21" y1="10" y2="10"/><path d="M8 14h.01"/><path d="M12 14h.01"/><path d="M16 14h.01"/><path d="M8 18h.01"/><path d="M12 18h.01"/><path d="M16 18h.01"/></svg>`;

  // ── Validation ───────────────────────────────────────────────────────────────
  const PHONE_REGEX = /^[0-9]{7,12}$/;
  const EMAIL_REGEX = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

  function validate(): boolean {
    const e: Record<string, string> = {};

    if (!name.trim() || name.trim().length < 3) {
      e.name = isAr ? 'الاسم مطلوب (3 أحرف على الأقل)' : 'Name is required (min 3 characters)';
    }

    if (!mobileNumber.trim()) {
      e.mobile = isAr ? 'رقم الهاتف مطلوب' : 'Phone number is required';
    } else if (!PHONE_REGEX.test(mobileNumber.trim())) {
      e.mobile = isAr ? 'صيغة غير صحيحة. يجب أن يحتوي على 7-12 رقماً' : 'Invalid format. Must be 7-12 digits';
    }

    if (email.trim() && !EMAIL_REGEX.test(email.trim())) {
      e.email = isAr ? 'صيغة البريد الإلكتروني غير صحيحة' : 'Invalid email format';
    }

    if (message.trim().length > 500) {
      e.message = isAr ? 'الرسالة يجب ألا تتجاوز 500 حرف' : 'Message must not exceed 500 characters';
    }

    if (eventDate) {
      const d = new Date(eventDate);
      if (isNaN(d.getTime()) || d <= new Date()) {
        e.eventDate = isAr ? 'يجب أن يكون التاريخ في المستقبل' : 'Date must be in the future';
      }
    }

    errors = e;
    return Object.keys(e).length === 0;
  }

  // ── Submit Handler ───────────────────────────────────────────────────────────
  async function handleSubmit(event: SubmitEvent) {
    event.preventDefault();

    if (formState === 'submitting' || formState === 'validating') {
      return;
    }

    formState = 'validating';
    if (!validate()) {
      formState = 'idle';
      return;
    }

    formState = 'submitting';
    serverError = '';

    try {
      const fullMobile = `${countryCode.trim()}${mobileNumber.trim()}`;
      
      const payload: Record<string, any> = {
        name: name.trim(),
        mobile: fullMobile,
        is_whatsapp: isWhatsapp,
        type: inquiryType,
        // Honeypot field — bots will fill this, BFF will silently reject
        website: honeypot,
      };

      if (listingId) payload.listing_id = listingId;
      if (email.trim()) payload.email = email.trim();
      if (message.trim()) payload.message = message.trim();
      if (eventDate) payload.event_date = eventDate;

      // Submit to same-origin BFF (handles rate limiting + forwarding)
      const res = await fetch('/bff/v1/public/inquiries', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });

      const data = await res.json();

      if (res.ok && data.status === 'success') {
        formState = 'success';
        trackingId = data.tracking_id || '';
        trackBlogFunnelEvent('inquiry_start');
        toasts.push('success', isAr ? 'تم إرسال استفسارك بنجاح!' : 'Your inquiry has been submitted successfully!');
        onSuccess?.(trackingId);
      } else if (res.status === 429) {
        formState = 'error';
        serverError = isAr ? 'عدد كبير من الطلبات. يرجى المحاولة لاحقاً.' : 'Too many submissions. Please try again later.';
      } else if (res.status === 400 && data.errors) {
        // Server-side validation errors — merge into field errors
        const serverErrors: Record<string, string> = {};
        for (const [key, val] of Object.entries(data.errors)) {
          serverErrors[key] = String(val);
        }
        errors = { ...errors, ...serverErrors };
        if (serverErrors.listing_id || serverErrors.vendor_id || serverErrors.type) {
           formState = 'error';
           serverError = isAr ? 'بيانات غير صالحة. يرجى المحاولة مرة أخرى.' : 'Invalid data. Please try again.';
        } else {
           formState = 'idle';
        }
      } else {
        formState = 'error';
        serverError = data.message || (isAr ? 'حدث خطأ. يرجى المحاولة مرة أخرى.' : 'An error occurred. Please try again.');
      }
    } catch (err) {
      console.error('[InquiryForm] Submission error:', err);
      formState = 'error';
      serverError = isAr ? 'فشل الاتصال بالخادم.' : 'Connection to the server failed.';
    }
  }

  function resetForm() {
    formState = 'idle';
    name = '';
    countryCode = '+966';
    mobileNumber = '';
    email = '';
    message = '';
    isWhatsapp = false;
    eventDate = '';
    errors = {};
    serverError = '';
    trackingId = '';
  }
</script>

<!-- ═══ SUCCESS STATE ═══ -->
{#if formState === 'success'}
  <div class="flex flex-col items-center text-center gap-4 py-6 {variant === 'modal' ? 'px-6' : 'p-6'}">
    <!-- Animated checkmark -->
    <div class="w-16 h-16 rounded-full bg-green-100 flex items-center justify-center animate-[scale-in_0.3s_ease-out]">
      <svg class="w-8 h-8 text-green-600" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="20 6 9 17 4 12"></polyline>
      </svg>
    </div>

    <div>
      <h3 class="font-display text-lg font-bold text-[var(--color-secondary)] mb-1">
        {isAr ? 'تم إرسال استفسارك بنجاح!' : 'Inquiry Sent Successfully!'}
      </h3>
      {#if vendorName}
        <p class="text-sm text-[var(--color-muted)]">
          {isAr ? `سيتواصل معك فريق ${vendorName} قريباً` : `${vendorName} will contact you shortly`}
        </p>
      {:else}
        <p class="text-sm text-[var(--color-muted)]">
          {isAr ? 'سنتواصل معك قريباً' : 'We will contact you shortly'}
        </p>
      {/if}
    </div>

    {#if trackingId}
      <div class="bg-[var(--color-surface-alt)] border border-[var(--color-border)] rounded-xl px-4 py-3 w-full">
        <p class="text-xs text-[var(--color-muted)] mb-1">{isAr ? 'رقم المتابعة' : 'Tracking ID'}</p>
        <p class="font-mono text-sm font-bold text-[var(--color-secondary)]">{trackingId}</p>
      </div>
    {/if}

    <button
      onclick={resetForm}
      class="text-sm text-[var(--color-primary)] hover:text-[var(--color-primary-dark)] font-medium underline underline-offset-2 transition-colors mt-2"
    >
      {isAr ? 'إرسال استفسار آخر' : 'Send another inquiry'}
    </button>
  </div>

<!-- ═══ FORM STATE ═══ -->
{:else}
  <form
    onsubmit={handleSubmit}
    class="flex flex-col gap-4 {variant === 'modal' ? 'px-6 pb-6' : variant === 'sidebar' ? 'p-6' : 'p-0'}"
    novalidate
  >
    <!-- Header -->
    {#if variant !== 'inline'}
      <div class="flex items-center justify-between">
        <div>
          <h3 class="font-display text-base font-bold text-[var(--color-secondary)]">
            {inquiryType === 'concierge'
              ? (isAr ? 'استشارة مجانية' : 'Free Consultation')
              : (isAr ? 'استفسار سريع' : 'Quick Inquiry')}
          </h3>
          <p class="text-xs text-[var(--color-muted)] mt-0.5">
            {isAr ? 'لا تحتاج تسجيل دخول' : 'No account needed'}
          </p>
        </div>
        {#if variant === 'modal' && onCancel}
          <button
            type="button"
            onclick={onCancel}
            class="text-[var(--color-muted)] hover:text-[var(--color-secondary)] transition-colors p-1"
            aria-label={isAr ? 'إغلاق' : 'Close'}
          >
            <svg viewBox="0 0 24 24" class="w-5 h-5" fill="none" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12"/>
            </svg>
          </button>
        {/if}
      </div>
    {/if}

    <!-- Honeypot — hidden from real users, bots auto-fill -->
    <div class="absolute -left-[9999px] opacity-0 h-0 w-0 overflow-hidden" aria-hidden="true">
      <label for="inquiry-website">Website</label>
      <input id="inquiry-website" type="text" name="website" bind:value={honeypot} tabindex="-1" autocomplete="off" />
    </div>

    <!-- Name -->
    <Input
      id="inquiry-name"
      label={isAr ? 'الاسم الكامل' : 'Full Name'}
      placeholder={isAr ? 'مثال: أحمد محمد' : 'e.g. Ahmed Mohammed'}
      bind:value={name}
      error={errors.name}
      required
      leadingIcon={UserIcon}
    />

    <!-- Mobile -->
    <div class="flex flex-col gap-1.5">
      <label for="inquiry-mobile" class="text-sm font-medium text-[var(--color-text)]">
        {isAr ? 'رقم الهاتف' : 'Phone Number'}
        <span class="text-[var(--color-error)] ms-0.5" aria-hidden="true">*</span>
      </label>
      <div class="flex gap-2 relative">
        <div class="w-1/3 min-w-[80px]">
          <input
            type="text"
            bind:value={countryCode}
            class="w-full rounded-lg border bg-white px-3 py-3 text-center text-[var(--color-text)] placeholder:text-[var(--color-muted)] transition-colors duration-150 focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] focus:border-[var(--color-primary)] border-[var(--color-border)] hover:border-[var(--color-primary-light)]"
            placeholder="+966"
            dir="ltr"
          />
        </div>
        <div class="relative w-2/3 flex-1">
          <input
            id="inquiry-mobile"
            type="tel"
            bind:value={mobileNumber}
            class="w-full rounded-lg border bg-white px-4 py-3 text-[var(--color-text)] placeholder:text-[var(--color-muted)] transition-colors duration-150 focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] focus:border-[var(--color-primary)] {errors.mobile ? 'border-[var(--color-error)] focus:ring-[var(--color-error)]' : 'border-[var(--color-border)] hover:border-[var(--color-primary-light)]'}"
            placeholder="512345678"
            dir="ltr"
          />
        </div>
      </div>
      {#if errors.mobile}
        <p class="text-sm text-[var(--color-error)]" role="alert">{errors.mobile}</p>
      {/if}

      <!-- WhatsApp toggle -->
      <label class="flex flex-row items-center justify-between cursor-pointer select-none px-3 py-2.5 mt-1 bg-slate-50 hover:bg-slate-100 rounded-lg border border-slate-200 transition-colors">
        <span class="text-sm font-medium text-slate-700 flex items-center gap-2">
          <svg viewBox="0 0 24 24" class="w-4 h-4 text-green-600" fill="currentColor"><path d="M17.472 14.382c-.297-.149-1.758-.867-2.03-.967-.273-.099-.471-.148-.67.15-.197.297-.767.966-.94 1.164-.173.199-.347.223-.644.075-.297-.15-1.255-.463-2.39-1.475-.883-.788-1.48-1.761-1.653-2.059-.173-.297-.018-.458.13-.606.134-.133.298-.347.446-.52.149-.174.198-.298.298-.497.099-.198.05-.371-.025-.52-.075-.149-.669-1.612-.916-2.207-.242-.579-.487-.5-.669-.51-.173-.008-.371-.01-.57-.01-.198 0-.52.074-.792.372-.272.297-1.04 1.016-1.04 2.479 0 1.462 1.065 2.875 1.213 3.074.149.198 2.096 3.2 5.077 4.487.709.306 1.262.489 1.694.625.712.227 1.36.195 1.871.118.571-.085 1.758-.719 2.006-1.413.248-.694.248-1.289.173-1.413-.074-.124-.272-.198-.57-.347z"/><path d="M12 2C6.477 2 2 6.477 2 12c0 1.89.525 3.66 1.438 5.168L2 22l4.832-1.438A9.955 9.955 0 0012 22c5.523 0 10-4.477 10-10S17.523 2 12 2zm0 18a8 8 0 01-4.108-1.132l-.288-.172-2.98.78.796-2.907-.189-.3A7.96 7.96 0 014 12c0-4.411 3.589-8 8-8s8 3.589 8 8-3.589 8-8 8z"/></svg>
          {isAr ? 'نعم، هذا الرقم عليه واتساب' : 'Yes, this number has WhatsApp'}
        </span>
        <input
          type="checkbox"
          bind:checked={isWhatsapp}
          class="w-4 h-4 rounded border-slate-300 text-green-600 focus:ring-green-500 accent-green-600"
        />
      </label>
    </div>

    <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
      <!-- Email (optional) -->
      <Input
        id="inquiry-email"
        label={isAr ? 'البريد الإلكتروني (اختياري)' : 'Email (optional)'}
        type="email"
        placeholder="your@email.com"
        bind:value={email}
        error={errors.email}
        leadingIcon={MailIcon}
      />

      <!-- Event Date (optional) -->
      <Input
        id="inquiry-event-date"
        label={isAr ? 'تاريخ المناسبة (اختياري)' : 'Event Date (optional)'}
        type="date"
        bind:value={eventDate}
        error={errors.eventDate}
        leadingIcon={CalendarIcon}
      />
    </div>

    <!-- Message (optional) -->
    <div class="flex flex-col gap-1.5">
      <label for="inquiry-message" class="text-sm font-medium text-[var(--color-text)]">
        {isAr ? 'رسالة (اختياري)' : 'Message (optional)'}
      </label>
      <textarea
        id="inquiry-message"
        bind:value={message}
        placeholder={isAr ? 'أي تفاصيل إضافية عن مناسبتك...' : 'Any additional details about your event...'}
        rows="2"
        maxlength="500"
        class="w-full rounded-lg border bg-white px-4 py-3 text-[var(--color-text)] placeholder:text-[var(--color-muted)] transition-colors duration-150 focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] focus:border-[var(--color-primary)] resize-none
          {errors.message
            ? 'border-[var(--color-error)] focus:ring-[var(--color-error)]'
            : 'border-[var(--color-border)] hover:border-[var(--color-primary-light)]'}"
      ></textarea>
      <div class="flex justify-between items-center">
        {#if errors.message}
          <p class="text-sm text-[var(--color-error)]" role="alert">{errors.message}</p>
        {:else}
          <span></span>
        {/if}
        <span class="text-xs text-[var(--color-muted)]">{message.length}/500</span>
      </div>
    </div>


    <!-- Server Error -->
    {#if serverError}
      <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-xl text-sm" role="alert">
        {serverError}
      </div>
    {/if}

    <!-- Submit Button -->
    <button
      type="submit"
      disabled={formState === 'submitting' || formState === 'validating'}
      class="w-full flex items-center justify-center gap-2 py-3.5 px-6 rounded-xl font-bold text-base transition-all duration-200
        bg-[var(--color-primary)] text-[var(--color-secondary)]
        hover:bg-[var(--color-primary-dark)]
        shadow-[0_4px_14px_rgba(217,119,6,0.3)]
        hover:shadow-[0_6px_20px_rgba(217,119,6,0.4)]
        hover:-translate-y-0.5
        disabled:opacity-60 disabled:cursor-not-allowed disabled:hover:translate-y-0 disabled:hover:shadow-[0_4px_14px_rgba(217,119,6,0.3)]"
    >
      {#if formState === 'submitting'}
        <svg class="animate-spin w-5 h-5" viewBox="0 0 24 24" fill="none">
          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25"></circle>
          <path d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" fill="currentColor" class="opacity-75"></path>
        </svg>
        {isAr ? 'جاري الإرسال...' : 'Submitting...'}
      {:else}
        📩 {isAr ? 'إرسال الاستفسار' : 'Send Inquiry'}
      {/if}
    </button>

    <!-- Privacy note -->
    <p class="text-[10px] text-[var(--color-muted)] text-center leading-relaxed">
      {isAr
        ? 'بيانات الاتصال الخاصة بك ستُشارك فقط مع المزود المحدد.'
        : 'Your contact details will only be shared with the selected vendor.'}
    </p>
  </form>
{/if}

<style>
  @keyframes scale-in {
    from { transform: scale(0.5); opacity: 0; }
    to { transform: scale(1); opacity: 1; }
  }
</style>
