<script lang="ts">
    import type { PageData } from './$types';
    import { TrendingUp, Users, Star, Package, MessageSquare, BarChart2, Activity } from 'lucide-svelte';

    let { data }: { data: PageData } = $props();
    import { getI18n } from '$lib/i18n/i18n.svelte';

    const i18n = getI18n();

    const metrics = $derived(data.metrics);
    const vendor  = $derived(data.vendor);
    const products = $derived(data.products ?? []);
    const leads   = $derived(data.leadInquiries ?? []);

    // ── KPI derivations ──────────────────────────────────────────────────────
    const activeHalls   = $derived(products.filter((p: any) => p.status === 'active').length);
    const totalHalls    = $derived(products.length);
    const activeLeads   = $derived(metrics?.activeLeadsCount ?? 0);
    const pendingPipe   = $derived(metrics?.totalPendingRevenuePipeline ?? 0);
    const totalReviews  = $derived(metrics?.total_reviews ?? 0);
    const avgRating     = $derived(metrics?.avg_overall ?? 0);
    const activePackages = $derived(metrics?.active_packages ?? 0);

    // ── Lead urgency breakdown ────────────────────────────────────────────────
    const urgencyBreakdown = $derived(
        leads.reduce((acc: Record<string, number>, lead: any) => {
            const index = lead.urgencyIndex ?? 'LOW';
            acc[index] = (acc[index] || 0) + 1;
            return acc;
        }, { CRITICAL: 0, HIGH: 0, MEDIUM: 0, LOW: 0 })
    );

    // ── Product status breakdown ──────────────────────────────────────────────
    const productStatusBreakdown = $derived(
        products.reduce((acc: Record<string, number>, p: any) => {
            const status = p.status ?? 'draft';
            acc[status] = (acc[status] || 0) + 1;
            return acc;
        }, { active: 0, draft: 0, pending_approval: 0, suspended: 0, archived: 0 })
    );

    // ── Rating axis breakdown ─────────────────────────────────────────────────
    const ratingAxes = $derived([
        { label: i18n.t.statisticsInsights.quality, value: metrics?.avg_quality ?? 0, color: '#10b981' },
        { label: i18n.t.statisticsInsights.staff, value: metrics?.avg_staff ?? 0, color: '#f59e0b' },
        { label: i18n.t.statisticsInsights.communication, value: metrics?.avg_communication ?? 0, color: '#6366f1' }
    ]);

    function formatSAR(n: number): string {
        const suffix = i18n.locale === 'ar' ? 'ر.س' : 'SAR';
        if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}${i18n.locale === 'ar' ? ' مليون ' : 'M '}${suffix}`;
        if (n >= 1_000)     return `${(n / 1_000).toFixed(0)}${i18n.locale === 'ar' ? ' ألف ' : 'K '}${suffix}`;
        return `${Math.round(n)} ${suffix}`;
    }

    function starWidth(v: number): string {
        return `${Math.min(100, (v / 5) * 100)}%`;
    }

    const statusColors: Record<string, string> = {
        active:           '#10b981',
        draft:            '#64748b',
        pending_approval: '#f59e0b',
        suspended:        '#ef4444',
        archived:         '#374151',
    };
</script>

<svelte:head>
    <title>{i18n.t.statisticsInsights.title} — {i18n.t.common.appName}</title>
</svelte:head>

<div class="stats-page" dir={i18n.locale === 'ar' ? 'rtl' : 'ltr'}>
    <!-- ─── HEADER ──────────────────────────────────────────────────────── -->
    <div class="page-header">
        <div class="header-text">
            <span class="page-tag">{i18n.t.statisticsInsights.tag}</span>
            <h1>{i18n.t.statisticsInsights.title}</h1>
            <p class="subtitle">{i18n.t.statisticsInsights.subtitle}</p>
        </div>
        {#if vendor}
            <div class="vendor-chip">
                <span class="vendor-name">{i18n.locale === 'ar' ? (vendor.name_ar || vendor.name_en) : vendor.name_en}</span>
                <span class="vendor-status" class:status-active={vendor.status === 'active'}>
                    {vendor.status === 'active' ? (i18n.locale === 'ar' ? 'نشط' : 'Active') : (vendor.status === 'suspended' ? (i18n.locale === 'ar' ? 'معلق' : 'Suspended') : vendor.status)}
                </span>
            </div>
        {/if}
    </div>

    {#if !metrics}
        <div class="empty-state">
            <Activity size={36} />
            <h2>{i18n.t.statisticsInsights.noDataTitle}</h2>
            <p>{i18n.t.statisticsInsights.noDataDesc}</p>
        </div>
    {:else}
        <!-- ─── KPI GRID ──────────────────────────────────────────────── -->
        <div class="kpi-grid">
            <div class="kpi-card kpi-leads">
                <div class="kpi-icon"><MessageSquare size={22} /></div>
                <div class="kpi-body">
                    <span class="kpi-label">{i18n.t.statisticsInsights.activeLeads}</span>
                    <span class="kpi-value">{activeLeads}</span>
                    <span class="kpi-sub">{i18n.t.statisticsInsights.pendingResponse}</span>
                </div>
            </div>
            <div class="kpi-card kpi-pipeline">
                <div class="kpi-icon"><TrendingUp size={22} /></div>
                <div class="kpi-body">
                    <span class="kpi-label">{i18n.t.statisticsInsights.revenuePipeline}</span>
                    <span class="kpi-value">{formatSAR(pendingPipe)}</span>
                    <span class="kpi-sub">{i18n.t.statisticsInsights.pendingBookings}</span>
                </div>
            </div>
            <div class="kpi-card kpi-halls">
                <div class="kpi-icon"><Package size={22} /></div>
                <div class="kpi-body">
                    <span class="kpi-label">{i18n.t.statisticsInsights.activeHalls}</span>
                    <span class="kpi-value">{activeHalls} <span class="kpi-of">/ {totalHalls}</span></span>
                    <span class="kpi-sub">{i18n.t.statisticsInsights.liveMarketplace}</span>
                </div>
            </div>
            <div class="kpi-card kpi-reviews">
                <div class="kpi-icon"><Star size={22} /></div>
                <div class="kpi-body">
                    <span class="kpi-label">{i18n.t.statisticsInsights.overallRating}</span>
                    <span class="kpi-value">{avgRating.toFixed(1)} <span class="kpi-of">/ 5</span></span>
                    <span class="kpi-sub">{i18n.t.statisticsInsights.approvedReviews.replace('{count}', totalReviews.toString())}</span>
                </div>
            </div>
            <div class="kpi-card kpi-packages">
                <div class="kpi-icon"><BarChart2 size={22} /></div>
                <div class="kpi-body">
                    <span class="kpi-label">{i18n.t.statisticsInsights.activePackages}</span>
                    <span class="kpi-value">{activePackages}</span>
                    <span class="kpi-sub">{i18n.t.statisticsInsights.promotionalOffers}</span>
                </div>
            </div>
            <div class="kpi-card kpi-users">
                <div class="kpi-icon"><Users size={22} /></div>
                <div class="kpi-body">
                    <span class="kpi-label">{i18n.t.statisticsInsights.leadInquiries}</span>
                    <span class="kpi-value">{leads.length}</span>
                    <span class="kpi-sub">{i18n.t.statisticsInsights.totalReceived}</span>
                </div>
            </div>
        </div>

        <!-- ─── CHARTS ROW ────────────────────────────────────────────── -->
        <div class="charts-row">
            <!-- Lead Urgency Breakdown -->
            <div class="chart-card">
                <h3 class="chart-title">{i18n.t.statisticsInsights.urgencyDistribution}</h3>
                <p class="chart-sub">{i18n.t.statisticsInsights.dateProximity}</p>
                <div class="urgency-bars">
                    {#each [
                        { key: 'CRITICAL', label: i18n.t.statisticsInsights.urgencyCritical,  color: '#ef4444' },
                        { key: 'HIGH',     label: i18n.t.statisticsInsights.urgencyHigh,     color: '#f97316' },
                        { key: 'MEDIUM',   label: i18n.t.statisticsInsights.urgencyMedium,   color: '#f59e0b' },
                        { key: 'LOW',      label: i18n.t.statisticsInsights.urgencyLow,      color: '#10b981' },
                    ] as item}
                        {@const count = urgencyBreakdown[item.key] ?? 0}
                        {@const pct = leads.length > 0 ? Math.round((count / leads.length) * 100) : 0}
                        <div class="urgency-row">
                            <div class="urgency-label-group">
                                <span class="urgency-dot" style="background:{item.color}"></span>
                                <span class="urgency-label">{item.label}</span>
                            </div>
                            <div class="urgency-bar-track">
                                <div class="urgency-bar-fill" style="width:{pct}%;background:{item.color}"></div>
                            </div>
                            <span class="urgency-count">{count}</span>
                        </div>
                    {/each}
                </div>
            </div>

            <!-- Rating Axis Breakdown -->
            <div class="chart-card">
                <h3 class="chart-title">{i18n.t.statisticsInsights.reviewScoreBreakdown}</h3>
                <p class="chart-sub">{i18n.t.statisticsInsights.avgReviews.replace('{count}', totalReviews.toString())}</p>
                <div class="rating-axes">
                    {#each ratingAxes as axis}
                        <div class="axis-row">
                            <span class="axis-label">{axis.label}</span>
                            <div class="axis-track">
                                <div class="axis-fill" style="width:{starWidth(axis.value)};background:{axis.color}"></div>
                            </div>
                            <span class="axis-score" style="color:{axis.color}">{axis.value.toFixed(1)}</span>
                        </div>
                    {/each}

                    <div class="overall-rating">
                        <span class="overall-label">{i18n.t.statisticsInsights.overall}</span>
                        <div class="stars-row">
                            {#each [1,2,3,4,5] as star}
                                <span class="star" class:filled={star <= Math.round(avgRating)}>★</span>
                            {/each}
                        </div>
                        <span class="overall-score">{avgRating.toFixed(1)}/5</span>
                    </div>
                </div>
            </div>

            <!-- Hall Status Breakdown -->
            <div class="chart-card">
                <h3 class="chart-title">{i18n.t.statisticsInsights.statusOverview}</h3>
                <p class="chart-sub">{i18n.t.statisticsInsights.totalHalls.replace('{count}', totalHalls.toString())}</p>
                <div class="hall-status-list">
                    {#if products.length === 0}
                        <p class="no-data">{i18n.t.statisticsInsights.noHallsConfigured}</p>
                    {:else}
                        {@const statusLabels = {
                            active: i18n.t.listings.statusActive,
                            draft: i18n.t.listings.statusDraft,
                            pending_approval: i18n.t.listings.statusPendingApproval,
                            suspended: i18n.t.listings.statusSuspended,
                            archived: i18n.t.listings.statusArchived
                        } as Record<string, string>}
                        {#each Object.entries(productStatusBreakdown) as [status, count]}
                            {@const color = statusColors[status] ?? '#64748b'}
                            <div class="status-row">
                                <div class="status-indicator" style="background:{color}"></div>
                                <span class="status-name">{statusLabels[status] ?? status.replace('_', ' ')}</span>
                                <span class="status-count" style="color:{color}">{count}</span>
                            </div>
                        {/each}
                    {/if}
                </div>

                <!-- Mini hall list -->
                {#if products.length > 0}
                    <div class="mini-halls">
                        {#each products.slice(0, 4) as p}
                            <div class="mini-hall">
                                <span class="mini-hall-name">{i18n.locale === 'ar' ? (p.nameAr || p.nameEn) : p.nameEn}</span>
                                <span class="mini-hall-cap">
                                    {p.totalCapacity ?? '—'} {i18n.t.statisticsInsights.guests}
                                </span>
                                <span class="mini-hall-price">
                                    {p.basePriceSar ? `${Math.round(p.basePriceSar).toLocaleString()} ${i18n.locale === 'ar' ? 'ر.س' : 'SAR'}` : '—'}
                                </span>
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>
        </div>

        <!-- ─── RECENT LEADS TABLE ────────────────────────────────────── -->
        {#if leads.length > 0}
            {@const urgencyLabels = {
                CRITICAL: i18n.t.statisticsInsights.urgencyCritical,
                HIGH: i18n.t.statisticsInsights.urgencyHigh,
                MEDIUM: i18n.t.statisticsInsights.urgencyMedium,
                LOW: i18n.t.statisticsInsights.urgencyLow
            } as Record<string, string>}
            {@const leadStatusLabels = {
                new: i18n.t.couples.statusNew,
                negotiating: i18n.t.couples.statusNegot,
                done: i18n.t.couples.statusDone,
                expired: i18n.t.couples.statusExpired,
                rejected: i18n.t.couples.statusRejected,
                unreachable: i18n.t.couples.statusUnreach,
                paid: i18n.t.couples.statusPaid
            } as Record<string, string>}
            <div class="leads-section">
                <h3 class="section-title">{i18n.t.statisticsInsights.recentLeads}</h3>
                <div class="leads-table-wrap">
                    <table class="leads-table">
                        <thead>
                            <tr>
                                <th>{i18n.t.statisticsInsights.customer}</th>
                                <th>{i18n.t.statisticsInsights.weddingDate}</th>
                                <th>{i18n.t.statisticsInsights.urgency}</th>
                                <th>{i18n.t.statisticsInsights.status}</th>
                                <th>{i18n.t.statisticsInsights.responseWindow}</th>
                            </tr>
                        </thead>
                        <tbody>
                            {#each leads.slice(0, 8) as lead}
                                <tr>
                                    <td>
                                        <div class="lead-name-cell">
                                            <div class="lead-avatar">{lead.customerName?.[0] ?? '?'}</div>
                                            <div>
                                                <span class="lead-name">{lead.customerName}</span>
                                                <span class="lead-phone">{lead.phone}</span>
                                            </div>
                                        </div>
                                    </td>
                                    <td class="td-date">{lead.weddingDate}</td>
                                    <td>
                                        <span class="urgency-badge urgency-{(lead.urgencyIndex ?? 'LOW').toLowerCase()}">
                                            {urgencyLabels[lead.urgencyIndex ?? 'LOW'] ?? (lead.urgencyIndex ?? 'LOW')}
                                        </span>
                                    </td>
                                    <td>
                                        <span class="status-pill status-{lead.status}">{leadStatusLabels[lead.status] ?? lead.status}</span>
                                    </td>
                                    <td class="td-countdown">{lead.responseLatencyCountdown}</td>
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                </div>
            </div>
        {/if}
    {/if}
</div>

<style>
    .stats-page {
        display: flex;
        flex-direction: column;
        gap: 28px;
        max-width: 1300px;
        margin: 0 auto;
        animation: fade-in 0.35s ease-out;
    }

    @keyframes fade-in {
        from { opacity: 0; transform: translateY(8px); }
        to   { opacity: 1; transform: translateY(0); }
    }

    /* ─── HEADER ──────────────────────────────────────────────────────── */
    .page-header {
        display: flex;
        align-items: flex-end;
        justify-content: space-between;
        gap: 16px;
        padding-bottom: 20px;
        border-bottom: 1px solid var(--border);
    }

    .page-tag {
        font-size: 0.6rem;
        text-transform: uppercase;
        letter-spacing: 2px;
        font-weight: 700;
        color: var(--color-primary);
    }

    .page-header h1 {
        margin: 4px 0 0;
        font-size: 1.8rem;
        font-weight: 800;
        color: var(--text);
        letter-spacing: -0.5px;
    }

    .subtitle { margin: 4px 0 0; font-size: 0.85rem; color: var(--text-sec); }

    .vendor-chip {
        display: flex;
        align-items: center;
        gap: 8px;
        background: var(--white);
        border: 1px solid var(--border);
        border-radius: 10px;
        padding: 8px 14px;
        flex-shrink: 0;
        box-shadow: var(--shadow-sm);
    }

    .vendor-name { font-size: 0.85rem; font-weight: 600; color: var(--text); }
    .vendor-status {
        font-size: 0.65rem;
        font-weight: 700;
        text-transform: uppercase;
        padding: 2px 8px;
        border-radius: 6px;
        background: var(--border);
        color: var(--text-sec);
    }
    .vendor-status.status-active { background: rgba(16,185,129,0.15); color: #15803d; }

    /* ─── EMPTY STATE ─────────────────────────────────────────────────── */
    .empty-state {
        display: flex; flex-direction: column; align-items: center;
        justify-content: center; gap: 12px; padding: 4rem 2rem;
        border: 1px dashed var(--border); border-radius: 16px;
        background: var(--white); text-align: center; color: var(--text-sec);
    }
    .empty-state h2 { font-size: 1.2rem; font-weight: 700; color: var(--text); margin: 0; }
    .empty-state p  { margin: 0; font-size: 0.875rem; }

    /* ─── KPI GRID ────────────────────────────────────────────────────── */
    .kpi-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
        gap: 16px;
    }

    .kpi-card {
        display: flex;
        align-items: center;
        gap: 14px;
        background: var(--white);
        border: 1px solid var(--border);
        border-radius: 14px;
        padding: 18px;
        transition: border-color 0.2s, box-shadow 0.2s;
        box-shadow: var(--shadow);
    }

    .kpi-card:hover {
        border-color: rgba(91, 33, 182, 0.2);
        box-shadow: var(--shadow-md);
    }

    .kpi-icon {
        width: 44px; height: 44px;
        border-radius: 12px;
        display: flex; align-items: center; justify-content: center;
        flex-shrink: 0;
    }

    .kpi-leads   .kpi-icon { background: rgba(239,68,68,0.12);  color: #dc2626; }
    .kpi-pipeline .kpi-icon { background: rgba(91, 33, 182, 0.12); color: var(--color-primary); }
    .kpi-halls   .kpi-icon { background: rgba(16,185,129,0.12); color: #16a34a; }
    .kpi-reviews  .kpi-icon { background: rgba(251,191,36,0.12); color: #d97706; }
    .kpi-packages .kpi-icon { background: rgba(59,130,246,0.12); color: #2563eb; }
    .kpi-users   .kpi-icon { background: rgba(245,158,11,0.12); color: #d97706; }

    .kpi-body   { display: flex; flex-direction: column; gap: 2px; }
    .kpi-label  { font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-sec); font-weight: 600; }
    .kpi-value  { font-size: 1.4rem; font-weight: 800; color: var(--text); line-height: 1.1; }
    .kpi-of     { font-size: 0.85rem; font-weight: 500; color: var(--text-sec); }
    .kpi-sub    { font-size: 0.72rem; color: var(--text-sec); }

    /* ─── CHARTS ROW ──────────────────────────────────────────────────── */
    .charts-row {
        display: grid;
        grid-template-columns: repeat(3, 1fr);
        gap: 18px;
    }

    .chart-card {
        background: var(--white);
        border: 1px solid var(--border);
        border-radius: 14px;
        padding: 20px;
        display: flex;
        flex-direction: column;
        gap: 14px;
        box-shadow: var(--shadow);
    }

    .chart-title { margin: 0; font-size: 0.95rem; font-weight: 700; color: var(--text); }
    .chart-sub   { margin: -8px 0 0; font-size: 0.75rem; color: var(--text-sec); }

    /* Urgency bars */
    .urgency-bars { display: flex; flex-direction: column; gap: 10px; }
    .urgency-row  { display: flex; align-items: center; gap: 10px; }
    .urgency-label-group { display: flex; align-items: center; gap: 6px; width: 120px; flex-shrink: 0; }
    .urgency-dot  { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
    .urgency-label { font-size: 0.72rem; font-weight: 600; color: var(--text-sec); }
    .urgency-bar-track { flex: 1; background: var(--border-light); border-radius: 4px; height: 7px; overflow: hidden; }
    .urgency-bar-fill  { height: 100%; border-radius: 4px; transition: width 0.4s ease; }
    .urgency-count { font-size: 0.75rem; font-weight: 700; color: var(--text); width: 20px; text-align: end; flex-shrink: 0; }

    /* Rating axes */
    .rating-axes { display: flex; flex-direction: column; gap: 12px; }
    .axis-row    { display: flex; align-items: center; gap: 10px; }
    .axis-label  { font-size: 0.75rem; font-weight: 600; color: var(--text-sec); width: 100px; flex-shrink: 0; }
    .axis-track  { flex: 1; background: var(--border-light); border-radius: 4px; height: 7px; overflow: hidden; }
    .axis-fill   { height: 100%; border-radius: 4px; transition: width 0.5s ease; }
    .axis-score  { font-size: 0.75rem; font-weight: 700; width: 28px; text-align: end; flex-shrink: 0; }

    .overall-rating {
        display: flex; align-items: center; gap: 10px;
        padding-top: 12px; border-top: 1px solid var(--border-light);
        margin-top: 4px;
    }
    .overall-label { font-size: 0.75rem; font-weight: 700; color: var(--text); }
    .stars-row { display: flex; gap: 2px; }
    .star { font-size: 1rem; color: var(--border); transition: color 0.2s; }
    .star.filled { color: #fbbf24; }
    .overall-score { font-size: 0.8rem; font-weight: 700; color: #fbbf24; margin-inline-start: auto; }

    /* Hall status */
    .hall-status-list { display: flex; flex-direction: column; gap: 8px; }
    .status-row { display: flex; align-items: center; gap: 8px; }
    .status-indicator { width: 10px; height: 10px; border-radius: 50%; flex-shrink: 0; }
    .status-name  { flex: 1; font-size: 0.78rem; color: var(--text-sec); text-transform: capitalize; }
    .status-count { font-size: 0.85rem; font-weight: 700; color: var(--text); }
    .no-data { font-size: 0.8rem; color: var(--text-sec); text-align: center; padding: 12px 0; }

    .mini-halls { margin-top: 8px; border-top: 1px solid var(--border-light); padding-top: 12px; display: flex; flex-direction: column; gap: 8px; }
    .mini-hall  { display: flex; align-items: center; gap: 8px; font-size: 0.75rem; }
    .mini-hall-name  { flex: 1; color: var(--text); font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
    .mini-hall-cap   { color: var(--text-sec); white-space: nowrap; }
    .mini-hall-price { color: var(--color-primary); font-weight: 700; white-space: nowrap; }

    /* ─── LEADS TABLE ─────────────────────────────────────────────────── */
    .leads-section { display: flex; flex-direction: column; gap: 14px; }
    .section-title { margin: 0; font-size: 1rem; font-weight: 700; color: var(--text); }

    .leads-table-wrap { overflow-x: auto; border-radius: 14px; border: 1px solid var(--border); }
    .leads-table { width: 100%; border-collapse: collapse; }
    .leads-table th {
        background: var(--border-light);
        padding: 10px 16px;
        text-align: start;
        font-size: 0.68rem;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        color: var(--text);
        border-bottom: 1px solid var(--border);
    }
    .leads-table td {
        padding: 12px 16px;
        font-size: 0.82rem;
        color: var(--text);
        border-bottom: 1px solid var(--border-light);
        vertical-align: middle;
    }
    .leads-table tr:last-child td { border-bottom: none; }
    .leads-table tr:hover td { background: var(--bg); }

    .lead-name-cell { display: flex; align-items: center; gap: 10px; }
    .lead-avatar {
        width: 32px; height: 32px; border-radius: 50%;
        background: rgba(91, 33, 182, 0.1); border: 1px solid rgba(91, 33, 182, 0.2);
        display: flex; align-items: center; justify-content: center;
        font-size: 0.8rem; font-weight: 700; color: var(--color-primary); flex-shrink: 0;
    }
    .lead-name  { display: block; font-weight: 600; }
    .lead-phone { display: block; font-size: 0.7rem; color: var(--text-sec); }

    .td-date     { color: var(--text-sec); }
    .td-countdown { font-size: 0.75rem; color: var(--text-sec); }

    .urgency-badge {
        display: inline-block; font-size: 0.62rem; font-weight: 800;
        padding: 2px 7px; border-radius: 4px; text-transform: uppercase;
    }
    .urgency-critical { background: rgba(239,68,68,0.12); color: #dc2626; }
    .urgency-high     { background: rgba(249,115,22,0.12); color: #ea580c; }
    .urgency-medium   { background: rgba(245,158,11,0.12); color: #d97706; }
    .urgency-low      { background: rgba(16,185,129,0.12); color: #16a34a; }

    .status-pill {
        display: inline-block; font-size: 0.65rem; font-weight: 700;
        padding: 2px 8px; border-radius: 6px; text-transform: capitalize;
        background: var(--border); color: var(--text-sec);
    }
    .status-pill.status-pending  { background: rgba(245,158,11,0.12);  color: #d97706; }
    .status-pill.status-won      { background: rgba(16,185,129,0.12);  color: #16a34a; }
    .status-pill.status-lost     { background: rgba(239,68,68,0.12);   color: #dc2626; }
    .status-pill.status-closed   { background: var(--border); color: var(--text-sec); }

    @media (max-width: 1100px) { .charts-row { grid-template-columns: 1fr 1fr; } }
    @media (max-width: 768px)  { .charts-row { grid-template-columns: 1fr; } .kpi-grid { grid-template-columns: 1fr 1fr; } }
    @media (max-width: 480px)  { .kpi-grid { grid-template-columns: 1fr; } }
</style>
