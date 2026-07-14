<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { getI18n } from '$lib/i18n/i18n.svelte';

  // Svelte 5 Props
  let {
    title = '',
    data = [] as number[],
    labels = [] as string[],
    color = '#1a9e7a'
  } = $props<{
    title: string;
    data: number[];
    labels: string[];
    color?: string;
  }>();

  const i18n = getI18n();

  let canvas = $state<HTMLCanvasElement | null>(null);
  let chart: any = null;

  async function createChart() {
    if (chart) {
      chart.destroy();
      chart = null;
    }

    if (!canvas) return;

    const { Chart, registerables } = await import('chart.js');
    Chart.register(...registerables);

    const activeFont = i18n.locale === 'ar' ? 'Cairo' : 'Inter';

    chart = new Chart(canvas, {
      type: 'bar',
      data: {
        labels,
        datasets: [{
          data,
          backgroundColor: color,
          borderRadius: 6,
          borderSkipped: false,
        }]
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          legend: { display: false }
        },
        scales: {
          x: {
            grid: { display: false },
            ticks: {
              font: { size: 10, family: activeFont },
              color: '#64748b'
            }
          },
          y: {
            grid: { color: '#f1f5f9' },
            ticks: {
              font: { size: 10, family: activeFont },
              color: '#64748b'
            },
            max: 80,
            beginAtZero: true,
          }
        }
      }
    });
  }

  onMount(() => {
    createChart();
  });

  // Re-render chart on locale or data change to update fonts and values
  $effect(() => {
    if (canvas && (i18n.locale || data.length > 0)) {
      createChart();
    }
  });

  onDestroy(() => {
    if (chart) chart.destroy();
  });
</script>

<div class="chart-card">
  <div class="chart-title">{title}</div>
  <div class="chart-wrap">
    <canvas bind:this={canvas}></canvas>
  </div>
</div>

<style>
.chart-card {
  background: var(--white); border-radius: var(--radius);
  border: 1px solid var(--border); padding: 20px; box-shadow: var(--shadow);
}
.chart-title { font-size: 14.5px; font-weight: 700; color: var(--text); margin-bottom: 16px; text-align: var(--text-align); }
.chart-wrap  { position: relative; height: 220px; }
</style>
