import { writable } from 'svelte/store';
import { goto } from '$app/navigation';
import { browser } from '$app/environment';

export interface UpgradeState {
    showModal: boolean;
    limitType: string;
    currentTier: string;
    message: string;
}

export const upgradeStore = writable<UpgradeState>({
    showModal: false,
    limitType: '',
    currentTier: '',
    message: ''
});

export function triggerUpgrade(limitType: string, currentTier: string = '', message: string = '') {
    upgradeStore.set({ showModal: true, limitType, currentTier, message });
}

export function closeUpgradeModal() {
    upgradeStore.update(s => ({ ...s, showModal: false }));
}
