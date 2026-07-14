export class MarketingState {
    // Data
    campaigns = $state<any[]>([]);

    // Filters
    statusFilter = $state<"all" | "pending" | "approved" | "rejected">("all");

    // Selection (Bulk Actions)
    checkedIds = $state<string[]>([]);

    // Action State
    isSubmitting = $state(false);

    // Modals
    selectedCampaign = $state<any>(null);
    isDetailModalOpen = $state(false);

    activeRejectId = $state<string | null>(null);
    isRejectModalOpen = $state(false);
    rejectionReason = $state("");

    isBulkRejectModalOpen = $state(false);
    bulkRejectionReason = $state("");

    // Telemetry KPIs
    totalCount = $derived(this.campaigns.length);
    pendingCount = $derived(this.campaigns.filter((c: any) => c.status === "pending").length);
    approvedCount = $derived(this.campaigns.filter((c: any) => c.status === "approved").length);
    rejectedCount = $derived(this.campaigns.filter((c: any) => c.status === "rejected").length);

    filteredCampaigns = $derived.by(() => {
        if (this.statusFilter === "all") return this.campaigns;
        return this.campaigns.filter((c: any) => c.status === this.statusFilter);
    });

    constructor(initialCampaigns: any[] = []) {
        this.campaigns = initialCampaigns;
    }

    setCampaigns(newCampaigns: any[]) {
        this.campaigns = newCampaigns;
        // Clean up checked IDs if they no longer exist
        const existingIds = new Set(newCampaigns.map(c => c.id));
        this.checkedIds = this.checkedIds.filter(id => existingIds.has(id));
    }

    // Toggle Actions
    toggleCheckAll() {
        if (this.checkedIds.length === this.filteredCampaigns.length && this.filteredCampaigns.length > 0) {
            this.checkedIds = [];
        } else {
            this.checkedIds = this.filteredCampaigns.map((c: any) => c.id);
        }
    }

    toggleCheck(id: string) {
        if (this.checkedIds.includes(id)) {
            this.checkedIds = this.checkedIds.filter((x) => x !== id);
        } else {
            this.checkedIds = [...this.checkedIds, id];
        }
    }

    // Modal Actions
    openDetail(campaign: any) {
        this.selectedCampaign = campaign;
        this.isDetailModalOpen = true;
    }

    openReject(id: string) {
        this.activeRejectId = id;
        this.rejectionReason = "";
        this.isRejectModalOpen = true;
    }

    openBulkReject() {
        if (this.checkedIds.length === 0) return;
        this.bulkRejectionReason = "";
        this.isBulkRejectModalOpen = true;
    }

    // Optimistic Updates
    optimisticApprove(id: string) {
        const idx = this.campaigns.findIndex(c => c.id === id);
        if (idx !== -1) {
            this.campaigns[idx].status = "approved";
        }
    }

    optimisticReject(id: string) {
        const idx = this.campaigns.findIndex(c => c.id === id);
        if (idx !== -1) {
            this.campaigns[idx].status = "rejected";
        }
    }

    optimisticBulkApprove(ids: string[]) {
        const idSet = new Set(ids);
        for (let i = 0; i < this.campaigns.length; i++) {
            if (idSet.has(this.campaigns[i].id)) {
                this.campaigns[i].status = "approved";
            }
        }
    }

    optimisticBulkReject(ids: string[]) {
        const idSet = new Set(ids);
        for (let i = 0; i < this.campaigns.length; i++) {
            if (idSet.has(this.campaigns[i].id)) {
                this.campaigns[i].status = "rejected";
            }
        }
    }
}
