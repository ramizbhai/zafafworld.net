<script lang="ts">
  import { enhance } from "$app/forms";
  import { invalidateAll } from "$app/navigation";
  import { lang } from "$lib/i18n/index.js";
  import type { MarketingState } from "$lib/features/admin/marketing/marketingState.svelte.js";

  let { state }: { state: MarketingState } = $props();

  function handleActionEnhance() {
    state.isSubmitting = true;
    return async ({ result, update, action }: any) => {
      state.isSubmitting = false;
      if (result.type === "success") {
        if (action.search.includes('reject')) {
            if (state.activeRejectId) {
                state.optimisticReject(state.activeRejectId);
            }
        } else if (action.search.includes('bulkReject')) {
            state.optimisticBulkReject(state.checkedIds);
        }
        state.isRejectModalOpen = false;
        state.isBulkRejectModalOpen = false;
        state.checkedIds = [];
        await invalidateAll();
      }
      await update();
    };
  }

  function statusCls(s: string) {
    if (s === "approved") return "badge badge-dot badge-success";
    if (s === "pending") return "badge badge-dot badge-warning";
    if (s === "rejected") return "badge badge-dot badge-danger";
    return "badge badge-dot badge-muted";
  }

  function statusLbl(s: string) {
    const m: Record<string, string> = {
      approved: $lang === "ar" ? "نشط / مقبول" : "Active / Approved",
      pending: $lang === "ar" ? "قيد المراجعة" : "Pending Approval",
      rejected: $lang === "ar" ? "مرفوض" : "Rejected",
    };
    return m[s] ?? s;
  }

  function fmtDate(d: string) {
    return new Date(d).toLocaleDateString($lang === "ar" ? "ar-SA" : "en-US", {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }
</script>

<!-- ─── REJECT MODAL ────────────────────────────────────────────── -->
{#if state.isRejectModalOpen}
  <div
    class="modal-backdrop"
    role="button"
    tabindex="0"
    aria-label="Close modal"
    onclick={() => (state.isRejectModalOpen = false)}
    onkeydown={(e) =>
      (e.key === "Enter" || e.key === "Escape") &&
      (state.isRejectModalOpen = false)}
  >
    <div
      class="modal"
      role="presentation"
      onclick={(e) => e.stopPropagation()}
    >
      <form method="POST" action="?/reject" use:enhance={handleActionEnhance}>
        <div class="modal-header">
          <h2 class="modal-title">
            {$lang === "ar"
              ? "رفض العرض الترويجي"
              : "Reject Promotional Campaign"}
          </h2>
          <button
            type="button"
            onclick={() => (state.isRejectModalOpen = false)}
            class="modal-close">✕</button
          >
        </div>
        <div class="modal-body">
          <input type="hidden" name="id" value={state.activeRejectId} />
          <div class="form-group">
            <label class="form-label" for="rejection_reason"
              >{$lang === "ar"
                ? "سبب الرفض (إلزامي للمورد) *"
                : "Rejection Reason (Required for vendor visibility) *"}</label
            >
            <textarea
              id="rejection_reason"
              name="rejection_reason"
              class="form-input"
              style="height: 100px; resize: vertical;"
              bind:value={state.rejectionReason}
              required
              placeholder="Specify why this promotion is rejected..."
            ></textarea>
          </div>
        </div>
        <div class="modal-footer">
          <button
            type="button"
            onclick={() => (state.isRejectModalOpen = false)}
            class="btn btn-ghost">Cancel</button
          >
          <button
            type="submit"
            disabled={state.isSubmitting}
            class="btn btn-primary btn-danger">Confirm Rejection</button
          >
        </div>
      </form>
    </div>
  </div>
{/if}

<!-- ─── BULK REJECT MODAL ────────────────────────────────────────── -->
{#if state.isBulkRejectModalOpen}
  <div
    class="modal-backdrop"
    role="button"
    tabindex="0"
    aria-label="Close modal"
    onclick={() => (state.isBulkRejectModalOpen = false)}
    onkeydown={(e) =>
      (e.key === "Enter" || e.key === "Escape") &&
      (state.isBulkRejectModalOpen = false)}
  >
    <div
      class="modal"
      role="presentation"
      onclick={(e) => e.stopPropagation()}
    >
      <form
        method="POST"
        action="?/bulkReject"
        use:enhance={handleActionEnhance}
      >
        <div class="modal-header">
          <h2 class="modal-title">
            Bulk Reject ({state.checkedIds.length}) Campaigns
          </h2>
          <button
            type="button"
            onclick={() => (state.isBulkRejectModalOpen = false)}
            class="modal-close">✕</button
          >
        </div>
        <div class="modal-body">
          <input type="hidden" name="ids" value={state.checkedIds.join(",")} />
          <div class="form-group">
            <label class="form-label" for="bulk_rejection_reason"
              >Rejection Reason for All Selected Campaigns *</label
            >
            <textarea
              id="bulk_rejection_reason"
              name="rejection_reason"
              class="form-input"
              style="height: 100px; resize: vertical;"
              bind:value={state.bulkRejectionReason}
              required
              placeholder="Specify rejection reason for all selected..."
            ></textarea>
          </div>
        </div>
        <div class="modal-footer">
          <button
            type="button"
            onclick={() => (state.isBulkRejectModalOpen = false)}
            class="btn btn-ghost">Cancel</button
          >
          <button
            type="submit"
            disabled={state.isSubmitting}
            class="btn btn-primary btn-danger">Confirm Bulk Reject</button
          >
        </div>
      </form>
    </div>
  </div>
{/if}

<!-- ─── DETAILS PREVIEW MODAL ────────────────────────────────────── -->
{#if state.isDetailModalOpen && state.selectedCampaign}
  <div
    class="modal-backdrop"
    role="button"
    tabindex="0"
    aria-label="Close modal"
    onclick={() => (state.isDetailModalOpen = false)}
    onkeydown={(e) =>
      (e.key === "Enter" || e.key === "Escape") &&
      (state.isDetailModalOpen = false)}
  >
    <div
      class="modal large-modal"
      role="presentation"
      onclick={(e) => e.stopPropagation()}
    >
      <div class="modal-header">
        <h2 class="modal-title">
          {$lang === "ar"
            ? "تفاصيل العرض الترويجي"
            : "Promotion Details Preview"}
        </h2>
        <button
          type="button"
          onclick={() => (state.isDetailModalOpen = false)}
          class="modal-close">✕</button
        >
      </div>
      <div class="modal-body">
        <div class="detail-grid">
          <div class="detail-row">
            <span class="detail-lbl">ID:</span>
            <span class="detail-val">{state.selectedCampaign.id}</span>
          </div>
          <div class="detail-row">
            <span class="detail-lbl">Vendor Brand:</span>
            <span class="detail-val"
              >{state.selectedCampaign.vendor_name} ({state.selectedCampaign.vendor_id})</span
            >
          </div>
          <div class="detail-row">
            <span class="detail-lbl">Title (Arabic):</span>
            <span class="detail-val" style="font-weight: 700;"
              >{state.selectedCampaign.title_ar}</span
            >
          </div>
          <div class="detail-row">
            <span class="detail-lbl">Title (English):</span>
            <span class="detail-val" style="font-weight: 700;"
              >{state.selectedCampaign.title_en}</span
            >
          </div>
          <div class="detail-row">
            <span class="detail-lbl">Offer Value / Type:</span>
            <span
              class="detail-val text-success"
              style="font-weight: 800; font-size: 16px;"
            >
              {#if state.selectedCampaign.promo_type === 'discount'}
                {#if state.selectedCampaign.discount_type === 'percentage'}
                  {state.selectedCampaign.discount_percentage}% OFF
                {:else}
                  SAR {Number(state.selectedCampaign.discount_fixed_amount).toLocaleString()} OFF
                {/if}
              {:else}
                🎁 Added-Value Benefit
              {/if}
            </span>
          </div>
          {#if state.selectedCampaign.promo_type === 'benefit'}
            <div class="detail-row">
              <span class="detail-lbl">Benefit (Arabic):</span>
              <span class="detail-val" style="font-weight: 500;">
                {state.selectedCampaign.benefit_description_ar || '—'}
              </span>
            </div>
            <div class="detail-row">
              <span class="detail-lbl">Benefit (English):</span>
              <span class="detail-val" style="font-weight: 500;">
                {state.selectedCampaign.benefit_description_en || '—'}
              </span>
            </div>
          {/if}
          <div class="detail-row">
            <span class="detail-lbl">Schedule Range:</span>
            <span class="detail-val">
              Start: {fmtDate(state.selectedCampaign.start_at)} <br />
              End: {fmtDate(state.selectedCampaign.end_at)}
            </span>
          </div>
          <div class="detail-row">
            <span class="detail-lbl">Current Status:</span>
            <span class="detail-val">
              <span class={statusCls(state.selectedCampaign.status)}
                >{statusLbl(state.selectedCampaign.status)}</span
              >
            </span>
          </div>
          <div class="detail-row">
            <span class="detail-lbl">Target Listing:</span>
            <div class="detail-val">
              <strong style="color: var(--teal);">{state.selectedCampaign.listing_title_en || 'Listing Name'}</strong> <br />
              <code class="uuid-pill" style="font-size: 11px;">{state.selectedCampaign.listing_id}</code>
            </div>
          </div>
          <div class="detail-row">
            <span class="detail-lbl">Created At:</span>
            <span class="detail-val"
              >{fmtDate(state.selectedCampaign.created_at)}</span
            >
          </div>
        </div>
      </div>
      <div class="modal-footer">
        <button
          type="button"
          onclick={() => (state.isDetailModalOpen = false)}
          class="btn btn-primary">Close Preview</button
        >
      </div>
    </div>
  </div>
{/if}

<style>
  .large-modal {
    max-width: 600px;
    width: 90%;
  }

  .detail-grid {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .detail-row {
    display: flex;
    flex-direction: column;
    gap: 4px;
    border-bottom: 1px dashed var(--border-light);
    padding-bottom: 8px;
  }
  .detail-lbl {
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    color: var(--text-ghost);
  }
  .detail-val {
    font-size: 13.5px;
    color: var(--text);
  }

  .uuid-pill {
    background-color: var(--bg);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 2px 8px;
    font-family: monospace;
    font-size: 11.5px;
    color: var(--text-sec);
  }
</style>
