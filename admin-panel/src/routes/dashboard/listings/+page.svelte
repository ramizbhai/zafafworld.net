<script lang="ts">
    import { enhance } from '$app/forms';
    import { invalidateAll } from '$app/navigation';
    import { fade } from 'svelte/transition';
    import { Check, X, Clock, AlertCircle, RefreshCw, Eye, Trash2, CheckCircle2, XCircle, Search, Filter } from 'lucide-svelte';

    interface Props {
        data: {
            listings: any[];
        };
        form?: { success?: boolean; error?: string; message?: string };
    }

    let { data, form }: Props = $props();

    // Grouping tabs
    let activeTab = $state<'pending' | 'active' | 'rejected' | 'draft' | 'archived'>('pending');

    let searchQuery = $state('');
    let selectedCategory = $state('all');
    let minPrice = $state('');
    let maxPrice = $state('');

    let listings = $derived(data.listings || []);

    // Filter by tab
    let pendingListings = $derived(listings.filter(l => l.status === 'pending_approval' || l.status === 'pending'));
    let approvedListings = $derived(listings.filter(l => l.status === 'active'));
    let rejectedListings = $derived(listings.filter(l => l.status === 'rejected'));
    let draftListings = $derived(listings.filter(l => l.status === 'draft'));
    let archivedListings = $derived(listings.filter(l => l.status === 'archived' || l.status === 'suspended'));

    // Counts
    let pendingCount = $derived(pendingListings.length);
    let approvedCount = $derived(approvedListings.length);
    let rejectedCount = $derived(rejectedListings.length);
    let draftCount = $derived(draftListings.length);
    let archivedCount = $derived(archivedListings.length);

    // List of categories (dynamic)
    let categories = $derived(['all', ...new Set(listings.map(l => l.product_category).filter(Boolean))]);

    // Apply filters
    let currentList = $derived.by(() => {
        let items = activeTab === 'pending' ? pendingListings : 
                    (activeTab === 'active' ? approvedListings : 
                    (activeTab === 'rejected' ? rejectedListings : 
                    (activeTab === 'draft' ? draftListings : archivedListings)));

        return items.filter(l => {
            // Search match
            const q = searchQuery.toLowerCase();
            const matchSearch = !searchQuery || 
                (l.title || '').toLowerCase().includes(q) ||
                (l.title_en || '').toLowerCase().includes(q) ||
                (l.title_ar || '').toLowerCase().includes(q) ||
                (l.vendor_name_en || '').toLowerCase().includes(q) ||
                (l.vendor_name_ar || '').toLowerCase().includes(q);

            // Category match
            const matchCategory = selectedCategory === 'all' || l.product_category === selectedCategory;

            // Price match
            const price = parseFloat(l.base_price_sar) || 0;
            const min = parseFloat(minPrice) || 0;
            const max = parseFloat(maxPrice) || Infinity;
            const matchPrice = (!minPrice || price >= min) && (!maxPrice || price <= max);

            return matchSearch && matchCategory && matchPrice;
        });
    });

    // Modals
    let showRejectModal = $state(false);
    let rejectListing = $state<any>(null);
    let rejectReason = $state('');

    function openRejectModal(listing: any) {
        rejectListing = listing;
        rejectReason = '';
        showRejectModal = true;
    }

    function closeModals() {
        showRejectModal = false;
        rejectListing = null;
    }
</script>

