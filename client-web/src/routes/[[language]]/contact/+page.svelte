<script lang="ts">
  import * as m from '$lib/paraglide/messages.js';
  import { getLocale } from '$lib/paraglide/runtime.js';
  import { getLocalizedField } from '$lib/utils/localize.js';
  import Button from '$lib/components/ui/Button.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import { enhance } from '$app/forms';

  let { form }: { form?: { success?: boolean; error?: string; values?: any } } = $props();

  let name = $state(form?.values?.name || '');
  let email = $state(form?.values?.email || '');
  let phone = $state(form?.values?.phone || '');
  let subject = $state(form?.values?.subject || '');
  let message = $state(form?.values?.message || '');

  let sending = $state(false);

  const contactInfo = $derived([
    { icon: '📞', label: m.contact_info_phone(),   value: '+966 59 211 2517' },
    { icon: '✉️', label: m.contact_info_email(),   value: 'contact@zafafworld.net' },
    { icon: '📍', label: m.contact_info_address(), value: m.auto_riyadh_saudi_arabia() },
    { icon: '🕐', label: m.contact_info_hours(),   value: m.contact_info_hoursValue() },
  ]);
</script>

<svelte:head>
  <title>{m.contact_title()} - {m.meta_siteName()}</title>
</svelte:head>

<div class="bg-[var(--color-surface-alt)] border-b border-[var(--color-border)]">
  <div class="container-page py-12">
    <span class="divider-gold"></span>
    <h1 class="font-display text-3xl sm:text-4xl font-bold text-[var(--color-secondary)] mt-4 mb-2">
      {m.contact_title()}
    </h1>
    <p class="text-[var(--color-muted)]">{m.contact_subtitle()}</p>
  </div>
</div>

<div class="container-page py-16">
  <div class="grid grid-cols-1 lg:grid-cols-3 gap-12">

    <!-- Contact info -->
    <aside>
      <div class="flex flex-col gap-6">
        {#each contactInfo as info}
          <div class="flex items-start gap-4 p-5 rounded-2xl bg-white border border-[var(--color-border)]">
            <span class="text-2xl mt-0.5" aria-hidden="true">{info.icon}</span>
            <div>
              <p class="text-xs font-semibold uppercase tracking-wide text-[var(--color-muted)] mb-1">{info.label}</p>
              <p class="text-sm text-[var(--color-text)]">{info.value}</p>
            </div>
          </div>
        {/each}
      </div>
    </aside>

    <!-- Contact form -->
    <div class="lg:col-span-2">
      {#if form?.success}
        <div class="flex flex-col items-center justify-center py-16 text-center bg-white rounded-2xl border border-[var(--color-border)]">
          <div class="text-5xl mb-4" aria-hidden="true">✅</div>
          <h2 class="font-display text-2xl font-bold text-[var(--color-secondary)] mb-3">
            {m.contact_form_success()}
          </h2>
          <p class="text-[var(--color-muted)] mb-6">
            {getLocale() === 'ar' ? 'سنتواصل معك قريباً' : "We'll get back to you soon"}
          </p>
          <a href="/contact" class="px-6 py-3 rounded-xl border border-[var(--color-border)] text-sm font-semibold hover:bg-[var(--color-surface-alt)] transition-colors">
            {m.auto_send_another_message()}
          </a>
        </div>
      {:else}
        <form method="POST" use:enhance={() => {
          sending = true;
          return async ({ update }) => {
            sending = false;
            await update();
          };
        }} class="bg-white rounded-2xl border border-[var(--color-border)] p-8 flex flex-col gap-5">
          {#if form?.error}
            <div class="bg-red-50 text-red-600 border border-red-200 rounded-xl p-4 text-sm" role="alert">
              {form.error}
            </div>
          {/if}

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-5">
            <Input
              name="name"
              label={m.contact_form_name()}
              bind:value={name}
              required
              placeholder={m.auto_john_smith()}
            />
            <Input
              name="email"
              type="email"
              label={m.contact_form_email()}
              bind:value={email}
              required
              placeholder="hello@example.com"
            />
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-5">
            <Input
              name="phone"
              type="tel"
              label={m.contact_form_phone()}
              bind:value={phone}
              placeholder="+966 5X XXX XXXX"
            />
            <Input
              name="subject"
              label={m.contact_form_subject()}
              bind:value={subject}
              required
              placeholder={m.auto_message_subject()}
            />
          </div>

          <div class="flex flex-col gap-1.5">
            <label for="contact-message" class="text-sm font-medium text-[var(--color-text)]">
              {m.contact_form_message()}
              <span class="text-[var(--color-error)] ms-0.5" aria-hidden="true">*</span>
            </label>
            <textarea
              id="contact-message"
              name="message"
              bind:value={message}
              required
              rows="5"
              placeholder={m.auto_write_your_message_h()}
              class="w-full rounded-lg border border-[var(--color-border)] px-4 py-3
                text-[var(--color-text)] placeholder:text-[var(--color-muted)]
                focus:outline-none focus:ring-2 focus:ring-[var(--color-primary)] focus:border-[var(--color-primary)]
                resize-y text-sm"
            ></textarea>
          </div>

          <Button type="submit" variant="primary" size="lg" loading={sending} fullWidth>
            {m.contact_form_submit()}
          </Button>
        </form>
      {/if}
    </div>
  </div>
</div>
