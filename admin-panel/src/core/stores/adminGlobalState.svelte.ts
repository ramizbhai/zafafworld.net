import { notificationStore } from '../../features/notifications/stores/notificationStore.svelte';

function createAdminGlobalState() {
    let sidebarCollapsed = $state(true);
    let notifOpen = $state(false);
    let searchQuery = $state('');
    let unreadChatsCount = $state(0);
    
    // We export a derived value for unread inquiries from the notificationStore
    let unreadInquiriesCount = $derived(notificationStore.unreadCount);

    return {
        get sidebarCollapsed() { return sidebarCollapsed; },
        set sidebarCollapsed(v) { sidebarCollapsed = v; },
        
        get notifOpen() { return notifOpen; },
        set notifOpen(v) { notifOpen = v; },
        
        get searchQuery() { return searchQuery; },
        set searchQuery(v) { searchQuery = v; },
        
        get unreadChatsCount() { return unreadChatsCount; },
        set unreadChatsCount(v) { unreadChatsCount = v; },
        
        get unreadInquiriesCount() { return unreadInquiriesCount; }
    };
}

export const adminGlobalState = createAdminGlobalState();
