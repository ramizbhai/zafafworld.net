import { writable } from 'svelte/store'; export const vendorStore = writable({ tier_id: 'free', policy_limits: { description_blocks: 5 } });
