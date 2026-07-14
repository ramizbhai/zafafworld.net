<script lang="ts">
    import { enhance } from '$app/forms';
    import { invalidateAll } from '$app/navigation';
    import { fade } from 'svelte/transition';
    import { Check, X, Clock, AlertCircle, RefreshCw, Crown, Info } from 'lucide-svelte';

    interface Props {
        data: {
            requests: any[];
        };
    }

    let { data }: Props = $props();

    let requests = $derived(data.requests || []);
    let submittingIds = $state<string[]>([]);
    let activeTab = $state<'pending' | 'reviewed'>('pending');

    let pendingRequests = $derived(requests.filter(r => r.status === 'pending'));
    let reviewedRequests = $derived(requests.filter(r => r.status !== 'pending'));

    let currentList = $derived(activeTab === 'pending' ? pendingRequests : reviewedRequests);

    let promptAction = $state<{ action: 'approve' | 'reject', id: string } | null>(null);
    let adminNotes = $state('');

    function openActionModal(action: 'approve' | 'reject', id: string) {
        promptAction = { action, id };
        adminNotes = '';
    }

    function closeActionModal() {
        promptAction = null;
        adminNotes = '';
    }

    function getStatusColor(status: string) {
        if (status === 'approved') return 'var(--success)';
        if (status === 'rejected') return 'var(--danger)';
        return 'var(--warning)';
    }

    function getStatusBg(status: string) {
        if (status === 'approved') return 'var(--success-dim)';
        if (status === 'rejected') return 'var(--danger-dim)';
        return 'var(--warning-dim)';
    }
</script>