<div class="page-container">
    <div class="page-header">
        <div class="header-left">
            <div class="header-icon">
                <Filter size={22} />
            </div>
            <div>
                <h1 class="page-title">Ads & Listings Moderation</h1>
                <p class="page-subtitle">Review, approve, and manage vendor-submitted listings and services</p>
            </div>
        </div>
        <div class="header-actions">
            <button class="btn btn-outline" onclick={() => invalidateAll()}>
                <RefreshCw size={16} />
                Refresh
            </button>
        </div>
    </div>

    <!-- TABS BAR -->
    <div class="tabs">
        <button class="tab" class:active={activeTab === 'pending'} onclick={() => activeTab = 'pending'}>
            Pending Approval <span class="tab-badge warning">{pendingCount}</span>
        </button>
        <button class="tab" class:active={activeTab === 'active'} onclick={() => activeTab = 'active'}>
            Approved / Active <span class="tab-badge success">{approvedCount}</span>
        </button>
        <button class="tab" class:active={activeTab === 'rejected'} onclick={() => activeTab = 'rejected'}>
            Rejected <span class="tab-badge danger">{rejectedCount}</span>
        </button>
        <button class="tab" class:active={activeTab === 'draft'} onclick={() => activeTab = 'draft'}>
            Draft <span class="tab-badge">{draftCount}</span>
        </button>
        <button class="tab" class:active={activeTab === 'archived'} onclick={() => activeTab = 'archived'}>
            Archived <span class="tab-badge">{archivedCount}</span>
        </button>
    </div>

    <!-- ADVANCED FILTER ROW -->
    <div class="filters-row">
        <!-- Search Keyword -->
        <div class="filter-item search-item">
            <label style="font-size: 11px; font-weight: 700; color: var(--text-ghost); text-transform: uppercase;">Search Listings</label>
            <div style="position: relative; display: flex; align-items: center;">
                <Search size={14} style="position: absolute; left: 12px; color: var(--text-ghost);" />
                <input 
                    type="search" 
                    placeholder="Search by title or vendor brand..." 
                    bind:value={searchQuery} 
                    style="width: 100%; height: 38px; padding: 8px 12px 8px 36px; border-radius: 8px; border: 1px solid var(--glass-border); background: var(--bg-surface); color: var(--text-primary); font-size: 13px; outline: none;"
                />
            </div>
        </div>

        <!-- Category Dropdown -->
        <div class="filter-item">
            <label for="cat_sel" style="font-size: 11px; font-weight: 700; color: var(--text-ghost); text-transform: uppercase;">Category</label>
            <select id="cat_sel" bind:value={selectedCategory} style="width: 100%; height: 38px; padding: 0 12px; border-radius: 8px; border: 1px solid var(--glass-border); background: var(--bg-surface); color: var(--text-primary); font-size: 13px; outline: none; cursor: pointer;">
                {#each categories as cat}
                    <option value={cat}>{cat === 'all' ? 'All Categories' : cat}</option>
                {/each}
            </select>
        </div>

        <!-- Price Range Min -->
        <div class="filter-item" style="max-width: 130px;">
            <label for="min_pr" style="font-size: 11px; font-weight: 700; color: var(--text-ghost); text-transform: uppercase;">Min Price (SAR)</label>
            <input id="min_pr" type="number" placeholder="0" bind:value={minPrice} style="width: 100%; height: 38px; padding: 0 12px; border-radius: 8px; border: 1px solid var(--glass-border); background: var(--bg-surface); color: var(--text-primary); font-size: 13px; outline: none;" />
        </div>

        <!-- Price Range Max -->
        <div class="filter-item" style="max-width: 130px;">
            <label for="max_pr" style="font-size: 11px; font-weight: 700; color: var(--text-ghost); text-transform: uppercase;">Max Price (SAR)</label>
            <input id="max_pr" type="number" placeholder="Any" bind:value={maxPrice} style="width: 100%; height: 38px; padding: 0 12px; border-radius: 8px; border: 1px solid var(--glass-border); background: var(--bg-surface); color: var(--text-primary); font-size: 13px; outline: none;" />
        </div>
    </div>

    <!-- ERROR/SUCCESS BANNERS -->
    {#if form?.error}
        <div class="notice-banner error">
            <AlertCircle size={18} />
            <span>{form.error}</span>
        </div>
    {/if}
    {#if form?.success}
        <div class="notice-banner success">
            <CheckCircle2 size={18} />
            <span>Operation completed successfully</span>
        </div>
    {/if}

    <!-- DATA TABLE -->
    <div class="tab-content-panel">
        {#if currentList.length === 0}
            <div class="empty-state">
                <AlertCircle size={32} />
                <h3>No listings found</h3>
                <p>No advertisements match the selected status and active filter criteria.</p>
            </div>
        {:else}
            <div class="table-container">
                <table>
                    <thead>
                        <tr>
                            <th>Listing Title / Service</th>
                            <th>Vendor Brand</th>
                            <th>Category</th>
                            <th>Base Price</th>
                            <th style="text-align:center">Moderation Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each currentList as l (l.id)}
                            <tr>
                                <td>
                                    <div class="listing-title">{l.title_en || l.title}</div>
                                    {#if l.title_ar}
                                        <div class="listing-title-ar">{l.title_ar}</div>
                                    {/if}
                                    <div class="listing-date">{new Date(l.created_at).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })}</div>
                                </td>
                                <td>
                                    <div class="vendor-name">{l.vendor_name_en}</div>
                                    <div class="vendor-email">{l.vendor_email || 'No email contact'}</div>
                                </td>
                                <td>
                                    <span class="badge badge-muted">{l.product_category}</span>
                                </td>
                                <td class="price-cell">
                                    {l.base_price_sar ? `${parseFloat(l.base_price_sar).toLocaleString()} SAR` : 'N/A'}
                                </td>
                                <td>
                                    <div class="actions-cell">
                                        {#if l.status !== 'active'}
                                            <form method="POST" action="?/approve" use:enhance={() => {
                                                return async ({ update }) => {
                                                    await invalidateAll();
                                                    update();
                                                };
                                            }}>
                                                <input type="hidden" name="vendor_id" value={l.vendor_id} />
                                                <input type="hidden" name="product_id" value={l.id} />
                                                <button type="submit" class="btn btn-sm btn-success">
                                                    <Check size={12} /> Approve
                                                </button>
                                            </form>
                                        {/if}

                                        {#if l.status !== 'rejected'}
                                            <button class="btn btn-sm btn-outline danger" onclick={() => openRejectModal(l)}>
                                                <X size={12} /> Reject
                                            </button>
                                        {/if}
                                    </div>
                                </td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        {/if}
    </div>
</div>

<!-- REJECT REASON MODAL -->
{#if showRejectModal && rejectListing}
    <div class="modal-backdrop" onclick={closeModals}>
        <div class="modal-card" onclick={(e) => e.stopPropagation()}>
            <div class="modal-header">
                <h2>Reject Listing</h2>
                <button class="btn-icon" onclick={closeModals}><X size={18} /></button>
            </div>
            <form method="POST" action="?/reject" use:enhance={() => {
                return async ({ update }) => {
                    closeModals();
                    await invalidateAll();
                    update();
                };
            }}>
                <input type="hidden" name="vendor_id" value={rejectListing.vendor_id} />
                <input type="hidden" name="product_id" value={rejectListing.id} />
                <div class="form-group full-width" style="margin-bottom:20px;">
                    <label for="reject_reason_text">Reason for Rejection * (Required)</label>
                    <textarea id="reject_reason_text" name="reason" required bind:value={rejectReason} rows="3" class="form-textarea" placeholder="Please specify the reasons or missing guidelines..."></textarea>
                </div>
                <div class="modal-actions">
                    <button type="button" class="btn btn-outline" onclick={closeModals}>Cancel</button>
                    <button type="submit" class="btn btn-danger">Confirm Rejection</button>
                </div>
            </form>
        </div>
    </div>
{/if}

<style>
    .page-container { padding: 24px; display: flex; flex-direction: column; gap: 20px; }
    .page-header { display: flex; justify-content: space-between; align-items: center; }
    .header-left { display: flex; align-items: center; gap: 12px; }
    .header-icon { background: var(--bg-elevated); padding: 10px; border-radius: 8px; color: var(--gold); border: 1px solid var(--glass-border); }
    .page-title { font-size: 22px; font-weight: 750; margin: 0; color: var(--text-primary); }
    .page-subtitle { font-size: 13px; color: var(--text-ghost); margin-top: 2px; }

    .tabs { display: flex; gap: 8px; border-bottom: 1px solid var(--glass-border); padding-bottom: 2px; }
    .tab { background: transparent; border: none; padding: 10px 16px; font-size: 13.5px; font-weight: 600; color: var(--text-ghost); cursor: pointer; display: flex; align-items: center; gap: 8px; position: relative; border-radius: 6px 6px 0 0; transition: all 0.2s; }
    .tab:hover { color: var(--text-primary); background: var(--bg-elevated); }
    .tab.active { color: var(--gold); background: var(--bg-surface); border-bottom: 2px solid var(--gold); }
    
    .tab-badge { font-size: 10.5px; font-weight: 700; padding: 2px 8px; border-radius: 99px; background: var(--bg-elevated); color: var(--text-secondary); }
    .tab-badge.warning { background: var(--warning-dim); color: var(--warning); }
    .tab-badge.success { background: var(--success-dim); color: var(--success); }
    .tab-badge.danger { background: var(--danger-dim); color: var(--danger); }

    .filters-row { display: flex; flex-wrap: wrap; gap: 16px; align-items: flex-end; padding: 16px; background: var(--bg-elevated); border: 1px solid var(--glass-border); border-radius: 12px; }
    .filter-item { display: flex; flex-direction: column; gap: 6px; flex: 1; min-width: 160px; }
    .search-item { flex: 2; min-width: 260px; }

    .notice-banner { display: flex; align-items: center; gap: 12px; padding: 12px 16px; border-radius: 8px; font-size: 13.5px; font-weight: 600; }
    .notice-banner.error { background: var(--danger-dim); color: var(--danger); border: 1px solid var(--danger-border); }
    .notice-banner.success { background: var(--success-dim); color: var(--success); border: 1px solid var(--success-border); }

    .table-container { background: var(--bg-elevated); border: 1px solid var(--glass-border); border-radius: 10px; overflow: hidden; margin-top: 10px; }
    table { width: 100%; border-collapse: collapse; text-align: left; }
    th { padding: 12px 16px; font-size: 11px; font-weight: 700; text-transform: uppercase; color: var(--text-ghost); border-bottom: 1px solid var(--glass-border); background: var(--bg-surface); }
    td { padding: 14px 16px; border-bottom: 1px solid var(--glass-border); font-size: 13px; vertical-align: middle; }
    tr:hover { background: var(--bg-surface); }

    .listing-title { font-weight: 700; color: var(--text-primary); }
    .listing-title-ar { font-size: 12px; color: var(--text-secondary); margin-top: 2px; direction: rtl; }
    .listing-date { font-size: 11.5px; color: var(--text-ghost); margin-top: 4px; }
    .vendor-name { font-weight: 700; color: var(--text-secondary); }
    .vendor-email { font-size: 11.5px; color: var(--text-ghost); margin-top: 1px; }

    .price-cell { font-weight: 700; color: var(--gold); }
    .actions-cell { display: flex; gap: 8px; justify-content: flex-start; }

    .empty-state { text-align: center; padding: 60px 20px; display: flex; flex-direction: column; align-items: center; gap: 10px; color: var(--text-ghost); }
    .empty-state h3 { font-size: 15px; font-weight: 700; color: var(--text-primary); margin: 0; }
    .empty-state p { font-size: 13px; margin: 0; }

    .btn { display: inline-flex; align-items: center; gap: 6px; padding: 6px 12px; border-radius: 6px; font-size: 12px; font-weight: 700; border: none; cursor: pointer; transition: all 0.2s; text-decoration: none; }
    .btn-gold { background: var(--gold); color: var(--bg-surface); }
    .btn-gold:hover { opacity: 0.9; }
    .btn-outline { background: transparent; border: 1px solid var(--glass-border); color: var(--text-secondary); }
    .btn-outline:hover { background: var(--bg-elevated); color: var(--text-primary); }
    .btn-outline.danger { color: var(--danger); border-color: var(--danger-border); }
    .btn-outline.danger:hover { background: var(--danger-dim); }
    .btn-success { background: var(--success); color: white; }
    .btn-success:hover { opacity: 0.9; }
    .btn-danger { background: var(--danger); color: white; }
    .btn-danger:hover { opacity: 0.9; }

    .badge { display: inline-block; padding: 2px 8px; border-radius: 4px; font-size: 11px; font-weight: 600; }
    .badge-muted { background: var(--bg-surface); color: var(--text-secondary); border: 1px solid var(--glass-border); }

    .modal-backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.6); backdrop-filter: blur(4px); display: flex; align-items: center; justify-content: center; z-index: 1000; }
    .modal-card { background: var(--bg-elevated); border: 1px solid var(--glass-border); border-radius: 12px; width: 100%; max-width: 500px; padding: 24px; }
    .modal-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
    .modal-header h2 { font-size: 16px; font-weight: 750; margin: 0; }
    .form-group { display: flex; flex-direction: column; gap: 6px; }
    .form-group label { font-size: 12px; font-weight: 600; color: var(--text-secondary); }
    .form-textarea { padding: 8px 12px; border-radius: 6px; border: 1px solid var(--glass-border); background: var(--bg-surface); color: var(--text-primary); outline: none; font-size: 13px; }
    .modal-actions { display: flex; justify-content: flex-end; gap: 12px; margin-top: 24px; }
</style>
