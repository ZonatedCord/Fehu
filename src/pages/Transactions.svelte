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

  const emptyForm = (): TransactionInput => ({
    amount: 0, type: 'expense', category_id: null,
    date: format(new Date(), 'yyyy-MM-dd'), description: '', notes: '',
  });
  let form = $state<TransactionInput>(emptyForm());
  let saving = $state(false);

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
    if (!confirm('Delete transaction?')) return;
    try { await api.deleteTransaction(id); await load(); }
    catch (e: any) { error = e.message ?? String(e); }
  }

  function fmt(n: number) {
    return new Intl.NumberFormat('it-IT', { style: 'currency', currency: 'EUR' }).format(n);
  }
</script>

<div class="page">
  <div class="page-header">
    <h1>Transactions</h1>
    <button class="btn-primary" onclick={openCreate}>+ Add</button>
  </div>
  <div class="filters">
    <select bind:value={filterType}>
      <option value="all">All types</option>
      <option value="income">Income</option>
      <option value="expense">Expense</option>
    </select>
    <select bind:value={filterCat}>
      <option value="">All categories</option>
      {#each categories as cat (cat.id)}
        <option value={cat.id}>{cat.name}</option>
      {/each}
    </select>
  </div>
  {#if error}<p class="error">{error}</p>{/if}
  <table>
    <thead><tr><th>Date</th><th>Description</th><th>Category</th><th>Amount</th><th></th></tr></thead>
    <tbody>
      {#each filtered as tx (tx.id)}
        <tr>
          <td class="date">{tx.date}</td>
          <td>{tx.description || '—'}</td>
          <td>{#if tx.category_name}<span class="tag">{tx.category_name}</span>{:else}<span class="muted">—</span>{/if}</td>
          <td class="amount" class:income={tx.type === 'income'} class:expense={tx.type === 'expense'}>
            {tx.type === 'income' ? '+' : '-'}{fmt(tx.amount)}
          </td>
          <td class="row-actions">
            <button onclick={() => openEdit(tx)}>Edit</button>
            <button class="danger" onclick={() => remove(tx.id)}>Del</button>
          </td>
        </tr>
      {/each}
      {#if filtered.length === 0}
        <tr><td colspan="5" class="empty">No transactions</td></tr>
      {/if}
    </tbody>
  </table>
</div>

<Modal title={editing ? 'Edit Transaction' : 'New Transaction'} open={modalOpen} onclose={() => modalOpen = false}>
  <form onsubmit={(e) => { e.preventDefault(); save(); }}>
    <label>Type
      <select bind:value={form.type}>
        <option value="expense">Expense</option>
        <option value="income">Income</option>
      </select>
    </label>
    <label>Amount (€)<input type="number" bind:value={form.amount} min="0.01" step="0.01" required /></label>
    <label>Date<input type="date" bind:value={form.date} required /></label>
    <label>Description<input bind:value={form.description} placeholder="e.g. Caffè al bar" /></label>
    <label>Category
      <select bind:value={form.category_id}>
        <option value={null}>— none —</option>
        {#each categories as cat (cat.id)}
          <option value={cat.id}>{cat.name}</option>
        {/each}
      </select>
    </label>
    <label>Notes<textarea bind:value={form.notes} rows="2"></textarea></label>
    <div class="form-actions">
      <button type="button" onclick={() => modalOpen = false}>Cancel</button>
      <button type="submit" class="btn-primary" disabled={saving}>{saving ? 'Saving…' : 'Save'}</button>
    </div>
  </form>
</Modal>

<style>
  .page { max-width: 900px; }
  .page-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem; }
  h1 { margin: 0; font-size: 1.5rem; }
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
  form { display: flex; flex-direction: column; gap: 0.75rem; }
  label { display: flex; flex-direction: column; gap: 0.25rem; font-size: 0.85rem; color: #aaa; }
  input, textarea { padding: 0.5rem 0.75rem; border: 1px solid #2e2e4e; border-radius: 6px; background: #0f0f1a; color: #e0e0f0; font-size: 0.9rem; }
  textarea { resize: vertical; font-family: inherit; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.5rem; margin-top: 0.5rem; }
  .btn-primary { background: #6366f1; color: #fff; border: none; padding: 0.5rem 1rem; border-radius: 6px; cursor: pointer; font-size: 0.9rem; }
  .btn-primary:hover { background: #5254cc; }
  .btn-primary:disabled { opacity: 0.6; cursor: not-allowed; }
</style>
