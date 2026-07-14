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

  let email    = $state('');
  let password = $state('');
  let loading  = $state(false);

  let { form } = $props();

  // Detect ?error=wrong_portal redirect from the client dashboard guard
  let portalError = $derived($page.url.searchParams.get('error'));
  let redirectedRole = $derived($page.url.searchParams.get('role') ?? '');

  const handleLogin: SubmitFunction = () => {
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
        toasts.push('success', 'Authentication Successful! Redirecting...');
        const redirectTo = $page.url.searchParams.get('redirect') ?? '/dashboard';
        // CRITICAL: Use full page navigation (not goto()) so +layout.server.ts
        // re-runs with the newly-set session cookie, fully hydrating auth state.
        window.location.href = decodeURIComponent(redirectTo);
      } else if (result.type === 'failure') {
        toasts.push('error', result.data?.message || 'An unexpected validation exception occurred.');
      } else {
        toasts.push('error', 'An unexpected error occurred.');
      }
    };
  };
</script>

<svelte:head>
  <title>{m.auth_login_title()} — {m.meta_siteName()}</title>
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
      <div class="text-center mb-8">
        <h1 class="font-display text-2xl font-bold text-[var(--color-secondary)]">{m.auth_login_title()}</h1>
        <p class="text-[var(--color-muted)] mt-2 text-sm">{m.auth_login_subtitle()}</p>
      </div>

      {#if portalError === 'wrong_portal'}
        <!-- Wrong portal error — shown when admin/vendor tries to access client dashboard -->
        <div class="mb-6 p-4 rounded-xl border text-sm text-start bg-amber-50 border-amber-300 text-amber-900">
          <div class="flex items-start gap-3">
            <span class="text-xl leading-none mt-0.5">⚠️</span>
            <div>
              {#if redirectedRole === 'admin'}
                <p class="font-bold mb-1">
                  {m.auto_this_is_the_client_p()}
                </p>
                <p>
                  {m.auto_your_account_has_adm()}
                  {#if getLocale() !== 'ar'}
                    <a href="https://admin.zafafworld.net" class="font-semibold underline hover:text-amber-700 ms-1">
                      Admin Panel ↗
                    </a>
                  {:else}
                    <a href="https://admin.zafafworld.net" class="font-semibold underline hover:text-amber-700 me-1">
                      لوحة الإدارة ↗
                    </a>
                  {/if}
                </p>
              {:else if redirectedRole === 'vendor'}
                <p class="font-bold mb-1">
                  {m.auto_this_is_the_client_p()}
                </p>
                <p>
                  {m.auto_your_account_is_a_ve()}
                  {#if getLocale() !== 'ar'}
                    <a href="https://vendor.zafafworld.net" class="font-semibold underline hover:text-amber-700 ms-1">
                      Vendor Portal ↗
                    </a>
                  {:else}
                    <a href="https://vendor.zafafworld.net" class="font-semibold underline hover:text-amber-700 me-1">
                      بوابة الموردين ↗
                    </a>
                  {/if}
                </p>
              {:else}
                <p class="font-bold mb-1">
                  {m.auto_access_denied()}
                </p>
                <p>
                  {m.auto_this_portal_is_for_c()}
                </p>
              {/if}
            </div>
          </div>
        </div>
      {:else}
        <div class="mb-8 p-4 rounded-xl bg-blue-50 border border-blue-200 text-sm text-blue-800 text-center">
          New to the platform? Please <a href="/auth/register{$page.url.searchParams.has('redirect') ? `?redirect=${encodeURIComponent($page.url.searchParams.get('redirect') || '')}` : ''}" class="font-semibold underline">Register</a> first to access personalized planning utilities.
        </div>
      {/if}


      <form method="POST" use:enhance={handleLogin} novalidate class="flex flex-col gap-5">
        <Input
          name="email"
          type="email"
          label={m.auth_login_email()}
          bind:value={email}
          required
          placeholder="you@example.com"
          leadingIcon="✉️"
        />

        <div>
          <Input
            name="password"
            type="password"
            label={m.auth_login_password()}
            bind:value={password}
            required
            placeholder="••••••••"
            leadingIcon="🔒"
          />
          <div class="mt-2 text-end">
            <a href="/auth/forgot-password" class="text-xs text-[var(--color-primary)] hover:underline">
              {m.auth_login_forgotPassword()}
            </a>
          </div>
        </div>

        <Button type="submit" variant="primary" size="lg" fullWidth {loading}>
          {m.auth_login_submit()}
        </Button>
      </form>

      <div class="mt-6 text-center">
        <p class="text-sm text-[var(--color-muted)]">
          {m.auth_login_noAccount()}
          <a href="/auth/register{$page.url.searchParams.has('redirect') ? `?redirect=${encodeURIComponent($page.url.searchParams.get('redirect') || '')}` : ''}" class="text-[var(--color-primary)] font-medium hover:underline ms-1">
            {m.auth_login_register()}
          </a>
        </p>
      </div>
    </div>
  </div>
</div>
