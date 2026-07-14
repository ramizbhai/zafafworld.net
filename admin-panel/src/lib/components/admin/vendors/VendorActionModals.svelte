<script lang="ts">
    import { enhance } from "$app/forms";
    import { getStatusColor } from "$lib/utils/adminFormatter.js";
    import type { VendorAdminState } from "../../../../core/stores/adminState.svelte.js";

    let { state } = $props<{ state: VendorAdminState }>();
</script>

<!-- Vendor Status Modal -->
{#if state.showStatusModal}
    <div
        class="modal-overlay"
        role="button"
        tabindex="-1"
        onclick={() => (state.showStatusModal = false)}
        onkeydown={(e) => {
            if (e.key === "Escape") state.showStatusModal = false;
        }}
    >
        <div
            class="modal"
            role="dialog"
            aria-modal="true"
            tabindex="-1"
            onclick={(e) => e.stopPropagation()}
            onkeydown={(e) => e.stopPropagation()}
        >
            <h3 class="modal-title">
                {state.pendingVendorStatus === "approved"
                    ? "✅ Approve Vendor"
                    : state.pendingVendorStatus === "suspended"
                      ? "⏸ Suspend Vendor"
                      : "❌ Reject Vendor"}
            </h3>
            <p class="modal-sub">
                You are about to change this vendor's status to
                <strong style="color:{getStatusColor(state.pendingVendorStatus)}"
                    >{state.pendingVendorStatus}</strong
                >.
            </p>

            <form
                method="POST"
                action="?/updateStatus"
                use:enhance={() => {
                    state.isSubmittingVendor = true;
                    state.showStatusModal = false;
                    return async ({ update }) => {
                        state.isSubmittingVendor = false;
                        await update();
                    };
                }}
            >
                <input
                    type="hidden"
                    name="status"
                    value={state.pendingVendorStatus}
                />

                <div class="modal-field">
                    <label for="reason_input">Reason (optional)</label>
                    <textarea
                        id="reason_input"
                        name="reason"
                        rows="3"
                        placeholder="e.g. Violation of platform policy, re-approval after verification..."
                        bind:value={state.statusReason}
                    ></textarea>
                </div>

                <div class="modal-actions">
                    <button
                        type="button"
                        class="modal-btn-cancel"
                        onclick={() => (state.showStatusModal = false)}
                    >
                        Cancel
                    </button>
                    <button
                        type="submit"
                        class="modal-btn-confirm"
                        style="background:{getStatusColor(state.pendingVendorStatus)}"
                        disabled={state.isSubmittingVendor}
                    >
                        {state.isSubmittingVendor
                            ? "Processing…"
                            : `Confirm ${state.pendingVendorStatus}`}
                    </button>
                </div>
            </form>
        </div>
    </div>
{/if}

<!-- Promote Product Modal -->
{#if state.showPromoteModal && state.promoteProductItem}
    <div
        class="modal-overlay"
        role="button"
        tabindex="-1"
        onclick={() => (state.showPromoteModal = false)}
        onkeydown={(e) => {
            if (e.key === "Escape") state.showPromoteModal = false;
        }}
    >
        <div
            class="modal"
            role="dialog"
            aria-modal="true"
            tabindex="-1"
            onclick={(e) => e.stopPropagation()}
            onkeydown={(e) => e.stopPropagation()}
        >
            <h3 class="modal-title">⭐ Promote Listing</h3>
            <p class="modal-sub">
                Feature <strong>{state.promoteProductItem.title}</strong> at the top of search results.
            </p>

            <form
                method="POST"
                action="?/promoteProduct"
                use:enhance={() => {
                    state.isSubmittingProduct = state.promoteProductItem.id;
                    state.showPromoteModal = false;
                    return async ({ update }) => {
                        state.isSubmittingProduct = null;
                        await update();
                    };
                }}
            >
                <input
                    type="hidden"
                    name="product_id"
                    value={state.promoteProductItem.id}
                />

                <div class="modal-field">
                    <label for="days_input">Duration (Days)</label>
                    <input
                        id="days_input"
                        name="days"
                        type="number"
                        min="1"
                        max="365"
                        bind:value={state.promoteDays}
                        style="background: rgba(255,255,255,0.04); border: 1px solid var(--glass-border); border-radius: 10px; padding: 10px; color: var(--text-primary); width: 100%;"
                    />
                </div>

                <div class="modal-actions">
                    <button
                        type="button"
                        class="modal-btn-cancel"
                        onclick={() => (state.showPromoteModal = false)}
                    >
                        Cancel
                    </button>
                    <button
                        type="submit"
                        class="modal-btn-confirm"
                        style="background: var(--gold); color: #000;"
                        disabled={state.isSubmittingProduct === state.promoteProductItem.id}
                    >
                        {state.isSubmittingProduct === state.promoteProductItem.id
                            ? "Processing…"
                            : "Promote Listing"}
                    </button>
                </div>
            </form>
        </div>
    </div>
{/if}

<style>
    .modal-overlay {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.75);
        backdrop-filter: blur(6px);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 9999;
    }
    .modal {
        background: var(--bg-elevated);
        border: 1px solid var(--glass-border);
        border-radius: 18px;
        padding: 28px;
        width: 100%;
        max-width: 480px;
        box-shadow: var(--shadow-xl);
    }
    .modal-title {
        font-size: 1.1rem;
        font-weight: 800;
        color: var(--text-primary);
        margin: 0 0 6px;
    }
    .modal-sub {
        font-size: 0.85rem;
        color: #94a3b8;
        margin: 0 0 20px;
    }
    .modal-field {
        display: flex;
        flex-direction: column;
        gap: 6px;
        margin-bottom: 20px;
    }
    .modal-field label {
        font-size: 0.78rem;
        font-weight: 600;
        color: #94a3b8;
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }
    .modal-field textarea {
        background: rgba(255, 255, 255, 0.04);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 10px;
        padding: 10px 12px;
        color: #e2e8f0;
        font-size: 0.85rem;
        resize: vertical;
        min-height: 80px;
        transition: border-color 0.15s;
    }
    .modal-field textarea:focus {
        outline: none;
        border-color: rgba(168, 85, 247, 0.5);
    }
    .modal-actions {
        display: flex;
        gap: 10px;
        justify-content: flex-end;
    }
    .modal-btn-cancel {
        padding: 9px 18px;
        border-radius: 9px;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        color: #94a3b8;
        font-size: 0.83rem;
        font-weight: 600;
        cursor: pointer;
        transition: background 0.15s;
    }
    .modal-btn-cancel:hover {
        background: rgba(255, 255, 255, 0.08);
    }
    .modal-btn-confirm {
        padding: 9px 20px;
        border-radius: 9px;
        border: none;
        color: #fff;
        font-size: 0.83rem;
        font-weight: 700;
        cursor: pointer;
        transition: opacity 0.15s;
    }
    .modal-btn-confirm:hover:not(:disabled) {
        opacity: 0.85;
    }
    .modal-btn-confirm:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }
</style>
