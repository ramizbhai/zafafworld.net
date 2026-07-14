import { writable } from 'svelte/store';

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
    console.warn('Admin Panel does not need upgrades');
}

export function closeUpgradeModal() {
    upgradeStore.update(s => ({ ...s, showModal: false }));
}
