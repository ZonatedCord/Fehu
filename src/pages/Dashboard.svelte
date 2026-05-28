<script lang="ts">
  import { onMount } from 'svelte';
  import { Chart, registerables } from 'chart.js';
  import { api } from '../lib/api';
  import type { DashboardStats } from '../lib/types';

  Chart.register(...registerables);

  let stats = $state<DashboardStats | null>(null);
  let error = $state('');
  let monthlyCanvas = $state<HTMLCanvasElement | null>(null);
  let categoryCanvas = $state<HTMLCanvasElement | null>(null);
  let monthlyChart: Chart | null = null;
  let categoryChart: Chart | null = null;

  async function load() {
    try {
      stats = await api.getDashboardStats();
      await new Promise(r => setTimeout(r, 0));
      renderCharts();
    } catch (e: any) { error = e.message ?? String(e); }
  }

  function renderCharts() {
    if (!stats) return;
    if (monthlyChart) { monthlyChart.destroy(); monthlyChart = null; }
    if (categoryChart) { categoryChart.destroy(); categoryChart = null; }

    if (monthlyCanvas && stats.monthly.length > 0) {
      monthlyChart = new Chart(monthlyCanvas, {
        type: 'bar',
        data: {
          labels: stats.monthly.map(m => m.month),
          datasets: [
            { label: 'Income', data: stats.monthly.map(m => m.income), backgroundColor: '#4ade8066', borderColor: '#4ade80', borderWidth: 1 },
            { label: 'Expense', data: stats.monthly.map(m => m.expense), backgroundColor: '#f8717166', borderColor: '#f87171', borderWidth: 1 },
          ],
        },
        options: {
          responsive: true,
          plugins: { legend: { labels: { color: '#ccc' } } },
          scales: {
            x: { ticks: { color: '#888' }, grid: { color: '#2e2e4e' } },
            y: { ticks: { color: '#888' }, grid: { color: '#2e2e4e' } },
          },
        },
      });
    }

    if (categoryCanvas && stats.by_category.length > 0) {
      categoryChart = new Chart(categoryCanvas, {
        type: 'doughnut',
        data: {
          labels: stats.by_category.map(c => c.category_name ?? 'Uncategorized'),
          datasets: [{ data: stats.by_category.map(c => c.total), backgroundColor: stats.by_category.map(c => c.color ?? '#6366f1'), borderWidth: 0 }],
        },
        options: { responsive: true, plugins: { legend: { labels: { color: '#ccc' } } } },
      });
    }
  }

  onMount(load);

  function fmt(n: number) {
    return new Intl.NumberFormat('it-IT', { style: 'currency', currency: 'EUR' }).format(n);
  }
</script>

<div class="page">
  <div class="page-header">
    <h1>Dashboard</h1>
    <button class="btn-refresh" onclick={load}>↻ Refresh</button>
  </div>
  {#if error}<p class="error">{error}</p>{/if}
  {#if stats}
    <div class="kpi-row">
      <div class="kpi income"><span class="kpi-label">Total Income</span><span class="kpi-value">{fmt(stats.total_income)}</span></div>
      <div class="kpi expense"><span class="kpi-label">Total Expense</span><span class="kpi-value">{fmt(stats.total_expense)}</span></div>
      <div class="kpi balance"><span class="kpi-label">Balance</span><span class="kpi-value">{fmt(stats.total_income - stats.total_expense)}</span></div>
    </div>
    <div class="charts-row">
      <div class="chart-card wide">
        <h2>Monthly Income vs Expense</h2>
        {#if stats.monthly.length === 0}<p class="muted">Add transactions to see data</p>
        {:else}<canvas bind:this={monthlyCanvas}></canvas>{/if}
      </div>
      <div class="chart-card">
        <h2>Expenses by Category</h2>
        {#if stats.by_category.length === 0}<p class="muted">No expense data</p>
        {:else}<canvas bind:this={categoryCanvas}></canvas>{/if}
      </div>
    </div>
  {:else if !error}
    <p class="muted">Loading…</p>
  {/if}
</div>

<style>
  .page { max-width: 1000px; }
  .page-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1.5rem; }
  h1 { margin: 0; font-size: 1.5rem; }
  .btn-refresh { background: #1a1a2e; border: 1px solid #2e2e4e; color: #888; padding: 0.35rem 0.75rem; border-radius: 6px; cursor: pointer; font-size: 0.85rem; }
  .kpi-row { display: flex; gap: 1rem; margin-bottom: 1.5rem; flex-wrap: wrap; }
  .kpi { flex: 1; min-width: 160px; background: #1a1a2e; border-radius: 8px; padding: 1rem 1.25rem; display: flex; flex-direction: column; gap: 0.25rem; }
  .kpi-label { font-size: 0.8rem; color: #888; text-transform: uppercase; letter-spacing: 0.05em; }
  .kpi-value { font-size: 1.5rem; font-weight: 700; }
  .income .kpi-value { color: #4ade80; }
  .expense .kpi-value { color: #f87171; }
  .balance .kpi-value { color: #a5b4fc; }
  .charts-row { display: flex; gap: 1rem; flex-wrap: wrap; }
  .chart-card { background: #1a1a2e; border-radius: 8px; padding: 1.25rem; flex: 1; min-width: 280px; }
  .chart-card.wide { flex: 2; }
  h2 { margin: 0 0 1rem; font-size: 0.95rem; color: #aaa; font-weight: 500; }
  .error { color: #f87171; }
  .muted { color: #555; }
</style>
