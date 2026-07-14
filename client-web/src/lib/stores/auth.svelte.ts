export type User = {
  id: string;
  name: string;
  avatar?: string;
  isVip?: boolean;
} | null;

class AuthStore {
  user = $state<User>(null);
  isAuthenticated = $derived(this.user !== null);

  setUser(user: User) {
    this.user = user;
  }

  async logout() {
    this.user = null;
    try {
      await fetch('/bff/v1/auth/logout', { method: 'POST' });
    } catch (err) {
      console.error('[authStore] Logout request failed:', err);
    }
    // Perform a full reload to clear all SvelteKit layout cache and return to landing page
    window.location.href = '/';
  }
}

export const authStore = new AuthStore();
