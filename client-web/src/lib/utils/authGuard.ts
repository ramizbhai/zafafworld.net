import { goto } from '$app/navigation';
import { authStore } from '$lib/stores/auth.svelte.js';

export function requireAuth(action: () => void) {
  if (authStore.isAuthenticated) {
    action();
  } else {
    goto('/auth/login');
  }
}

export function authGuard(e: Event) {
  if (!authStore.isAuthenticated) {
    e.preventDefault();
    goto('/auth/login');
  }
}
