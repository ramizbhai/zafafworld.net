import { untrack } from 'svelte';

export function createInquiriesState(initialData: () => any) {
    let rawInquiries = $state<any[]>(initialData()?.inquiries || []);
    
    // We synchronize our local state if the server data updates (e.g., after invalidateAll)
    // but without overwriting optimistic updates unnecessarily if we want to be robust. 
    // For simplicity, whenever `initialData().inquiries` changes identity, we re-sync.
    $effect(() => {
        const dataInquiries = initialData()?.inquiries || [];
        untrack(() => {
            rawInquiries = dataInquiries;
        });
    });

    let searchQuery = $state('');
    let statusFilter = $state('all');
    let selectedInquiry = $state<any>(null);
    let isDrawerOpen = $state(false);
    let isSubmitting = $state(false);
    
    let currentPage = $state(1);
    const pageSize = 5;

    // Reset pagination when filter changes
    $effect(() => {
        if (searchQuery || statusFilter) {
            untrack(() => {
                currentPage = 1;
            });
        }
    });

    let activeSelectedInquiry = $derived(
        rawInquiries.find((inq: any) => inq.id === selectedInquiry?.id) || selectedInquiry
    );

    let filteredInquiries = $derived.by(() => {
        let list = rawInquiries;
        
        if (searchQuery) {
            const q = searchQuery.toLowerCase();
            list = list.filter((inq: any) => 
                (inq.client_first_name && inq.client_first_name.toLowerCase().includes(q)) ||
                (inq.client_last_name && inq.client_last_name.toLowerCase().includes(q)) ||
                (inq.client_email && inq.client_email.toLowerCase().includes(q)) ||
                (inq.client_phone && inq.client_phone.includes(q)) ||
                (inq.name && inq.name.toLowerCase().includes(q)) ||
                (inq.email && inq.email.toLowerCase().includes(q)) ||
                (inq.phone && inq.phone.includes(q)) ||
                (inq.message && inq.message.toLowerCase().includes(q))
            );
        }

        if (statusFilter !== 'all') {
            list = list.filter((inq: any) => inq.status === statusFilter);
        }

        // Sort by created_at descending by default
        return [...list].sort((a: any, b: any) => {
            const timeA = new Date(a.created_at).getTime();
            const timeB = new Date(b.created_at).getTime();
            return timeB - timeA;
        });
    });

    let totalCount = $derived(filteredInquiries.length);
    let totalPages = $derived(Math.max(1, Math.ceil(totalCount / pageSize)));
    let paginatedInquiries = $derived.by(() => {
        const start = (currentPage - 1) * pageSize;
        return filteredInquiries.slice(start, start + pageSize);
    });

    // Optimistic Update API
    function optimisticallyUpdateStatus(id: string, status: string) {
        rawInquiries = rawInquiries.map(inq => 
            inq.id === id ? { ...inq, status } : inq
        );
        if (selectedInquiry && selectedInquiry.id === id) {
            selectedInquiry = { ...selectedInquiry, status };
        }
    }

    return {
        get searchQuery() { return searchQuery; },
        set searchQuery(v) { searchQuery = v; },
        
        get statusFilter() { return statusFilter; },
        set statusFilter(v) { statusFilter = v; },
        
        get selectedInquiry() { return selectedInquiry; },
        set selectedInquiry(v) { selectedInquiry = v; },
        
        get isDrawerOpen() { return isDrawerOpen; },
        set isDrawerOpen(v) { isDrawerOpen = v; },
        
        get isSubmitting() { return isSubmitting; },
        set isSubmitting(v) { isSubmitting = v; },
        
        get currentPage() { return currentPage; },
        set currentPage(v) { currentPage = v; },
        
        get rawInquiries() { return rawInquiries; },
        
        get activeSelectedInquiry() { return activeSelectedInquiry; },
        get paginatedInquiries() { return paginatedInquiries; },
        get totalPages() { return totalPages; },
        
        optimisticallyUpdateStatus
    };
}
