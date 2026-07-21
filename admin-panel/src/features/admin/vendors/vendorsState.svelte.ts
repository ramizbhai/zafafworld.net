export function createVendorsState() {
    let activeTab = $state<'active' | 'stopped' | 'all'>('active');
    let search = $state('');
    let submittingIds = $state<string[]>([]);
    let submittingListingIds = $state<string[]>([]);
    
    let errorMessage = $state('');
    let successMessage = $state('');
    let rejectionReasonMap = $state<Record<string, string>>({});

    let promptStatusChange = $state<{ id: string, status: string } | null>(null);
    let statusReason = $state('');

    let promptAdModal = $state<any | null>(null);
    let adIsFeatured = $state(false);
    let adExpiryPreset = $state<'1month' | '3months' | 'custom'>('1month');
    let adCustomDate = $state('');

    let chatVendor = $state<any>(null);
    let chatMessages = $state<any[]>([]);
    let lightboxUrl = $state('');
    let chatInput = $state('');
    let isFetchingMessages = $state(false);
    let isSendingMessage = $state(false);
    let accountStatusFilter = $state('all');
    let subTierFilter = $state('all');
    let subStatusFilter = $state('all');
    let chatError = $state('');
    let pollInterval = $state<any>(null);

    return {
        get accountStatusFilter() { return accountStatusFilter; },
        set accountStatusFilter(v) { accountStatusFilter = v; },

        get subTierFilter() { return subTierFilter; },
        set subTierFilter(v) { subTierFilter = v; },

        get subStatusFilter() { return subStatusFilter; },
        set subStatusFilter(v) { subStatusFilter = v; },
        get activeTab() { return activeTab; },
        set activeTab(v) { activeTab = v; },

        get search() { return search; },
        set search(v) { search = v; },

        get submittingIds() { return submittingIds; },
        set submittingIds(v) { submittingIds = v; },

        get submittingListingIds() { return submittingListingIds; },
        set submittingListingIds(v) { submittingListingIds = v; },

        get errorMessage() { return errorMessage; },
        set errorMessage(v) { errorMessage = v; },

        get successMessage() { return successMessage; },
        set successMessage(v) { successMessage = v; },

        get rejectionReasonMap() { return rejectionReasonMap; },
        set rejectionReasonMap(v) { rejectionReasonMap = v; },

        get promptStatusChange() { return promptStatusChange; },
        set promptStatusChange(v) { promptStatusChange = v; },

        get statusReason() { return statusReason; },
        set statusReason(v) { statusReason = v; },

        get promptAdModal() { return promptAdModal; },
        set promptAdModal(v) { promptAdModal = v; },

        get adIsFeatured() { return adIsFeatured; },
        set adIsFeatured(v) { adIsFeatured = v; },

        get adExpiryPreset() { return adExpiryPreset; },
        set adExpiryPreset(v) { adExpiryPreset = v; },

        get adCustomDate() { return adCustomDate; },
        set adCustomDate(v) { adCustomDate = v; },

        get chatVendor() { return chatVendor; },
        set chatVendor(v) { chatVendor = v; },

        get chatMessages() { return chatMessages; },
        set chatMessages(v) { chatMessages = v; },

        get lightboxUrl() { return lightboxUrl; },
        set lightboxUrl(v) { lightboxUrl = v; },

        get chatInput() { return chatInput; },
        set chatInput(v) { chatInput = v; },

        get isFetchingMessages() { return isFetchingMessages; },
        set isFetchingMessages(v) { isFetchingMessages = v; },

        get isSendingMessage() { return isSendingMessage; },
        set isSendingMessage(v) { isSendingMessage = v; },

        get chatError() { return chatError; },
        set chatError(v) { chatError = v; },

        get pollInterval() { return pollInterval; },
        set pollInterval(v) { pollInterval = v; },
    };
}
