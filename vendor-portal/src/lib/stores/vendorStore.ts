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

const defaultLimits: PolicyLimits = {
    products: 1,
    cover_photos: 1,
    photos: 1,
    videos: 0,
    description_blocks: 1
};

export const vendorStore = writable<VendorState>({
    tier_id: 'free',
    policy_limits: defaultLimits
});

export function setVendorLimits(tier_id: string, limits: any) {
    if (!limits) return;
    const mappedLimits: PolicyLimits = {
        products: limits.max_products !== undefined ? limits.max_products : (limits.products ?? defaultLimits.products),
        cover_photos: limits.max_cover_photos !== undefined ? limits.max_cover_photos : (limits.cover_photos ?? defaultLimits.cover_photos),
        photos: limits.max_additional_photos !== undefined ? limits.max_additional_photos : (limits.photos ?? defaultLimits.photos),
        videos: limits.max_videos !== undefined ? limits.max_videos : (limits.videos ?? defaultLimits.videos),
        description_blocks: limits.max_description_blocks !== undefined ? limits.max_description_blocks : (limits.description_blocks ?? defaultLimits.description_blocks),
    };
    vendorStore.set({
        tier_id: tier_id || 'free',
        policy_limits: mappedLimits
    });
}
