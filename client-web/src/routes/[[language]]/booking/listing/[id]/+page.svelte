<script lang="ts">
  import { page } from '$app/stores';
  import * as m from '$lib/paraglide/messages.js';
import { getLocale } from '$lib/paraglide/runtime.js';
import { getLocalizedField, formatCurrency, formatNumber, formatDate } from '$lib/utils/localize.js';
  import { listingService } from '$lib/services/api/listing.service.js';
  import { resolveMediaUrl } from '$lib/shared/utils/media.js';
  import Button from '$lib/components/ui/Button.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import type { ListingDetail } from '$lib/types/index.js';
  import { env } from '$env/dynamic/public';
  import { trackBlogFunnelEvent } from '$lib/utils/analytics.js';
  import { countryStore } from '$lib/stores/country.svelte.js';

  let { data } = $props();
  const user = $derived(data?.user);

  // Country-aware tax rates — backend should be the source of truth
  const TAX_RATES: Record<string, number> = {
    SA: 0.15,  // Saudi Arabia VAT 15%
    AE: 0.05,  // UAE VAT 5%
    EG: 0.14,  // Egypt VAT 14%
    TR: 0.20,  // Turkey VAT 20%
  };
  const taxRate = $derived(TAX_RATES[countryStore.activeCode] ?? 0.15);

  // ── State ─────────────────────────────────────────────────────────────────
  let listing     = $derived(data.listing);
  let loading     = $state(false);
  let submitting  = $state(false);
  let step        = $state<1 | 2 | 3>(1);
  let confirmed   = $state(false);
  let bookingResult = $state<{ bookingNumber: string; totalPrice: number; depositPaid: number } | null>(null);

  // Idempotency key — regenerated after a successful booking to allow a new booking
  let idempotencyKey = $state(crypto.randomUUID());

  // Form fields
  let eventDate    = $state('');
  let eventType    = $state('wedding');
  let guestCount   = $state('');
  let specialNotes = $state('');
  let firstName    = $state('');
  let lastName     = $state('');
  let email        = $state('');
  let phone        = $state('');
  let errors       = $state<Record<string, string>>({});

  // Listing is now loaded via SSR

  // ── Derived values ─────────────────────────────────────────────────────────
  const listingName = $derived(
    listing ? listing.title : ''
  );
  const vendorName = $derived(
    listing ? (getLocalizedField(listing.vendor, 'name', getLocale())) : ''
  );
  const totalCapacity = $derived(
    (listing?.attributes?.menCapacity || 0) + (listing?.attributes?.womenCapacity || 0)
  );
  const subtotal = $derived(
    listing
      ? (listing.basePriceSar
          ? parseFloat(listing.basePriceSar)
          : (listing.startingPrice || 25000))
      : 0
  );
  const tax = $derived(Math.round(subtotal * taxRate));
  const total = $derived(subtotal + tax);
  const depositAmt = $derived(Math.round(total * (listing?.depositPercentage || 25) / 100));
  const isCalculatingPrice = false;

  const stepLabels = $derived([
    m.booking_steps_details(),
    m.booking_steps_review(),
    m.booking_steps_payment(),
  ]);

  const eventTypeOptions = $derived<any[]>(data.metadata?.venueTypes || []);

  // ── Validation ────────────────────────────────────────────────────────────
  function validateStep1(): boolean {
    const e: Record<string, string> = {};
    if (!eventDate)  e.eventDate  = m.errors_required();
    if (!guestCount || parseInt(guestCount) <= 0) e.guestCount = m.errors_required();
    errors = e;
    return Object.keys(e).length === 0;
  }

  function validateStep2(): boolean {
    const e: Record<string, string> = {};
    if (!firstName) e.firstName = m.errors_required();
    if (!lastName)  e.lastName  = m.errors_required();
    if (!email || !email.includes('@')) e.email = m.errors_required();
    if (!phone)  e.phone  = m.errors_required();
    errors = e;
    return Object.keys(e).length === 0;
  }

  function nextStep() {
    if (step === 1 && !validateStep1()) return;
    if (step === 2 && !validateStep2()) return;
    step = (step + 1) as 1 | 2 | 3;
  }
  function prevStep() {
    if (step > 1) step = (step - 1) as 1 | 2 | 3;
  }

  // ── Submit booking ─────────────────────────────────────────────────────────
  async function handleConfirm() {
    if (!listing) return;
    if (!user?.session?.access_token) {
      goto(`/auth/login?redirect=/booking/listing/${listing.id}`);
      return;
    }

    submitting = true;
    errors = {};

    try {
      const result = await listingService.createBooking(
        {
          listingId:       listing.id,
          eventDate,
          eventType,
          guestCount:      parseInt(guestCount),
          specialRequests: specialNotes || undefined,
          contactInfo:     { firstName, lastName, email, phone },
        },
        user.session.access_token,
        idempotencyKey,
      );

      bookingResult = result;
      confirmed = true;
      trackBlogFunnelEvent('booking_conversion');
      idempotencyKey = crypto.randomUUID(); // rotate key
    } catch (e: any) {
      errors = { submit: e.message ?? 'Failed to submit booking. Please try again.' };
    } finally {
      submitting = false;
    }
  }
