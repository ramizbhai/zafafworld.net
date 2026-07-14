<script lang="ts">
  import { env } from "$env/dynamic/public";
  import { X, Image, Flag } from "lucide-svelte";
  import { t, lang } from "$lib/i18n/index.js";
  import { RBACService, type User } from "../../../../core/auth/rbac.service.js";
  import type { MessagesState } from "../../../features/admin/messages/messagesState.svelte.js";

  let { state, user } = $props<{ state: MessagesState, user?: User | null }>();

  const API_BASE = env.PUBLIC_API_URL || "https://api.zafafworld.net";

  const clientPart = $derived(
    state.activeConv?.participants?.find((p: any) => p.role?.toLowerCase() === "client")
  );

  const vendorPart = $derived(
    state.activeConv?.participants?.find((p: any) => p.role?.toLowerCase() === "vendor")
  );
</script>

{#if state.selectedId && state.isDrawerOpen}
  <!-- Mobile backdrop -->
  <button
    type="button"
    class="fixed inset-0 bg-[rgba(15,22,41,0.15)] backdrop-blur-xs z-40 lg:hidden border-none cursor-default"
    onclick={() => (state.isDrawerOpen = false)}
    aria-label="Close details"
  ></button>

  <aside class="details-sidebar">
    <div class="details-header">
      <h3>{$lang === 'ar' ? 'التفاصيل' : 'Details'}</h3>
      <button
        onclick={() => (state.isDrawerOpen = false)}
        class="btn-icon lg:hidden"
      >
        <X size={16} />
      </button>
    </div>

    <div class="details-body">
      <!-- CLIENT CARD -->
      <div class="card-detail">
        <div class="detail-section-title flex justify-between items-center w-full">
          <div class="flex items-center gap-2">
            <div class="w-2 h-2 rounded-full bg-[var(--text-ghost)]"></div>
            <h4>{$lang === 'ar' ? 'ملف العميل' : 'Client Profile'}</h4>
          </div>
          {#if clientPart && RBACService.canFlagUser(user)}
            <button
              onclick={() => state.flagUser(clientPart.userId, user)}
              class="flag-btn"
              title={$lang === 'ar' ? 'الإبلاغ عن العميل' : 'Flag Client'}
            >
              <Flag size={12} />
            </button>
          {/if}
        </div>
        <div class="detail-rows">
          <div class="detail-row">
            <span class="row-label">{$lang === 'ar' ? 'الاسم' : 'Name'}</span>
            <span class="row-val">
              {state.activeConv?.clientFirstName
                ? `${state.activeConv.clientFirstName} ${state.activeConv.clientLastName || ""}`.trim()
                : clientPart?.name || "Client"}
            </span>
          </div>
          {#if state.activeConv?.clientEmail || clientPart?.email}
            <div class="detail-row">
              <span class="row-label">{$lang === 'ar' ? 'البريد الإلكتروني' : 'Email'}</span>
              <span class="row-val break-all">
                {state.activeConv?.clientEmail || clientPart?.email}
              </span>
            </div>
          {/if}
          {#if state.activeConv?.clientPhone}
            <div class="detail-row">
              <span class="row-label">{$lang === 'ar' ? 'الهاتف' : 'Phone'}</span>
              <span class="row-val">{state.activeConv.clientPhone}</span>
            </div>
          {/if}
          {#if state.activeConv?.clientWeddingDate}
            <div class="detail-row">
              <span class="row-label">{$lang === 'ar' ? 'تاريخ الزفاف' : 'Wedding'}</span>
              <span class="row-val">
                {new Date(state.activeConv.clientWeddingDate).toLocaleDateString(
                  $lang === 'ar' ? 'ar-SA' : 'en-US',
                  { day: "numeric", month: "short", year: "numeric" },
                )}
              </span>
            </div>
          {/if}
        </div>
      </div>

      <!-- VENDOR CARD -->
      <div class="card-detail">
        <div class="detail-section-title flex justify-between items-center w-full">
          <div class="flex items-center gap-2">
            <div class="w-2 h-2 rounded-full bg-[var(--gold)]"></div>
            <h4>{$lang === 'ar' ? 'ملف المورد' : 'Vendor Profile'}</h4>
          </div>
          {#if vendorPart && RBACService.canFlagUser(user)}
            <button
              onclick={() => state.flagUser(vendorPart.userId, user)}
              class="flag-btn text-[var(--gold)] hover:bg-[var(--gold)]/10"
              title={$lang === 'ar' ? 'الإبلاغ عن المورد' : 'Flag Vendor'}
            >
              <Flag size={12} />
            </button>
          {/if}
        </div>
        <div class="detail-rows">
          <div class="detail-row">
            <span class="row-label">{$lang === 'ar' ? 'الاسم' : 'Name'}</span>
            <span class="row-val">
              {state.activeConv?.vendorNameEn ||
                state.activeConv?.vendorNameAr ||
                vendorPart?.name ||
                "Vendor"}
            </span>
          </div>
          {#if state.activeConv?.vendorEmail || vendorPart?.email}
            <div class="detail-row">
              <span class="row-label">{$lang === 'ar' ? 'البريد الإلكتروني' : 'Email'}</span>
              <span class="row-val break-all">
                {state.activeConv?.vendorEmail || vendorPart?.email}
              </span>
            </div>
          {/if}
          {#if state.activeConv?.vendorPhone}
            <div class="detail-row">
              <span class="row-label">{$lang === 'ar' ? 'الهاتف' : 'Phone'}</span>
              <span class="row-val">{state.activeConv.vendorPhone}</span>
            </div>
          {/if}
          {#if state.activeConv?.vendorCategory}
            <div class="detail-row">
              <span class="row-label">{$lang === 'ar' ? 'التصنيف' : 'Category'}</span>
              <span class="row-val">{state.activeConv.vendorCategory}</span>
            </div>
          {/if}
        </div>
      </div>

      <!-- LISTING CARD -->
      <div class="card-detail-listing">
        {#if state.activeConv?.productCoverImage}
          <img
            src={state.activeConv.productCoverImage.startsWith("http")
              ? state.activeConv.productCoverImage
              : `${API_BASE}${state.activeConv.productCoverImage}`}
            alt="Listing"
            class="listing-img"
          />
        {:else}
          <div class="listing-fallback">
            <Image size={24} class="text-[var(--text-ghost)]" />
          </div>
        {/if}
        <div class="listing-info">
          <h4 class="listing-title">
            {state.activeConv?.productNameEn ||
              state.activeConv?.productNameAr ||
              ($lang === 'ar' ? 'استفسار عام' : 'General Inquiry')}
          </h4>
          {#if state.activeConv?.vendorCategory}
            <p class="listing-desc">
              {state.activeConv.vendorCategory}
            </p>
          {/if}
          <div class="listing-footer">
            {#if state.activeConv?.productPrice}
              <span class="listing-price">
                {Number(state.activeConv.productPrice).toLocaleString()} SAR
              </span>
            {:else}
              <span></span>
            {/if}
            {#if state.activeConv?.status}
              <span class="badge badge-muted badge-pill text-[9px] px-2 py-0.5 font-[700]">
                {state.activeConv.status}
              </span>
            {/if}
          </div>
        </div>
      </div>

      <!-- CONVERSATION INFO -->
      <div class="card-detail">
        <div class="detail-section-title">
          <div class="w-2 h-2 rounded-full bg-[var(--text-ghost)]"></div>
          <h4>{$lang === 'ar' ? 'تفاصيل المحادثة' : 'Conversation Details'}</h4>
        </div>
        <div class="detail-rows">
          <div class="detail-row">
            <span class="row-label">ID</span>
            <span class="row-val mono">
              {state.selectedId?.slice(0, 8)}…{state.selectedId?.slice(-4)}
            </span>
          </div>
          <div class="detail-row">
            <span class="row-label">{$lang === 'ar' ? 'إجمالي الرسائل' : 'Total Messages'}</span>
            <span class="row-val">{state.messages.length}</span>
          </div>
        </div>
      </div>
    </div>
  </aside>
{/if}

<style>
  .details-sidebar {
    width: 350px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    background: var(--bg-raised);
    border-inline-start: 1px solid var(--glass-border);
  }
  .details-header {
    height: 64px;
    border-bottom: 1px solid var(--glass-border);
    padding: 0 20px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: var(--bg-elevated);
  }
  .details-header h3 {
    font-size: 14px;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
  }
  .details-body {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .card-detail {
    background: var(--bg-base);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-md);
    padding: 14px 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .detail-section-title {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 2px;
  }
  .detail-section-title h4 {
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: var(--text-ghost);
    margin: 0;
  }
  .flag-btn {
    padding: 4px;
    border-radius: var(--radius-sm);
    color: var(--text-ghost);
    background: transparent;
    border: none;
    cursor: pointer;
    transition: all 160ms;
  }
  .flag-btn:hover {
    color: var(--danger);
    background: rgba(239, 68, 68, 0.1);
  }
  .detail-rows {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .detail-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 12px;
    font-size: 13px;
    padding-bottom: 8px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.02);
  }
  .detail-row:last-child {
    padding-bottom: 0;
    border-bottom: none;
  }
  .row-label {
    color: var(--text-secondary);
    font-weight: 500;
    flex-shrink: 0;
  }
  .row-val {
    color: var(--text-primary);
    font-weight: 600;
    text-align: end;
  }
  .row-val.mono {
    font-family: var(--font-mono);
  }
  .card-detail-listing {
    background: var(--bg-base);
    border: 1px solid var(--glass-border);
    border-radius: var(--radius-md);
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
  .listing-img {
    width: 100%;
    aspect-ratio: 16/10;
    object-fit: cover;
    background: var(--bg-deep);
    border-bottom: 1px solid var(--glass-border);
  }
  .listing-fallback {
    width: 100%;
    aspect-ratio: 16/10;
    background: var(--bg-deep);
    border-bottom: 1px solid var(--glass-border);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .listing-info {
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .listing-title {
    font-size: 13.5px;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
    line-height: 1.3;
  }
  .listing-desc {
    font-size: 12px;
    color: var(--text-secondary);
    margin: 0;
  }
  .listing-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 4px;
  }
  .listing-price {
    font-size: 13.5px;
    font-weight: 750;
    color: var(--text-primary);
  }
  .btn-icon {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: transparent;
    border: none;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 160ms;
  }
  .btn-icon:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }
  .details-body::-webkit-scrollbar {
    width: 4px;
  }
  .details-body::-webkit-scrollbar-thumb {
    background: rgba(91, 33, 182, 0.08);
    border-radius: 2px;
  }
  .details-body::-webkit-scrollbar-thumb:hover {
    background: rgba(91, 33, 182, 0.16);
  }
</style>
