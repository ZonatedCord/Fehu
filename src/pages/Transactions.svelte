<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';
  import Modal from '../components/Modal.svelte';
  import type { Category, Transaction, TransactionFile, TransactionInput } from '../lib/types';
  import { format } from 'date-fns';
  import { open } from '@tauri-apps/plugin-dialog';
  import { openPath } from '@tauri-apps/plugin-opener';
  import { Paperclip } from '@lucide/svelte';
  import { keyboardAction } from '../lib/stores';

  let transactions = $state<Transaction[]>([]);
  let categories = $state<Category[]>([]);
  let error = $state('');
  let modalOpen = $state(false);
  let editing = $state<Transaction | null>(null);
  let filterType = $state<'all' | 'income' | 'expense'>('all');
  let filterCat = $state<number | string>('');
  let viewMode = $state<'list' | 'calendar'>('list');
  let searchText = $state('');
  let startDate = $state('');
  let endDate = $state('');
  let searchRef = $state<HTMLInputElement | null>(null);
  let searchTimer: ReturnType<typeof setTimeout> | null = null;

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
    date: format(new Date(), 'yyyy-MM-dd'), description: '', notes: '', metodo: 'carta', currency: 'EUR',
  });
  let form = $state<TransactionInput>(emptyForm());
  let saving = $state(false);

  // Attachments
  let attachments = $state<TransactionFile[]>([]);
  let attachLoading = $state(false);

  $effect(() => {
    if (editing) {
      attachLoading = true;
      api.listAttachments(editing.id)
        .then(a => { attachments = a; })
        .catch(() => { attachments = []; })
        .finally(() => { attachLoading = false; });
    } else {
      attachments = [];
    }
  });

  async function addAttachment() {
    if (!editing) return;
    const path = await open({ multiple: false }).catch(() => null);
    if (path && typeof path === 'string') {
      try {
        const att = await api.attachFile(editing.id, path);
        attachments = [...attachments, att];
      } catch (e: any) { error = e.message ?? String(e); }
    }
  }

  async function removeAttachment(id: number) {
    try {
      await api.deleteAttachment(id);
      attachments = attachments.filter(a => a.id !== id);
    } catch (e: any) { error = e.message ?? String(e); }
  }

  async function openAttachmentFile(path: string) {
    try { await openPath(path); } catch {}
  }

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
      [transactions, categories] = await Promise.all([
        api.listTransactions({
          start_date: startDate || undefined,
          end_date: endDate || undefined,
          search_text: searchText.trim() || undefined,
        }),
        api.listCategories(),
      ]);
    } catch (e: any) { error = e.message ?? String(e); }
  }

  function onSearchInput() {
    if (searchTimer) clearTimeout(searchTimer);
    searchTimer = setTimeout(load, 300);
  }

  function resetDateFilter() { startDate = ''; endDate = ''; load(); }

  onMount(load);

  // React to keyboard shortcuts from +page.svelte
  keyboardAction.subscribe(action => {
    if (!action) return;
    if (action === 'new-transaction') { editing = null; form = emptyForm(); modalOpen = true; }
    if (action === 'focus-search') { searchRef?.focus(); }
  });

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
    form = { amount: tx.amount, type: tx.type, category_id: tx.category_id, date: tx.date, description: tx.description, notes: tx.notes, metodo: tx.metodo ?? 'carta', currency: tx.currency ?? 'EUR' };
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
      <input
        class="search-input"
        bind:this={searchRef}
        bind:value={searchText}
        oninput={onSearchInput}
        placeholder="Cerca…"
      />
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
      <input type="date" class="date-filter" bind:value={startDate} onchange={load} title="Dal" />
      <input type="date" class="date-filter" bind:value={endDate} onchange={load} title="Al" />
      {#if startDate || endDate}
        <button class="btn-reset" onclick={resetDateFilter} title="Reset date">✕</button>
      {/if}
    </div>
    {#if error}<p class="error">{error}</p>{/if}
    <div class="table-wrap">
    <table>
      <thead><tr><th>Data</th><th>Descrizione</th><th>Categoria</th><th>Metodo</th><th>Importo</th><th></th></tr></thead>
      <tbody>
        {#each filtered as tx (tx.id)}
          <tr>
            <td class="date">{formatDate(tx.date)}</td>
            <td class="desc-cell">
              {tx.description || '—'}
              {#if tx.attachment_count > 0}<Paperclip size={12} class="attach-icon" />{/if}
            </td>
            <td>{#if tx.category_name}<span class="tag">{tx.category_name}</span>{:else}<span class="muted">—</span>{/if}</td>
            <td class="metodo">
              {#if tx.metodo === 'contanti'}<span class="badge cash">C</span>
              {:else if tx.metodo === 'carta'}<span class="badge card">K</span>
              {:else}<span class="badge other">?</span>{/if}
            </td>
            <td class="amount" class:income={tx.type === 'income'} class:expense={tx.type === 'expense'}>
              {tx.type === 'income' ? '+' : '-'}{fmt(tx.amount)}
              {#if tx.currency && tx.currency !== 'EUR'}<span class="currency-badge">{tx.currency}</span>{/if}
            </td>
            <td class="row-actions">
              <button onclick={() => openEdit(tx)}>Modifica</button>
              <button class="danger" onclick={() => remove(tx.id)}>Elimina</button>
            </td>
          </tr>
        {/each}
        {#if filtered.length === 0}
          <tr><td colspan="6" class="empty">
            {#if transactions.length === 0}Nessuna transazione ancora — aggiungi la prima spesa o entrata.
            {:else}Nessuna transazione corrisponde ai filtri.{/if}
          </td></tr>
        {/if}
      </tbody>
    </table>
    </div>

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
    <div class="form-row-2">
      <label>Metodo
        <select bind:value={form.metodo}>
          <option value="carta">Carta</option>
          <option value="contanti">Contanti</option>
          <option value="altro">Altro</option>
        </select>
      </label>
      <label>Valuta
        <select bind:value={form.currency}>
          <option value="EUR">EUR €</option>
          <option value="USD">USD $</option>
          <option value="GBP">GBP £</option>
          <option value="CHF">CHF</option>
          <option value="JPY">JPY ¥</option>
        </select>
      </label>
    </div>
    <label>Importo<input type="number" bind:value={form.amount} min="0.01" step="0.01" required /></label>
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

    {#if editing}
      <div class="att-section">
        <div class="att-header">
          <span class="att-title"><Paperclip size={13} /> Allegati</span>
          <button type="button" class="btn-att" onclick={addAttachment}>+ Aggiungi file</button>
        </div>
        {#if attachLoading}
          <p class="att-hint">Caricamento…</p>
        {:else if attachments.length === 0}
          <p class="att-hint">Nessun allegato — aggiungi fatture o scontrini</p>
        {:else}
          {#each attachments as att (att.id)}
            <div class="att-row">
              <span class="att-name" title={att.file_path}>{att.file_name}</span>
              <div class="att-actions">
                <button type="button" class="att-btn" onclick={() => openAttachmentFile(att.file_path)}>Apri</button>
                <button type="button" class="att-btn danger" onclick={() => removeAttachment(att.id)}>✕</button>
              </div>
            </div>
          {/each}
        {/if}
      </div>
    {/if}

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
  .view-toggle { display: flex; border-radius: 6px; overflow: hidden; border: 1px solid var(--border); }
  .view-toggle button { padding: 0.35rem 0.75rem; border: none; background: var(--bg-base); color: var(--text-muted); cursor: pointer; font-size: 0.82rem; }
  .view-toggle button.active { background: var(--bg-elevated); color: var(--accent-lt); }

  .filters { display: flex; gap: 0.5rem; margin-bottom: 1rem; flex-wrap: wrap; align-items: center; }
  .search-input { padding: 0.4rem 0.75rem; background: var(--bg-card); border: 1px solid var(--border); color: var(--text); border-radius: 6px; font-size: 0.85rem; min-width: 140px; flex: 1; max-width: 220px; }
  .date-filter { padding: 0.4rem 0.5rem; background: var(--bg-card); border: 1px solid var(--border); color: var(--text); border-radius: 6px; font-size: 0.82rem; width: 130px; }
  .btn-reset { background: none; border: none; color: var(--text-dim); cursor: pointer; font-size: 0.9rem; padding: 0.2rem 0.4rem; }
  .btn-reset:hover { color: var(--expense); }
  select { padding: 0.4rem 0.75rem; background: var(--bg-card); border: 1px solid var(--border); color: var(--text); border-radius: 6px; font-size: 0.85rem; }
  .table-wrap { background: var(--bg-card); border-radius: 12px; overflow: hidden; box-shadow: 0 1px 3px rgba(0,0,0,0.06); }
  table { width: 100%; border-collapse: collapse; font-size: 0.9rem; }
  th { text-align: left; padding: 0.6rem 0.75rem; border-bottom: 1px solid var(--border); color: var(--text-dim); font-weight: 600; font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.05em; background: var(--bg-card2); }
  td { padding: 0.65rem 0.75rem; border-bottom: 1px solid var(--border2); }
  .date { color: var(--text-muted); font-size: 0.8rem; white-space: nowrap; }
  .amount { font-weight: 600; text-align: right; }
  .income { color: var(--income); }
  .expense { color: var(--expense); }
  .tag { background: var(--bg-elevated); padding: 0.2rem 0.5rem; border-radius: 4px; font-size: 0.8rem; }
  .muted { color: var(--text-dim); }
  .desc-cell { display: flex; align-items: center; gap: 0.35rem; }
  :global(.attach-icon) { color: var(--text-dim); flex-shrink: 0; }
  .currency-badge { font-size: 0.65rem; background: var(--bg-elevated); color: var(--accent-lt); border: 1px solid var(--border); padding: 0.05rem 0.3rem; border-radius: 3px; vertical-align: middle; margin-left: 0.2rem; }
  .form-row-2 { display: flex; gap: 0.75rem; }
  .form-row-2 label { flex: 1; }
  .row-actions { display: flex; gap: 0.25rem; justify-content: flex-end; }
  .row-actions button { font-size: 0.75rem; padding: 0.2rem 0.4rem; background: var(--bg-elevated); border: none; color: var(--text-muted); cursor: pointer; border-radius: 4px; }
  .row-actions .danger { color: var(--expense); }
  .empty { text-align: center; color: var(--text-dim); padding: 2rem; }
  .error { color: var(--expense); }
  .metodo { text-align: center; }
  .badge { display: inline-block; width: 18px; height: 18px; border-radius: 3px; font-size: 0.65rem; font-weight: 700; line-height: 18px; text-align: center; }
  .badge.cash { background: color-mix(in srgb, var(--income) 15%, transparent); color: var(--income); }
  .badge.card { background: color-mix(in srgb, var(--accent) 15%, transparent); color: var(--accent-lt); }
  .badge.other { background: var(--bg-elevated); color: var(--text-muted); }

  /* Calendar */
  .cal-nav { display: flex; align-items: center; gap: 1rem; margin-bottom: 0.75rem; }
  .cal-nav button { background: var(--bg-card); border: 1px solid var(--border); color: var(--text-muted); padding: 0.3rem 0.75rem; border-radius: 6px; cursor: pointer; font-size: 1.1rem; }
  .cal-title { font-weight: 600; font-size: 1rem; }
  .cal-grid { display: grid; grid-template-columns: repeat(7, 1fr); gap: 2px; margin-bottom: 1rem; }
  .cal-dow { text-align: center; font-size: 0.75rem; color: var(--text-dim); padding: 0.3rem 0; font-weight: 500; }
  .cal-cell { min-height: 64px; border-radius: 6px; padding: 0.3rem; position: relative; }
  .empty-cell { background: transparent; }
  .day-cell {
    background: var(--bg-card); border: 1px solid transparent;
    cursor: pointer; display: flex; flex-direction: column; align-items: flex-start;
    transition: border-color 0.1s;
  }
  .day-cell:hover { border-color: var(--border); }
  .day-cell.today { border-color: var(--accent); }
  .day-cell.selected { background: var(--bg-elevated); border-color: var(--accent); }
  .day-num { font-size: 0.82rem; color: var(--text-muted); }
  .day-cell.today .day-num { color: var(--accent-lt); font-weight: 700; }
  .dots { display: flex; gap: 3px; margin-top: auto; padding-top: 0.25rem; }
  .dot { width: 6px; height: 6px; border-radius: 50%; }
  .dot.income { background: var(--income); }
  .dot.expense { background: var(--expense); }

  /* Day detail panel */
  .day-detail { background: var(--bg-card); border-radius: 8px; padding: 1rem; }
  .day-detail-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 0.75rem; font-weight: 600; font-size: 0.9rem; }
  .btn-sm-add { background: var(--accent); color: #fff; border: none; padding: 0.25rem 0.6rem; border-radius: 4px; cursor: pointer; font-size: 0.8rem; }
  .day-tx { display: flex; align-items: center; gap: 0.5rem; padding: 0.4rem 0; border-bottom: 1px solid var(--border); font-size: 0.88rem; }
  .day-tx:last-child { border-bottom: none; }
  .day-tx-desc { flex: 1; }
  .empty-day { color: var(--text-dim); font-size: 0.85rem; }
  .btn-edit-sm { background: none; border: none; color: var(--text-muted); cursor: pointer; font-size: 0.9rem; padding: 0.1rem 0.3rem; }
  .btn-del-sm { background: none; border: none; color: var(--expense); cursor: pointer; font-size: 0.85rem; padding: 0.1rem 0.3rem; opacity: 0.6; }
  .btn-del-sm:hover { opacity: 1; }

  form { display: flex; flex-direction: column; gap: 0.75rem; }
  label { display: flex; flex-direction: column; gap: 0.25rem; font-size: 0.85rem; color: var(--text-muted); }
  input, textarea { padding: 0.5rem 0.75rem; border: 1px solid var(--border); border-radius: 6px; background: var(--bg-base); color: var(--text); font-size: 0.9rem; }
  textarea { resize: vertical; font-family: inherit; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.5rem; margin-top: 0.5rem; }
  .btn-primary { background: var(--accent); color: #fff; border: none; padding: 0.5rem 1rem; border-radius: 6px; cursor: pointer; font-size: 0.9rem; }
  .btn-primary:hover { opacity: 0.9; }
  .btn-primary:disabled { opacity: 0.6; cursor: not-allowed; }
  .cat-row { display: flex; gap: 0.4rem; align-items: center; }
  .cat-row select { flex: 1; }
  .btn-new-cat { white-space: nowrap; background: var(--bg-elevated); border: 1px solid var(--border); color: var(--accent-lt); padding: 0.4rem 0.6rem; border-radius: 6px; cursor: pointer; font-size: 0.78rem; }
  .btn-new-cat:hover { opacity: 0.85; }
  .new-cat-inline { display: flex; gap: 0.4rem; margin-top: 0.4rem; align-items: center; }
  .new-cat-inline input:not([type="color"]) { flex: 1; }
  .new-cat-inline input[type="color"] { width: 36px; height: 32px; padding: 0.1rem; cursor: pointer; flex-shrink: 0; }
  .btn-sm { padding: 0.4rem 0.75rem; font-size: 0.82rem; }

  /* Attachments */
  .att-section { border: 1px solid var(--border); border-radius: 7px; padding: 0.75rem; display: flex; flex-direction: column; gap: 0.4rem; }
  .att-header { display: flex; align-items: center; justify-content: space-between; }
  .att-title { display: flex; align-items: center; gap: 0.35rem; font-size: 0.82rem; color: var(--text-muted); }
  .btn-att { background: var(--bg-card); border: 1px solid var(--border); color: #a5b4fc; padding: 0.25rem 0.6rem; border-radius: 5px; cursor: pointer; font-size: 0.78rem; }
  .btn-att:hover { border-color: #6366f1; }
  .att-hint { font-size: 0.78rem; color: var(--text-dim); margin: 0.1rem 0; }
  .att-row { display: flex; align-items: center; justify-content: space-between; background: var(--bg-base); border-radius: 5px; padding: 0.35rem 0.6rem; }
  .att-name { font-size: 0.82rem; color: var(--text); flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .att-actions { display: flex; gap: 0.25rem; flex-shrink: 0; }
  .att-btn { background: none; border: none; color: var(--text-muted); cursor: pointer; font-size: 0.78rem; padding: 0.15rem 0.4rem; border-radius: 3px; }
  .att-btn:hover { color: var(--text); background: var(--bg-card); }
  .att-btn.danger { color: var(--expense); }
  .att-btn.danger:hover { opacity: 0.8; }
</style>