</script>

<svelte:head>
  <title>
    {listing ? `${m.booking_title()} - ${listingName}` : m.booking_title()} | {m.meta_siteName()}
  </title>
</svelte:head>

<!-- ── Page Header ──────────────────────────────────────────────────────────── -->
<div class="bg-[var(--color-surface-alt)] border-b border-[var(--color-border)]">
  <div class="container-page py-8">
    <div class="flex items-center gap-2 text-sm text-[var(--color-muted)] mb-4">
      <a href="/listings" class="hover:text-[var(--color-primary)] transition-colors">
        {m.auto_listings()}
      </a>
      {#if listing}
        <span>/</span>
        <a href={listing.detailUrl} class="hover:text-[var(--color-primary)] transition-colors truncate max-w-[160px]">{listingName}</a>
        <span>/</span>
      {/if}
      <span class="text-[var(--color-secondary)] font-medium">{m.booking_title()}</span>
    </div>

    <h1 class="font-display text-2xl font-bold text-[var(--color-secondary)] mb-6">
      {m.booking_title()}
      {#if listing}
        <span class="text-[var(--color-primary)]"> — {listingName}</span>
      {/if}
    </h1>

    <!-- Step indicator -->
    {#if !confirmed}
      <nav aria-label="Booking steps">
        <ol class="flex items-center gap-0" role="list">
          {#each stepLabels as label, i}
            {@const stepNum = i + 1}
            {@const isActive    = step === stepNum}
            {@const isCompleted = step > stepNum}
            <li class="flex items-center flex-1 last:flex-none">
              <div class="flex items-center gap-2">
                <div
                  class="w-8 h-8 rounded-full flex items-center justify-center text-sm font-bold flex-shrink-0
                    {isCompleted ? 'bg-[var(--color-primary)] text-[var(--color-secondary)]'
                      : isActive ? 'bg-[var(--color-secondary)] text-white'
                      : 'bg-[var(--color-border)] text-[var(--color-muted)]'}"
                  aria-current={isActive ? 'step' : undefined}
                >
                  {#if isCompleted}✓{:else}{stepNum}{/if}
                </div>
                <span class="text-sm font-medium hidden sm:block {isActive ? 'text-[var(--color-secondary)]' : 'text-[var(--color-muted)]'}">{label}</span>
              </div>
              {#if i < stepLabels.length - 1}
                <div class="flex-1 h-px mx-4 {isCompleted ? 'bg-[var(--color-primary)]' : 'bg-[var(--color-border)]'}"></div>
              {/if}
            </li>
          {/each}
        </ol>
      </nav>
    {/if}
  </div>
</div>

<div class="container-page py-10">

  <!-- ═══ Confirmed ═══ -->
  {#if confirmed && bookingResult}
    <div class="max-w-lg mx-auto text-center py-16">
      <div class="w-24 h-24 rounded-full bg-green-100 flex items-center justify-center mx-auto mb-6 shadow-lg">
        <svg viewBox="0 0 24 24" class="w-12 h-12 text-green-600" fill="none" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/>
        </svg>
      </div>
      <h2 class="font-display text-3xl font-bold text-[var(--color-secondary)] mb-3">
        {m.booking_success_title()}
      </h2>
      <p class="text-[var(--color-muted)] text-base mb-6">{m.booking_success_description()}</p>

      <div class="bg-amber-50 border border-amber-200 rounded-2xl p-6 mb-6 text-start">
        <div class="flex justify-between mb-2 text-sm">
          <span class="text-[var(--color-muted)]">{m.booking_success_bookingId()}:</span>
          <span class="font-bold text-[var(--color-primary)] font-mono">{bookingResult.bookingNumber}</span>
        </div>
        <div class="flex justify-between mb-2 text-sm">
          <span class="text-[var(--color-muted)]">{m.auto_total()}:</span>
          <span class="font-bold text-[var(--color-secondary)]">{formatCurrency(bookingResult.totalPrice)}</span>
        </div>
        <div class="flex justify-between text-sm">
          <span class="text-[var(--color-muted)]">{m.auto_deposit()}:</span>
          <span class="font-semibold text-[var(--color-secondary)]">{formatCurrency(bookingResult.depositPaid)}</span>
        </div>
      </div>

      <div class="flex flex-col sm:flex-row gap-3 justify-center">
        <Button href="/" variant="primary">{m.booking_success_home()}</Button>
        <Button href="/listings" variant="outline">{m.auto_browse_listings()}</Button>
      </div>
    </div>

  <!-- ═══ Loading ═══ -->
  {:else if loading}
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-10">
      <div class="lg:col-span-2 animate-pulse flex flex-col gap-4">
        <div class="h-8 bg-gray-200 rounded w-1/3"></div>
        <div class="h-12 bg-gray-100 rounded-xl"></div>
        <div class="h-12 bg-gray-100 rounded-xl"></div>
        <div class="h-12 bg-gray-100 rounded-xl"></div>
      </div>
      <div class="h-64 bg-gray-100 rounded-2xl animate-pulse"></div>
    </div>

  <!-- ═══ Not found ═══ -->
  {:else if !listing}
    <div class="text-center py-24">
      <div class="text-5xl mb-4" aria-hidden="true">🏛️</div>
      <h2 class="font-display text-xl font-bold text-[var(--color-secondary)] mb-2">
        {m.auto_listing_not_found()}
      </h2>
      <p class="text-[var(--color-muted)] mb-6">{m.errors_notFound()}</p>
      <Button href="/listings" variant="primary">{m.auto_browse_listings()}</Button>
    </div>

  <!-- ═══ Main form ═══ -->
  {:else}
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-10">

      <!-- ── Form area ──────────────────────────────────────────────────────── -->
      <div class="lg:col-span-2">
        <div class="bg-white rounded-2xl border border-[var(--color-border)] p-8">

          <!-- Step 1: Event details -->
          {#if step === 1}
            <h2 class="font-display text-xl font-bold text-[var(--color-secondary)] mb-6">
              {m.booking_steps_details()}
            </h2>
            <div class="flex flex-col gap-5">

              <!-- Event date -->
              <div>
                <label for="event-date" class="block text-sm font-medium text-[var(--color-text)] mb-1.5">
                  {m.booking_form_eventDate()}
                  <span class="text-[var(--color-error)] ms-0.5">*</span>
                </label>
                <input
                  id="event-date"
                  type="date"
                  bind:value={eventDate}
                  min={new Date(Date.now() + 86400000).toISOString().split('T')[0]}
                  class="w-full rounded-lg border px-4 py-3 text-sm
                    focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] focus:border-[var(--color-primary)]
                    {errors.eventDate ? 'border-[var(--color-error)]' : 'border-[var(--color-border)]'}"
                />
                {#if errors.eventDate}
                  <p class="text-xs text-[var(--color-error)] mt-1">{errors.eventDate}</p>
                {/if}
              </div>

              <!-- Event type -->
              <div>
                <label for="event-type" class="block text-sm font-medium text-[var(--color-text)] mb-1.5">
                  {m.booking_form_eventType()}
                </label>
                <select id="event-type" bind:value={eventType}
                  class="w-full rounded-lg border border-[var(--color-border)] px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] bg-white">
                  {#each eventTypeOptions as opt}
                    <option value={opt.key || opt.value || opt.id}>
                      {getLocale() === 'ar' ? (opt.labelAr || opt.arLabel || opt.ar || opt.label || opt.name_ar) : (opt.labelEn || opt.enLabel || opt.en || opt.label || opt.name_en)}
                    </option>
                  {/each}
                </select>
              </div>

              <!-- Guest count -->
              <div>
                <label for="guest-count" class="block text-sm font-medium text-[var(--color-text)] mb-1.5">
                  {m.booking_form_guestCount()}
                  <span class="text-[var(--color-error)] ms-0.5">*</span>
                </label>
                <input
                  id="guest-count"
                  type="number"
                  bind:value={guestCount}
                  min="1"
                  placeholder={totalCapacity > 0
                    ? (getLocale() === 'ar' ? `حتى ${totalCapacity} ضيف` : `Up to ${totalCapacity} guests`)
                    : ''}
                  class="w-full rounded-lg border px-4 py-3 text-sm
                    focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] focus:border-[var(--color-primary)]
                    {errors.guestCount ? 'border-[var(--color-error)]' : 'border-[var(--color-border)]'}"
                />
                {#if errors.guestCount}
                  <p class="text-xs text-[var(--color-error)] mt-1">{errors.guestCount}</p>
                {/if}
                {#if totalCapacity > 0}
                  <p class="text-xs text-[var(--color-muted)] mt-1">
                    <svg viewBox="0 0 24 24" class="w-3.5 h-3.5 inline-block me-1 -mt-0.5" fill="none" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"/>
                    </svg>
                    {getLocale() === 'ar'
                      ? `الاستيعاب الأقصى: ${totalCapacity.toLocaleString()} ضيف`
                      : `Max capacity: ${totalCapacity.toLocaleString()} guests`}
                  </p>
                {/if}
              </div>

              <!-- Special requests -->
              <div>
                <label for="special-requests" class="block text-sm font-medium text-[var(--color-text)] mb-1.5">
                  {m.booking_form_specialRequests()}
                </label>
                <textarea id="special-requests" bind:value={specialNotes} rows="3"
                  placeholder={m.auto_any_special_requests()}
                  class="w-full rounded-lg border border-[var(--color-border)] px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] resize-y"
                ></textarea>
              </div>
            </div>

          <!-- Step 2: Contact info -->
          {:else if step === 2}
            <h2 class="font-display text-xl font-bold text-[var(--color-secondary)] mb-6">
              {m.auto_contact_information()}
            </h2>
            <div class="flex flex-col gap-5">
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-5">
                <Input label={m.booking_form_firstName()} bind:value={firstName} required error={errors.firstName} id="first-name" />
                <Input label={m.booking_form_lastName()}  bind:value={lastName}  required error={errors.lastName}  id="last-name" />
              </div>
              <Input type="email" label={m.booking_form_email()} bind:value={email} required error={errors.email} placeholder="you@example.com" id="email" />
              <Input type="tel"   label={m.booking_form_phone()} bind:value={phone} required error={errors.phone}  placeholder="+966 5X XXX XXXX" id="phone" />
            </div>

          <!-- Step 3: Review & confirm -->
          {:else if step === 3}
            <h2 class="font-display text-xl font-bold text-[var(--color-secondary)] mb-6">
              {m.booking_steps_payment()}
            </h2>

            {#if errors.submit}
              <div class="mb-6 p-4 rounded-xl bg-red-50 border border-red-200 text-red-600 text-sm flex items-start gap-3">
                <svg class="w-5 h-5 flex-shrink-0 mt-0.5" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.28 7.22a.75.75 0 00-1.06 1.06L8.94 10l-1.72 1.72a.75.75 0 101.06 1.06L10 11.06l1.72 1.72a.75.75 0 101.06-1.06L11.06 10l1.72-1.72a.75.75 0 00-1.06-1.06L10 8.94 8.28 7.22z" clip-rule="evenodd"/>
                </svg>
                <span>{errors.submit}</span>
              </div>
            {/if}

            <!-- Booking summary -->
            <div class="bg-[var(--color-surface-alt)] rounded-xl p-5 mb-6 space-y-3 text-sm">
              <div class="flex justify-between">
                <span class="text-[var(--color-muted)]">{m.auto_listing()}</span>
                <span class="font-medium text-end max-w-[60%]">{listingName}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-[var(--color-muted)]">{m.auto_venue()}</span>
                <span class="font-medium">{vendorName}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-[var(--color-muted)]">{m.booking_summary_date()}</span>
                <span class="font-medium">{eventDate}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-[var(--color-muted)]">{m.booking_summary_guests()}</span>
                <span class="font-medium">{guestCount}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-[var(--color-muted)]">{m.auto_name()}</span>
                <span class="font-medium">{firstName} {lastName}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-[var(--color-muted)]">{m.auto_email()}</span>
                <span class="font-medium">{email}</span>
              </div>
            </div>

            <!-- Payment placeholder -->
            <div class="rounded-xl border-2 border-dashed border-[var(--color-border)] p-8 text-center text-[var(--color-muted)]">
              <div class="text-4xl mb-3" aria-hidden="true">💳</div>
              <p class="font-medium">{m.auto_payment_gateway()}</p>
              <p class="text-sm mt-1">{m.auto_payment_gateway_inte()}</p>
            </div>
          {/if}

          <!-- Navigation buttons -->
          <div class="flex items-center justify-between mt-8 pt-6 border-t border-[var(--color-border)]">
            {#if step > 1}
              <Button onclick={prevStep} variant="ghost" size="md">
                <svg viewBox="0 0 20 20" class="w-4 h-4 ltr:rotate-180" fill="currentColor">
                  <path fill-rule="evenodd" d="M12.79 5.23a.75.75 0 01-.02 1.06L8.832 10l3.938 3.71a.75.75 0 11-1.04 1.08l-4.5-4.25a.75.75 0 010-1.08l4.5-4.25a.75.75 0 011.06.02z" clip-rule="evenodd"/>
                </svg>
                {m.booking_form_back()}
              </Button>
            {:else}
              <div></div>
            {/if}

            {#if step < 3}
              <Button onclick={nextStep} variant="primary" size="md">
                {m.booking_form_next()}
                <svg viewBox="0 0 20 20" class="w-4 h-4 rtl:rotate-180" fill="currentColor">
                  <path fill-rule="evenodd" d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z" clip-rule="evenodd"/>
                </svg>
              </Button>
            {:else}
              <Button onclick={handleConfirm} variant="primary" size="lg" loading={submitting}>
                {m.booking_form_submit()}
              </Button>
            {/if}
          </div>
        </div>
      </div>

      <!-- ── Booking summary sidebar ──────────────────────────────────────── -->
      <aside>
        <div class="sticky top-24 bg-white rounded-2xl border border-[var(--color-border)] shadow-[var(--shadow-md)] overflow-hidden">
          <!-- Listing cover image -->
          <div class="h-40 overflow-hidden">
            <img src={resolveMediaUrl(listing.coverImage)} alt={listingName} class="w-full h-full object-cover" loading="lazy" />
          </div>

          <div class="p-6">
            <h3 class="font-display text-lg font-bold text-[var(--color-secondary)] mb-0.5">{listingName}</h3>
            <p class="text-sm text-[var(--color-primary)] mb-1">{vendorName}</p>
            {#if listing.cityEn || listing.cityAr}
              <p class="text-xs text-[var(--color-muted)] mb-4 flex items-center gap-1">
                📍 {getLocalizedField(listing, 'city', getLocale())}
              </p>
            {/if}

            <!-- Price breakdown -->
            {#if subtotal > 0}
              <div class="flex flex-col gap-2.5 text-sm {isCalculatingPrice ? 'opacity-50' : ''}">
                <div class="flex justify-between">
                  <span class="text-[var(--color-muted)]">{m.booking_summary_subtotal()}</span>
                  <span>{formatCurrency(subtotal)}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-[var(--color-muted)]">{m.booking_summary_taxes()} (15%)</span>
                  <span>{formatCurrency(tax)}</span>
                </div>
                <div class="flex justify-between pt-2.5 border-t border-[var(--color-border)] font-bold text-base">
                  <span>{m.booking_summary_total()}</span>
                  <span class="text-[var(--color-primary)]">{formatCurrency(total)}</span>
                </div>
              </div>
              <p class="text-xs text-[var(--color-muted)] mt-3 text-center bg-[var(--color-surface-alt)] rounded-lg p-2 {isCalculatingPrice ? 'opacity-50' : ''}">
                {listing.depositPercentage}% {m.auto_deposit_1()} =
                <strong>{formatCurrency(depositAmt)}</strong>
              </p>
            {:else}
              <p class="text-sm text-[var(--color-muted)] italic text-center py-3">
                {m.auto_enter_date_and_guest()}
              </p>
            {/if}

            <!-- Security note -->
            <div class="mt-5 flex items-center gap-2 text-xs text-[var(--color-muted)]">
              <svg viewBox="0 0 20 20" class="w-4 h-4 flex-shrink-0 text-green-500" fill="currentColor">
                <path fill-rule="evenodd" d="M10 1a4.5 4.5 0 00-4.5 4.5V9H5a2 2 0 00-2 2v6a2 2 0 002 2h10a2 2 0 002-2v-6a2 2 0 00-2-2h-.5V5.5A4.5 4.5 0 0010 1zm3 8V5.5a3 3 0 10-6 0V9h6z" clip-rule="evenodd"/>
              </svg>
              <span>{m.auto_secure__encrypted_b()}</span>
            </div>
          </div>
        </div>
      </aside>
    </div>
  {/if}
</div>
