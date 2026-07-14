import { goto } from '$app/navigation';
import { untrack } from 'svelte';

export function createCouplesState(pageUrl: URL, getData: () => any) {
    let searchQuery = $state(pageUrl.searchParams.get('q') || '');
    let statusFilter = $state(pageUrl.searchParams.get('status') || 'all');
    let selectedInquiry = $state<any>(null);
    let isDrawerOpen = $state(false);
    let isSubmitting = $state(false);
    
    let currentPage = $state(parseInt(pageUrl.searchParams.get('page') || '1'));
    
    let sortField = $state<'customer_name' | 'wedding_date' | 'created_at' | 'status'>(pageUrl.searchParams.get('sort') as any || 'created_at');
    let sortAsc = $state(pageUrl.searchParams.get('order') === 'asc');

    let isAddModalOpen = $state(false);
    let newName = $state('');
    let newPhone = $state('');
    let newDate = $state('');
    let newMessage = $state('');

    $effect(() => {
        const q = searchQuery;
        const status = statusFilter;
        const field = sortField;
        const asc = sortAsc;
        const pageNum = currentPage;

        untrack(() => {
            const url = new URL(window.location.href);
            if (q) url.searchParams.set('q', q);
            else url.searchParams.delete('q');
            
            if (status !== 'all') url.searchParams.set('status', status);
            else url.searchParams.delete('status');
            
            url.searchParams.set('sort', field);
            url.searchParams.set('order', asc ? 'asc' : 'desc');
            url.searchParams.set('page', pageNum.toString());
            
            goto(url.toString(), { keepFocus: true, noScroll: true, replaceState: true });
        });
    });

    let activeSelectedInquiry = $derived(
        getData().inquiries?.find((inq: any) => inq.id === selectedInquiry?.id) || selectedInquiry
    );

    let paginatedInquiries = $derived(getData().inquiries || []);
    let totalPages = $derived(getData().totalPages || 1);

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
        
        get sortField() { return sortField; },
        set sortField(v) { sortField = v; },
        
        get sortAsc() { return sortAsc; },
        set sortAsc(v) { sortAsc = v; },
        
        get isAddModalOpen() { return isAddModalOpen; },
        set isAddModalOpen(v) { isAddModalOpen = v; },
        
        get newName() { return newName; },
        set newName(v) { newName = v; },
        
        get newPhone() { return newPhone; },
        set newPhone(v) { newPhone = v; },
        
        get newDate() { return newDate; },
        set newDate(v) { newDate = v; },
        
        get newMessage() { return newMessage; },
        set newMessage(v) { newMessage = v; },
        
        get activeSelectedInquiry() { return activeSelectedInquiry; },
        get paginatedInquiries() { return paginatedInquiries; },
        get totalPages() { return totalPages; }
    };
}
