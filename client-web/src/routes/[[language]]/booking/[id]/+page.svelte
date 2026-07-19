<script lang="ts">
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { getLocale } from "$lib/paraglide/runtime.js";
  import {
    getLocalizedField,
    formatCurrency,
    formatNumber,
    formatDate,
  } from "$lib/utils/localize.js";
  import { env } from "$env/dynamic/public";
  import { safeFetch } from "$lib/utils/api";
  import { vendorService } from "$lib/services/api/vendor.service.js";
  import Button from "$lib/components/ui/Button.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import Skeleton from "$lib/components/ui/Skeleton.svelte";
  import type { Venue, VenueCategory } from "$lib/types/index.js";
  import * as m from "$lib/paraglide/messages.js";
  import { countryStore } from '$lib/stores/country.svelte.js';

  let { data } = $props();

  // ── Idempotency ──────────────────────────────────────────────────────────
  // Minted once per page-session. The backend idempotent_gate_middleware uses
  // this key to deduplicate accidental retries / double-taps on the same
  // booking form. A new key is issued only after a successful confirmation so
  // the user can safely book again from this page without stale lock state.
  let idempotencyKey = $state(crypto.randomUUID());

  // Form fields
  let eventDate = $state("");
  let venue = $derived<Venue | null>(data?.venue || null);
  let loading = $state(false);
  let submitting = $state(false);
  let step = $state<1 | 2 | 3>(1);
  let confirmed = $state(false);
  let bookingId = $state("");
  let eventType = $state<VenueCategory>("wedding");
  let guestCount = $state("");
  let specialNotes = $state("");
  let firstName = $state("");
  let lastName = $state("");
  let email = $state("");
  let phone = $state("");
  let errors = $state<Record<string, string>>({});

  // Country-aware tax rates — backend should be the source of truth
  const TAX_RATES: Record<string, number> = {
    SA: 0.15,  // Saudi Arabia VAT 15%
    AE: 0.05,  // UAE VAT 5%
    EG: 0.14,  // Egypt VAT 14%
    TR: 0.20,  // Turkey VAT 20%
  };
  const taxRate = $derived(TAX_RATES[countryStore.activeCode] ?? 0.15);

  const subtotal = $derived(venue?.pricing.basePrice ?? 0);
  const tax = $derived(Math.round(subtotal * taxRate));
  const total = $derived(subtotal + tax);

  const venueName = $derived(
    venue ? getLocalizedField(venue, "name", getLocale()) : "",
  );

  const stepLabels = $derived([
    m.booking_steps_details(),
    m.booking_steps_review(),
    m.booking_steps_payment(),
  ]);

  const eventTypeOptions = $derived<any[]>(
    $page.data.metadata?.venueTypes || [],
  );

  // ── Validation ────────────────────────────────────────────────────────────
  function validateStep1(): boolean {
    const e: Record<string, string> = {};
    if (!eventDate) e.eventDate = m.errors_required();
    if (!guestCount) e.guestCount = m.errors_required();
    errors = e;
    return Object.keys(e).length === 0;
  }

  function validateStep2(): boolean {
    const e: Record<string, string> = {};
    if (!firstName) e.firstName = m.errors_required();
    if (!lastName) e.lastName = m.errors_required();
    if (!email) e.email = m.errors_required();
    if (!phone) e.phone = m.errors_required();
    errors = e;
    return Object.keys(e).length === 0;
  }

  // ── Navigation ────────────────────────────────────────────────────────────
  function nextStep() {
    if (step === 1 && !validateStep1()) return;
    if (step === 2 && !validateStep2()) return;
    step = (step + 1) as 1 | 2 | 3;
  }

  function prevStep() {
    if (step > 1) step = (step - 1) as 1 | 2 | 3;
  }

  async function handleConfirm() {
    submitting = true;
    errors = {};

    const API_BASE = env.PUBLIC_API_URL || "http://localhost:8080";

    try {
      const response = await safeFetch<{ bookingNumber: string }>(
        fetch,
        `${API_BASE}/api/v1/public/bookings`,
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            // Single-use deduplication token — exercised by the backend
            // idempotent_gate_middleware (DashMap<String, IdempotentState>).
            // Ensures that a network retry or accidental double-submit within
            // the same booking session is blocked server-side with 409.
            "Idempotency-Key": idempotencyKey,
          },
          body: JSON.stringify({
            venue_id: venue?.id,
            event_date: eventDate,
            event_type: eventType,
            guest_count: parseInt(guestCount) || 0,
            special_requests: specialNotes || null,
            first_name: firstName,
            last_name: lastName,
            email: email,
            phone: phone,
          }),
        },
      );

      if (response.success && response.data) {
        bookingId = response.data.bookingNumber;
        confirmed = true;
        // Rotate the key so a subsequent booking from this page gets a fresh
        // deduplication token rather than re-using the consumed one.
        idempotencyKey = crypto.randomUUID();
      } else {
        const errorMsg =
          response.error?.message ||
          "Failed to submit booking inquiry. Please try again.";
        errors = {
          submit: errorMsg,
          eventDate:
            response.status === 400 || response.status === 409 ? errorMsg : "",
        };
      }
    } catch (err: any) {
      errors = {
        submit:
          "A connection failure occurred. Please verify your internet connection and try again.",
      };
    } finally {
      submitting = false;
    }
  }
