<script lang="ts">
  import { onMount } from 'svelte';
  import { get } from 'svelte/store';
  import { Chart, registerables } from 'chart.js';
  import { api } from '../lib/api';
  import type { BalanceAdjustment, BudgetAlert, DashboardStats, PatrimonioStats } from '../lib/types';
  import SankeyChart from '../components/SankeyChart.svelte';
  import { Trash2, BarChart3 } from '@lucide/svelte';
  import { theme } from '../lib/stores';

  Chart.register(...registerables);

  let stats = $state<DashboardStats | null>(null);
  let patrimonio = $state<PatrimonioStats | null>(null);
  let error = $state('');
  let monthlyCanvas = $state<HTMLCanvasElement | null>(null);
  let categoryCanvas = $state<HTMLCanvasElement | null>(null);
  let monthlyChart: Chart | null = null;
  let categoryChart: Chart | null = null;

  let showRettifica = $state(false);
  let rettMetodo = $state<'contanti' | 'carta'>('contanti');
  let rettAmount = $state(0);
  let rettNote = $state('');
  let rettDate = $state(new Date().toISOString().slice(0, 10));
  let rettSaving = $state(false);
  let rettifiche = $state<BalanceAdjustment[]>([]);
  let showRettifiche = $state(false);
  let budgetAlerts = $state<BudgetAlert[]>([]);

  async function load() {
    try {
      const currentMonth = new Date().toISOString().slice(0, 7);
      [stats, patrimonio, rettifiche, budgetAlerts] = await Promise.all([
        api.getDashboardStats(),
        api.getPatrimonio(),
        api.listBalanceAdjustments(),
        api.getBudgetAlerts(currentMonth),
      ]);
      await new Promise(r => setTimeout(r, 0));
      renderCharts();
    } catch (e: any) { error = e.message ?? String(e); }
  }

  async function eliminaRettifica(id: number) {
    try {
      await api.deleteBalanceAdjustment(id);
      await load();
    } catch (e: any) { error = e.message ?? String(e); }
  }

  function renderCharts() {
    if (!stats) return;
    if (monthlyChart) { monthlyChart.destroy(); monthlyChart = null; }
    if (categoryChart) { categoryChart.destroy(); categoryChart = null; }

    const isLight = get(theme) === 'light';
    const labelColor  = isLight ? '#374151' : '#b0b8cc';
    const gridColor   = isLight ? '#d1d5eb' : '#2e2e4e';
    const incomeColor = isLight ? '#16a34a' : '#4ade80';
    const expenseColor = isLight ? '#dc2626' : '#f87171';

    if (monthlyCanvas && stats.monthly.length > 0) {
      monthlyChart = new Chart(monthlyCanvas, {
        type: 'bar',
        data: {
          labels: stats.monthly.map(m => m.month),
          datasets: [
            { label: 'Entrate', data: stats.monthly.map(m => m.income), backgroundColor: incomeColor + '55', borderColor: incomeColor, borderWidth: 1 },
            { label: 'Uscite', data: stats.monthly.map(m => m.expense), backgroundColor: expenseColor + '55', borderColor: expenseColor, borderWidth: 1 },
          ],
        },
        options: {
          responsive: true,
          plugins: { legend: { labels: { color: labelColor } } },
          scales: {
            x: { ticks: { color: labelColor }, grid: { color: gridColor } },
            y: { ticks: { color: labelColor }, grid: { color: gridColor } },
          },
        },
      });
    }

    if (categoryCanvas && stats.by_category.length > 0) {
      categoryChart = new Chart(categoryCanvas, {
        type: 'doughnut',
        data: {
          labels: stats.by_category.map(c => c.category_name ?? 'Senza categoria'),
          datasets: [{ data: stats.by_category.map(c => c.total), backgroundColor: stats.by_category.map(c => c.color ?? '#6366f1'), borderWidth: 0 }],
        },
        options: { responsive: true, plugins: { legend: { labels: { color: labelColor } } } },
      });
    }
  }

  async function salvaRettifica() {
    if (rettAmount === 0) return;
    rettSaving = true;
    try {
      await api.createBalanceAdjustment(rettMetodo, rettAmount, rettNote, rettDate);
      showRettifica = false;
      rettAmount = 0;
      rettNote = '';
      await load();
    } catch (e: any) { error = e.message ?? String(e); }
    finally { rettSaving = false; }
  }

  onMount(() => {
    load();
    const unsub = theme.subscribe(() => { if (stats) renderCharts(); });
    return unsub;
  });

  function fmt(n: number) {
    return new Intl.NumberFormat('it-IT', { style: 'currency', currency: 'EUR' }).format(n);
  }
