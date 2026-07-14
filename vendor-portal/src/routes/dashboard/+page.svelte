<script lang="ts">
    import { getI18n } from '$lib/i18n/i18n.svelte';
    import { createDashboardState } from '$lib/stores/dashboardState.svelte';

    import StatCard from '$lib/components/StatCard.svelte';
    import SubscriptionOverview from '$lib/components/SubscriptionOverview.svelte';
    
    import WelcomeExperience from '$lib/components/dashboard/WelcomeExperience.svelte';
    import DashboardStatusBanner from '$lib/components/dashboard/DashboardStatusBanner.svelte';
    import WeeklyInquiriesChart from '$lib/components/dashboard/WeeklyInquiriesChart.svelte';
    import ReviewPerformanceCard from '$lib/components/dashboard/ReviewPerformanceCard.svelte';
    import RecentInquiriesTable from '$lib/components/dashboard/RecentInquiriesTable.svelte';

    import '$lib/components/dashboard/styles.css';

    let { data } = $props();
    const i18n = getI18n();
    const dashboardState = createDashboardState(() => data);

    $effect(() => {
        dashboardState.initStreams();
    });

    let stats = $derived<Array<{
        label: string;
        value: string | number;
        icon: string;
        color: 'teal' | 'blue' | 'orange' | 'red' | 'purple' | 'gold';
        trend?: number;
        suffix?: string;
        trendLabel?: string;
        premiumBorder?: boolean;
    }>>([
        {
            label: i18n.t.dashboard.offerCount,
            value: dashboardState.metrics.active_packages ?? 0,
            icon: '🏷',
            color: 'teal' as const
        },
        {
            label: i18n.t.dashboard.activeProducts,
            value: dashboardState.metrics.active_products ?? 0,
            icon: '🏛',
            color: 'blue' as const,
            premiumBorder: true
        },
        {
            label: i18n.t.dashboard.pendingRequests,
            value: dashboardState.metrics.activeLeadsCount ?? 0,
            icon: '💬',
            color: 'orange' as const
        },
        {
            label: i18n.t.dashboard.revenuePipeline,
            value: dashboardState.metrics.totalPendingRevenuePipeline ?? 0,
            icon: '💰',
            color: 'purple' as const,
            suffix: i18n.locale === 'ar' ? 'ريال' : 'SAR',
            premiumBorder: true
        },
        {
            label: i18n.t.dashboard.totalReviews,
            value: dashboardState.metrics.total_reviews ?? 0,
            icon: '⭐',
            color: 'gold' as const
        },
    ]);
</script>

<svelte:head>
    <title>{i18n.t.nav.dashboard} – {i18n.t.common.appName}</title>
</svelte:head>

<div class="dashboard-home stagger">
    {#if !dashboardState.hasListings}
        <WelcomeExperience {i18n} userFirstName={data.user?.first_name ?? (i18n.locale === 'ar' ? 'شريكنا' : 'Partner')} />
    {:else}
        <DashboardStatusBanner {i18n} vendorStatus={dashboardState.vendor.status} />

        <SubscriptionOverview vendor={dashboardState.vendor} />

        <div class="stats-grid">
            {#each stats as s}
                <StatCard
                    label={s.label}
                    value={s.value}
                    icon={s.icon}
                    color={s.color}
                    trend={s.trend}
                    suffix={s.suffix}
                    premiumBorder={s.premiumBorder}
                    trendLabel={i18n.locale === 'ar' ? 'هذا الأسبوع' : 'this week'}
                />
            {/each}
        </div>

        <div class="insights-grid">
            <WeeklyInquiriesChart {i18n} inquiries={dashboardState.inquiries} />
            <ReviewPerformanceCard {i18n} metrics={dashboardState.metrics} />
        </div>

        <RecentInquiriesTable {i18n} {dashboardState} />
    {/if}
</div>
