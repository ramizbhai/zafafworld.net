import { setVendorLimits } from '$lib/stores/vendorStore';

export function createLayoutState() {
    let sidebarOpen = $state(false);
    let userMenuOpen = $state(false);
    let langMenuOpen = $state(false);
    let statsExpanded = $state(false);
    let searchOpen = $state(false);
    let notifOpen = $state(false);
    let gateLock = $state('none');

    let vendor = $state<any>({
        name_en: 'Partner',
        name_ar: 'شريك',
        category: '',
        status: 'active',
        tier_id: 'free_tier',
        subscriptionTierId: 'free_tier',
        policy_limits: { max_products: 1, max_categories: 1 }
    });

    let notifications = $state<any[]>([]);
    let tiers = $state<any[]>([]);

    function initStreams(data: any) {
        if (data.streamed?.telemetry) {
            data.streamed.telemetry.then((res: any) => {
                if (res?.status === 'stopped') {
                    gateLock = 'stopped';
                }
                if (res?.data?.vendor) {
                    vendor = res.data.vendor;
                    setVendorLimits(vendor.tier_id, vendor.policy_limits);
                }
            });
        }
        if (data.streamed?.notifications) {
            data.streamed.notifications.then((res: any) => {
                notifications = res?.notifications || [];
            });
        }
        if (data.streamed?.tiers) {
            data.streamed.tiers.then((res: any) => {
                tiers = res?.tiers || [];
            });
        }
    }

    function toggleSidebar() { sidebarOpen = !sidebarOpen; }
    function closeSidebar() { sidebarOpen = false; }
    
    function closeAllMenus() {
        userMenuOpen = false;
        langMenuOpen = false;
        notifOpen = false;
        searchOpen = false;
    }

    function handleWindowClick(e: MouseEvent) {
        const t = e.target as HTMLElement;
        if (!t.closest('.user-menu-wrap')) userMenuOpen = false;
        if (!t.closest('.lang-menu-wrap')) langMenuOpen = false;
        if (!t.closest('.notif-wrap')) notifOpen = false;
        if (!t.closest('.search-bar')) searchOpen = false;
    }

    return {
        get sidebarOpen() { return sidebarOpen; },
        set sidebarOpen(v) { sidebarOpen = v; },
        get userMenuOpen() { return userMenuOpen; },
        set userMenuOpen(v) { userMenuOpen = v; },
        get langMenuOpen() { return langMenuOpen; },
        set langMenuOpen(v) { langMenuOpen = v; },
        get statsExpanded() { return statsExpanded; },
        set statsExpanded(v) { statsExpanded = v; },
        get searchOpen() { return searchOpen; },
        set searchOpen(v) { searchOpen = v; },
        get notifOpen() { return notifOpen; },
        set notifOpen(v) { notifOpen = v; },
        get gateLock() { return gateLock; },
        get vendor() { return vendor; },
        get notifications() { return notifications; },
        get tiers() { return tiers; },
        initStreams,
        toggleSidebar,
        closeSidebar,
        closeAllMenus,
        handleWindowClick
    };
}
