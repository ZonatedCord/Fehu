<script lang="ts">
  import { onMount } from 'svelte';
  import { Chart, registerables } from 'chart.js';
  import { SankeyController, Flow } from 'chartjs-chart-sankey';
  import type { DashboardStats } from '../lib/types';

  Chart.register(...registerables, SankeyController, Flow);

  let { stats }: { stats: DashboardStats } = $props();

  let canvas = $state<HTMLCanvasElement | null>(null);
  let chart: Chart | null = null;

  function buildData() {
    const flows: { from: string; to: string; flow: number }[] = [];
    const colors: Record<string, string> = {};

    const source = 'Entrate';
    colors[source] = '#4ade80';

    for (const cat of stats.by_category) {
      const name = cat.category_name ?? 'Senza categoria';
      if (cat.total > 0) {
        flows.push({ from: source, to: name, flow: Math.round(cat.total * 100) / 100 });
        colors[name] = cat.color ?? '#6366f1';
      }
    }

    const savings = stats.total_income - stats.total_expense;
    if (savings > 0.01) {
      flows.push({ from: source, to: 'Risparmio', flow: Math.round(savings * 100) / 100 });
      colors['Risparmio'] = '#a5b4fc';
    }

    return { flows, colors };
  }

  function render() {
    if (!canvas || stats.by_category.length === 0) return;
    if (chart) { chart.destroy(); chart = null; }

    const { flows, colors } = buildData();
    if (flows.length === 0) return;

    chart = new Chart(canvas, {
      type: 'sankey' as any,
      data: {
        datasets: [{
          label: 'Flusso denaro',
          data: flows,
          colorFrom: (c: any) => colors[c.dataset.data[c.dataIndex]?.from] ?? '#4ade80',
          colorTo: (c: any) => colors[c.dataset.data[c.dataIndex]?.to] ?? '#6366f1',
          colorMode: 'gradient',
          borderWidth: 0,
        }],
      },
      options: {
        responsive: true,
        plugins: {
          legend: { display: false },
          tooltip: {
            callbacks: {
              label: (ctx: any) => {
                const d = ctx.dataset.data[ctx.dataIndex];
                return `${d.from} → ${d.to}: €${d.flow.toFixed(2)}`;
              },
            },
          },
        },
      },
    });
  }

  onMount(() => {
    render();
    return () => { if (chart) chart.destroy(); };
  });

  $effect(() => {
    stats;
    render();
  });
</script>

{#if stats.by_category.length === 0}
  <p class="muted">Aggiungi uscite per categoria per vedere il flusso</p>
{:else}
  <canvas bind:this={canvas}></canvas>
{/if}

<style>
  .muted { color: var(--text-dim); font-size: 0.85rem; }
</style>
