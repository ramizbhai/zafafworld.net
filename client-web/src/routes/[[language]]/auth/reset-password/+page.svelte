<script lang="ts">
  import * as m from '$lib/paraglide/messages.js';
import { getLocale } from '$lib/paraglide/runtime.js';
import { getLocalizedField } from '$lib/utils/localize.js';
  import Button from '$lib/components/ui/Button.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import { enhance } from '$app/forms';
  import { toasts } from '$lib/stores/toast.svelte.js';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import type { SubmitFunction } from '@sveltejs/kit';

  let password        = $state('');
  let confirmPassword = $state('');
  let loading         = $state(false);
  let success         = $state(false);

  let { form } = $props();

  // Extract token from query params reactively
  let token = $derived($page.url.searchParams.get('token') ?? '');

  const handleReset: SubmitFunction = () => {
    if (password.length < 8) {
      toasts.push('error', m.auto_password_must_be_at_());
      return () => {};
    }
    if (password !== confirmPassword) {
      toasts.push('error', m.auto_passwords_do_not_mat());
      return () => {};
    }

    loading = true;
    return async ({ result }) => {
      loading = false;
      if (result.type === 'success' && result.data?.success) {
        success = true;
        toasts.push('success', m.auto_password_reset_succe());
      } else if (result.type === 'failure') {
        toasts.push('error', result.data?.message || 'An error occurred during password reset.');
      } else {
        toasts.push('error', 'An unexpected error occurred.');
      }
    };
  };
</script>

<svelte:head>
  <title>
    {m.auto_reset_password()} — {m.meta_siteName()}
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
            {m.auto_success()}
          </h1>
          <p class="text-[var(--color-muted)] text-sm mb-8 leading-relaxed">
            {m.auto_your_password_has_be()}
          </p>
          <Button href="/auth/login" variant="primary" size="lg" fullWidth>
            {m.auto_sign_in_now()}
          </Button>
        </div>
      {:else if !token}
        <!-- Invalid/Missing Token State -->
        <div class="text-center py-6">
          <div class="w-16 h-16 bg-red-50 border border-red-200 text-red-500 rounded-full flex items-center justify-center mx-auto mb-6 text-2xl">
            ⚠️
          </div>
          <h1 class="font-display text-xl font-bold text-[var(--color-secondary)] mb-4">
            {m.auto_invalid_reset_link()}
          </h1>
          <p class="text-[var(--color-muted)] text-sm mb-8 leading-relaxed">
            {m.auto_the_password_recover()}
          </p>
          <Button href="/auth/forgot-password" variant="secondary" size="lg" fullWidth>
            {m.auto_request_new_link()}
          </Button>
        </div>
      {:else}
        <!-- Form State -->
        <div class="text-center mb-8">
          <h1 class="font-display text-2xl font-bold text-[var(--color-secondary)]">
            {m.auto_reset_password()}
          </h1>
          <p class="text-[var(--color-muted)] mt-2 text-sm">
            {m.auto_choose_a_strong_and_()}
          </p>
        </div>

        {#if form?.message}
          <div class="mb-6 p-4 rounded-xl border text-sm text-start bg-red-50 border-red-200 text-red-900 flex items-start gap-3">
            <span class="text-xl leading-none mt-0.5">⚠️</span>
            <p>{form.message}</p>
          </div>
        {/if}

        <form method="POST" use:enhance={handleReset} novalidate class="flex flex-col gap-5">
          <input type="hidden" name="token" value={token} />

          <Input
            name="password"
            type="password"
            label={m.auto_new_password()}
            bind:value={password}
            required
            placeholder="••••••••"
            leadingIcon="🔒"
          />

          <Input
            name="confirmPassword"
            type="password"
            label={m.auto_confirm_new_password()}
            bind:value={confirmPassword}
            required
            placeholder="••••••••"
            leadingIcon="🔒"
          />

          <Button type="submit" variant="primary" size="lg" fullWidth {loading}>
            {m.auto_save_password()}
          </Button>
        </form>
      {/if}
    </div>
  </div>
</div>
