<script lang="ts">
    import { Shield, CheckCircle, XCircle, AlertTriangle } from "lucide-svelte";
    import { getStatusColor } from "$lib/utils/adminFormatter.js";
    import { RBACService, type User } from "../../../../core/auth/rbac.service.js";
    import type { VendorAdminState } from "../../../../core/stores/adminState.svelte.js";

    let { vendor, user, state } = $props<{ vendor: any, user: User, state: VendorAdminState }>();
</script>

{#if RBACService.canApproveVendor(user) || RBACService.canSuspendVendor(user)}
    <div class="info-card action-card">
        <h3 class="card-title">
            <Shield size={15} /> Vendor Status Control
        </h3>
        <p class="card-sub">
            Current status: <strong style="color:{getStatusColor(vendor.status)}">
                {vendor.status}
            </strong>
        </p>
        <div class="action-btns">
            {#if vendor.status !== "approved" && RBACService.canApproveVendor(user)}
                <button
                    class="action-btn btn-approve"
                    onclick={() => state.openStatusModal("approved")}
                    type="button"
                >
                    <CheckCircle size={14} /> Approve
                </button>
            {/if}
            
            {#if vendor.status !== "suspended" && RBACService.canSuspendVendor(user)}
                <button
                    class="action-btn btn-suspend"
                    onclick={() => state.openStatusModal("suspended")}
                    type="button"
                >
                    <XCircle size={14} /> Suspend
                </button>
            {/if}
            
            {#if vendor.status !== "rejected" && RBACService.canSuspendVendor(user)}
                <button
                    class="action-btn btn-reject"
                    onclick={() => state.openStatusModal("rejected")}
                    type="button"
                >
                    <AlertTriangle size={14} /> Reject
                </button>
            {/if}
        </div>
    </div>
{/if}

<style>
    .info-card {
        background: var(--bg-elevated);
        border: 1px solid var(--glass-border);
        border-radius: 14px;
        padding: 18px;
    }
    .action-card {
        border-color: rgba(168, 85, 247, 0.2);
    }
    .card-title {
        display: flex;
        align-items: center;
        gap: 7px;
        font-size: 0.82rem;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        color: #94a3b8;
        margin: 0 0 14px;
    }
    .card-sub {
        font-size: 0.82rem;
        color: #64748b;
        margin: 0 0 12px;
    }
    .action-btns {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
        margin-top: 12px;
    }
    .action-btn {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        font-size: 0.78rem;
        font-weight: 700;
        padding: 7px 14px;
        border-radius: 8px;
        border: 1px solid transparent;
        cursor: pointer;
        transition: all 0.15s;
    }
    .action-btn.btn-approve {
        background: rgba(16, 185, 129, 0.1);
        color: #34d399;
        border-color: rgba(16, 185, 129, 0.25);
    }
    .action-btn.btn-approve:hover {
        background: rgba(16, 185, 129, 0.2);
    }
    .action-btn.btn-suspend {
        background: rgba(245, 158, 11, 0.1);
        color: #fbbf24;
        border-color: rgba(245, 158, 11, 0.25);
    }
    .action-btn.btn-suspend:hover {
        background: rgba(245, 158, 11, 0.2);
    }
    .action-btn.btn-reject {
        background: rgba(239, 68, 68, 0.1);
        color: #f87171;
        border-color: rgba(239, 68, 68, 0.25);
    }
    .action-btn.btn-reject:hover {
        background: rgba(239, 68, 68, 0.2);
    }
</style>