</script>

<div class="page">
  <div class="page-header">
    <h1>Dashboard</h1>
  </div>
  {#if error}<p class="error">{error}</p>{/if}
  {#if stats}
    <div class="kpi-row">
      <div class="kpi income"><span class="kpi-label">Entrate totali</span><span class="kpi-value">{fmt(stats.total_income)}</span></div>
      <div class="kpi expense"><span class="kpi-label">Uscite totali</span><span class="kpi-value">{fmt(stats.total_expense)}</span></div>
      <div class="kpi balance"><span class="kpi-label">Saldo</span><span class="kpi-value">{fmt(stats.total_income - stats.total_expense)}</span></div>
    </div>
    {#if patrimonio}
      <div class="kpi-row patrimonio-row">
        <div class="kpi cash">
          <span class="kpi-label">Saldo contanti</span>
          <span class="kpi-value" class:negative={patrimonio.saldo_contanti < 0}>{fmt(patrimonio.saldo_contanti)}</span>
        </div>
        <div class="kpi card">
          <span class="kpi-label">Saldo carta</span>
          <span class="kpi-value" class:negative={patrimonio.saldo_carta < 0}>{fmt(patrimonio.saldo_carta)}</span>
        </div>
        <div class="kpi total">
          <span class="kpi-label">Patrimonio totale</span>
          <span class="kpi-value" class:negative={patrimonio.totale < 0}>{fmt(patrimonio.totale)}</span>
        </div>
      </div>
      <div class="rettifica-link">
        <button class="btn-ghost-small" onclick={() => showRettifica = true}>+ Rettifica</button>
        {#if rettifiche.length > 0}
          <button class="btn-ghost-small" onclick={() => showRettifiche = !showRettifiche}>
            {showRettifiche ? 'Nascondi' : `Rettifiche (${rettifiche.length})`}
          </button>
        {/if}
      </div>
      {#if showRettifiche && rettifiche.length > 0}
        <div class="rettifiche-list">
          {#each rettifiche as r (r.id)}
            <div class="rett-row">
              <span class="rett-date">{r.date}</span>
              <span class="badge-metodo" class:cash={r.metodo === 'contanti'} class:card={r.metodo === 'carta'}>
                {r.metodo === 'contanti' ? 'C' : 'K'}
              </span>
              <span class="rett-amount" class:pos={r.amount >= 0} class:neg={r.amount < 0}>
                {r.amount >= 0 ? '+' : ''}{fmt(r.amount)}
              </span>
              <span class="rett-note">{r.note || '—'}</span>
              <button class="btn-del-rett" onclick={() => eliminaRettifica(r.id)} title="Elimina">
                <Trash2 size={13} />
              </button>
            </div>
          {/each}
        </div>
      {/if}
    {/if}
    {#if budgetAlerts.length > 0}
      <div class="budget-alerts">
        {#each budgetAlerts as a (a.category_id)}
          <div class="budget-alert">
            <span class="alert-icon">!</span>
            <span class="alert-text">
              <strong>{a.category_name}</strong>: spesi {fmt(a.spent)} su {fmt(a.budget_limit)} mensili
            </span>
          </div>
        {/each}
      </div>
    {/if}
    <div class="charts-row">
      <div class="chart-card wide">
        <h2>Entrate vs Uscite mensili</h2>
        {#if stats.monthly.length === 0}<p class="muted">Aggiungi transazioni per vedere i dati</p>
        {:else}<canvas bind:this={monthlyCanvas}></canvas>{/if}
      </div>
      <div class="chart-card">
        <h2>Uscite per categoria</h2>
        {#if stats.by_category.length === 0}<p class="muted">Nessuna uscita</p>
        {:else}<canvas bind:this={categoryCanvas}></canvas>{/if}
      </div>
    </div>
    <div class="chart-card full">
      <h2>Flusso denaro</h2>
      <SankeyChart {stats} />
    </div>
  {:else if !error}
    <div class="dash-empty">
      <BarChart3 size={48} class="dash-empty-icon" />
      <p>Caricamento…</p>
    </div>
  {/if}
</div>

{#if showRettifica}
  <div class="backdrop" role="dialog" aria-modal="true">
    <div class="modal">
      <div class="modal-header">
        <h2>Rettifica saldo</h2>
        <button class="btn-close" onclick={() => showRettifica = false}>✕</button>
      </div>
      <div class="modal-body">
        <label>
          Metodo
          <select bind:value={rettMetodo}>
            <option value="contanti">Contanti</option>
            <option value="carta">Carta</option>
          </select>
        </label>
        <label>
          Importo (€) — positivo per aggiungere, negativo per sottrarre
          <input type="number" bind:value={rettAmount} step="0.01" />
        </label>
        <label>
          Nota (opzionale)
          <input bind:value={rettNote} placeholder="Es. Saldo iniziale" />
        </label>
        <label>
          Data
          <input type="date" bind:value={rettDate} />
        </label>
      </div>
      <div class="modal-footer">
        <button class="btn-primary" onclick={salvaRettifica} disabled={rettSaving || rettAmount === 0}>
          {rettSaving ? '…' : 'Salva'}
        </button>
        <button class="btn-ghost" onclick={() => showRettifica = false}>Annulla</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .page { max-width: 1000px; }
  .page-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1.5rem; }
  h1 { margin: 0; font-size: 1.5rem; }
  .btn-refresh { background: var(--bg-card); border: 1px solid var(--border); color: var(--text-muted); padding: 0.35rem 0.75rem; border-radius: 6px; cursor: pointer; font-size: 0.85rem; }
  .kpi-row { display: flex; gap: 1rem; margin-bottom: 1.5rem; flex-wrap: wrap; }
  .kpi { flex: 1; min-width: 160px; background: var(--bg-card); border-radius: 14px; padding: 1.1rem 1.25rem; display: flex; flex-direction: column; gap: 0.25rem; box-shadow: 0 1px 3px rgba(0,0,0,0.06); }
  .kpi-label { font-size: 0.8rem; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.05em; }
  .kpi-value { font-size: 1.5rem; font-weight: 700; }
  .income .kpi-value { color: var(--income); }
  .expense .kpi-value { color: var(--expense); }
  .balance .kpi-value { color: var(--accent-lt); }
  .charts-row { display: flex; gap: 1rem; flex-wrap: wrap; }
  .chart-card { background: var(--bg-card); border-radius: 14px; padding: 1.25rem; flex: 1; min-width: 280px; box-shadow: 0 1px 3px rgba(0,0,0,0.06); }
  .chart-card.wide { flex: 2; }
  .chart-card.full { flex: 1 0 100%; }
  h2 { margin: 0 0 1rem; font-size: 0.95rem; color: var(--text-muted); font-weight: 500; }
  .error { color: var(--expense); }
  .muted { color: var(--text-dim); }
  .dash-empty { display: flex; flex-direction: column; align-items: center; padding: 4rem; gap: 0.75rem; color: var(--text-dim); }
  :global(.dash-empty-icon) { color: var(--border); }
  .patrimonio-row { margin-top: -0.5rem; }
  .kpi.cash .kpi-value { color: var(--income); }
  .kpi.card .kpi-value { color: #60a5fa; }
  .kpi.total .kpi-value { color: #c084fc; }
  .kpi-value.negative { color: var(--expense) !important; }
  .rettifica-link { text-align: right; margin-bottom: 1rem; }
  .btn-ghost-small { background: none; border: none; color: var(--text-dim); cursor: pointer; font-size: 0.78rem; padding: 0.25rem 0.5rem; text-decoration: underline; }
  .btn-ghost-small:hover { color: var(--text-muted); }
  .rettifiche-list { background: var(--bg-card2); border: 1px solid var(--border2); border-radius: 8px; padding: 0.5rem; margin-bottom: 1rem; display: flex; flex-direction: column; gap: 0.15rem; }
  .rett-row { display: flex; align-items: center; gap: 0.6rem; padding: 0.35rem 0.5rem; border-radius: 4px; font-size: 0.82rem; }
  .rett-row:hover { background: var(--bg-card); }
  .rett-date { color: var(--text-dim); width: 90px; flex-shrink: 0; }
  .badge-metodo { width: 18px; height: 18px; border-radius: 3px; font-size: 0.65rem; font-weight: 700; line-height: 18px; text-align: center; flex-shrink: 0; }
  .badge-metodo.cash { background: color-mix(in srgb, var(--income) 15%, transparent); color: var(--income); }
  .badge-metodo.card { background: color-mix(in srgb, var(--accent) 15%, transparent); color: var(--accent-lt); }
  .rett-amount { font-weight: 600; width: 90px; flex-shrink: 0; }
  .rett-amount.pos { color: var(--income); }
  .rett-amount.neg { color: var(--expense); }
  .rett-note { flex: 1; color: var(--text-dim); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .btn-del-rett { background: none; border: none; color: var(--text-dim); cursor: pointer; padding: 0.1rem 0.25rem; border-radius: 3px; display: flex; align-items: center; }
  .btn-del-rett:hover { color: var(--expense); }
  .backdrop { position: fixed; inset: 0; background: rgba(0,0,0,0.6); display: flex; align-items: center; justify-content: center; z-index: 100; }
  .modal { background: var(--bg-card); border-radius: 18px; padding: 1.5rem; width: 360px; max-width: 95vw; }
  .modal-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1.25rem; }
  .modal-header h2 { margin: 0; font-size: 1.1rem; color: var(--text); }
  .btn-close { background: none; border: none; color: var(--text-muted); cursor: pointer; font-size: 1.1rem; }
  .modal-body { display: flex; flex-direction: column; gap: 0.75rem; margin-bottom: 1.25rem; }
  .modal-body label { display: flex; flex-direction: column; gap: 0.25rem; font-size: 0.85rem; color: var(--text-muted); }
  .modal-body input, .modal-body select { padding: 0.5rem 0.75rem; border: 1px solid var(--border); border-radius: 6px; background: var(--bg-base); color: var(--text); font-size: 0.9rem; }
  .modal-footer { display: flex; gap: 0.5rem; justify-content: flex-end; }
  .btn-primary { background: var(--accent); color: #fff; border: none; padding: 0.5rem 1rem; border-radius: 6px; cursor: pointer; font-size: 0.9rem; }
  .btn-primary:disabled { opacity: 0.6; cursor: not-allowed; }
  .btn-ghost { background: none; border: none; color: var(--text-dim); cursor: pointer; padding: 0.5rem; }
  .budget-alerts { display: flex; flex-direction: column; gap: 0.4rem; margin-bottom: 1.25rem; }
  .budget-alert { display: flex; align-items: center; gap: 0.6rem; background: rgba(239,68,68,0.08); border: 1px solid rgba(239,68,68,0.3); border-radius: 6px; padding: 0.5rem 0.75rem; font-size: 0.85rem; }
  .alert-icon { color: var(--expense); font-weight: 700; font-size: 1rem; }
  .alert-text { color: var(--expense); opacity: 0.85; }
  .alert-text strong { color: var(--expense); }
</style>
