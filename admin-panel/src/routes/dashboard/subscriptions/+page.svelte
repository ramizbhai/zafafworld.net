<script lang="ts">
    import { enhance } from '$app/forms';
    import { invalidateAll } from '$app/navigation';
    import { fade } from 'svelte/transition';
    import { Check, X, Clock, AlertCircle, RefreshCw, Crown, Info, Trash2, Edit2, ExternalLink } from 'lucide-svelte';

    interface Props {
        data: {
            requests: any[];
            vendors: any[];
            tiers: any[];
        };
        form?: { success?: boolean; error?: string; message?: string };
    }

    let { data, form }: Props = $props();

    // Grouping tabs
    let activeTab = $state<'requests' | 'Free' | 'Gold' | 'VIP' | 'Diamond'>('requests');

    let requests = $derived(data.requests || []);
    let vendors = $derived(data.vendors || []);
    let tiers = $derived(data.tiers || []);

    // Filtered lists
    let pendingRequests = $derived(requests.filter(r => r.status === 'pending'));
    
    let freeVendors = $derived(vendors.filter(v => v.current_tier === 'Free'));
    let goldenVendors = $derived(vendors.filter(v => v.current_tier === 'Gold' || v.current_tier === 'Golden'));
    let vipVendors = $derived(vendors.filter(v => v.current_tier === 'VIP'));
    let diamondVendors = $derived(vendors.filter(v => v.current_tier === 'Diamond'));

    // Count badges
    let pendingCount = $derived(pendingRequests.length);
    let freeCount = $derived(freeVendors.length);
    let goldenCount = $derived(goldenVendors.length);
    let vipCount = $derived(vipVendors.length);
    let diamondCount = $derived(diamondVendors.length);

    // Modal forms
    let showActionModal = $state(false);
    let selectedVendor = $state<any>(null);
    let formTierId = $state('');
    let formExpiresAt = $state('');
    let isRemoving = $state(false);

    let promptReqAction = $state<{ action: 'approve' | 'reject', id: string } | null>(null);
    let adminNotes = $state('');

    function openUpgradeModal(vendor: any) {
        selectedVendor = vendor;
        isRemoving = false;
        formTierId = vendor.subscription_tier_id || '';
        formExpiresAt = vendor.subscription_expires_at ? vendor.subscription_expires_at.slice(0, 10) : '';
        showActionModal = true;
    }

    function openRemoveModal(vendor: any) {
        selectedVendor = vendor;
        isRemoving = true;
        showActionModal = true;
    }

    function closeModals() {
        showActionModal = false;
        promptReqAction = null;
        selectedVendor = null;
    }

    function getFreeTierId() {
        const freeTier = tiers.find(t => t.name === 'Free');
        return freeTier ? freeTier.id : '';
    }

    function formatDate(dt: string) {
        if (!dt) return 'Never / Unlimited';
        return new Date(dt).toLocaleDateString('en-US', {
            year: 'numeric',
            month: 'short',
            day: 'numeric'
        });
    }
</script>

