export function createVendorAdminState() {
  let isSubmittingVendor = $state(false);
  let isSubmittingProduct = $state<string | null>(null);
  let statusReason = $state("");
  let productReason = $state("");
  
  let showStatusModal = $state(false);
  let pendingVendorStatus = $state("");
  
  let showPromoteModal = $state(false);
  let promoteProductItem = $state<any>(null);
  let promoteDays = $state(30);

  function openStatusModal(status: string) {
    pendingVendorStatus = status;
    statusReason = "";
    showStatusModal = true;
  }

  function openPromoteModal(product: any) {
    promoteProductItem = product;
    promoteDays = 30;
    showPromoteModal = true;
  }

  return {
    get isSubmittingVendor() { return isSubmittingVendor; },
    set isSubmittingVendor(v) { isSubmittingVendor = v; },

    get isSubmittingProduct() { return isSubmittingProduct; },
    set isSubmittingProduct(v) { isSubmittingProduct = v; },

    get statusReason() { return statusReason; },
    set statusReason(v) { statusReason = v; },

    get productReason() { return productReason; },
    set productReason(v) { productReason = v; },

    get showStatusModal() { return showStatusModal; },
    set showStatusModal(v) { showStatusModal = v; },

    get pendingVendorStatus() { return pendingVendorStatus; },
    set pendingVendorStatus(v) { pendingVendorStatus = v; },

    get showPromoteModal() { return showPromoteModal; },
    set showPromoteModal(v) { showPromoteModal = v; },

    get promoteProductItem() { return promoteProductItem; },
    set promoteProductItem(v) { promoteProductItem = v; },

    get promoteDays() { return promoteDays; },
    set promoteDays(v) { promoteDays = v; },

    openStatusModal,
    openPromoteModal
  };
}

export type VendorAdminState = ReturnType<typeof createVendorAdminState>;

// Later on, if a global admin state is needed for system health, it can be exported here too.

import {
  TrendingUp, Users, Building2, CalendarCheck, Clock,
  CheckCircle2, AlertCircle, Activity
} from 'lucide-svelte';

export class AdminDashboardState {
  summary = $state<any>(null);
  dashboard = $state<any>(null);
  approvals = $state<any[]>([]);

  selectedMarket = $state<string | null>(null);
  chartHovered = $state<number | null>(null);

  constructor(data: any) {
    this.summary = data.summary;
    this.dashboard = data.dashboard;
    this.approvals = data.approvals || [];
  }

  kpis = $derived.by(() => {
    return [
      {
        key: 'dash.total_revenue',
        value_ar: `${(this.summary?.total_revenue ?? 0).toLocaleString('ar-SA')} ر.س`,
        value_en: `SAR ${(this.summary?.total_revenue ?? 0).toLocaleString('en-US')}`,
        icon: TrendingUp,
        iconClass: 'icon-gold',
        edgeColor: 'var(--gold)',
        trend: this.summary?.monthly_revenue && this.summary.monthly_revenue.length >= 2
          ? `${(((this.summary.monthly_revenue[this.summary.monthly_revenue.length - 1].revenue - this.summary.monthly_revenue[this.summary.monthly_revenue.length - 2].revenue) / (this.summary.monthly_revenue[this.summary.monthly_revenue.length - 2].revenue || 1)) * 100).toFixed(1)}%`
          : '0.0%',
        trendDir: 'up',
        sub_ar: 'مقارنة بالشهر الماضي',
        sub_en: 'vs last month',
      },
      {
        key: 'dash.active_vendors',
        value_ar: (this.dashboard?.total_active_vendors ?? 0).toLocaleString('ar-SA'),
        value_en: (this.dashboard?.total_active_vendors ?? 0).toLocaleString('en-US'),
        icon: Building2,
        iconClass: 'icon-purple',
        edgeColor: 'var(--purple)',
        trend: (this.summary?.regional_distribution?.length ?? 0) > 0 ? 'نشط' : 'خامل',
        trend_en: (this.summary?.regional_distribution?.length ?? 0) > 0 ? 'Active' : 'Idle',
        trendDir: 'up',
        sub_ar: `في ${this.summary?.regional_distribution?.length ?? 0} أسواق`,
        sub_en: `across ${this.summary?.regional_distribution?.length ?? 0} markets`,
      },
      {
        key: 'dash.active_subscriptions',
        value_ar: (this.dashboard?.active_subscriptions_count ?? 0).toLocaleString('ar-SA'),
        value_en: (this.dashboard?.active_subscriptions_count ?? 0).toLocaleString('en-US'),
        icon: CalendarCheck,
        iconClass: 'icon-success',
        edgeColor: 'var(--success)',
        trend: 'مباشر',
        trend_en: 'Live',
        trendDir: 'up',
        sub_ar: 'النشطة والتجريبية',
        sub_en: 'active & trial',
      },
      {
        key: 'dash.total_inquiries',
        value_ar: (this.dashboard?.total_inquiries_count ?? 0).toLocaleString('ar-SA'),
        value_en: (this.dashboard?.total_inquiries_count ?? 0).toLocaleString('en-US'),
        icon: Users,
        iconClass: 'icon-info',
        edgeColor: 'var(--info)',
        trend: 'منذ البداية',
        trend_en: 'all time',
        trendDir: 'up',
        sub_ar: 'استفسارات المستخدمين',
        sub_en: 'user inquiries',
      },
      {
        key: 'dash.pending_reviews',
        value_ar: (this.dashboard?.pending_reviews_count ?? 0).toLocaleString('ar-SA'),
        value_en: (this.dashboard?.pending_reviews_count ?? 0).toLocaleString('en-US'),
        icon: Clock,
        iconClass: 'icon-warning',
        edgeColor: 'var(--warning)',
        trend: (this.dashboard?.pending_reviews_count ?? 0) > 0 ? 'عاجل' : 'مستقر',
        trend_en: (this.dashboard?.pending_reviews_count ?? 0) > 0 ? 'Urgent' : 'Stable',
        trendDir: (this.dashboard?.pending_reviews_count ?? 0) > 0 ? 'alert' : 'flat',
        sub_ar: 'تتطلب مراجعة فورية',
        sub_en: 'require immediate action',
      },
      {
        key: 'dash.pending_approvals',
        value_ar: (this.summary?.pending_approvals_count ?? 0).toLocaleString('ar-SA'),
        value_en: (this.summary?.pending_approvals_count ?? 0).toLocaleString('en-US'),
        icon: Clock,
        iconClass: 'icon-danger',
        edgeColor: 'var(--danger)',
        trend: (this.summary?.pending_approvals_count ?? 0) > 0 ? 'مراجعة' : 'واضح',
        trend_en: (this.summary?.pending_approvals_count ?? 0) > 0 ? 'Review' : 'Clear',
        trendDir: (this.summary?.pending_approvals_count ?? 0) > 0 ? 'alert' : 'flat',
        sub_ar: 'طلبات انضمام الموردين',
        sub_en: 'supplier onboarding requests',
      },
    ];
  });

