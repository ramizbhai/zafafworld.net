import { goto } from '$app/navigation';
import { authStore } from '$lib/stores/auth.svelte.js';

export function requireAuth(action: () => void) {
  if (authStore.isAuthenticated) {
    action();
  } else {
    const currentUrl = typeof window !== 'undefined' ? window.location.pathname + window.location.search : '/';
    goto(`/auth/login?redirect=${encodeURIComponent(currentUrl)}`);
  }
}

export function authGuard(e: Event) {
  if (!authStore.isAuthenticated) {
    e.preventDefault();
    const currentUrl = typeof window !== 'undefined' ? window.location.pathname + window.location.search : '/';
    goto(`/auth/login?redirect=${encodeURIComponent(currentUrl)}`);
  }
}
