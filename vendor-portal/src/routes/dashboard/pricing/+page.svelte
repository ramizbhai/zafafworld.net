<script lang="ts">
  import StatusBadge     from '$lib/components/StatusBadge.svelte';
  import Pagination      from '$lib/components/Pagination.svelte';
  import PaymentModal    from '$lib/components/PaymentModal.svelte';
  import PageSelectorModal from '$lib/components/PageSelectorModal.svelte';
  import { getI18n } from '$lib/i18n/i18n.svelte';
  import type { PricingPlan } from '$lib/types';
  import type { PageData } from './$types';

  let { data } = $props<{ data: PageData }>();

  const i18n = getI18n();

  let activeTab = $state<'subscription' | 'plans'>('subscription');
  let currentPage = $state(1);

  // Modal Flow
  let showPageModal = $state(false);
  let showPaymentModal = $state(false);
  let selectedPlan = $state<PricingPlan | null>(null);

  // Subscriptions reactive variables
  let totalActivePages = $derived(data.pages.filter((p: any) => p.status === 'published').length);
  
  // Calculate total paid dynamically
  let totalPaidAmount = $derived.by(() => {
    return data.subscriptions
      .filter((s: any) => s.status === 'paid')
      .reduce((sum: number, s: any) => {
        const num = parseInt(s.amount.replace(/[^0-9]/g, '')) || 0;
        return sum + num;
      }, 0);
  });

  // Paginated Subscriptions
  let itemsPerPage = 10;
  let pagedSubscriptions = $derived(
    data.subscriptions.slice((currentPage - 1) * itemsPerPage, currentPage * itemsPerPage)
  );
  let totalPages = $derived(Math.max(1, Math.ceil(data.subscriptions.length / itemsPerPage)));

  function openPlanFlow(plan: PricingPlan | null) {
    selectedPlan = plan;
    showPageModal = true;
  }

  function onPageNext() {
    showPageModal = false;
    showPaymentModal = true;
  }

  function onPageClose() {
    showPageModal = false;
  }

  function onPaymentClose() {
    showPaymentModal = false;
  }

  function onPaymentConfirm() {
    showPaymentModal = false;
    alert(i18n.t.pricing.paymentGatewayRedirect);
    
    // Simulate successful plan upgrade / checkout
    if (selectedPlan) {
      const todayStr = i18n.locale === 'ar' ? '١٢ أكتوبر ٢٠٢٦' : '12 Oct 2026';
      data.subscriptions.unshift({
        id: data.subscriptions.length + 1,
        page: data.pages[0]?.title || 'قاعة الأفراح الملكية',
        plan: selectedPlan.name,
        date: todayStr,
        amount: `${selectedPlan.price * 55} ${i18n.locale === 'ar' ? 'ريال سعودي' : 'SAR'}`,
        status: 'paid',
        canRenew: false
      });
      activeTab = 'subscription';
    }
  }
</script>

<svelte:head>
  <title>{i18n.t.pricing.title} – {i18n.t.common.appName}</title>
</svelte:head>