  markets = $derived.by(() => {
    return (this.summary?.regional_distribution || []).map((m: any) => {
      const totalBase = this.summary?.total_bookings_count || this.summary?.active_vendors_count || 1;
      const count = m.bookings_count ?? m.active_vendors_count ?? 0;
      const regionName = m.region ?? m.country_name_en ?? 'Unknown';
      const share = Math.round((count / Math.max(totalBase, 1)) * 100);
      return {
        code: m.country_id ?? regionName.substring(0, 3).toUpperCase(),
        name_ar: m.country_name_ar ?? regionName,
        name_en: m.country_name_en ?? regionName,
        vendors: count,
        share: share || 0
      };
    });
  });

  revenueMonths = $derived((this.summary?.monthly_revenue || []).map((m: any) => m.month_ar));
  revenueMonthsEn = $derived((this.summary?.monthly_revenue || []).map((m: any) => m.month_en));
  revenueData = $derived((this.summary?.monthly_revenue || []).map((m: any) => m.revenue));
  maxRevenue = $derived(Math.max(...this.revenueData, 1));
  revenueAverage = $derived(this.revenueData.length > 0 ? this.revenueData.reduce((a: number, b: number) => a + b, 0) / this.revenueData.length : 0);
  peakMonthIndex = $derived(this.revenueData.length > 0 ? this.revenueData.indexOf(Math.max(...this.revenueData)) : -1);
  yoyGrowthText = $derived(this.summary?.yoy_growth ?? 'N/A');

  getActivityIcon(eventType: string) {
    switch (eventType) {
      case 'vendor_approved':    return { icon: CheckCircle2, iconClass: 'act-success' };
      case 'vendor_suspended':   return { icon: AlertCircle,  iconClass: 'act-danger' };
      case 'vendor_rejected':    return { icon: AlertCircle,  iconClass: 'act-danger' };
      case 'review_approved':    return { icon: CheckCircle2, iconClass: 'act-success' };
      case 'review_rejected':    return { icon: AlertCircle,  iconClass: 'act-danger' };
      case 'system_alert':       return { icon: AlertCircle,  iconClass: 'act-danger' };
      default:                   return { icon: Activity,     iconClass: 'act-purple' };
    }
  }

  activities = $derived.by(() => {
    return (this.summary?.recent_activities ?? []).map((ev: any) => {
      const { icon, iconClass } = this.getActivityIcon(ev.event_type);
      return { icon, iconClass, msg_ar: ev.message_ar, msg_en: ev.message_en, time: ev.relative_time };
    });
  });

  queue = $derived.by(() => {
    return this.approvals.slice(0, 4).map((item: any, idx: number) => {
      let relativeTime = 'recent';
      if (item.created_at) {
        const diffMs = new Date().getTime() - new Date(item.created_at).getTime();
        const diffHrs = Math.floor(diffMs / (1000 * 60 * 60));
        if (diffHrs < 1) relativeTime = '1h';
        else if (diffHrs < 24) relativeTime = `${diffHrs}h`;
        else relativeTime = `${Math.floor(diffHrs / 24)}d`;
      }
      return {
        priority: idx === 0 ? 'critical' : 'pending',
        name_ar: item.name_ar,
        name_en: item.name_en,
        city_ar: item.city_name_ar,
        city_en: item.city_name_en,
        time: relativeTime
      };
    });
  });
}
