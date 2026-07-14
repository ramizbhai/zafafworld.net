<script lang="ts">
    import { enhance } from "$app/forms";
    import { Star } from "lucide-svelte";
    import { RBACService, type User } from "../../../../core/auth/rbac.service.js";
    import type { VendorAdminState } from "../../../../core/stores/adminState.svelte.js";

    let { vendor, tiers, user, state } = $props<{ vendor: any, tiers: any[], user: User, state: VendorAdminState }>();
</script>

{#if RBACService.canOverrideSubscription(user)}
    <div class="info-card action-card">
        <h3 class="card-title">
            <Star size={15} /> Subscription Override (God Mode)
        </h3>
        <p class="card-sub">
            Manually set tier, expiry, and status.
        </p>
        
        <form
            method="POST"
            action="?/updateSubscription"
            use:enhance={() => {
                state.isSubmittingVendor = true;
                return async ({ update }) => {
                    state.isSubmittingVendor = false;
                    await update();
                };
            }}
            class="subscription-form"
        >
            <div class="modal-field">
                <label for="sub_status">Subscription Status</label>
                <select id="sub_status" name="subscription_status" value={vendor.subscription_status}>
                    <option value="trial">Trial</option>
                    <option value="active">Active</option>
                    <option value="stopped">Stopped</option>
                </select>
            </div>

            <div class="modal-field">
                <label for="sub_tier">Tier</label>
                <select id="sub_tier" name="subscription_tier_id" value={vendor.subscription_tier_id || ""}>
                    {#each tiers as tier}
                        <option value={tier.id}>{tier.name}</option>
                    {/each}
                </select>
            </div>

            <div class="modal-field">
                <label for="sub_expiry">Expires At</label>
                <input 
                    type="datetime-local" 
                    id="sub_expiry" 
                    name="subscription_expires_at" 
                    value={vendor.subscription_expires_at ? new Date(vendor.subscription_expires_at).toISOString().slice(0, 16) : ""}
                />
            </div>

            <button type="submit" class="action-btn btn-approve submit-btn" disabled={state.isSubmittingVendor}>
                {state.isSubmittingVendor ? 'Saving...' : 'Save Subscription'}
            </button>
        </form>
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
    .modal-field {
        display: flex;
        flex-direction: column;
        gap: 6px;
        margin-bottom: 12px;
    }
    .modal-field label {
        font-size: 0.78rem;
        font-weight: 600;
        color: #94a3b8;
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }
    .modal-field select,
    .modal-field input {
        background: rgba(255, 255, 255, 0.04);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 10px;
        padding: 10px 12px;
        color: #e2e8f0;
        font-size: 0.85rem;
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
    .submit-btn {
        width: 100%;
        justify-content: center;
        margin-top: 10px;
    }
</style>