<div class="page-container">
    <div class="page-header">
        <div class="header-left">
            <div class="header-icon">
                <Crown size={22} />
            </div>
            <div>
                <h1 class="page-title">Subscription Requests</h1>
                <p class="page-subtitle">Manage vendor plan upgrade requests</p>
            </div>
        </div>
        <div class="header-actions">
            <button class="btn btn-outline" onclick={() => window.location.reload()}>
                <RefreshCw size={16} />
                Refresh
            </button>
        </div>
    </div>

    <div class="tabs">
        <button 
            class="tab" 
            class:active={activeTab === 'pending'} 
            onclick={() => activeTab = 'pending'}
        >
            Pending ({pendingRequests.length})
        </button>
        <button 
            class="tab" 
            class:active={activeTab === 'reviewed'} 
            onclick={() => activeTab = 'reviewed'}
        >
            Reviewed ({reviewedRequests.length})
        </button>
    </div>

    {#if currentList.length === 0}
        <div class="empty-state" in:fade={{ duration: 200 }}>
            <div class="empty-icon">
                <Info size={32} />
            </div>
            <h3>No requests found</h3>
            <p>There are currently no {activeTab} subscription requests.</p>
        </div>
    {:else}
        <div class="table-container" in:fade={{ duration: 200 }}>
            <table class="requests-table">
                <thead>
                    <tr>
                        <th>Vendor</th>
                        <th>Requested Tier</th>
                        <th>Date</th>
                        <th>Status</th>
                        <th>Admin Notes</th>
                        {#if activeTab === 'pending'}
                            <th class="actions-col">Actions</th>
                        {/if}
                    </tr>
                </thead>
                <tbody>
                    {#each currentList as req (req.id)}
                        <tr>
                            <td>
                                <div class="vendor-info">
                                    <div class="vendor-name">{req.vendor_name_en}</div>
                                    <div class="vendor-id">ID: {req.vendor_id.split('-')[0]}...</div>
                                </div>
                            </td>
                            <td>
                                <div class="tier-badge">
                                    <Crown size={12} />
                                    {req.requested_tier_name || 'Premium'}
                                </div>
                            </td>
                            <td>
                                <div class="date-text">
                                    {new Date(req.created_at).toLocaleDateString()}
                                </div>
                                <div class="time-text">
                                    {new Date(req.created_at).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                                </div>
                            </td>
                            <td>
                                <span class="status-badge" style="background: {getStatusBg(req.status)}; color: {getStatusColor(req.status)}">
                                    {#if req.status === 'pending'}
                                        <Clock size={12} />
                                    {:else if req.status === 'approved'}
                                        <Check size={12} />
                                    {:else}
                                        <X size={12} />
                                    {/if}
                                    {req.status.toUpperCase()}
                                </span>
                            </td>
                            <td>
                                <span class="notes-text" title={req.admin_notes || ''}>
                                    {req.admin_notes || '-'}
                                </span>
                            </td>
                            {#if activeTab === 'pending'}
                                <td class="actions-col">
                                    <div class="action-buttons">
                                        <button 
                                            class="btn-icon btn-approve" 
                                            title="Approve Request"
                                            disabled={submittingIds.includes(req.id)}
                                            onclick={() => openActionModal('approve', req.id)}
                                        >
                                            <Check size={16} />
                                        </button>
                                        <button 
                                            class="btn-icon btn-reject" 
                                            title="Reject Request"
                                            disabled={submittingIds.includes(req.id)}
                                            onclick={() => openActionModal('reject', req.id)}
                                        >
                                            <X size={16} />
                                        </button>
                                    </div>
                                </td>
                            {/if}
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    {/if}
</div>

{#if promptAction}
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="modal-backdrop" in:fade={{ duration: 150 }} onclick={closeActionModal} role="button" tabindex="0">
        <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div class="modal-content" onclick={(e) => e.stopPropagation()} role="document">
            <div class="modal-header">
                <h3>{promptAction.action === 'approve' ? 'Approve' : 'Reject'} Subscription Request</h3>
                <button class="close-btn" onclick={closeActionModal}>
                    <X size={18} />
                </button>
            </div>
            
            <form 
                method="POST" 
                action="?/{promptAction.action}"
                use:enhance={() => {
                    if (promptAction) {
                        submittingIds = [...submittingIds, promptAction.id];
                        const targetId = promptAction.id;
                        return async ({ result, update }) => {
                            submittingIds = submittingIds.filter(id => id !== targetId);
                            
                            if (result.type === 'success' && result.data?.success) {
                                promptAction = null;
                                invalidateAll();
                            } else {
                                alert((result as any).data?.error || 'Failed to update request');
                            }
                            update({ reset: false });
                        };
                    }
                }}
            >
                <input type="hidden" name="id" value={promptAction.id} />
                
                <div class="modal-body">
                    {#if promptAction.action === 'approve'}
                        <div class="alert info-alert">
                            <AlertCircle size={16} />
                            <p>Approving this request will automatically upgrade the vendor's subscription tier and set their expiry date to 1 year from today.</p>
                        </div>
                    {:else}
                        <div class="alert warning-alert">
                            <AlertCircle size={16} />
                            <p>Are you sure you want to reject this subscription request? The vendor will remain on their current plan.</p>
                        </div>
                    {/if}

                    <div class="form-group" style="margin-top: 1rem;">
                        <label for="admin_notes">Admin Notes (Optional)</label>
                        <textarea 
                            id="admin_notes" 
                            name="admin_notes" 
                            bind:value={adminNotes} 
                            placeholder="Add reason or notes visible to the system..."
                            rows="3"
                        ></textarea>
                    </div>
                </div>

                <div class="modal-footer">
                    <button type="button" class="btn btn-outline" onclick={closeActionModal}>Cancel</button>
                    <button 
                        type="submit" 
                        class="btn {promptAction.action === 'approve' ? 'btn-primary' : 'btn-danger'}"
                    >
                        {promptAction.action === 'approve' ? 'Confirm Approval' : 'Confirm Rejection'}
                    </button>
                </div>
            </form>
        </div>
    </div>
{/if}

<style>
    .page-container {
        padding: 24px;
        max-width: 1200px;
        margin: 0 auto;
    }

    .page-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 24px;
    }

    .header-left {
        display: flex;
        align-items: center;
        gap: 16px;
    }

    .header-icon {
        width: 48px;
        height: 48px;
        border-radius: 12px;
        background: linear-gradient(135deg, var(--purple-dim) 0%, var(--gold-dim) 100%);
        color: var(--purple);
        display: flex;
        align-items: center;
        justify-content: center;
        box-shadow: 0 4px 12px var(--glass-border);
    }

    .page-title {
        font-size: 24px;
        font-weight: 700;
        margin: 0 0 4px 0;
        color: var(--text-primary);
    }

    .page-subtitle {
        font-size: 14px;
        color: var(--text-tertiary);
        margin: 0;
    }

    .tabs {
        display: flex;
        gap: 8px;
        margin-bottom: 24px;
        border-bottom: 1px solid var(--glass-border);
        padding-bottom: 8px;
    }

    .tab {
        padding: 8px 16px;
        border: none;
        background: transparent;
        color: var(--text-secondary);
        font-weight: 600;
        font-size: 14px;
        cursor: pointer;
        border-radius: 8px;
        transition: all 0.2s;
    }

    .tab:hover {
        background: var(--bg-hover);
        color: var(--text-primary);
    }

    .tab.active {
        background: var(--purple-dim);
        color: var(--purple);
    }

    .table-container {
        background: var(--bg-elevated);
        border: 1px solid var(--glass-border);
        border-radius: 12px;
        overflow-x: auto;
    }

    .requests-table {
        width: 100%;
        border-collapse: collapse;
        text-align: left;
    }

    .requests-table th, .requests-table td {
        padding: 16px;
        border-bottom: 1px solid var(--glass-border);
    }

    .requests-table th {
        font-size: 12px;
        text-transform: uppercase;
        color: var(--text-ghost);
        font-weight: 700;
        letter-spacing: 0.5px;
    }

    .requests-table tr:last-child td {
        border-bottom: none;
    }

    .vendor-info {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .vendor-name {
        font-weight: 600;
        color: var(--text-primary);
    }

    .vendor-id {
        font-size: 12px;
        color: var(--text-ghost);
        font-family: monospace;
    }

    .tier-badge {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        padding: 4px 10px;
        background: var(--gold-dim);
        color: var(--gold);
        border: 1px solid var(--gold-border);
        border-radius: 20px;
        font-size: 12px;
        font-weight: 700;
    }

    .date-text {
        font-weight: 500;
        color: var(--text-secondary);
        font-size: 14px;
    }

    .time-text {
        font-size: 12px;
        color: var(--text-ghost);
        margin-top: 2px;
    }

    .status-badge {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        padding: 4px 10px;
        border-radius: 6px;
        font-size: 11px;
        font-weight: 700;
    }

    .notes-text {
        color: var(--text-tertiary);
        font-size: 13px;
        max-width: 200px;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        display: inline-block;
    }

    .action-buttons {
        display: flex;
        gap: 8px;
    }

    .btn-icon {
        width: 32px;
        height: 32px;
        border-radius: 8px;
        border: 1px solid var(--glass-border);
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        background: var(--bg-base);
        color: var(--text-secondary);
        transition: all 0.2s;
    }

    .btn-icon:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .btn-approve:not(:disabled):hover {
        background: var(--success-dim);
        border-color: var(--success-border);
        color: var(--success);
    }

    .btn-reject:not(:disabled):hover {
        background: var(--danger-dim);
        border-color: var(--danger-border);
        color: var(--danger);
    }

    .empty-state {
        padding: 64px 24px;
        text-align: center;
        background: var(--bg-elevated);
        border: 1px dashed var(--glass-border);
        border-radius: 12px;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 12px;
    }

    .empty-icon {
        width: 64px;
        height: 64px;
        border-radius: 50%;
        background: var(--bg-raised);
        color: var(--text-ghost);
        display: flex;
        align-items: center;
        justify-content: center;
        margin-bottom: 8px;
    }

    .empty-state h3 {
        margin: 0;
        font-size: 18px;
        color: var(--text-primary);
    }

    .empty-state p {
        margin: 0;
        color: var(--text-tertiary);
        font-size: 14px;
    }

    /* Modal Styles */
    .modal-backdrop {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.5);
        backdrop-filter: blur(4px);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 1000;
        padding: 24px;
    }

    .modal-content {
        background: var(--bg-elevated);
        border: 1px solid var(--glass-border);
        border-radius: 16px;
        width: 100%;
        max-width: 480px;
        box-shadow: 0 24px 48px rgba(0, 0, 0, 0.2);
        overflow: hidden;
    }

    .modal-header {
        padding: 20px 24px;
        border-bottom: 1px solid var(--glass-border);
        display: flex;
        align-items: center;
        justify-content: space-between;
    }

    .modal-header h3 {
        margin: 0;
        font-size: 18px;
        font-weight: 600;
        color: var(--text-primary);
    }

    .close-btn {
        background: transparent;
        border: none;
        color: var(--text-ghost);
        cursor: pointer;
        padding: 4px;
        border-radius: 6px;
    }

    .close-btn:hover {
        background: var(--bg-hover);
        color: var(--text-primary);
    }

    .modal-body {
        padding: 24px;
    }

    .alert {
        padding: 12px 16px;
        border-radius: 8px;
        display: flex;
        gap: 12px;
        align-items: flex-start;
        font-size: 13px;
        line-height: 1.5;
    }

    .alert p {
        margin: 0;
    }

    .info-alert {
        background: var(--purple-dim);
        border: 1px solid var(--purple-border);
        color: var(--purple);
    }

    .warning-alert {
        background: var(--warning-dim);
        border: 1px solid var(--warning-border);
        color: var(--warning);
    }

    .form-group label {
        display: block;
        margin-bottom: 8px;
        font-size: 13px;
        font-weight: 600;
        color: var(--text-secondary);
    }

    .form-group textarea {
        width: 100%;
        padding: 12px;
        border-radius: 8px;
        border: 1px solid var(--glass-border);
        background: var(--bg-base);
        color: var(--text-primary);
        font-family: inherit;
        resize: vertical;
    }

    .form-group textarea:focus {
        outline: none;
        border-color: var(--purple);
        box-shadow: 0 0 0 3px var(--purple-dim);
    }

    .modal-footer {
        padding: 16px 24px;
        border-top: 1px solid var(--glass-border);
        background: var(--bg-base);
        display: flex;
        justify-content: flex-end;
        gap: 12px;
    }

    .btn {
        padding: 8px 16px;
        border-radius: 8px;
        font-size: 14px;
        font-weight: 600;
        cursor: pointer;
        display: inline-flex;
        align-items: center;
        gap: 8px;
        border: 1px solid transparent;
        transition: all 0.2s;
    }

    .btn-outline {
        background: transparent;
        border-color: var(--glass-border);
        color: var(--text-secondary);
    }

    .btn-outline:hover {
        background: var(--bg-hover);
        color: var(--text-primary);
    }

    .btn-primary {
        background: var(--purple);
        color: white;
    }

    .btn-primary:hover {
        background: var(--purple-glow);
    }

    .btn-danger {
        background: var(--danger);
        color: white;
    }

    .btn-danger:hover {
        filter: brightness(1.1);
    }
</style>