<div class="pricing-container" dir={i18n.isRtl ? 'rtl' : 'ltr'}>
  <!-- Tabs Row -->
  <div class="pricing-tabs">
    <button
      class="tab-btn {activeTab === 'subscription' ? 'active' : ''}"
      onclick={() => activeTab = 'subscription'}
    >
      {i18n.t.pricing.tabSubscription}
    </button>
    <button
      class="tab-btn {activeTab === 'plans' ? 'active' : ''}"
      onclick={() => activeTab = 'plans'}
    >
      {i18n.t.pricing.tabPlans}
    </button>
  </div>

  <!-- ── TAB: Subscription Details ── -->
  {#if activeTab === 'subscription'}
    <div class="summary-cards">
      <div class="summary-card">
        <div class="summary-info">
          <div class="summary-label">{i18n.t.pricing.pageCount}</div>
          <div class="summary-value">{totalActivePages}</div>
        </div>
        <div class="stat-icon icon-teal">📄</div>
      </div>
      
      <div class="summary-card">
        <div class="summary-info">
          <div class="summary-label">{i18n.t.pricing.totalPaid}</div>
          <div class="summary-value">
            {totalPaidAmount}
          </div>
        </div>
        <div class="stat-icon icon-orange">💰</div>
      </div>
    </div>

    <div class="table-card">
      <div class="table-header">
        <span class="table-title">{i18n.t.pricing.subTableTitle}</span>
      </div>
      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>{i18n.t.couples.pageName}</th>
              <th>{i18n.t.pricing.planName}</th>
              <th>{i18n.t.pricing.date}</th>
              <th>{i18n.t.pricing.amount}</th>
              <th>{i18n.t.pricing.status}</th>
              <th>{i18n.t.common.actions}</th>
            </tr>
          </thead>
          <tbody>
            {#each pagedSubscriptions as row (row.id)}
              <tr>
                <td>{row.page}</td>
                <td>{row.plan}</td>
                <td>{row.date}</td>
                <td>{row.amount}</td>
                <td><StatusBadge status={row.status} /></td>
                <td>
                  {#if row.canRenew}
                    <button class="btn btn-primary btn-sm" onclick={() => openPlanFlow(null)}>
                      {i18n.t.pricing.renewBtn}
                    </button>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
      <Pagination bind:current={currentPage} total={totalPages} totalRows={data.subscriptions.length} />
    </div>
  {/if}

  <!-- ── TAB: Plan Pricing ── -->
  {#if activeTab === 'plans'}
    <div class="plans-header">
      <h2>{i18n.t.pricing.plansHeader}</h2>
      <p>{i18n.t.pricing.plansSub}</p>
    </div>

    <div class="plans-grid">
      {#each data.plans as plan (plan.id)}
        <div class="plan-card {plan.popular ? 'featured' : ''} {plan.current ? 'current' : ''}">
          {#if plan.popular}
            <div class="popular-badge">{i18n.t.pricing.popular}</div>
          {/if}

          <div class="plan-name">{plan.name}</div>
          <div class="plan-price">
            <span class="price-num">{plan.price}</span>
            <span class="price-unit">/ {i18n.t.pricing.monthly}</span>
          </div>
          <p class="plan-desc">{plan.desc}</p>

          <ul class="plan-features">
            {#each plan.features as feat}
              <li>
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" class="check">
                  <polyline points="20 6 9 17 4 12"/>
                </svg>
                {feat}
              </li>
            {/each}
          </ul>

          {#if plan.current}
            <button class="btn btn-outline plan-btn" disabled>{i18n.t.pricing.currentPlan}</button>
          {:else}
            <button class="btn btn-primary plan-btn" onclick={() => openPlanFlow(plan)}>
              {i18n.t.pricing.changePlan}
            </button>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>

<!-- Checkout Modals Flow -->
<PageSelectorModal
  open={showPageModal}
  pages={data.pages}
  onClose={onPageClose}
  onNext={onPageNext}
/>

<PaymentModal
  open={showPaymentModal}
  onClose={onPaymentClose}
  onConfirm={onPaymentConfirm}
/>

<style>
  /* Container logical adjustments */
  .pricing-container {
    animation: fadeIn 0.3s ease-out;
  }

  /* ── Tabs ── */
  .pricing-tabs {
    display: flex; border-bottom: 1px solid var(--border); margin-bottom: 24px;
  }
  .tab-btn {
    padding: 11px 22px; font-size: 14px; font-weight: 500; color: var(--text-sec);
    cursor: pointer; border: none; background: none; font-family: var(--font);
    border-bottom: 2px solid transparent; margin-bottom: -1px; transition: all 0.15s;
    outline: none;
  }
  .tab-btn:hover  { color: var(--teal); }
  .tab-btn.active { color: var(--teal); border-color: var(--teal); font-weight: 600; }

  /* ── Summary ── */
  .summary-cards {
    display: grid; grid-template-columns: repeat(auto-fit, minmax(200px,1fr));
    gap: 14px; max-width: 500px; margin-bottom: 24px;
  }
  .summary-card {
    background: var(--white); border-radius: var(--radius);
    border: 1px solid var(--border); padding: 16px 20px;
    display: flex; align-items: center; justify-content: space-between;
    box-shadow: var(--shadow);
  }
  .summary-label { font-size: 12px; color: var(--text-sec); margin-bottom: 4px; text-align: var(--text-align); }
  .summary-value { font-size: 26px; font-weight: 700; color: var(--text); }
  .stat-icon { width: 44px; height: 44px; border-radius: 10px; display: flex; align-items: center; justify-content: center; font-size: 20px; }
  .icon-teal   { background: var(--teal-light); color: var(--teal); }
  .icon-orange { background: #fef3c7; color: var(--orange); }

  /* ── Plans ── */
  .plans-header { margin-bottom: 24px; }
  .plans-header h2 { font-size: 20px; font-weight: 800; text-align: var(--text-align); margin-bottom: 6px; }
  .plans-header p  { font-size: 13px; color: var(--text-sec); text-align: var(--text-align); line-height: 1.7; }

  .plans-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: 18px;
    margin-bottom: 24px;
  }
  .plan-card {
    background: var(--white); border-radius: var(--radius);
    border: 1.5px solid var(--border); padding: 24px;
    position: relative; transition: all 0.2s;
    display: flex; flex-direction: column;
  }
  .plan-card:hover { box-shadow: var(--shadow-md); }
  .plan-card.featured { border-color: var(--teal); box-shadow: 0 0 0 3px rgba(26,158,122,0.1); }

  .popular-badge {
    display: inline-block; background: var(--teal); color: #fff;
    font-size: 11px; font-weight: 700; padding: 3px 12px;
    border-radius: 20px; margin-bottom: 14px; align-self: flex-start;
  }
  .plan-name { font-size: 18px; font-weight: 700; color: var(--text); margin-bottom: 8px; text-align: var(--text-align); }
  .plan-price { display: flex; align-items: baseline; gap: 6px; margin-bottom: 4px; }
  .price-num  { font-size: 36px; font-weight: 800; color: var(--text); line-height: 1; }
  .price-unit { font-size: 14px; color: var(--text-sec); }
  .plan-desc  { font-size: 13px; color: var(--text-sec); margin: 10px 0 18px; line-height: 1.5; text-align: var(--text-align); }

  .plan-features { list-style: none; margin-bottom: 24px; flex: 1; }
  .plan-features li {
    display: flex; align-items: flex-start; gap: 8px;
    font-size: 13px; color: var(--text); padding: 6px 0;
    border-bottom: 1px solid var(--border-light);
  }
  .plan-features li:last-child { border-bottom: none; }
  .check { color: var(--teal); flex-shrink: 0; margin-top: 1px; }

  .plan-btn { width: 100%; justify-content: center; margin-top: auto; }

  @keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }

  @media (max-width: 600px) {
    .plans-grid { grid-template-columns: 1fr; }
  }
</style>
