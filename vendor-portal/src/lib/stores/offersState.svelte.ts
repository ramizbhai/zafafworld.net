export type TabId = "all" | "active" | "scheduled" | "pending" | "history";

export const tabs: { id: TabId; labelEn: string; labelAr: string; statusFilter?: string }[] = [
    { id: "all",       labelEn: "All",       labelAr: "الكل" },
    { id: "active",    labelEn: "Active",    labelAr: "نشط",         statusFilter: "approved" },
    { id: "scheduled", labelEn: "Scheduled", labelAr: "مجدول",       statusFilter: "approved" },
    { id: "pending",   labelEn: "Pending",   labelAr: "قيد المراجعة", statusFilter: "pending" },
    { id: "history",   labelEn: "History",   labelAr: "السابقة" },
];

export function createOffersState(getData: () => any) {
    let activeTab = $state<TabId>("all");
    let searchQuery = $state("");
    
    // Delete confirmation modal
    let isDeleteConfirmOpen = $state(false);
    let offerToDelete = $state<any>(null);

    // Action button loading states
    let pausingId = $state<string | null>(null);
    let resumingId = $state<string | null>(null);
    let duplicatingId = $state<string | null>(null);
    let renewingId = $state<string | null>(null);

    // Renew Modal State
    let isRenewModalOpen = $state(false);
    let offerToRenew = $state<any>(null);
    let renewDays = $state(14);

    let currentPage = $derived(getData().filters?.page || 1);
    let totalPages = $derived(Math.max(1, Math.ceil((getData().total || 0) / (getData().filters?.limit || 12))));

    let filteredOffers = $derived.by(() => {
        const offers = getData().offers || [];
        if (activeTab === "all") return offers;
        if (activeTab === "active") {
            return offers.filter((o: any) => o.derived_status === "Active");
        }
        if (activeTab === "scheduled") {
            return offers.filter((o: any) => o.derived_status === "Scheduled");
        }
        if (activeTab === "pending") {
            return offers.filter((o: any) => o.derived_status === "Pending" || o.derived_status === "Rejected");
        }
        if (activeTab === "history") {
            return offers.filter((o: any) =>
                o.derived_status === "Expired" || o.derived_status === "Cancelled" || o.derived_status === "Paused"
            );
        }
        return offers;
    });

    let tabCounts = $derived.by(() => {
        const offers = getData().offers || [];
        return {
            all: offers.length,
            active: offers.filter((o: any) => o.derived_status === "Active").length,
            scheduled: offers.filter((o: any) => o.derived_status === "Scheduled").length,
            pending: offers.filter((o: any) => o.derived_status === "Pending" || o.derived_status === "Rejected").length,
            history: offers.filter((o: any) => ["Expired", "Cancelled", "Paused"].includes(o.derived_status)).length,
        };
    });

    return {
        get activeTab() { return activeTab; },
        set activeTab(v) { activeTab = v; },
        
        get searchQuery() { return searchQuery; },
        set searchQuery(v) { searchQuery = v; },

        get isDeleteConfirmOpen() { return isDeleteConfirmOpen; },
        set isDeleteConfirmOpen(v) { isDeleteConfirmOpen = v; },
        
        get offerToDelete() { return offerToDelete; },
        set offerToDelete(v) { offerToDelete = v; },

        get pausingId() { return pausingId; },
        set pausingId(v) { pausingId = v; },
        
        get resumingId() { return resumingId; },
        set resumingId(v) { resumingId = v; },
        
        get duplicatingId() { return duplicatingId; },
        set duplicatingId(v) { duplicatingId = v; },
        
        get renewingId() { return renewingId; },
        set renewingId(v) { renewingId = v; },

        get isRenewModalOpen() { return isRenewModalOpen; },
        set isRenewModalOpen(v) { isRenewModalOpen = v; },
        
        get offerToRenew() { return offerToRenew; },
        set offerToRenew(v) { offerToRenew = v; },
        
        get renewDays() { return renewDays; },
        set renewDays(v) { renewDays = v; },

        get currentPage() { return currentPage; },
        get totalPages() { return totalPages; },
        get filteredOffers() { return filteredOffers; },
        get tabCounts() { return tabCounts; }
    };
}
