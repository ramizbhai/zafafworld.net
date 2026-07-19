<script lang="ts">
  import * as m from '$lib/paraglide/messages.js';
import { getLocale } from '$lib/paraglide/runtime.js';
import { getLocalizedField } from '$lib/utils/localize.js';
  import Button from '$lib/components/ui/Button.svelte';
  import Input from '$lib/components/ui/Input.svelte';

  let name    = $state('');
  let email   = $state('');
  let phone   = $state('');
  let subject = $state('');
  let message = $state('');
  let sent    = $state(false);
  let sending = $state(false);

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    sending = true;
    // Simulate API call
    await new Promise((r) => setTimeout(r, 1500));
    sent    = true;
    sending = false;
  }

  const contactInfo = $derived([
    { icon: '📞', label: m.contact_info_phone(),   value: '+966 11 000 0000' },
    { icon: '✉️', label: m.contact_info_email(),   value: 'hello@zafafworld.net' },
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
      {#if sent}
        <div class="flex flex-col items-center justify-center py-16 text-center bg-white rounded-2xl border border-[var(--color-border)]">
          <div class="text-5xl mb-4" aria-hidden="true">✅</div>
          <h2 class="font-display text-2xl font-bold text-[var(--color-secondary)] mb-3">
            {m.contact_form_success()}
          </h2>
          <p class="text-[var(--color-muted)] mb-6">
            {getLocale() === 'ar' ? 'سنتواصل معك قريباً' : "We'll get back to you soon"}
          </p>
          <Button onclick={() => sent = false} variant="outline">
            {m.auto_send_another_message()}
          </Button>
        </div>
      {:else}
        <form onsubmit={handleSubmit} novalidate class="bg-white rounded-2xl border border-[var(--color-border)] p-8 flex flex-col gap-5">
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-5">
            <Input
              label={m.contact_form_name()}
              bind:value={name}
              required
              placeholder={m.auto_john_smith()}
            />
            <Input
              type="email"
              label={m.contact_form_email()}
              bind:value={email}
              required
              placeholder="hello@example.com"
            />
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-5">
            <Input
              type="tel"
              label={m.contact_form_phone()}
              bind:value={phone}
              placeholder="+966 5X XXX XXXX"
            />
            <Input
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
