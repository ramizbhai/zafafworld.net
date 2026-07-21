<script lang="ts">
    import { enhance } from '$app/forms';
    import { invalidateAll } from '$app/navigation';
    import { fade } from 'svelte/transition';
    import { Check, X, Clock, AlertCircle, RefreshCw, Eye, Sparkles, MessageCircle, Building2, MapPin, Mail, Phone, ShieldAlert, Crown, ExternalLink, Search } from 'lucide-svelte';

    interface Props {
        data: {
            vendors: any[];
            tiers: any[];
            pendingListings: any[];
        };
        form?: { success?: boolean; error?: string; message?: string };
    }

    let { data, form }: Props = $props();

    // Grouping tabs
    let activeTab = $state<'pending' | 'active' | 'suspended' | 'banned' | 'all'>('pending');

    // Filter properties
    let searchQuery = $state('');
    let selectedSubTier = $state('all');
    let selectedSubStatus = $state('all');

    let vendors = $derived(data.vendors || []);
    let tiers = $derived(data.tiers || []);

    // Filtered lists for tab counts
    let pendingVendors = $derived(vendors.filter(v => v.status === 'pending'));
    let activeVendors = $derived(vendors.filter(v => v.status === 'active'));
    let suspendedVendors = $derived(vendors.filter(v => v.status === 'suspended'));
    let bannedVendors = $derived(vendors.filter(v => v.status === 'banned' || v.status === 'rejected'));

    // Count badges
    let pendingCount = $derived(pendingVendors.length);
    let activeCount = $derived(activeVendors.length);
    let suspendedCount = $derived(suspendedVendors.length);
    let bannedCount = $derived(bannedVendors.length);
    let totalCount = $derived(vendors.length);

    // Apply filters on the current active tab
    let currentVendors = $derived(
        (activeTab === 'pending' ? pendingVendors :
         activeTab === 'active' ? activeVendors :
         activeTab === 'suspended' ? suspendedVendors :
         activeTab === 'banned' ? bannedVendors : vendors).filter(v => {
            // Search text match
            const q = searchQuery.toLowerCase();
            const matchSearch = !searchQuery ||
                (v.name_en || '').toLowerCase().includes(q) ||
                (v.name_ar || '').toLowerCase().includes(q) ||
                (v.email || '').toLowerCase().includes(q) ||
                (v.phone || '').toLowerCase().includes(q) ||
                (v.category || '').toLowerCase().includes(q) ||
                (v.city_name_en || '').toLowerCase().includes(q);

            // Subscription Tier match
            const matchTier = selectedSubTier === 'all' || 
                (selectedSubTier === 'Free' && (v.current_tier || 'Free') === 'Free') ||
                (selectedSubTier === 'Gold' && ((v.current_tier || '') === 'Gold' || (v.current_tier || '') === 'Golden')) ||
                ((v.current_tier || '').toLowerCase() === selectedSubTier.toLowerCase());

            // Subscription Status match
            const matchSubStatus = selectedSubStatus === 'all' || v.subscription_status === selectedSubStatus;

            return matchSearch && matchTier && matchSubStatus;
        })
    );

    // Modals & Action States
    let showStatusModal = $state(false);
    let showSubscriptionModal = $state(false);
    let showFeaturedModal = $state(false);
    let selectedVendor = $state<any>(null);

    let statusReason = $state('');
    let statusTarget = $state<'active' | 'suspended' | 'banned'>('active');

    let formTierId = $state('');
    let formSubStatus = $state('active');
    let formExpiresAt = $state('');

    let isRemovingSubscription = $state(false);

    let adIsFeatured = $state(false);
    let adExpiry = $state('');

    function openStatusModal(vendor: any, target: 'active' | 'suspended' | 'banned') {
        selectedVendor = vendor;
        statusTarget = target;
        statusReason = '';
        showStatusModal = true;
    }

    function openSubscriptionModal(vendor: any, remove = false) {
        selectedVendor = vendor;
        isRemovingSubscription = remove;
        formTierId = vendor.subscription_tier_id || '';
        formSubStatus = vendor.subscription_status || 'active';
        formExpiresAt = vendor.subscription_expires_at ? vendor.subscription_expires_at.slice(0, 10) : '';
        showSubscriptionModal = true;
    }

    function openFeaturedModal(vendor: any) {
        selectedVendor = vendor;
        adIsFeatured = vendor.is_featured || false;
        adExpiry = vendor.featured_expires_at ? vendor.featured_expires_at.slice(0, 10) : '';
        showFeaturedModal = true;
    }

    function closeModals() {
        showStatusModal = false;
        showSubscriptionModal = false;
        showFeaturedModal = false;
        selectedVendor = null;
    }

    function getFreeTierId() {
        const freeTier = tiers.find(t => t.name === 'Free');
        return freeTier ? freeTier.id : '';
    }

    function formatDate(dt: string) {
        if (!dt) return 'Unlimited / Free';
        try {
            const parsed = new Date(dt);
            if (isNaN(parsed.getTime())) {
                return 'Unlimited / Free';
            }
            return parsed.toLocaleDateString('en-US', {
                year: 'numeric',
                month: 'short',
                day: 'numeric'
            });
        } catch (e) {
            return 'Unlimited / Free';
        }
    }
