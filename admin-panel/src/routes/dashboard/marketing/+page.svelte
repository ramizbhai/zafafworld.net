<script lang="ts">
  import { lang } from "$lib/i18n/index.js";
  import { AlertTriangle, ThumbsUp } from "lucide-svelte";
  import { MarketingState } from "$lib/features/admin/marketing/marketingState.svelte.js";
  import MarketingFilterBar from "$lib/components/admin/marketing/MarketingFilterBar.svelte";
  import CampaignList from "$lib/components/admin/marketing/CampaignList.svelte";
  import CampaignActionModals from "$lib/components/admin/marketing/CampaignActionModals.svelte";

  let { data, form } = $props<{ data: any; form: any }>();

  // Initialize the state controller
  const state = new MarketingState(data.campaigns || []);

  // Sync campaigns if they change from load function
  $effect(() => {
    state.setCampaigns(data.campaigns || []);
  });
</script>

<div
  class="fade-in promotions-moderation-container"
  dir={$lang === "ar" ? "rtl" : "ltr"}
>
  <div class="page-header">
    <div class="page-header-left">
      <h1 class="page-title">
        {$lang === "ar" ? "مراجعة العروض الترويجية" : "Promotions Moderation"}
      </h1>
      <p class="page-subtitle">
        {$lang === "ar"
          ? "إدارة طابور المراجعة، واعتماد أو رفض عروض الخصومات الخاصة بالموردين."
          : "Review targeted vendor promotions queue, view targeted listings, approve or reject campaigns."}
      </p>
    </div>
  </div>

  <MarketingFilterBar {state} />

  <!-- Status banner for operations responses -->
  {#if form?.error}
    <div class="alert alert-danger" style="margin-bottom: 20px;">
      <AlertTriangle size={18} />
      <span>{form.error}</span>
    </div>
  {/if}
  {#if form?.success}
    <div class="alert alert-success" style="margin-bottom: 20px;">
      <ThumbsUp size={18} />
      <span>{form.message}</span>
    </div>
  {/if}

  <CampaignList {state} />

  <CampaignActionModals {state} />
</div>

<style>
  .promotions-moderation-container {
    display: flex;
    flex-direction: column;
    gap: 24px;
    max-width: 1200px;
    margin: 0 auto;
  }

  .alert {
    padding: 12px 18px;
    border-radius: var(--radius);
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 13px;
    font-weight: 600;
  }
  .alert-danger {
    background-color: hsl(0, 80%, 95%);
    border: 1.5px solid hsl(0, 80%, 85%);
    color: hsl(0, 80%, 30%);
  }
  .alert-success {
    background-color: hsl(142, 70%, 93%);
    border: 1.5px solid hsl(142, 70%, 85%);
    color: hsl(142, 70%, 25%);
  }
</style>