<div class="page-container">
    <div class="page-header">
        <div class="header-left">
            <div class="header-icon">
                <Crown size={22} />
            </div>
            <div>
                <h1 class="page-title">Subscriptions Management</h1>
                <p class="page-subtitle">Oversee vendor plans, upgrade requests, and active tier assignments</p>
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
        <button class="tab" class:active={activeTab === 'requests'} onclick={() => activeTab = 'requests'}>
            Requests <span class="tab-badge warning">{pendingCount}</span>
        </button>
        <button class="tab" class:active={activeTab === 'Free'} onclick={() => activeTab = 'Free'}>
            Free <span class="tab-badge">{freeCount}</span>
        </button>
        <button class="tab" class:active={activeTab === 'Gold'} onclick={() => activeTab = 'Gold'}>
            Golden <span class="tab-badge gold">{goldenCount}</span>
        </button>
        <button class="tab" class:active={activeTab === 'VIP'} onclick={() => activeTab = 'VIP'}>
            VIP <span class="tab-badge purple">{vipCount}</span>
        </button>
        <button class="tab" class:active={activeTab === 'Diamond'} onclick={() => activeTab = 'Diamond'}>
            Diamond <span class="tab-badge cyan">{diamondCount}</span>
        </button>
    </div>

    <!-- ACTIONS NOTICE -->
    {#if form?.error}
        <div class="notice-banner error">
            <AlertCircle size={18} />
            <span>{form.error}</span>
        </div>
    {/if}
    {#if form?.success}
        <div class="notice-banner success">
            <Check size={18} />
            <span>{form.message || 'Operation successful'}</span>
        </div>
    {/if}

    <!-- TAB CONTENTS -->
    <div class="tab-content-panel">
        {#if activeTab === 'requests'}
            {#if pendingRequests.length === 0}
                <div class="empty-state">
                    <Info size={32} />
                    <h3>No pending requests</h3>
                    <p>Vendors have not submitted any plan upgrade requests.</p>
                </div>
            {:else}
                <div class="table-container">
                    <table>
                        <thead>
                            <tr>
                                <th>Vendor Brand</th>
                                <th>Requested Tier</th>
                                <th>Submitted Date</th>
                                <th style="text-align:center">Moderation</th>
                            </tr>
                        </thead>
                        <tbody>
                            {#each pendingRequests as r (r.id)}
                                <tr>
                                    <td>
                                        <div class="vendor-name">{r.vendor_name || r.vendor_name_en || 'Unknown'}</div>
                                        <div class="vendor-email">{r.vendor_email || ''}</div>
                                    </td>
                                    <td>
                                        <span class="badge badge-gold font-bold">{r.requested_tier_name}</span>
                                    </td>
                                    <td class="text-muted">{formatDate(r.created_at)}</td>
                                    <td>
                                        <div class="actions-cell">
                                            <button class="btn btn-sm btn-success" onclick={() => promptReqAction = { action: 'approve', id: r.id }}>
                                                <Check size={12} /> Approve
                                            </button>
                                            <button class="btn btn-sm btn-outline danger" onclick={() => promptReqAction = { action: 'reject', id: r.id }}>
                                                <X size={12} /> Reject
                                            </button>
                                        </div>
                                    </td>
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                </div>
            {/if}
        {:else}
            <!-- PLANS VENDORS LIST -->
            {@const currentVendors = activeTab === 'Free' ? freeVendors : (activeTab === 'Gold' ? goldenVendors : (activeTab === 'VIP' ? vipVendors : diamondVendors))}
            {#if currentVendors.length === 0}
                <div class="empty-state">
                    <Info size={32} />
                    <h3>No active vendors</h3>
                    <p>No vendors are currently assigned to the {activeTab} tier plan.</p>
                </div>
            {:else}
                <div class="table-container">
                    <table>
                        <thead>
                            <tr>
                                <th>Vendor</th>
                                <th>City / Area</th>
                                <th>Subscription Expiry</th>
                                <th>Status</th>
                                <th style="text-align:center">Actions</th>
                            </tr>
                        </thead>
                        <tbody>
                            {#each currentVendors as v (v.id)}
                                <tr>
                                    <td>
                                        <div class="vendor-name">{v.name_en}</div>
                                        {#if v.name_ar}
                                            <div class="vendor-name-ar" style="font-size:12px; color:var(--text-ghost)">{v.name_ar}</div>
                                        {/if}
                                        <div class="vendor-email">{v.email || 'No email'}</div>
                                    </td>
                                    <td>
                                        <span class="badge badge-muted">{v.city_name_en}</span>
                                    </td>
                                    <td class="text-muted font-mono">{formatDate(v.subscription_expires_at)}</td>
                                    <td>
                                        <span class="badge" style="text-transform:uppercase; font-size:11px; font-weight:700; background: {v.subscription_status === 'active' ? 'var(--success-dim)' : 'var(--danger-dim)'}; color: {v.subscription_status === 'active' ? 'var(--success)' : 'var(--danger)'}">
                                            {v.subscription_status}
                                        </span>
                                    </td>
                                    <td>
                                        <div class="actions-cell">
                                            <a href="/dashboard/vendors/{v.id}" class="btn btn-sm btn-outline">
                                                <ExternalLink size={12} /> View
                                            </a>
                                            <button class="btn btn-sm btn-outline" onclick={() => openUpgradeModal(v)}>
                                                <Edit2 size={12} /> Change Plan
                                            </button>
                                            {#if activeTab !== 'Free'}
                                                <button class="btn btn-sm btn-outline danger" onclick={() => openRemoveModal(v)}>
                                                    <Trash2 size={12} /> Remove
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
        {/if}
    </div>
</div>

<!-- UPGRADE / ASSIGN PLAN MODAL -->
{#if showActionModal && selectedVendor}
    <div class="modal-backdrop" onclick={closeModals}>
        <div class="modal-card" onclick={(e) => e.stopPropagation()}>
            <div class="modal-header">
                <h2>{isRemoving ? 'Remove Subscription Plan' : 'Assign / Change Plan'}</h2>
                <button class="btn-icon" onclick={closeModals}><X size={18} /></button>
            </div>
            
            {#if isRemoving}
                <form method="POST" action="?/removeSubscription" use:enhance={() => {
                    return async ({ update }) => {
                        closeModals();
                        await invalidateAll();
                        update();
                    };
                }}>
                    <input type="hidden" name="id" value={selectedVendor.id} />
                    <input type="hidden" name="free_tier_id" value={getFreeTierId()} />
                    <p style="margin-bottom:20px; font-size:14px; line-height:1.6;">
                        Are you sure you want to remove the paid subscription for <strong>{selectedVendor.name_en}</strong>? 
                        The vendor account status will be reset back to the <strong>Free Plan</strong> instantly.
                    </p>
                    <div class="modal-actions">
                        <button type="button" class="btn btn-outline" onclick={closeModals}>Cancel</button>
                        <button type="submit" class="btn btn-danger">Confirm & Downgrade</button>
                    </div>
                </form>
            {:else}
                <form method="POST" action="?/assignSubscription" use:enhance={() => {
                    return async ({ update }) => {
                        closeModals();
                        await invalidateAll();
                        update();
                    };
                }}>
                    <input type="hidden" name="id" value={selectedVendor.id} />
                    <div class="form-grid">
                        <div class="form-group full-width">
                            <label>Vendor Account</label>
                            <input type="text" readonly disabled value={selectedVendor.name_en} class="form-input disabled" />
                        </div>
                        <div class="form-group">
                            <label for="sub_tier_select">Subscription Tier Plan *</label>
                            <select id="sub_tier_select" name="subscription_tier_id" required bind:value={formTierId} class="form-select">
                                {#each tiers as t}
                                    <option value={t.id}>{t.name} ({t.price} SAR/yr)</option>
                                {/each}
                            </select>
                        </div>
                        <div class="form-group">
                            <label for="sub_expiry_date">Expiration Date</label>
                            <input id="sub_expiry_date" type="date" name="expires_at" bind:value={formExpiresAt} class="form-input" />
                        </div>
                    </div>
                    <div class="modal-actions">
                        <button type="button" class="btn btn-outline" onclick={closeModals}>Cancel</button>
                        <button type="submit" class="btn btn-gold">Update Plan</button>
                    </div>
                </form>
            {/if}
        </div>
    </div>
{/if}

<!-- REQUEST ACTION MODAL -->
{#if promptReqAction}
    <div class="modal-backdrop" onclick={closeModals}>
        <div class="modal-card" onclick={(e) => e.stopPropagation()}>
            <div class="modal-header">
                <h2>{promptReqAction.action === 'approve' ? 'Approve Subscription Request' : 'Reject Subscription Request'}</h2>
                <button class="btn-icon" onclick={closeModals}><X size={18} /></button>
            </div>
            <form method="POST" action={promptReqAction.action === 'approve' ? '?/approve' : '?/reject'} use:enhance={() => {
                return async ({ update }) => {
                    closeModals();
                    await invalidateAll();
                    update();
                };
            }}>
                <input type="hidden" name="id" value={promptReqAction.id} />
                <div class="form-group full-width" style="margin-bottom:20px;">
                    <label for="sub_admin_notes">Administrative Notes / Message to Vendor</label>
                    <textarea id="sub_admin_notes" name="admin_notes" bind:value={adminNotes} rows="3" class="form-textarea" placeholder="Provide notes or feedback regarding this tier request..."></textarea>
                </div>
                <div class="modal-actions">
                    <button type="button" class="btn btn-outline" onclick={closeModals}>Cancel</button>
                    <button type="submit" class="btn {promptReqAction.action === 'approve' ? 'btn-success' : 'btn-danger'}">
                        {promptReqAction.action === 'approve' ? 'Approve & Activate' : 'Reject Request'}
                    </button>
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
    .tab-badge.gold { background: var(--warning-dim); color: var(--gold); }
    .tab-badge.purple { background: rgba(124, 58, 237, 0.15); color: rgb(167, 139, 250); }
    .tab-badge.cyan { background: rgba(6, 182, 212, 0.15); color: rgb(103, 232, 249); }

    .notice-banner { display: flex; align-items: center; gap: 12px; padding: 12px 16px; border-radius: 8px; font-size: 13.5px; font-weight: 600; }
    .notice-banner.error { background: var(--danger-dim); color: var(--danger); border: 1px solid var(--danger-border); }
    .notice-banner.success { background: var(--success-dim); color: var(--success); border: 1px solid var(--success-border); }

    .table-container { background: var(--bg-elevated); border: 1px solid var(--glass-border); border-radius: 10px; overflow: hidden; margin-top: 10px; }
    table { width: 100%; border-collapse: collapse; text-align: left; }
    th { padding: 12px 16px; font-size: 11px; font-weight: 700; text-transform: uppercase; color: var(--text-ghost); border-bottom: 1px solid var(--glass-border); background: var(--bg-surface); }
    td { padding: 14px 16px; border-bottom: 1px solid var(--glass-border); font-size: 13px; vertical-align: middle; }
    tr:hover { background: var(--bg-surface); }

    .vendor-name { font-weight: 700; color: var(--text-primary); }
    .vendor-email { font-size: 11.5px; color: var(--text-ghost); margin-top: 1px; }
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
    .badge-gold { background: var(--warning-dim); color: var(--gold); }
    .badge-muted { background: var(--bg-surface); color: var(--text-secondary); border: 1px solid var(--glass-border); }

    .modal-backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.6); backdrop-filter: blur(4px); display: flex; align-items: center; justify-content: center; z-index: 1000; }
    .modal-card { background: var(--bg-elevated); border: 1px solid var(--glass-border); border-radius: 12px; width: 100%; max-width: 500px; padding: 24px; }
    .modal-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; }
    .modal-header h2 { font-size: 16px; font-weight: 750; margin: 0; }
    .form-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
    .full-width { grid-column: span 2; }
    .form-group { display: flex; flex-direction: column; gap: 6px; }
    .form-group label { font-size: 12px; font-weight: 600; color: var(--text-secondary); }
    .form-input, .form-select, .form-textarea { padding: 8px 12px; border-radius: 6px; border: 1px solid var(--glass-border); background: var(--bg-surface); color: var(--text-primary); outline: none; font-size: 13px; }
    .form-input.disabled { background: var(--bg-surface); opacity: 0.6; cursor: not-allowed; }
    .modal-actions { display: flex; justify-content: flex-end; gap: 12px; margin-top: 24px; }
</style>
