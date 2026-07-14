<script lang="ts">
    import { createVendorsState } from '../../../features/admin/vendors/vendorsState.svelte';
    import { filterVendors } from '../../../features/admin/vendors/vendors.service';
    
    import VendorsHeader from '../../../features/admin/vendors/components/VendorsHeader.svelte';
    import VendorsFilterBar from '../../../features/admin/vendors/components/VendorsFilterBar.svelte';
    import VendorGrid from '../../../features/admin/vendors/components/VendorGrid.svelte';
    import PendingListingsQueue from '../../../features/admin/vendors/components/PendingListingsQueue.svelte';
    import VendorActionModal from '../../../features/admin/vendors/components/VendorActionModal.svelte';
    import VendorChatDrawer from '../../../features/admin/vendors/components/VendorChatDrawer.svelte';

    import '../../../features/admin/vendors/components/styles.css';

    let { data } = $props<{ data: { vendors: any[]; pendingListings: any[]; tiers?: any[] } }>();

    let vendorsState = createVendorsState();

    let vendors = $derived(data.vendors || []);
    let pendingListings = $derived(data.pendingListings || []);
    
    let activeCount = $derived(vendors.filter((v: any) => v.status === 'active' && v.subscription_status !== 'stopped').length);
    let stoppedCount = $derived(vendors.filter((v: any) => v.subscription_status === 'stopped' && v.status !== 'pending').length);
    
    let filteredVendors = $derived(filterVendors(vendors, vendorsState.activeTab, vendorsState.search));
</script>

<svelte:head>
    <title>Vendor Governance | ZafafWorld Admin</title>
</svelte:head>

<div class="curation-container">
    <VendorsHeader {vendorsState} />
    
    <VendorsFilterBar 
        {vendorsState} 
        {activeCount} 
        {stoppedCount} 
        totalCount={vendors.length} 
    />
    
    <VendorGrid 
        vendors={filteredVendors} 
        {vendorsState} 
        tiers={data.tiers || []} 
    />
</div>

<PendingListingsQueue 
    {pendingListings} 
    {vendorsState} 
/>

<VendorActionModal {vendorsState} />
<VendorChatDrawer {vendorsState} />