</script>

<svelte:head>
  <title>{m.booking_title()} - {m.meta_siteName()}</title>
</svelte:head>

<!-- Page header -->
<div
  class="bg-[var(--color-surface-alt)] border-b border-[var(--color-border)]"
>
  <div class="container-page py-8">
    <h1
      class="font-display text-2xl font-bold text-[var(--color-secondary)] mb-6"
    >
      {m.booking_title()}
    </h1>

    <!-- Step indicator -->
    {#if !confirmed}
      <nav aria-label="Booking steps">
        <ol class="flex items-center gap-0" role="list">
          {#each stepLabels as label, i}
            {@const stepNum = i + 1}
            {@const isActive = step === stepNum}
            {@const isCompleted = step > stepNum}
            <li class="flex items-center flex-1 last:flex-none">
              <div class="flex items-center gap-2">
                <div
                  class="w-8 h-8 rounded-full flex items-center justify-center text-sm font-bold flex-shrink-0
                  {isCompleted
                    ? 'bg-[var(--color-primary)] text-[var(--color-secondary)]'
                    : isActive
                      ? 'bg-[var(--color-secondary)] text-white'
                      : 'bg-[var(--color-border)] text-[var(--color-muted)]'}"
                  aria-current={isActive ? "step" : undefined}
                >
                  {#if isCompleted}✓{:else}{stepNum}{/if}
                </div>
                <span
                  class="text-sm font-medium hidden sm:block
                  {isActive
                    ? 'text-[var(--color-secondary)]'
                    : 'text-[var(--color-muted)]'}">{label}</span
                >
              </div>
              {#if i < stepLabels.length - 1}
                <div
                  class="flex-1 h-px mx-4 {isCompleted ? 'bg-[var(--color-primary)]'
                    : 'bg-[var(--color-border)]'}"
                ></div>
              {/if}
            </li>
          {/each}
        </ol>
      </nav>
    {/if}
  </div>
</div>

<div class="container-page py-10">
  {#if confirmed}
    <!-- ── Success screen ── -->
    <div class="max-w-lg mx-auto text-center py-16">
      <div
        class="w-20 h-20 rounded-full bg-green-100 flex items-center justify-center mx-auto mb-6"
      >
        <svg
          viewBox="0 0 24 24"
          class="w-10 h-10 text-green-600"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
          />
        </svg>
      </div>
      <h2
        class="font-display text-3xl font-bold text-[var(--color-secondary)] mb-3"
      >
        {m.booking_success_title()}
      </h2>
      <p class="text-[var(--color-muted)] text-base mb-4">
        {m.booking_success_description()}
      </p>
      <div
        class="inline-flex items-center gap-2 bg-[var(--color-surface-alt)] border border-[var(--color-border)] rounded-xl px-6 py-3 mb-8"
      >
        <span class="text-sm text-[var(--color-muted)]"
          >{m.booking_success_bookingId()}:</span
        >
        <span class="font-bold text-[var(--color-primary)] font-mono text-lg"
          >{bookingId}</span
        >
      </div>
      <div class="flex flex-col sm:flex-row gap-3 justify-center">
        <Button href="/" variant="primary">{m.booking_success_home()}</Button>
        <Button href="/venues" variant="outline">{m.nav_venues()}</Button>
      </div>
    </div>
  {:else if loading}
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-10">
      <div class="lg:col-span-2 flex flex-col gap-4">
        <Skeleton height="h-10" width="w-1/3" />
        <Skeleton height="h-12" rounded />
        <Skeleton height="h-12" rounded />
        <Skeleton height="h-12" rounded />
        <Skeleton height="h-12" rounded />
      </div>
      <Skeleton height="h-64" rounded />
    </div>
  {:else if !venue}
    <div class="text-center py-24">
      <p class="text-[var(--color-muted)]">{m.errors_notFound()}</p>
      <Button href="/venues" variant="primary" class="mt-4"
        >{m.nav_venues()}</Button
      >
    </div>
  {:else}
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-10">
      <!-- ── Form area ── -->
      <div class="lg:col-span-2">
        <div
          class="bg-white rounded-2xl border border-[var(--color-border)] p-8"
        >
          {#if step === 1}
            <!-- Step 1: Event details -->
            <h2
              class="font-display text-xl font-bold text-[var(--color-secondary)] mb-6"
            >
              {m.booking_steps_details()}
            </h2>
            <div class="flex flex-col gap-5">
              <!-- Event date -->
              <div>
                <label
                  for="event-date"
                  class="block text-sm font-medium text-[var(--color-text)] mb-1.5"
                >
                  {m.booking_form_eventDate()}
                  <span class="text-[var(--color-error)] ms-0.5">*</span>
                </label>
                <input
                  id="event-date"
                  type="date"
                  bind:value={eventDate}
                  min={new Date().toISOString().split("T")[0]}
                  class="w-full rounded-lg border px-4 py-3 text-sm
                    focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] focus:border-[var(--color-primary)]
                    {errors.eventDate
                    ? 'border-[var(--color-error)]'
                    : 'border-[var(--color-border)]'}"
                />
                {#if errors.eventDate}
                  <p class="text-xs text-[var(--color-error)] mt-1">
                    {errors.eventDate}
                  </p>
                {/if}
              </div>

              <!-- Event type -->
              <div>
                <label
                  for="event-type"
                  class="block text-sm font-medium text-[var(--color-text)] mb-1.5"
                >
                  {m.booking_form_eventType()}
                </label>
                <select
                  id="event-type"
                  bind:value={eventType}
                  class="w-full rounded-lg border border-[var(--color-border)] px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] bg-white"
                >
                  {#each eventTypeOptions as opt}
                    <option value={opt.key || opt.value || opt.id}>
                      {getLocale() === "ar"
                        ? opt.labelAr ||
                          opt.arLabel ||
                          opt.ar ||
                          opt.label ||
                          opt.name_ar
                        : opt.labelEn ||
                          opt.enLabel ||
                          opt.en ||
                          opt.label ||
                          opt.name_en}
                    </option>
                  {/each}
                </select>
              </div>

              <!-- Guest count -->
              <Input
                type="number"
                label={m.booking_form_guestCount()}
                bind:value={guestCount}
                required
                error={errors.guestCount}
                placeholder={`${venue.capacity.min} – ${venue.capacity.max}`}
                hint={getLocale() === "ar"
                  ? `السعة: ${venue.capacity.min} – ${venue.capacity.max} ضيف`
                  : `Capacity: ${venue.capacity.min} – ${venue.capacity.max} guests`}
              />

              <!-- Special requests -->
              <div>
                <label
                  for="special-requests"
                  class="block text-sm font-medium text-[var(--color-text)] mb-1.5"
                >
                  {m.booking_form_specialRequests()}
                </label>
                <textarea
                  id="special-requests"
                  bind:value={specialNotes}
                  rows="3"
                  placeholder={m.auto_any_special_requests()}
                  class="w-full rounded-lg border border-[var(--color-border)] px-4 py-3 text-sm focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] resize-y"
                ></textarea>
              </div>
            </div>
          {:else if step === 2}
            <!-- Step 2: Contact info -->
            <h2
              class="font-display text-xl font-bold text-[var(--color-secondary)] mb-6"
            >
              {m.auto_contact_information()}
            </h2>
            <div class="flex flex-col gap-5">
              <div class="grid grid-cols-1 sm:grid-cols-2 gap-5">
                <Input
                  label={m.booking_form_firstName()}
                  bind:value={firstName}
                  required
                  error={errors.firstName}
                />
                <Input
                  label={m.booking_form_lastName()}
                  bind:value={lastName}
                  required
                  error={errors.lastName}
                />
              </div>
              <Input
                type="email"
                label={m.booking_form_email()}
                bind:value={email}
                required
                error={errors.email}
                placeholder="you@example.com"
              />
              <Input
                type="tel"
                label={m.booking_form_phone()}
                bind:value={phone}
                required
                error={errors.phone}
                placeholder="+966 5X XXX XXXX"
              />
            </div>
          {:else if step === 3}
            <!-- Step 3: Review & payment -->
            <h2
              class="font-display text-xl font-bold text-[var(--color-secondary)] mb-6"
            >
              {m.booking_steps_payment()}
            </h2>

            {#if errors.submit}
              <div
                class="mb-6 p-4 rounded-xl bg-red-50 border border-red-200 text-red-600 text-sm flex items-start gap-3"
              >
                <svg
                  class="w-5 h-5 flex-shrink-0 mt-0.5"
                  viewBox="0 0 20 20"
                  fill="currentColor"
                >
                  <path
                    fill-rule="evenodd"
                    d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.28 7.22a.75.75 0 00-1.06 1.06L8.94 10l-1.72 1.72a.75.75 0 101.06 1.06L10 11.06l1.72 1.72a.75.75 0 101.06-1.06L11.06 10l1.72-1.72a.75.75 0 00-1.06-1.06L10 8.94 8.28 7.22z"
                    clip-rule="evenodd"
                  />
                </svg>
                <span>{errors.submit}</span>
              </div>
            {/if}

            <!-- Review summary -->
            <div
              class="bg-[var(--color-surface-alt)] rounded-xl p-5 mb-6 text-sm space-y-2"
            >
              <div class="flex justify-between">
                <span class="text-[var(--color-muted)]"
                  >{m.booking_summary_venue()}</span
                >
                <span class="font-medium">{venueName}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-[var(--color-muted)]"
                  >{m.booking_summary_date()}</span
                >
                <span class="font-medium">{eventDate}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-[var(--color-muted)]"
                  >{m.booking_summary_guests()}</span
                >
                <span class="font-medium">{guestCount}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-[var(--color-muted)]">{m.auto_name()}</span>
                <span class="font-medium">{firstName} {lastName}</span>
              </div>
            </div>

            <!-- Payment placeholder -->
            <div
              class="rounded-xl border-2 border-dashed border-[var(--color-border)] p-8 text-center text-[var(--color-muted)]"
            >
              <div class="text-4xl mb-3" aria-hidden="true">💳</div>
              <p class="font-medium">{m.auto_payment_gateway()}</p>
              <p class="text-sm mt-1">{m.auto_payment_gateway_inte()}</p>
            </div>
          {/if}

          <!-- Navigation buttons -->
          <div
            class="flex items-center justify-between mt-8 pt-6 border-t border-[var(--color-border)]"
          >
            {#if step > 1}
              <Button onclick={prevStep} variant="ghost" size="md">
                <svg
                  viewBox="0 0 20 20"
                  class="w-4 h-4 ltr:rotate-180"
                  fill="currentColor"
                >
                  <path
                    fill-rule="evenodd"
                    d="M12.79 5.23a.75.75 0 01-.02 1.06L8.832 10l3.938 3.71a.75.75 0 11-1.04 1.08l-4.5-4.25a.75.75 0 010-1.08l4.5-4.25a.75.75 0 011.06.02z"
                    clip-rule="evenodd"
                  />
                </svg>
                {m.booking_form_back()}
              </Button>
            {:else}
              <div></div>
            {/if}

            {#if step < 3}
              <Button onclick={nextStep} variant="primary" size="md">
                {m.booking_form_next()}
                <svg
                  viewBox="0 0 20 20"
                  class="w-4 h-4 rtl:rotate-180"
                  fill="currentColor"
                >
                  <path
                    fill-rule="evenodd"
                    d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z"
                    clip-rule="evenodd"
                  />
                </svg>
              </Button>
            {:else}
              <Button
                onclick={handleConfirm}
                variant="primary"
                size="lg"
                loading={submitting}
              >
                {m.booking_form_submit()}
              </Button>
            {/if}
          </div>
        </div>
      </div>

      <!-- ── Booking summary sidebar ── -->
      <aside>
        <div
          class="sticky top-24 bg-white rounded-2xl border border-[var(--color-border)] shadow-[var(--shadow-md)] overflow-hidden"
        >
          <!-- Venue image -->
          {#if venue.images[0]}
            <div class="h-40 overflow-hidden">
              <img
                src={venue.images[0].url}
                alt={venueName}
                class="w-full h-full object-cover"
                loading="lazy"
              />
            </div>
          {/if}

          <div class="p-6">
            <h3
              class="font-display text-lg font-bold text-[var(--color-secondary)] mb-1"
            >
              {venueName}
            </h3>
            <p class="text-sm text-[var(--color-muted)] mb-5">
              {venue.location.city}، {venue.location.district}
            </p>

            <div class="flex flex-col gap-3 text-sm">
              <div class="flex justify-between">
                <span class="text-[var(--color-muted)]"
                  >{m.booking_summary_subtotal()}</span
                >
                <span>{formatCurrency(subtotal)}</span>
              </div>
              <div class="flex justify-between">
                <span class="text-[var(--color-muted)]"
                  >{m.booking_summary_taxes()}</span
                >
                <span>{formatCurrency(tax)}</span>
              </div>
              <div
                class="flex justify-between pt-3 border-t border-[var(--color-border)] font-bold text-base"
              >
                <span>{m.booking_summary_total()}</span>
                <span class="text-[var(--color-primary)]"
                  >{formatCurrency(total)}</span
                >
              </div>
            </div>

            <p class="text-xs text-[var(--color-muted)] mt-4 text-center">
              {getLocale() === "ar"
                ? `الدفعة المقدمة ${venue.pricing.depositPercentage}% = ${formatCurrency(Math.round((total * venue.pricing.depositPercentage) / 100))}`
                : `${venue.pricing.depositPercentage}% deposit = ${formatCurrency(Math.round((total * venue.pricing.depositPercentage) / 100))}`}
            </p>
          </div>
        </div>
      </aside>
    </div>
  {/if}
</div>
