<script lang="ts">
  import * as m from '$lib/paraglide/messages.js';
import { getLocale } from '$lib/paraglide/runtime.js';
import { getLocalizedField } from '$lib/utils/localize.js';
  import Button from '$lib/components/ui/Button.svelte';
  import Input from '$lib/components/ui/Input.svelte';
  import { enhance } from '$app/forms';
  import { authStore } from '$lib/stores/auth.svelte.js';
  import { toasts } from '$lib/stores/toast.svelte.js';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import type { SubmitFunction } from '@sveltejs/kit';
  import { z } from 'zod';

  const registerSchema = z.object({
    firstName: z.string().min(2, 'First name must be at least 2 characters'),
    lastName: z.string().min(2, 'Last name must be at least 2 characters'),
    email: z.string().email('Invalid email format'),
    phone: z.string().regex(/^\+?[1-9]\d{1,14}$/, 'Invalid phone number format'),
    city: z.string().min(1, 'City is required'),
    password: z.string().min(8, 'Password must be at least 8 characters').regex(/[A-Z]/, 'Must contain uppercase').regex(/[a-z]/, 'Must contain lowercase').regex(/[0-9]/, 'Must contain number'),
    confirm: z.string(),
    agreed: z.boolean().refine(val => val === true, { message: 'You must agree to the terms' })
  }).refine((data) => data.password === data.confirm, {
    message: "Passwords don't match",
    path: ['confirm'],
  });

  let firstName = $state('');
  let lastName  = $state('');
  let email     = $state('');
  let phone     = $state('');
  let city      = $state('');
  let password  = $state('');
  let confirm   = $state('');
  let agreed    = $state(false);
  let loading   = $state(false);
  let errors    = $state<Record<string, string>>({});
  let termsContainer = $state<HTMLElement | null>(null);

  let { form, data } = $props();

  function validate(): boolean {
    const result = registerSchema.safeParse({ firstName, lastName, email, phone, city, password, confirm, agreed });
    if (result.success) {
      errors = {};
      return true;
    } else {
      const e: Record<string, string> = {};
      result.error.issues.forEach(issue => {
        if (issue.path[0]) {
          e[issue.path[0].toString()] = issue.message;
        }
      });
      errors = e;
      if (e.agreed && termsContainer) {
        termsContainer.scrollIntoView({ behavior: 'smooth', block: 'center' });
      }
      return false;
    }
  }

  const handleRegister: SubmitFunction = ({ cancel }) => {
    if (!validate()) {
      cancel();
      return;
    }
    loading = true;
    return async ({ result }) => {
      loading = false;
      if (result.type === 'success' && result.data?.success) {
        const rawUser = result.data.user;
        authStore.setUser(rawUser ? {
          id: rawUser.id,
          name: rawUser.first_name
            ? `${rawUser.first_name} ${rawUser.last_name}`
            : rawUser.email,
          avatar: rawUser.avatar,
          isVip: rawUser.isVip
        } : null);
        toasts.push('success', 'Registration Successful! Redirecting...');
        const redirectTo = $page.url.searchParams.get('redirect') ?? '/dashboard';
        goto(decodeURIComponent(redirectTo), { invalidateAll: true });
      } else if (result.type === 'failure') {
        toasts.push('error', result.data?.message || 'Registration failed. Please check your constraints.');
      } else {
        toasts.push('error', 'An unexpected error occurred.');
      }
    };
  };
</script>

<svelte:head>
  <title>{m.auth_register_title()} - {m.meta_siteName()}</title>
</svelte:head>

<div class="min-h-screen flex items-center justify-center bg-[var(--color-surface-alt)] px-4 py-12">
  <div class="w-full max-w-lg">
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
      <div class="text-center mb-8">
        <h1 class="font-display text-2xl font-bold text-[var(--color-secondary)]">{m.auth_register_title()}</h1>
        <p class="text-[var(--color-muted)] mt-2 text-sm">{m.auth_register_subtitle()}</p>
      </div>

      <form method="POST" use:enhance={handleRegister} novalidate class="flex flex-col gap-4">
        <input type="hidden" name="role" value="client" />
        <div class="grid grid-cols-2 gap-4">
          <Input name="firstName" label={m.auth_register_firstName()} bind:value={firstName} required error={errors.firstName} />
          <Input name="lastName" label={m.auth_register_lastName()}  bind:value={lastName}  required error={errors.lastName} />
        </div>

        <Input name="email" type="email" label={m.auth_register_email()}  bind:value={email}    required error={errors.email}    placeholder="you@example.com" leadingIcon="✉️" />
        
        <div class="grid grid-cols-2 gap-4">
          <Input name="phone" type="tel"   label={m.auth_register_phone()}  bind:value={phone}    placeholder="+966 5X XXX XXXX"   leadingIcon="📱" />
          
          <div class="form-group flex flex-col gap-1">
            <label class="text-sm font-medium text-[var(--color-text)]" for="city">City</label>
            <div class="relative">
              <span class="absolute inset-y-0 start-0 flex items-center ps-3 text-gray-500 pointer-events-none">📍</span>
              <select name="city" id="city" bind:value={city} class="w-full bg-[var(--color-surface)] border border-[var(--color-border)] text-[var(--color-text)] text-sm rounded-xl focus:ring-primary focus:border-primary block p-2.5 ps-10" required>
                <option value="" disabled selected>Select a city</option>
                {#each data.cities || [] as c}
                  <option value={c.id}>{getLocalizedField(c, 'name', getLocale())}</option>
                {/each}
              </select>
            </div>
            {#if errors.city}
              <p class="text-xs text-[var(--color-error)] mt-1">{errors.city}</p>
            {/if}
          </div>
        </div>

        <Input name="password" type="password" label={m.auth_register_password()}        bind:value={password} required error={errors.password} leadingIcon="🔒" />
        <Input name="confirm" type="password" label={m.auth_register_confirmPassword()} bind:value={confirm}  required error={errors.confirm}  leadingIcon="🔒" />

        <!-- Terms -->
        <div bind:this={termsContainer} class="transition-colors duration-300 {errors.agreed ? 'bg-red-50 border border-red-200 p-3 rounded-xl' : 'p-1'}">
          <label class="flex items-start gap-3 cursor-pointer">
            <input
              type="checkbox"
              bind:checked={agreed}
              class="mt-1 accent-[var(--color-primary)] w-4 h-4 flex-shrink-0"
            />
            <span class="text-sm text-[var(--color-text)]">
              {m.auth_register_agreeTerms()}
              <a href="/terms" class="text-[var(--color-primary)] hover:underline">{m.footer_terms()}</a>
            </span>
          </label>
          {#if errors.agreed}
            <p class="text-xs text-[var(--color-error)] mt-1">{errors.agreed}</p>
          {/if}
        </div>

        <Button type="submit" variant="primary" size="lg" fullWidth {loading} class="mt-2">
          {m.auth_register_submit()}
        </Button>
      </form>

      <div class="mt-6 text-center">
        <p class="text-sm text-[var(--color-muted)]">
          {m.auth_register_hasAccount()}
          <a href="/auth/login{$page.url.searchParams.has('redirect') ? `?redirect=${encodeURIComponent($page.url.searchParams.get('redirect') || '')}` : ''}" class="text-[var(--color-primary)] font-medium hover:underline ms-1">
            {m.auth_register_login()}
          </a>
        </p>
      </div>
    </div>
  </div>
</div>
