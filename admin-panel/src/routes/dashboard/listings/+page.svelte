<script lang="ts">
    import { enhance } from '$app/forms';
    import { invalidate } from '$app/navigation';
    import { CheckCircle2, XCircle, Clock, Search, LayoutGrid, CheckSquare } from 'lucide-svelte';

    let { data } = $props();

    let searchQuery = $state('');
    let selectedStatus = $state('all');
    let processingId = $state<string | null>(null);
    let rejectingId = $state<string | null>(null);

    let listings = $derived(data.listings || []);

    let filteredListings = $derived(
        listings.filter((listing: any) => {
            const matchSearch = (listing.title || '').toLowerCase().includes(searchQuery.toLowerCase()) ||
                (listing.title_en || '').toLowerCase().includes(searchQuery.toLowerCase()) ||
                (listing.title_ar || '').toLowerCase().includes(searchQuery.toLowerCase()) ||
                (listing.vendor_name_en || '').toLowerCase().includes(searchQuery.toLowerCase()) ||
                (listing.vendor_name_ar || '').toLowerCase().includes(searchQuery.toLowerCase());
            const matchStatus = selectedStatus === 'all' || listing.status === selectedStatus;
            return matchSearch && matchStatus;
        })
    );

    const statusMap: Record<string, any> = {
        'active': { color: 'var(--success)', bg: 'var(--success-dim)', label: 'Active', icon: CheckCircle2 },
        'draft': { color: 'var(--text-ghost)', bg: 'var(--bg-elevated)', label: 'Draft', icon: Clock },
        'pending_approval': { color: 'var(--warning)', bg: 'var(--warning-dim)', label: 'Pending', icon: Clock },
        'suspended': { color: 'var(--danger)', bg: 'var(--danger-dim)', label: 'Suspended', icon: XCircle },
        'rejected': { color: 'var(--danger)', bg: 'var(--danger-dim)', label: 'Rejected', icon: XCircle },
        'archived': { color: 'var(--text-ghost)', bg: 'var(--bg-elevated)', label: 'Archived', icon: CheckSquare }
    };
</script>

