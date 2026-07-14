import { writable } from 'svelte/store';

// ─── UI Store: sidebar state, modals, loading ──────────────────────────────
function createUIStore() {
  const { subscribe, update, set } = writable({
    sidebarCollapsed: false,
    commandPaletteOpen: false,
    notificationPanelOpen: false,
    globalLoading: false,
  });

  return {
    subscribe,
    toggleSidebar: () => update(s => ({ ...s, sidebarCollapsed: !s.sidebarCollapsed })),
    setSidebarCollapsed: (v: boolean) => update(s => ({ ...s, sidebarCollapsed: v })),
    openCommandPalette: () => update(s => ({ ...s, commandPaletteOpen: true })),
    closeCommandPalette: () => update(s => ({ ...s, commandPaletteOpen: false })),
    toggleNotifications: () => update(s => ({ ...s, notificationPanelOpen: !s.notificationPanelOpen })),
    setLoading: (v: boolean) => update(s => ({ ...s, globalLoading: v })),
  };
}

export const ui = createUIStore();