</script>

<svelte:head>
    <title>Vendors Governance | ZafafWorld Admin</title>
</svelte:head>

<div class="page-container">
    <div class="page-header">
        <div class="header-left">
            <div class="header-icon">
                <Building2 size={22} />
            </div>
            <div>
                <h1 class="page-title">Vendors Directory & Governance</h1>
                <p class="page-subtitle">Oversee vendor verification steps, plan upgrades, search prioritization, and administrative lockdowns</p>
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
            Active / Approved <span class="tab-badge success">{activeCount}</span>
        </button>
        <button class="tab" class:active={activeTab === 'suspended'} onclick={() => activeTab = 'suspended'}>
            Suspended <span class="tab-badge warning">{suspendedCount}</span>
        </button>
        <button class="tab" class:active={activeTab === 'banned'} onclick={() => activeTab = 'banned'}>
            Banned / Rejected <span class="tab-badge danger">{bannedCount}</span>
        </button>
        <button class="tab" class:active={activeTab === 'all'} onclick={() => activeTab = 'all'}>
            All Vendors <span class="tab-badge">{totalCount}</span>
        </button>
    </div>

    <!-- ADVANCED FILTER ROW -->
    <div class="filters-row">
        <!-- Search Keyword -->
        <div class="filter-item search-item">
            <label style="font-size: 11px; font-weight: 700; color: var(--text-ghost); text-transform: uppercase;">Search Keywords</label>
            <div style="position: relative; display: flex; align-items: center;">
                <Search size={14} style="position: absolute; left: 12px; color: var(--text-ghost);" />
                <input 
                    type="search" 
                    placeholder="Search by vendor brand, email, phone..." 
                    bind:value={searchQuery} 
                    style="width: 100%; height: 38px; padding: 8px 12px 8px 36px; border-radius: 8px; border: 1px solid var(--glass-border); background: var(--bg-surface); color: var(--text-primary); font-size: 13px; outline: none;"
                />
            </div>
        </div>

        <!-- Subscription Tier Filter -->
        <div class="filter-item">
            <label for="sub_tier_sel" style="font-size: 11px; font-weight: 700; color: var(--text-ghost); text-transform: uppercase;">Subscription Plan</label>
            <select id="sub_tier_sel" bind:value={selectedSubTier} style="width: 100%; height: 38px; padding: 0 12px; border-radius: 8px; border: 1px solid var(--glass-border); background: var(--bg-surface); color: var(--text-primary); font-size: 13px; outline: none; cursor: pointer;">
                <option value="all">All Plans</option>
                <option value="Free">Free Plan</option>
                <option value="Gold">Golden Plan</option>
                <option value="VIP">VIP Plan</option>
                <option value="Diamond">Diamond Plan</option>
            </select>
        </div>

        <!-- Billing State Filter -->
        <div class="filter-item">
            <label for="sub_state_sel" style="font-size: 11px; font-weight: 700; color: var(--text-ghost); text-transform: uppercase;">Billing State</label>
            <select id="sub_state_sel" bind:value={selectedSubStatus} style="width: 100%; height: 38px; padding: 0 12px; border-radius: 8px; border: 1px solid var(--glass-border); background: var(--bg-surface); color: var(--text-primary); font-size: 13px; outline: none; cursor: pointer;">
                <option value="all">All States</option>
                <option value="active">Active</option>
                <option value="trial">Trial</option>
                <option value="stopped">Stopped</option>
            </select>
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
            <Check size={18} />
            <span>{form.message || 'Operation completed successfully'}</span>
        </div>
    {/if}

    <!-- TAB PANEL CONTENT -->
    <div class="tab-content-panel">
        {#if currentVendors.length === 0}
            <div class="empty-state">
                <AlertCircle size={32} />
                <h3>No vendors found</h3>
                <p>No vendor directories match the selected status and active filter criteria.</p>
            </div>
        {:else}
            <div class="table-container">
                <table>
                    <thead>
                        <tr>
                            <th>Vendor Brand Name</th>
                            <th>Category / Area</th>
                            <th>Prioritization & Tier</th>
                            <th>Verification State</th>
                            <th style="text-align:center">Administrative Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        {#each currentVendors as v (v.id)}
                            {@const tierName = v.current_tier || 'Free'}
                            <tr>
                                <td>
                                    <div class="vendor-name-row">
                                        <span class="vendor-name-title">{v.name_en}</span>
                                        {#if v.name_ar}
                                            <span class="vendor-name-ar">{v.name_ar}</span>
                                        {/if}
                                        {#if v.is_featured}
                                            <span class="badge badge-gold" style="font-size:10px; display:inline-flex; align-items:center; gap:2px;">
                                                <Sparkles size={10} /> Featured
                                            </span>
                                        {/if}
                                    </div>
                                    
                                    <!-- Contact pills -->
                                    <div class="contact-row" style="margin-top:6px; display:flex; gap:12px; flex-wrap:wrap;">
                                        {#if v.email}
                                            <div class="contact-pill"><Mail size={12} /> <span>{v.email}</span></div>
                                        {/if}
                                        {#if v.phone}
                                            <div class="contact-pill" style="display:flex; align-items:center; gap:6px;">
                                                <Phone size={12} /> <span>{v.phone}</span>
                                                <a href="https://wa.me/{v.phone.replace(/[^0-9]/g, '')}" target="_blank" rel="noopener noreferrer" class="wa-link" style="color:var(--success); display:inline-flex; align-items:center; gap:2px; text-decoration:none; font-weight:700;">
                                                    <MessageCircle size={12} /> WhatsApp
                                                </a>
                                            </div>
                                        {/if}
                                    </div>
                                </td>
                                <td>
                                    <div class="vendor-meta-cell">
                                        <span class="badge badge-muted" style="text-transform: capitalize;">{v.category}</span>
                                        <div style="font-size:11.5px; color:var(--text-ghost); margin-top:2px;">
                                            <MapPin size={11} /> {v.city_name_en || 'Saudi Arabia'}
                                        </div>
                                    </div>
                                </td>
                                <td>
                                    <!-- Premium Tier Gradient Badges -->
                                    <div class="tier-gradient-container" style="display:flex; flex-direction:column; gap:4px;">
                                        <span class="tier-badge" class:free={tierName === 'Free'} class:gold={tierName === 'Gold' || tierName === 'Golden'} class:vip={tierName === 'VIP'} class:diamond={tierName === 'Diamond'}>
                                            🌟 {tierName} Plan
                                        </span>
                                        <span class="sub-billing-badge" style="font-size:11px; font-weight:600; color:var(--text-ghost);">
                                            Billing: {v.subscription_status?.toUpperCase() || 'FREE'}
                                        </span>
                                    </div>
                                </td>
                                <td>
                                    <div style="display:flex; flex-direction:column; gap:4px;">
                                        <span class="badge" style="text-transform:uppercase; font-size:10px; font-weight:800; background: {v.status === 'active' ? 'var(--success-dim)' : (v.status === 'pending' ? 'var(--warning-dim)' : 'var(--danger-dim)')}; color: {v.status === 'active' ? 'var(--success)' : (v.status === 'pending' ? 'var(--warning)' : 'var(--danger)')}">
                                            {v.status}
                                        </span>
                                        {#if v.subscription_expires_at}
                                            <span style="font-size:11px; color:var(--text-ghost);">Expires: {formatDate(v.subscription_expires_at)}</span>
                                        {/if}
                                    </div>
                                </td>
                                <td>
                                    <div class="actions-cell">
                                        <a href="/dashboard/vendors/{v.id}" class="btn btn-sm btn-outline" title="Inspect profile & listings">
                                            <ExternalLink size={12} /> View Details
                                        </a>

                                        {#if v.status === 'pending'}
                                            <form method="POST" action="?/updateStatus" use:enhance={() => {
                                                return async ({ update }) => {
                                                    await invalidateAll();
                                                    update();
                                                };
                                            }}>
                                                <input type="hidden" name="id" value={v.id} />
                                                <input type="hidden" name="status" value="active" />
                                                <button type="submit" class="btn btn-sm btn-success">
                                                    Approve Vendor
                                                </button>
                                            </form>
                                        {:else if v.status === 'suspended' || v.status === 'banned' || v.status === 'rejected'}
                                            <form method="POST" action="?/reactivate" use:enhance={() => {
                                                return async ({ update }) => {
                                                    await invalidateAll();
                                                    update();
                                                };
                                            }}>
                                                <input type="hidden" name="id" value={v.id} />
                                                <button type="submit" class="btn btn-sm btn-success">
                                                    Restore Account
                                                </button>
                                            </form>
                                        {:else if v.status === 'active'}
                                            <button class="btn btn-sm btn-outline danger" onclick={() => openStatusModal(v, 'suspended')}>
                                                Suspend
                                            </button>
                                        {/if}

                                        <!-- Premium tier action controls -->
                                        <button class="btn btn-sm btn-outline" onclick={() => openSubscriptionModal(v)}>
                                            Configure Plan
                                        </button>
                                        {#if tierName !== 'Free'}
                                            <button class="btn btn-sm btn-outline danger" onclick={() => openSubscriptionModal(v, true)}>
                                                Downgrade
                                            </button>
                                        {/if}

                                        <!-- Featured toggle -->
                                        <button class="btn btn-sm btn-outline" onclick={() => openFeaturedModal(v)}>
                                            <Sparkles size={12} /> Featured Placement
                                        </button>
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

<!-- CHANGE ACCOUNT STATUS MODAL (SUSPEND/BAN) -->
{#if showStatusModal && selectedVendor}
    <div class="modal-backdrop" onclick={closeModals}>
        <div class="modal-card" onclick={(e) => e.stopPropagation()}>
            <div class="modal-header">
                <h2>Suspend / Lock Vendor Account</h2>
                <button class="btn-icon" onclick={closeModals}><X size={18} /></button>
            </div>
            <form method="POST" action="?/updateStatus" use:enhance={() => {
                return async ({ update }) => {
                    closeModals();
                    await invalidateAll();
                    update();
                };
            }}>
                <input type="hidden" name="id" value={selectedVendor.id} />
                <input type="hidden" name="status" value={statusTarget} />
                <div class="form-group full-width" style="margin-bottom:20px;">
                    <label for="lockout_reason_text">Reason for Suspension / Lockout *</label>
                    <textarea id="lockout_reason_text" name="reason" required bind:value={statusReason} rows="3" class="form-textarea" placeholder="Provide reason for lockdown. This will log in audit logs..."></textarea>
                </div>
                <div class="modal-actions">
                    <button type="button" class="btn btn-outline" onclick={closeModals}>Cancel</button>
                    <button type="submit" class="btn btn-danger">Confirm Lockdown</button>
                </div>
            </form>
        </div>
    </div>
{/if}

<!-- CONFIGURE PLAN / DOWNGRADE MODAL -->
{#if showSubscriptionModal && selectedVendor}
    <div class="modal-backdrop" onclick={closeModals}>
        <div class="modal-card" onclick={(e) => e.stopPropagation()}>
            <div class="modal-header">
                <h2>{isRemovingSubscription ? 'Remove Paid Subscription' : 'Configure Subscription Plan'}</h2>
                <button class="btn-icon" onclick={closeModals}><X size={18} /></button>
            </div>

            {#if isRemovingSubscription}
                <form method="POST" action="?/updateSubscription" use:enhance={() => {
                    return async ({ update }) => {
                        closeModals();
                        await invalidateAll();
                        update();
                    };
                }}>
                    <input type="hidden" name="id" value={selectedVendor.id} />
                    <input type="hidden" name="subscription_status" value="active" />
                    <input type="hidden" name="subscription_tier_id" value={getFreeTierId()} />
                    <p style="margin-bottom:20px; font-size:14px; line-height:1.6;">
                        Are you sure you want to remove the paid subscription plan for <strong>{selectedVendor.name_en}</strong>?
                        The vendor profile will be instantly downgraded to the <strong>Free Plan</strong>.
                    </p>
                    <div class="modal-actions">
                        <button type="button" class="btn btn-outline" onclick={closeModals}>Cancel</button>
                        <button type="submit" class="btn btn-danger">Downgrade to Free</button>
                    </div>
                </form>
            {:else}
                <form method="POST" action="?/updateSubscription" use:enhance={() => {
                    return async ({ update }) => {
                        closeModals();
                        await invalidateAll();
                        update();
                    };
                }}>
                    <input type="hidden" name="id" value={selectedVendor.id} />
                    <div style="display:flex; flex-direction:column; gap:16px;">
                        <div class="form-group">
                            <label for="vendor_sub_plan_sel">Select Plan Tier *</label>
                            <select id="vendor_sub_plan_sel" name="subscription_tier_id" required bind:value={formTierId} class="form-select" style="height:38px; padding:0 12px; border-radius:8px; border:1px solid var(--glass-border); background:var(--bg-surface); color:var(--text-primary);">
                                {#each tiers as t}
                                    <option value={t.id}>{t.name} ({t.price} SAR/yr)</option>
                                {/each}
                            </select>
                        </div>
                        <div class="form-group">
                            <label for="vendor_sub_state_sel">Billing / Subscription State *</label>
                            <select id="vendor_sub_state_sel" name="subscription_status" required bind:value={formSubStatus} class="form-select" style="height:38px; padding:0 12px; border-radius:8px; border:1px solid var(--glass-border); background:var(--bg-surface); color:var(--text-primary);">
                                <option value="active">Active (Subscribed)</option>
                                <option value="trial">Trial Period</option>
                                <option value="stopped">Stopped / Locked</option>
                            </select>
                        </div>
                    </div>
                    <div class="modal-actions">
                        <button type="button" class="btn btn-outline" onclick={closeModals}>Cancel</button>
                        <button type="submit" class="btn btn-gold">Save Changes</button>
                    </div>
                </form>
            {/if}
        </div>
    </div>
{/if}

<!-- CONFIGURE FEATURED PLACEMENT MODAL -->
{#if showFeaturedModal && selectedVendor}
    <div class="modal-backdrop" onclick={closeModals}>
        <div class="modal-card" onclick={(e) => e.stopPropagation()}>
            <div class="modal-header">
                <h2>Featured Promotion Configuration</h2>
                <button class="btn-icon" onclick={closeModals}><X size={18} /></button>
            </div>
            <form method="POST" action="?/updateFeatured" use:enhance={() => {
                return async ({ update }) => {
                    closeModals();
                    await invalidateAll();
                    update();
                };
            }}>
                <input type="hidden" name="id" value={selectedVendor.id} />
                <div style="display:flex; flex-direction:column; gap:16px;">
                    <div class="form-group" style="display:flex; flex-direction:row; align-items:center; gap:10px;">
                        <input id="featured_chk" type="checkbox" name="is_featured" value="true" bind:checked={adIsFeatured} style="width:16px; height:16px;" />
                        <label for="featured_chk" style="font-weight:700;">Promote to Featured Placement</label>
                    </div>
                    <div class="form-group">
                        <label for="featured_expiry_date">Placement Expiry Date</label>
                        <input id="featured_expiry_date" type="date" name="expires_at" bind:value={adExpiry} class="form-input" style="height:38px; padding:0 12px; border-radius:8px; border:1px solid var(--glass-border); background:var(--bg-surface); color:var(--text-primary);" />
                    </div>
                </div>
                <div class="modal-actions">
                    <button type="button" class="btn btn-outline" onclick={closeModals}>Cancel</button>
                    <button type="submit" class="btn btn-gold">Update Placement</button>
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
    .filter-item { display: flex; flex-direction: column; gap: 6px; flex: 1; min-width: 180px; }
    .search-item { flex: 2; min-width: 280px; }

    .notice-banner { display: flex; align-items: center; gap: 12px; padding: 12px 16px; border-radius: 8px; font-size: 13.5px; font-weight: 600; }
    .notice-banner.error { background: var(--danger-dim); color: var(--danger); border: 1px solid var(--danger-border); }
    .notice-banner.success { background: var(--success-dim); color: var(--success); border: 1px solid var(--success-border); }

    .table-container { background: var(--bg-elevated); border: 1px solid var(--glass-border); border-radius: 10px; overflow: hidden; margin-top: 10px; }
    table { width: 100%; border-collapse: collapse; text-align: left; }
    th { padding: 12px 16px; font-size: 11px; font-weight: 700; text-transform: uppercase; color: var(--text-ghost); border-bottom: 1px solid var(--glass-border); background: var(--bg-surface); }
    td { padding: 14px 16px; border-bottom: 1px solid var(--glass-border); font-size: 13px; vertical-align: middle; }
    tr:hover { background: var(--bg-surface); }

    .vendor-name-row { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
    .vendor-name-title { font-weight: 700; color: var(--text-primary); }
    .vendor-name-ar { font-size: 12px; color: var(--text-secondary); direction: rtl; }
    
    .contact-pill { display: inline-flex; align-items: center; gap: 6px; font-size: 11.5px; color: var(--text-ghost); background: var(--bg-surface); padding: 2px 8px; border-radius: 6px; border: 1px solid var(--glass-border); }
    
    .tier-badge {
        color: #fff;
        padding: 0.2rem 0.6rem;
        border-radius: 6px;
        font-size: 0.75rem;
        font-weight: 700;
        text-shadow: 0 1px 2px rgba(0,0,0,0.3);
        width: fit-content;
    }
    .tier-badge.free { background: #64748b; }
    .tier-badge.gold { background: linear-gradient(135deg, #fbbf24 0%, #d97706 100%); }
    .tier-badge.vip { background: linear-gradient(135deg, #8b5cf6 0%, #5b21b6 100%); }
    .tier-badge.diamond { background: linear-gradient(135deg, #06b6d4 0%, #0891b2 100%); }

    .actions-cell { display: flex; gap: 8px; justify-content: flex-start; flex-wrap: wrap; }

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
    .form-group { display: flex; flex-direction: column; gap: 6px; }
    .form-group label { font-size: 12px; font-weight: 600; color: var(--text-secondary); }
    .form-textarea { padding: 8px 12px; border-radius: 6px; border: 1px solid var(--glass-border); background: var(--bg-surface); color: var(--text-primary); outline: none; font-size: 13px; }
    .modal-actions { display: flex; justify-content: flex-end; gap: 12px; margin-top: 24px; }
</style>
