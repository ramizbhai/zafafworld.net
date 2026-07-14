export function createDashboardState(getData: () => any) {
    let currentPage = $state(1);
    let selectedInquiry = $state<any>(null);
    let selectedStatus = $state('');
    let isModalOpen = $state(false);

    let telemetry = $state<any>({
        vendor:  { name_en: 'New Vendor', name_ar: 'شريك زفاف', status: 'active' },
        metrics: {
            activeLeadsCount: 0,
            totalPendingRevenuePipeline: 0,
            active_packages: 0,
            active_products: 0,
            total_reviews: 0,
            avg_overall: 0,
            avg_quality: 0,
            avg_staff: 0,
            avg_communication: 0
        }
    });

    let tiers = $state<any[]>([]);

    function initStreams() {
        const data = getData();
        if (data.streamed?.telemetry) {
            data.streamed.telemetry.then((res: any) => {
                if (res?.data) telemetry = res.data;
            });
        }
        if (data.streamed?.tiers) {
            data.streamed.tiers.then((res: any) => {
                tiers = res?.tiers || [];
            });
        }
    }

    let vendor = $derived(telemetry.vendor);
    let metrics = $derived(telemetry.metrics);
    let hasListings = $derived((telemetry?.products?.length ?? 0) > 0 || (metrics?.active_products ?? 0) > 0);

    let inquiries = $derived(getData().inquiries ?? []);
    let itemsPerPage = 10;
    
    let pagedInquiries = $derived(
        inquiries.slice((currentPage - 1) * itemsPerPage, currentPage * itemsPerPage)
    );
    
    let totalPages = $derived(Math.ceil(inquiries.length / itemsPerPage) || 1);

    function openEditModal(inquiry: any) {
        selectedInquiry = inquiry;
        selectedStatus = inquiry.status;
        isModalOpen = true;
    }

    function closeEditModal() {
        isModalOpen = false;
        selectedInquiry = null;
    }

    return {
        get currentPage() { return currentPage; },
        set currentPage(v) { currentPage = v; },
        get selectedInquiry() { return selectedInquiry; },
        set selectedInquiry(v) { selectedInquiry = v; },
        get selectedStatus() { return selectedStatus; },
        set selectedStatus(v) { selectedStatus = v; },
        get isModalOpen() { return isModalOpen; },
        set isModalOpen(v) { isModalOpen = v; },
        
        get telemetry() { return telemetry; },
        get tiers() { return tiers; },
        get vendor() { return vendor; },
        get metrics() { return metrics; },
        get hasListings() { return hasListings; },
        
        get inquiries() { return inquiries; },
        get pagedInquiries() { return pagedInquiries; },
        get totalPages() { return totalPages; },
        get itemsPerPage() { return itemsPerPage; },
        
        initStreams,
        openEditModal,
        closeEditModal
    };
}
