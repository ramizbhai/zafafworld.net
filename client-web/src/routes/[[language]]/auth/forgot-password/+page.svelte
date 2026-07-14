<script lang="ts">
  import * as m from '$lib/paraglide/messages.js';
import { getLocale } from '$lib/paraglide/runtime.js';
import { getLocalizedField } from '$lib/utils/localize.js';
  import Button from '$lib/components/ui/Button.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import { enhance } from '$app/forms';
  import { toasts } from '$lib/stores/toast.svelte.js';
  import { page } from '$app/stores';
  import type { SubmitFunction } from '@sveltejs/kit';

  let email   = $state('');
  let loading = $state(false);
  let success = $state(false);

  let { form } = $props();

  const handleForgot: SubmitFunction = () => {
    loading = true;
    return async ({ result }) => {
      loading = false;
      if (result.type === 'success' && result.data?.success) {
        success = true;
        toasts.push('success', result.data.message || 'Recovery email successfully sent!');
      } else if (result.type === 'failure') {
        toasts.push('error', result.data?.message || 'An unexpected error occurred.');
      } else {
        toasts.push('error', 'An unexpected error occurred.');
      }
    };
  };
</script>

<svelte:head>
  <title>
    {m.auto_forgot_password()} — {m.meta_siteName()}
  </title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center bg-[var(--color-surface-alt)] px-4 py-12">
  <div class="w-full max-w-md">
    <!-- Logo -->
    <div class="text-center mb-8">
      <a href="/" class="inline-flex items-center gap-3">
        <div class="w-12 h-12 rounded-2xl bg-[var(--color-primary)] flex items-center justify-center">
          <span class="text-[var(--color-secondary)] font-bold text-lg font-display">ز</span>
        </div>
        <span class="font-display text-2xl font-bold text-[var(--color-secondary)]">
          {m.auto_zafafworld()}
        </span>
      </a>
    </div>

    <div class="bg-white rounded-2xl border border-[var(--color-border)] shadow-[var(--shadow-lg)] p-8">
      {#if success}
        <!-- Success State -->
        <div class="text-center py-6 animate-fade-in">
          <div class="w-16 h-16 bg-green-50 border border-green-200 text-green-600 rounded-full flex items-center justify-center mx-auto mb-6 text-2xl">
            ✓
          </div>
          <h1 class="font-display text-2xl font-bold text-[var(--color-secondary)] mb-4">
            {m.auto_recovery_link_sent()}
          </h1>
          <p class="text-[var(--color-muted)] text-sm mb-8 leading-relaxed">
            {m.auto_if_this_email_is_reg()}
          </p>
          <Button href="/auth/login" variant="primary" size="lg" fullWidth>
            {m.auto_return_to_login()}
          </Button>
        </div>
      {:else}
        <!-- Form State -->
        <div class="text-center mb-8">
          <h1 class="font-display text-2xl font-bold text-[var(--color-secondary)]">
            {m.auto_forgot_password_1()}
          </h1>
          <p class="text-[var(--color-muted)] mt-2 text-sm">
            {m.auto_enter_your_email_add()}
          </p>
        </div>

        {#if form?.message && !form?.success}
          <div class="mb-6 p-4 rounded-xl border text-sm text-start bg-red-50 border-red-200 text-red-900 flex items-start gap-3">
            <span class="text-xl leading-none mt-0.5">⚠️</span>
            <p>{form.message}</p>
          </div>
        {/if}

        <form method="POST" use:enhance={handleForgot} novalidate class="flex flex-col gap-5">
          <Input
            name="email"
            type="email"
            label={m.auto_email_address()}
            bind:value={email}
            required
            placeholder="you@example.com"
            leadingIcon="✉️"
          />

          <Button type="submit" variant="primary" size="lg" fullWidth {loading}>
            {m.auto_send_recovery_link()}
          </Button>
        </form>

        <div class="mt-6 text-center">
          <p class="text-sm text-[var(--color-muted)]">
            {m.auto_remembered_your_pass()}
            <a href="/auth/login" class="text-[var(--color-primary)] font-medium hover:underline ms-1">
              {m.auth_login_register() ? (m.auto_sign_in()) : 'Sign in'}
            </a>
          </p>
        </div>
      {/if}
    </div>
  </div>
</div>
