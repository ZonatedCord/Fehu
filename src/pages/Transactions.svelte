<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';
  import Modal from '../components/Modal.svelte';
  import type { Category, Transaction, TransactionInput } from '../lib/types';
  import { format } from 'date-fns';

  let transactions = $state<Transaction[]>([]);
  let categories = $state<Category[]>([]);
  let error = $state('');
  let modalOpen = $state(false);
  let editing = $state<Transaction | null>(null);
  let filterType = $state<'all' | 'income' | 'expense'>('all');
  let filterCat = $state<number | string>('');
  let viewMode = $state<'list' | 'calendar'>('list');

  // Calendar state
  const today = new Date();
  let calYear = $state(today.getFullYear());
  let calMonth = $state(today.getMonth()); // 0-indexed
  let selectedDay = $state<string | null>(null); // YYYY-MM-DD

  const MESI = ['Gennaio','Febbraio','Marzo','Aprile','Maggio','Giugno','Luglio','Agosto','Settembre','Ottobre','Novembre','Dicembre'];
  const GIORNI = ['Lun','Mar','Mer','Gio','Ven','Sab','Dom'];

  function prevMonth() {
    if (calMonth === 0) { calMonth = 11; calYear--; } else calMonth--;
    selectedDay = null;
  }
  function nextMonth() {
    if (calMonth === 11) { calMonth = 0; calYear++; } else calMonth++;
    selectedDay = null;
  }

  // Days grid for current calendar month
  let calDays = $derived(() => {
    const firstDay = new Date(calYear, calMonth, 1);
    // Monday=0 offset
    let startOffset = (firstDay.getDay() + 6) % 7;
    const daysInMonth = new Date(calYear, calMonth + 1, 0).getDate();
    const cells: Array<number | null> = [];
    for (let i = 0; i < startOffset; i++) cells.push(null);
    for (let d = 1; d <= daysInMonth; d++) cells.push(d);
    return cells;
  });

  // Map YYYY-MM-DD → transactions for fast lookup
  let txByDay = $derived(() => {
    const map = new Map<string, Transaction[]>();
    for (const t of transactions) {
      const key = t.date.slice(0, 10);
      if (!map.has(key)) map.set(key, []);
      map.get(key)!.push(t);
    }
    return map;
  });

  function dayKey(d: number) {
    return `${calYear}-${String(calMonth + 1).padStart(2,'0')}-${String(d).padStart(2,'0')}`;
  }

  let selectedDayTx = $derived(selectedDay ? (txByDay().get(selectedDay) ?? []) : []);

  const emptyForm = (): TransactionInput => ({
    amount: 0, type: 'expense', category_id: null,
    date: format(new Date(), 'yyyy-MM-dd'), description: '', notes: '',
  });
  let form = $state<TransactionInput>(emptyForm());
  let saving = $state(false);

  // Inline category creation
  let newCatOpen = $state(false);
  let newCatName = $state('');
  let newCatColor = $state('#6366f1');
  let newCatSaving = $state(false);

  async function createCategoryInline() {
    if (!newCatName.trim()) return;
    newCatSaving = true;
    try {
      const cat = await api.createCategory(newCatName.trim(), newCatColor, 'package');
      categories = [...categories, cat].sort((a, b) => a.name.localeCompare(b.name));
      form.category_id = cat.id;
      newCatOpen = false;
      newCatName = '';
    } catch (e: any) { error = e.message ?? String(e); }
    finally { newCatSaving = false; }
  }

  async function load() {
    try {
      [transactions, categories] = await Promise.all([api.listTransactions(), api.listCategories()]);
    } catch (e: any) { error = e.message ?? String(e); }
  }
  onMount(load);

  let filtered = $derived(transactions.filter(t => {
    if (filterType !== 'all' && t.type !== filterType) return false;
    if (filterCat !== '' && t.category_id !== filterCat) return false;
    return true;
  }));

  function openCreate() { editing = null; form = emptyForm(); modalOpen = true; }
  function openCreate_onDay(day: string) {
    editing = null; form = { ...emptyForm(), date: day }; modalOpen = true;
  }
  function openEdit(tx: Transaction) {
    editing = tx;
    form = { amount: tx.amount, type: tx.type, category_id: tx.category_id, date: tx.date, description: tx.description, notes: tx.notes };
    modalOpen = true;
  }

  async function save() {
    saving = true;
    try {
      if (editing) await api.updateTransaction(editing.id, form);
      else await api.createTransaction(form);
      modalOpen = false; await load();
    } catch (e: any) { error = e.message ?? String(e); }
    finally { saving = false; }
  }

  async function remove(id: number) {
    if (!confirm('Eliminare questa transazione?')) return;
    try { await api.deleteTransaction(id); await load(); }
    catch (e: any) { error = e.message ?? String(e); }
  }

  function formatDate(iso: string): string {
    if (!iso || iso.length < 10) return iso;
    const [y, m, d] = iso.split('-');
    return `${d}/${m}/${y}`;
  }

  function fmt(n: number) {
    return new Intl.NumberFormat('it-IT', { style: 'currency', currency: 'EUR' }).format(n);
  }

  const todayKey = `${today.getFullYear()}-${String(today.getMonth()+1).padStart(2,'0')}-${String(today.getDate()).padStart(2,'0')}`;
