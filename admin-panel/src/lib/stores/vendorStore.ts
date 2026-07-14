import { writable } from 'svelte/store';

export interface PolicyLimits {
    products: number;
    cover_photos: number;
    photos: number;
    videos: number;
    description_blocks: number;
}

export interface VendorState {
    tier_id: string;
    policy_limits: PolicyLimits;
}

// Unlimited for Admin Panel
const defaultLimits: PolicyLimits = {
    products: -1,
    cover_photos: -1,
    photos: -1,
    videos: -1,
    description_blocks: -1
};

export const vendorStore = writable<VendorState>({
    tier_id: 'diamond', // Always diamond for admins
    policy_limits: defaultLimits
});

export function setVendorLimits(tier_id: string, limits: any) {}
