<script lang="ts">
  import { enhance } from "$app/forms";
  import { invalidateAll } from "$app/navigation";
  import { t, lang } from "$lib/i18n/index.js";
  import {
    Megaphone,
    CheckSquare,
    Square,
    ThumbsUp,
    ThumbsDown,
    Eye,
  } from "lucide-svelte";
  import type { MarketingState } from "$lib/features/admin/marketing/marketingState.svelte.js";
  import { RBACService } from "../../../../core/auth/rbac.service.js";
  import { page } from "$app/stores";

  let { state }: { state: MarketingState } = $props();

  let canModerate = $derived(RBACService.canModeratePromotions($page.data.user));

  function handleActionEnhance() {
    state.isSubmitting = true;
    return async ({ result, update, action }: any) => {
      state.isSubmitting = false;
      if (result.type === "success") {
        if (action.search.includes('bulkApprove')) {
            state.optimisticBulkApprove(state.checkedIds);
        } else if (action.search.includes('approve')) {
            // we could parse formData for id, but optimistic updates are fine to skip here or do carefully
        }
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

{#if state.checkedIds.length > 0 && canModerate}
  <div class="bulk-actions-bar fade-in">
    <span class="bulk-title">
      Selected: <strong>{state.checkedIds.length}</strong> campaigns
    </span>
    <div class="bulk-buttons">
      <form
        method="POST"
        action="?/bulkApprove"
        use:enhance={handleActionEnhance}
      >
        <input type="hidden" name="ids" value={state.checkedIds.join(",")} />
        <button
          type="submit"
          class="btn btn-sm btn-success flex items-center gap-1"
        >
          <ThumbsUp size={14} /> Approve All
        </button>
      </form>
      <button
        onclick={() => state.openBulkReject()}
        class="btn btn-sm btn-danger flex items-center gap-1"
      >
        <ThumbsDown size={14} /> Reject All
      </button>
    </div>
  </div>
{/if}

<div class="table-container">
  <div class="table-head-bar">
    <span class="table-title"
      >{$lang === "ar"
        ? "طابور العروض الترويجية"
        : "Promotions Moderation Queue"}</span
    >
  </div>
  <div class="table-scroll">
    <table>
      <thead>
        <tr>
          <th style="width: 40px;">
            <button onclick={() => state.toggleCheckAll()} class="checkbox-btn" disabled={!canModerate}>
              {#if state.checkedIds.length === state.filteredCampaigns.length && state.filteredCampaigns.length > 0}
                <CheckSquare size={16} class="text-gold" />
              {:else}
                <Square size={16} />
              {/if}
            </button>
          </th>
          <th>{$lang === "ar" ? "العرض الترويجي" : "Promotional Offer"}</th>
          <th>{$lang === "ar" ? "المورد" : "Vendor Brand"}</th>
          <th style="text-align:end"
            >{$lang === "ar" ? "الخصم" : "Discount"}</th
          >
          <th>{$lang === "ar" ? "فترة العرض" : "Date Schedule"}</th>
          <th>{$lang === "ar" ? "المستهدف" : "Targets"}</th>
          <th>{$t("common.status")}</th>
          <th style="text-align:center"
            >{$lang === "ar" ? "الإجراءات" : "Moderation"}</th
          >
        </tr>
      </thead>
      <tbody>
        {#each state.filteredCampaigns as c}
          <tr>
            <td>
              <button onclick={() => state.toggleCheck(c.id)} class="checkbox-btn" disabled={!canModerate}>
                {#if state.checkedIds.includes(c.id)}
                  <CheckSquare size={16} class="text-gold" />
                {:else}
                  <Square size={16} />
                {/if}
              </button>
            </td>
            <td>
              <button
                onclick={() => state.openDetail(c)}
                class="title-link-btn"
                title="View details"
              >
                {$lang === "ar" ? c.title_ar : c.title_en}
              </button>
            </td>
            <td><span class="badge badge-muted">{c.vendor_name}</span></td>
            <td
              style="text-align:end; font-weight:700; color:var(--success); font-size: 14px;"
            >
              {#if c.promo_type === 'discount'}
                {#if c.discount_type === 'percentage'}
                  {c.discount_percentage}%
                {:else}
                  SAR {Number(c.discount_fixed_amount).toLocaleString()}
                {/if}
              {:else}
                🎁 {$lang === 'ar' ? 'عرض قيمة' : 'Added Value'}
              {/if}
            </td>
            <td class="text-muted" style="font-size:11.5px">
              <div style="display:flex; flex-direction:column; gap:2px;">
                <span><strong>Start:</strong> {fmtDate(c.start_at)}</span>
                <span><strong>End:</strong> {fmtDate(c.end_at)}</span>
              </div>
            </td>
            <td>
              <span class="badge badge-info" style="max-width: 180px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; display: inline-block;" title={$lang === 'ar' ? c.listing_title_ar : c.listing_title_en}>
                🎯 {$lang === 'ar' ? c.listing_title_ar || c.listing_id : c.listing_title_en || c.listing_id}
              </span>
            </td>
            <td
              ><span class={statusCls(c.status)}>{statusLbl(c.status)}</span
              ></td
            >
            <td style="text-align:center">
              <div class="actions-cell">
                <button
                  onclick={() => state.openDetail(c)}
                  class="icon-action-btn"
                  title="Preview"
                >
                  <Eye size={14} />
                </button>
                {#if c.status === "pending" && canModerate}
                  <form
                    method="POST"
                    action="?/approve"
                    use:enhance={handleActionEnhance}
                    style="display:inline;"
                  >
                    <input type="hidden" name="id" value={c.id} />
                    <button
                      type="submit"
                      class="icon-action-btn approve-btn"
                      title="Approve"
                    >
                      <ThumbsUp size={14} />
                    </button>
                  </form>
                  <button
                    onclick={() => state.openReject(c.id)}
                    class="icon-action-btn reject-btn"
                    title="Reject"
                  >
                    <ThumbsDown size={14} />
                  </button>
                {/if}
              </div>
            </td>
          </tr>
        {:else}
          <tr>
            <td
              colspan="8"
              style="text-align: center; padding: 48px; color: var(--text-ghost)"
            >
              <Megaphone
                size={36}
                style="margin: 0 auto 16px; color: var(--text-ghost);"
              />
              <h3>
                {$lang === "ar"
                  ? "لا توجد عروض ترويجية نشطة"
                  : "No promotional campaigns match"}
              </h3>
              <p>
                {$lang === "ar"
                  ? "طابور المراجعة فارغ للمرشح المختار"
                  : "No promotions match the selected filter in this queue."}
              </p>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>

<style>
  .bulk-actions-bar {
    background-color: hsl(45, 90%, 96%);
    border: 1.5px solid hsl(45, 90%, 80%);
    border-radius: var(--radius);
    padding: 12px 18px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
  }
  .bulk-title {
    font-size: 13.5px;
    color: hsl(45, 90%, 20%);
  }
  .bulk-buttons {
    display: flex;
    gap: 8px;
  }

  .checkbox-btn {
    border: none;
    background: transparent;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px;
    color: var(--text-sec);
  }
  .checkbox-btn:hover:not(:disabled) {
    color: var(--text);
  }
  .checkbox-btn:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }

  .title-link-btn {
    background: transparent;
    border: none;
    font-weight: 600;
    font-size: 13.5px;
    text-align: start;
    cursor: pointer;
    color: var(--text);
  }
  .title-link-btn:hover {
    color: var(--teal);
    text-decoration: underline;
  }

  .actions-cell {
    display: flex;
    gap: 4px;
    justify-content: center;
  }

  .icon-action-btn {
    width: 28px;
    height: 28px;
    border-radius: 6px;
    border: 1.5px solid var(--border);
    background: var(--card-bg);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-sec);
    transition: all 0.15s ease;
  }
  .icon-action-btn:hover:not(:disabled) {
    background: var(--bg);
    border-color: var(--teal);
    color: var(--text);
  }
  .approve-btn:hover:not(:disabled) {
    background-color: hsl(142, 70%, 93%);
    border-color: hsl(142, 70%, 50%);
    color: hsl(142, 70%, 25%);
  }
  .reject-btn:hover:not(:disabled) {
    background-color: hsl(0, 80%, 95%);
    border-color: hsl(0, 80%, 50%);
    color: hsl(0, 80%, 25%);
  }
</style>
