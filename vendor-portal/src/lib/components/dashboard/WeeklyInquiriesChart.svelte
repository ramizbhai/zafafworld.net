<script lang="ts">
    import { onMount } from 'svelte';
    import { getWeeklyData } from '../../services/dashboard.service';

    let { i18n, inquiries } = $props<{
        i18n: any;
        inquiries: any[];
    }>();

    let chartCanvas = $state<HTMLCanvasElement | null>(null);

    onMount(() => {
        if (!chartCanvas) return;

        let destroyed = false;
        let chartRef: any = null;

        (async () => {
            const { Chart, registerables } = await import('chart.js');
            Chart.register(...registerables);
            if (destroyed || !chartCanvas) return;

            const weekly = getWeeklyData(inquiries, i18n.locale as any);
            const ctx = chartCanvas.getContext('2d');
            if (!ctx) return;

            const gradient = ctx.createLinearGradient(0, 0, 0, 200);
            gradient.addColorStop(0, 'rgba(26, 158, 122, 0.20)');
            gradient.addColorStop(1, 'rgba(26, 158, 122, 0.00)');

            chartRef = new Chart(ctx, {
                type: 'line',
                data: {
                    labels: weekly.labels,
                    datasets: [{
                        label: i18n.locale === 'ar' ? 'الاستفسارات' : 'Inquiries',
                        data: weekly.data,
                        borderColor: 'hsl(162, 72%, 40%)',
                        backgroundColor: gradient,
                        borderWidth: 2.5,
                        fill: true,
                        tension: 0.45,
                        pointBackgroundColor: 'hsl(162, 72%, 40%)',
                        pointBorderColor: '#fff',
                        pointBorderWidth: 2,
                        pointRadius: 4,
                        pointHoverRadius: 6,
                    }]
                },
                options: {
                    responsive: true,
                    maintainAspectRatio: false,
                    interaction: { intersect: false, mode: 'index' },
                    plugins: {
                        legend: { display: false },
                        tooltip: {
                            backgroundColor: 'rgba(15, 23, 42, 0.90)',
                            titleColor: '#fff',
                            bodyColor: 'rgba(255,255,255,0.75)',
                            padding: 10,
                            cornerRadius: 8,
                            displayColors: false,
                            titleFont: { size: 13, weight: 'bold', family: i18n.locale === 'ar' ? 'Cairo' : 'Inter' },
                            bodyFont: { size: 12, family: i18n.locale === 'ar' ? 'Cairo' : 'Inter' },
                        }
                    },
                    scales: {
                        x: {
                            grid: { display: false },
                            border: { display: false },
                            ticks: {
                                color: '#94a3b8',
                                font: { size: 12, family: i18n.locale === 'ar' ? 'Cairo' : 'Inter' }
                            }
                        },
                        y: {
                            beginAtZero: true,
                            grid: { color: 'rgba(226, 232, 240, 0.6)', drawBorder: false } as any,
                            border: { display: false, dash: [4, 4] },
                            ticks: {
                                color: '#94a3b8',
                                font: { size: 11, family: 'Inter' },
                                maxTicksLimit: 5,
                                precision: 0
                            }
                        }
                    }
                }
            });
        })();

        return () => {
            destroyed = true;
            chartRef?.destroy();
        };
    });
</script>

<div class="card chart-card">
    <div class="chart-card-header">
        <div>
            <h3 class="chart-title">
                {i18n.locale === 'ar' ? 'استفسارات الأسبوع' : 'Weekly Inquiries'}
            </h3>
            <p class="chart-subtitle">
                {i18n.locale === 'ar' ? 'حركة الطلبات خلال الأيام السبعة الماضية' : 'Request activity over the past 7 days'}
            </p>
        </div>
        <span class="chart-badge">
            {i18n.locale === 'ar' ? 'آخر 7 أيام' : 'Last 7 days'}
        </span>
    </div>
    <div class="chart-wrap">
        {#if inquiries.length === 0}
            <div class="empty-chart-state" style="display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; min-height: 200px; color: var(--color-text-muted); gap: 8px;">
                <span style="font-size: 24px;">📊</span>
                <span style="font-size: 13px; font-weight: 500;">
                    {i18n.locale === 'ar' ? 'لا توجد استفسارات كافية لعرضها' : 'No inquiries available to display'}
                </span>
            </div>
        {:else}
            <canvas bind:this={chartCanvas}></canvas>
        {/if}
    </div>
</div>