</script>

<div class="page">
  <div class="page-header">
    <h1>Transazioni</h1>
    <div class="header-right">
      <div class="view-toggle">
        <button class:active={viewMode === 'list'} onclick={() => viewMode = 'list'}>Lista</button>
        <button class:active={viewMode === 'calendar'} onclick={() => viewMode = 'calendar'}>Calendario</button>
      </div>
      <button class="btn-primary" onclick={openCreate}>+ Aggiungi</button>
    </div>
  </div>

  {#if viewMode === 'list'}
    <div class="filters">
      <select bind:value={filterType}>
        <option value="all">Tutti i tipi</option>
        <option value="income">Entrata</option>
        <option value="expense">Uscita</option>
      </select>
      <select bind:value={filterCat}>
        <option value="">Tutte le categorie</option>
        {#each categories as cat (cat.id)}
          <option value={cat.id}>{cat.name}</option>
        {/each}
      </select>
    </div>
    {#if error}<p class="error">{error}</p>{/if}
    <table>
      <thead><tr><th>Data</th><th>Descrizione</th><th>Categoria</th><th>Importo</th><th></th></tr></thead>
      <tbody>
        {#each filtered as tx (tx.id)}
          <tr>
            <td class="date">{formatDate(tx.date)}</td>
            <td>{tx.description || '—'}</td>
            <td>{#if tx.category_name}<span class="tag">{tx.category_name}</span>{:else}<span class="muted">—</span>{/if}</td>
            <td class="amount" class:income={tx.type === 'income'} class:expense={tx.type === 'expense'}>
              {tx.type === 'income' ? '+' : '-'}{fmt(tx.amount)}
            </td>
            <td class="row-actions">
              <button onclick={() => openEdit(tx)}>Modifica</button>
              <button class="danger" onclick={() => remove(tx.id)}>Elimina</button>
            </td>
          </tr>
        {/each}
        {#if filtered.length === 0}
          <tr><td colspan="5" class="empty">Nessuna transazione</td></tr>
        {/if}
      </tbody>
    </table>

  {:else}
    <!-- CALENDAR VIEW -->
    <div class="cal-nav">
      <button onclick={prevMonth}>‹</button>
      <span class="cal-title">{MESI[calMonth]} {calYear}</span>
      <button onclick={nextMonth}>›</button>
    </div>

    <div class="cal-grid">
      {#each GIORNI as g}
        <div class="cal-dow">{g}</div>
      {/each}
      {#each calDays() as cell}
        {#if cell === null}
          <div class="cal-cell empty-cell"></div>
        {:else}
          {@const key = dayKey(cell)}
          {@const dayTx = txByDay().get(key) ?? []}
          {@const hasIncome = dayTx.some(t => t.type === 'income')}
          {@const hasExpense = dayTx.some(t => t.type === 'expense')}
          {@const isToday = key === todayKey}
          {@const isSelected = key === selectedDay}
          <button
            class="cal-cell day-cell"
            class:today={isToday}
            class:selected={isSelected}
            class:has-tx={dayTx.length > 0}
            onclick={() => selectedDay = isSelected ? null : key}
          >
            <span class="day-num">{cell}</span>
            {#if hasExpense || hasIncome}
              <div class="dots">
                {#if hasExpense}<span class="dot expense"></span>{/if}
                {#if hasIncome}<span class="dot income"></span>{/if}
              </div>
            {/if}
          </button>
        {/if}
      {/each}
    </div>

    {#if selectedDay}
      <div class="day-detail">
        <div class="day-detail-header">
          <span>{formatDate(selectedDay)}</span>
          <button class="btn-sm-add" onclick={() => openCreate_onDay(selectedDay!)}>+ Aggiungi</button>
        </div>
        {#if selectedDayTx.length === 0}
          <p class="empty-day">Nessuna transazione questo giorno</p>
        {:else}
          {#each selectedDayTx as tx (tx.id)}
            <div class="day-tx">
              <span class="day-tx-desc">{tx.description || '—'}</span>
              {#if tx.category_name}<span class="tag">{tx.category_name}</span>{/if}
              <span class="amount" class:income={tx.type === 'income'} class:expense={tx.type === 'expense'}>
                {tx.type === 'income' ? '+' : '-'}{fmt(tx.amount)}
              </span>
              <button class="btn-edit-sm" onclick={() => openEdit(tx)}>✎</button>
              <button class="btn-del-sm" onclick={() => remove(tx.id)}>✕</button>
            </div>
          {/each}
        {/if}
      </div>
    {/if}
  {/if}
</div>

<Modal title={editing ? 'Modifica transazione' : 'Nuova transazione'} open={modalOpen} onclose={() => modalOpen = false}>
  <form onsubmit={(e) => { e.preventDefault(); save(); }}>
    <label>Tipo
      <select bind:value={form.type}>
        <option value="expense">Uscita</option>
        <option value="income">Entrata</option>
      </select>
    </label>
    <label>Importo (€)<input type="number" bind:value={form.amount} min="0.01" step="0.01" required /></label>
    <label>Data<input type="date" bind:value={form.date} required /></label>
    <label>Descrizione<input bind:value={form.description} placeholder="es. Caffè al bar" /></label>
    <label>Categoria
      <div class="cat-row">
        <select bind:value={form.category_id}>
          <option value={null}>— nessuna —</option>
          {#each categories as cat (cat.id)}
            <option value={cat.id}>{cat.name}</option>
          {/each}
        </select>
        <button type="button" class="btn-new-cat" onclick={() => { newCatOpen = !newCatOpen; newCatName = ''; }}>
          {newCatOpen ? '✕' : '+ Nuova'}
        </button>
      </div>
      {#if newCatOpen}
        <div class="new-cat-inline">
          <input bind:value={newCatName} placeholder="Nome categoria" onkeydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); createCategoryInline(); } }} />
          <input type="color" bind:value={newCatColor} title="Colore" />
          <button type="button" class="btn-primary btn-sm" onclick={createCategoryInline} disabled={newCatSaving || !newCatName.trim()}>
            {newCatSaving ? '…' : 'Crea'}
          </button>
        </div>
      {/if}
    </label>
    <label>Note<textarea bind:value={form.notes} rows="2"></textarea></label>
    <div class="form-actions">
      <button type="button" onclick={() => modalOpen = false}>Annulla</button>
      <button type="submit" class="btn-primary" disabled={saving}>{saving ? 'Salvataggio…' : 'Salva'}</button>
    </div>
  </form>
</Modal>

<style>
  .page { max-width: 900px; }
  .page-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem; }
  h1 { margin: 0; font-size: 1.5rem; }
  .header-right { display: flex; align-items: center; gap: 0.75rem; }
  .view-toggle { display: flex; border-radius: 6px; overflow: hidden; border: 1px solid #2e2e4e; }
  .view-toggle button { padding: 0.35rem 0.75rem; border: none; background: #0f0f1a; color: #888; cursor: pointer; font-size: 0.82rem; }
  .view-toggle button.active { background: #1e1e3e; color: #a5b4fc; }

  .filters { display: flex; gap: 0.5rem; margin-bottom: 1rem; }
  select { padding: 0.4rem 0.75rem; background: #1a1a2e; border: 1px solid #2e2e4e; color: #e0e0f0; border-radius: 6px; font-size: 0.85rem; }
  table { width: 100%; border-collapse: collapse; font-size: 0.9rem; }
  th { text-align: left; padding: 0.5rem 0.75rem; border-bottom: 1px solid #2e2e4e; color: #888; font-weight: 500; font-size: 0.8rem; }
  td { padding: 0.6rem 0.75rem; border-bottom: 1px solid #1a1a2e; }
  .date { color: #888; font-size: 0.8rem; white-space: nowrap; }
  .amount { font-weight: 600; text-align: right; }
  .income { color: #4ade80; }
  .expense { color: #f87171; }
  .tag { background: #2a2a3e; padding: 0.2rem 0.5rem; border-radius: 4px; font-size: 0.8rem; }
  .muted { color: #555; }
  .row-actions { display: flex; gap: 0.25rem; justify-content: flex-end; }
  .row-actions button { font-size: 0.75rem; padding: 0.2rem 0.4rem; background: #2a2a3e; border: none; color: #ccc; cursor: pointer; border-radius: 4px; }
  .row-actions .danger { color: #f87171; }
  .empty { text-align: center; color: #555; padding: 2rem; }
  .error { color: #f87171; }

  /* Calendar */
  .cal-nav { display: flex; align-items: center; gap: 1rem; margin-bottom: 0.75rem; }
  .cal-nav button { background: #1a1a2e; border: 1px solid #2e2e4e; color: #ccc; padding: 0.3rem 0.75rem; border-radius: 6px; cursor: pointer; font-size: 1.1rem; }
  .cal-title { font-weight: 600; font-size: 1rem; }
  .cal-grid { display: grid; grid-template-columns: repeat(7, 1fr); gap: 2px; margin-bottom: 1rem; }
  .cal-dow { text-align: center; font-size: 0.75rem; color: #555; padding: 0.3rem 0; font-weight: 500; }
  .cal-cell { min-height: 64px; border-radius: 6px; padding: 0.3rem; position: relative; }
  .empty-cell { background: transparent; }
  .day-cell {
    background: #1a1a2e; border: 1px solid transparent;
    cursor: pointer; display: flex; flex-direction: column; align-items: flex-start;
    transition: border-color 0.1s;
  }
  .day-cell:hover { border-color: #2e2e4e; }
  .day-cell.today { border-color: #6366f1; }
  .day-cell.selected { background: #1e1e3e; border-color: #6366f1; }
  .day-num { font-size: 0.82rem; color: #888; }
  .day-cell.today .day-num { color: #a5b4fc; font-weight: 700; }
  .dots { display: flex; gap: 3px; margin-top: auto; padding-top: 0.25rem; }
  .dot { width: 6px; height: 6px; border-radius: 50%; }
  .dot.income { background: #4ade80; }
  .dot.expense { background: #f87171; }

  /* Day detail panel */
  .day-detail { background: #1a1a2e; border-radius: 8px; padding: 1rem; }
  .day-detail-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 0.75rem; font-weight: 600; font-size: 0.9rem; }
  .btn-sm-add { background: #6366f1; color: #fff; border: none; padding: 0.25rem 0.6rem; border-radius: 4px; cursor: pointer; font-size: 0.8rem; }
  .day-tx { display: flex; align-items: center; gap: 0.5rem; padding: 0.4rem 0; border-bottom: 1px solid #2e2e4e; font-size: 0.88rem; }
  .day-tx:last-child { border-bottom: none; }
  .day-tx-desc { flex: 1; }
  .empty-day { color: #555; font-size: 0.85rem; }
  .btn-edit-sm { background: none; border: none; color: #888; cursor: pointer; font-size: 0.9rem; padding: 0.1rem 0.3rem; }
  .btn-del-sm { background: none; border: none; color: #f87171; cursor: pointer; font-size: 0.85rem; padding: 0.1rem 0.3rem; opacity: 0.6; }
  .btn-del-sm:hover { opacity: 1; }

  form { display: flex; flex-direction: column; gap: 0.75rem; }
  label { display: flex; flex-direction: column; gap: 0.25rem; font-size: 0.85rem; color: #aaa; }
  input, textarea { padding: 0.5rem 0.75rem; border: 1px solid #2e2e4e; border-radius: 6px; background: #0f0f1a; color: #e0e0f0; font-size: 0.9rem; }
  textarea { resize: vertical; font-family: inherit; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.5rem; margin-top: 0.5rem; }
  .btn-primary { background: #6366f1; color: #fff; border: none; padding: 0.5rem 1rem; border-radius: 6px; cursor: pointer; font-size: 0.9rem; }
  .btn-primary:hover { background: #5254cc; }
  .btn-primary:disabled { opacity: 0.6; cursor: not-allowed; }
  .cat-row { display: flex; gap: 0.4rem; align-items: center; }
  .cat-row select { flex: 1; }
  .btn-new-cat { white-space: nowrap; background: #2a2a3e; border: 1px solid #3e3e5e; color: #a5b4fc; padding: 0.4rem 0.6rem; border-radius: 6px; cursor: pointer; font-size: 0.78rem; }
  .btn-new-cat:hover { background: #3a3a5e; }
  .new-cat-inline { display: flex; gap: 0.4rem; margin-top: 0.4rem; align-items: center; }
  .new-cat-inline input:not([type="color"]) { flex: 1; }
  .new-cat-inline input[type="color"] { width: 36px; height: 32px; padding: 0.1rem; cursor: pointer; flex-shrink: 0; }
  .btn-sm { padding: 0.4rem 0.75rem; font-size: 0.82rem; }
</style>
