<script lang="ts">
    import { ArrowLeft, AlertTriangle } from "lucide-svelte";
    import type { PageData, ActionData } from "./$types";

    import { createVendorAdminState } from "../../../../core/stores/adminState.svelte.js";
    import VendorHeader from "$lib/components/admin/vendors/VendorHeader.svelte";
    import VendorInfoPanel from "$lib/components/admin/vendors/VendorInfoPanel.svelte";
    import VendorStatusControl from "$lib/components/admin/vendors/VendorStatusControl.svelte";
    import VendorSubscriptionGodMode from "$lib/components/admin/vendors/VendorSubscriptionGodMode.svelte";
    import VendorListingsGrid from "$lib/components/admin/vendors/VendorListingsGrid.svelte";
    import VendorActionModals from "$lib/components/admin/vendors/VendorActionModals.svelte";

    let { data, form }: { data: PageData; form: ActionData } = $props();

    const vendor = $derived(data.vendor);
    const products = $derived(vendor?.products ?? []);
    const user = $derived(data.user);

    let state = createVendorAdminState();
</script>

<svelte:head>
    <title>{vendor?.name_en ?? "Vendor Detail"} — Admin Panel</title>
</svelte:head>

<div class="vendor-detail-page">
    <!-- BACK BUTTON -->
    <a href="/dashboard/vendors" class="back-btn">
        <ArrowLeft size={15} /> Back to Vendors
    </a>

    {#if !vendor}
        <div class="empty-state">
            <AlertTriangle size={36} />
            <h2>Vendor not found</h2>
            <p>This vendor record may have been deleted or you may not have access.</p>
        </div>
    {:else}
        <!-- FORM FEEDBACK -->
        {#if form?.error}
            <div class="alert alert-error">⚠️ {form.error}</div>
        {/if}
        {#if form?.success}
            <div class="alert alert-success">✅ {form.message}</div>
        {/if}

        <VendorHeader {vendor} productsCount={products.length} />

        <div class="content-grid">
            <div class="left-col">
                <VendorInfoPanel {vendor} />
                <VendorStatusControl {vendor} {user} {state} />
                <VendorSubscriptionGodMode {vendor} tiers={data.tiers} {user} {state} />
            </div>

            <div class="right-col">
                <VendorListingsGrid {products} {user} {state} />
            </div>
        </div>
    {/if}
</div>

<VendorActionModals {state} />

<style>
    .vendor-detail-page {
        max-width: 1280px;
        margin: 0 auto;
        display: flex;
        flex-direction: column;
        gap: 24px;
        animation: fade-in 0.3s ease-out;
    }

    @keyframes fade-in {
        from {
            opacity: 0;
            transform: translateY(6px);
        }
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }

    .back-btn {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        font-size: 0.82rem;
        font-weight: 600;
        color: #94a3b8;
        text-decoration: none;
        transition: color 0.15s;
    }
    .back-btn:hover {
        color: #e2e8f0;
    }

    .alert {
        padding: 12px 16px;
        border-radius: 10px;
        font-size: 0.85rem;
        font-weight: 500;
    }
    .alert-error {
        background: rgba(239, 68, 68, 0.1);
        border: 1px solid rgba(239, 68, 68, 0.2);
        color: #f87171;
    }
    .alert-success {
        background: rgba(16, 185, 129, 0.1);
        border: 1px solid rgba(16, 185, 129, 0.2);
        color: #34d399;
    }

    .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 12px;
        padding: 4rem;
        text-align: center;
        border: 1px dashed var(--glass-border);
        border-radius: 14px;
        background: var(--bg-raised);
        color: var(--text-secondary);
    }
    .empty-state h2 {
        font-size: 1.2rem;
        font-weight: 700;
        color: var(--text-primary);
        margin: 0;
    }
    .empty-state p {
        margin: 0;
        font-size: 0.875rem;
    }

    .content-grid {
        display: grid;
        grid-template-columns: 320px 1fr;
        gap: 20px;
        align-items: start;
    }

    .left-col,
    .right-col {
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    @media (max-width: 900px) {
        .content-grid {
            grid-template-columns: 1fr;
        }
    }
</style>