<div class="listings-page">
    <header class="page-header">
        <div class="header-left">
            <h1 class="page-title">Ads Management</h1>
            <p class="page-subtitle">Review, approve, and manage vendor listings.</p>
        </div>
    </header>

    <div class="controls-bar">
        <div class="search-box">
            <Search size={16} />
            <input type="text" bind:value={searchQuery} placeholder="Search by Ad Title or Vendor..." />
        </div>
        
        <div class="filter-group">
            <select bind:value={selectedStatus} class="status-select">
                <option value="all">All Statuses</option>
                <option value="active">Active</option>
                <option value="pending_approval">Pending Approval</option>
                <option value="draft">Draft</option>
                <option value="suspended">Suspended</option>
                <option value="rejected">Rejected</option>
            </select>
        </div>
    </div>

    <div class="table-container">
        <table class="listings-table">
            <thead>
                <tr>
                    <th>Ad Title</th>
                    <th>Vendor</th>
                    <th>Category</th>
                    <th>Price</th>
                    <th>Status</th>
                    <th>Actions</th>
                </tr>
            </thead>
            <tbody>
                {#each filteredListings as listing (listing.id)}
                    {@const statusInfo = statusMap[listing.status] || statusMap.draft}
                    <tr>
                        <td>
                            <div class="listing-title">{listing.title_en || listing.title}</div>
                            {#if listing.title_ar}
                                <div class="listing-title-ar">{listing.title_ar}</div>
                            {/if}
                            <div class="listing-date">{new Date(listing.created_at).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })}</div>
                        </td>
                        <td>
                            <div class="vendor-name">{listing.vendor_name_en}</div>
                            <div class="vendor-email">{listing.vendor_email || listing.vendor_phone || 'No contact'}</div>
                        </td>
                        <td>
                            <span class="category-badge">{listing.product_category}</span>
                        </td>
                        <td>
                            <div class="price">{listing.base_price_sar ? `SAR ${listing.base_price_sar}` : 'N/A'}</div>
                        </td>
                        <td>
                            <span class="status-badge" style="background: {statusInfo.bg}; color: {statusInfo.color}">
                                <statusInfo.icon size={12} />
                                {statusInfo.label}
                            </span>
                        </td>
                        <td>
                            <div class="actions">
                                {#if listing.status !== 'active'}
                                    <form method="POST" action="?/approve" use:enhance={() => {
                                        processingId = listing.id;
                                        return async ({ update }) => {
                                            await invalidate('app:admin-listings');
                                            processingId = null;
                                            update();
                                        };
                                    }}>
                                        <input type="hidden" name="vendor_id" value={listing.vendor_id} />
                                        <input type="hidden" name="product_id" value={listing.id} />
                                        <button type="submit" class="btn btn-approve" disabled={processingId === listing.id}>
                                            {#if processingId === listing.id}
                                                ...
                                            {:else}
                                                <CheckCircle2 size={14} /> Approve
                                            {/if}
                                        </button>
                                    </form>
                                {/if}

                                {#if listing.status !== 'rejected'}
                                    {#if rejectingId === listing.id}
                                        <form method="POST" action="?/reject" use:enhance={() => {
                                            processingId = listing.id;
                                            return async ({ update }) => {
                                                await invalidate('app:admin-listings');
                                                processingId = null;
                                                rejectingId = null;
                                                update();
                                            };
                                        }}>
                                            <input type="hidden" name="vendor_id" value={listing.vendor_id} />
                                            <input type="hidden" name="product_id" value={listing.id} />
                                            <textarea name="reason" required placeholder="Rejection reason..." rows="2"
                                                class="reject-reason-input"></textarea>
                                            <div class="reject-actions">
                                                <button type="submit" class="btn btn-reject"
                                                    disabled={processingId === listing.id}>
                                                    <XCircle size={14} /> Confirm
                                                </button>
                                                <button type="button" class="btn btn-cancel"
                                                    onclick={() => rejectingId = null}>
                                                    Cancel
                                                </button>
                                            </div>
                                        </form>
                                    {:else}
                                        <button type="button" class="btn btn-reject"
                                            onclick={() => rejectingId = listing.id}
                                            disabled={processingId === listing.id}>
                                            <XCircle size={14} /> Reject
                                        </button>
                                    {/if}
                                {/if}
                            </div>
                        </td>
                    </tr>
                {/each}
                {#if filteredListings.length === 0}
                    <tr>
                        <td colspan="6" class="empty-state">No listings found matching your criteria.</td>
                    </tr>
                {/if}
            </tbody>
        </table>
    </div>
</div>

<style>
    .listings-page { padding: 24px; display: flex; flex-direction: column; gap: 24px; }
    .page-header { display: flex; justify-content: space-between; align-items: flex-start; }
    .page-title { font-size: 24px; font-weight: 700; color: var(--text-primary); margin: 0; }
    .page-subtitle { font-size: 14px; color: var(--text-secondary); margin: 4px 0 0 0; }

    .controls-bar { display: flex; gap: 16px; align-items: center; }
    .search-box {
        display: flex; align-items: center; gap: 8px; flex: 1; max-width: 400px;
        background: var(--bg-elevated); border: 1px solid var(--glass-border);
        border-radius: 8px; padding: 0 12px; height: 40px;
    }
    .search-box input { border: none; background: transparent; color: var(--text-primary); outline: none; flex: 1; }
    .status-select {
        height: 40px; padding: 0 12px; border-radius: 8px;
        background: var(--bg-elevated); border: 1px solid var(--glass-border);
        color: var(--text-primary); outline: none;
    }

    .table-container { background: var(--bg-elevated); border-radius: 12px; border: 1px solid var(--glass-border); overflow: hidden; }
    .listings-table { width: 100%; border-collapse: collapse; text-align: left; }
    .listings-table th { padding: 12px 16px; border-bottom: 1px solid var(--glass-border); font-size: 12px; text-transform: uppercase; color: var(--text-ghost); font-weight: 600; }
    .listings-table td { padding: 16px; border-bottom: 1px solid var(--glass-border); vertical-align: middle; }
    
    .listing-title { font-weight: 600; font-size: 14px; color: var(--text-primary); }
    .listing-title-ar { font-size: 12px; color: var(--text-secondary); margin-top: 2px; direction: rtl; }
    .listing-date { font-size: 12px; color: var(--text-tertiary); margin-top: 4px; }
    
    .vendor-name { font-weight: 500; font-size: 13px; color: var(--text-secondary); }
    .vendor-email { font-size: 12px; color: var(--text-tertiary); margin-top: 2px; }

    .category-badge { display: inline-block; padding: 4px 8px; border-radius: 6px; background: var(--bg-float); border: 1px solid var(--glass-border); font-size: 11px; font-weight: 500; color: var(--text-secondary); }
    .price { font-weight: 600; font-size: 13px; color: var(--text-primary); }

    .status-badge { display: inline-flex; align-items: center; gap: 4px; padding: 4px 8px; border-radius: 6px; font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.5px; }

    .actions { display: flex; gap: 8px; }
    .btn { display: inline-flex; align-items: center; gap: 6px; padding: 6px 12px; border-radius: 6px; font-size: 12px; font-weight: 600; cursor: pointer; border: none; transition: all 0.2s; }
    .btn-approve { background: var(--success); color: white; }
    .btn-approve:hover:not(:disabled) { background: hsl(142, 69%, 45%); }
    .btn-reject { background: var(--danger); color: white; }
    .btn-reject:hover:not(:disabled) { background: hsl(0, 72%, 45%); }
    .btn-cancel { background: var(--bg-elevated); color: var(--text-secondary); border: 1px solid var(--glass-border); }
    .reject-reason-input { width: 100%; border-radius: 6px; border: 1px solid var(--glass-border); background: var(--bg-elevated); color: var(--text-primary); padding: 6px 8px; font-size: 12px; resize: none; margin-top: 6px; }
    .reject-actions { display: flex; gap: 6px; margin-top: 6px; }
    .btn:disabled { opacity: 0.5; cursor: not-allowed; }
    
    .empty-state { text-align: center; padding: 48px; color: var(--text-ghost); font-size: 14px; }
</style>
