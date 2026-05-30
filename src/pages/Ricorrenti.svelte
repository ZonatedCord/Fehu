<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';
  import Modal from '../components/Modal.svelte';
  import type { Category, RecurringInput, RecurringTemplate } from '../lib/types';
  import { Plus, Trash2, Pencil, RefreshCw } from '@lucide/svelte';
  import { format } from 'date-fns';

  let templates = $state<RecurringTemplate[]>([]);
  let categories = $state<Category[]>([]);
  let error = $state('');
  let toast = $state('');
  let modalOpen = $state(false);
  let editing = $state<RecurringTemplate | null>(null);
  let saving = $state(false);

  const emptyForm = (): RecurringInput => ({
    description: '', amount: 0, type: 'expense', category_id: null,
    metodo: 'carta', notes: '', frequency: 'monthly',
    next_date: format(new Date(), 'yyyy-MM-dd'),
  });
  let form = $state<RecurringInput>(emptyForm());

  const FREQ_LABELS: Record<string, string> = {
    daily: 'Giornaliero', weekly: 'Settimanale',
    monthly: 'Mensile', yearly: 'Annuale',
  };

  async function load() {
    try {
      [templates, categories] = await Promise.all([api.listRecurring(), api.listCategories()]);
    } catch (e: any) { error = e.message ?? String(e); }
  }

  onMount(async () => {
    await load();
    try {
      const n = await api.checkAndInsertRecurring();
      if (n > 0) {
        toast = `${n} transazion${n === 1 ? 'e ricorrente inserita' : 'i ricorrenti inserite'}.`;
        await load();
        setTimeout(() => { toast = ''; }, 4000);
      }
    } catch { /* ignore */ }
  });

  function openCreate() { editing = null; form = emptyForm(); modalOpen = true; }
  function openEdit(t: RecurringTemplate) {
    editing = t;
    form = {
      description: t.description, amount: t.amount, type: t.type,
      category_id: t.category_id, metodo: t.metodo, notes: t.notes,
      frequency: t.frequency, next_date: t.next_date,
    };
    modalOpen = true;
  }

  async function save() {
    if (!form.description.trim() || form.amount <= 0) return;
    saving = true;
    try {
      if (editing) await api.updateRecurring(editing.id, form);
      else await api.createRecurring(form);
      modalOpen = false;
      await load();
    } catch (e: any) { error = e.message ?? String(e); }
    finally { saving = false; }
  }

  async function remove(id: number) {
    if (!confirm('Eliminare questa spesa ricorrente?')) return;
    try { await api.deleteRecurring(id); await load(); }
    catch (e: any) { error = e.message ?? String(e); }
  }

  async function toggle(id: number) {
    try { await api.toggleRecurring(id); await load(); }
    catch (e: any) { error = e.message ?? String(e); }
  }

  function fmt(n: number) {
    return new Intl.NumberFormat('it-IT', { style: 'currency', currency: 'EUR' }).format(n);
  }
</script>

<div class="page">
  <div class="page-header">
    <h1>Ricorrenti</h1>
    <button class="btn-primary" onclick={openCreate}><Plus size={14} /> Nuova</button>
  </div>

  {#if toast}<div class="toast"><RefreshCw size={14} /> {toast}</div>{/if}
  {#if error}<p class="error">{error}</p>{/if}

  {#if templates.length === 0}
    <div class="empty-state">
      <RefreshCw size={40} class="empty-icon" />
      <p class="empty-title">Nessuna spesa ricorrente</p>
      <p class="empty-hint">Aggiungi affitto, abbonamenti, bollette — vengono inseriti automaticamente alla data prevista.</p>
    </div>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Descrizione</th><th>Importo</th><th>Frequenza</th>
          <th>Prossima data</th><th>Categoria</th><th>Attivo</th><th></th>
        </tr>
      </thead>
      <tbody>
        {#each templates as t (t.id)}
          <tr class:inactive={!t.active}>
            <td>{t.description}</td>
            <td class="amount" class:income={t.type === 'income'} class:expense={t.type === 'expense'}>
              {t.type === 'income' ? '+' : '-'}{fmt(t.amount)}
            </td>
            <td>{FREQ_LABELS[t.frequency] ?? t.frequency}</td>
            <td class="date">{t.next_date}</td>
            <td>{#if t.category_name}{t.category_name}{:else}<span class="muted">—</span>{/if}</td>
            <td>
              <button class="toggle-btn" class:on={t.active} onclick={() => toggle(t.id)} title={t.active ? 'Disattiva' : 'Attiva'}>
                {t.active ? 'Sì' : 'No'}
              </button>
            </td>
            <td class="row-actions">
              <button onclick={() => openEdit(t)}><Pencil size={13} /></button>
              <button class="danger" onclick={() => remove(t.id)}><Trash2 size={13} /></button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<Modal title={editing ? 'Modifica ricorrente' : 'Nuova ricorrente'} open={modalOpen} onclose={() => modalOpen = false}>
  <form onsubmit={(e) => { e.preventDefault(); save(); }}>
    <label>Descrizione
      <input bind:value={form.description} placeholder="es. Affitto" required />
    </label>
    <div class="form-row">
      <label>Tipo
        <select bind:value={form.type}>
          <option value="expense">Uscita</option>
          <option value="income">Entrata</option>
        </select>
      </label>
      <label>Importo (€)
        <input type="number" bind:value={form.amount} min="0.01" step="0.01" required />
      </label>
    </div>
    <div class="form-row">
      <label>Frequenza
        <select bind:value={form.frequency}>
          <option value="daily">Giornaliero</option>
          <option value="weekly">Settimanale</option>
          <option value="monthly">Mensile</option>
          <option value="yearly">Annuale</option>
        </select>
      </label>
      <label>Prima occorrenza
        <input type="date" bind:value={form.next_date} required />
      </label>
    </div>
    <label>Categoria
      <select bind:value={form.category_id}>
        <option value={null}>— nessuna —</option>
        {#each categories as cat (cat.id)}
          <option value={cat.id}>{cat.name}</option>
        {/each}
      </select>
    </label>
    <label>Metodo
      <select bind:value={form.metodo}>
        <option value="carta">Carta</option>
        <option value="contanti">Contanti</option>
        <option value="altro">Altro</option>
      </select>
    </label>
    <label>Note
      <input bind:value={form.notes} placeholder="opzionale" />
    </label>
    <div class="form-actions">
      <button type="button" onclick={() => modalOpen = false}>Annulla</button>
      <button type="submit" class="btn-primary" disabled={saving}>{saving ? '…' : 'Salva'}</button>
    </div>
  </form>
</Modal>

<style>
  .page { max-width: 900px; }
  .page-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1.5rem; }
  h1 { margin: 0; font-size: 1.5rem; }
  .btn-primary { display: inline-flex; align-items: center; gap: 0.3rem; background: #6366f1; color: #fff; border: none; padding: 0.5rem 1rem; border-radius: 6px; cursor: pointer; font-size: 0.9rem; }
  .error { color: #f87171; }
  .toast { display: flex; align-items: center; gap: 0.4rem; background: #1a2e1a; border: 1px solid #166534; color: #4ade80; padding: 0.5rem 0.75rem; border-radius: 6px; margin-bottom: 1rem; font-size: 0.85rem; }
  table { width: 100%; border-collapse: collapse; font-size: 0.9rem; }
  th { text-align: left; padding: 0.5rem 0.75rem; border-bottom: 1px solid #2e2e4e; color: #888; font-weight: 500; font-size: 0.8rem; }
  td { padding: 0.55rem 0.75rem; border-bottom: 1px solid #1a1a2e; }
  .inactive td { opacity: 0.45; }
  .date { color: #888; font-size: 0.82rem; }
  .amount { font-weight: 600; }
  .income { color: #4ade80; }
  .expense { color: #f87171; }
  .muted { color: #555; }
  .toggle-btn { font-size: 0.72rem; padding: 0.15rem 0.45rem; background: #2a2a3e; border: 1px solid #3e3e5e; color: #888; cursor: pointer; border-radius: 4px; }
  .toggle-btn.on { background: #166534; border-color: #166534; color: #4ade80; }
  .row-actions { display: flex; gap: 0.25rem; justify-content: flex-end; }
  .row-actions button { font-size: 0.75rem; padding: 0.25rem 0.4rem; background: #2a2a3e; border: none; color: #ccc; cursor: pointer; border-radius: 4px; display: flex; align-items: center; }
  .row-actions .danger { color: #f87171; }
  .empty-state { display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 4rem 2rem; gap: 0.75rem; }
  :global(.empty-icon) { color: #2e2e4e; }
  .empty-title { font-size: 1rem; color: #666; margin: 0; }
  .empty-hint { font-size: 0.85rem; color: #444; text-align: center; max-width: 400px; margin: 0; }
  form { display: flex; flex-direction: column; gap: 0.75rem; }
  .form-row { display: flex; gap: 0.75rem; }
  .form-row label { flex: 1; }
  label { display: flex; flex-direction: column; gap: 0.25rem; font-size: 0.85rem; color: #aaa; }
  input, select { padding: 0.5rem 0.75rem; border: 1px solid #2e2e4e; border-radius: 6px; background: #0f0f1a; color: #e0e0f0; font-size: 0.9rem; width: 100%; box-sizing: border-box; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.5rem; margin-top: 0.5rem; }
  .form-actions button { padding: 0.5rem 1rem; border-radius: 6px; cursor: pointer; font-size: 0.9rem; border: 1px solid #2e2e4e; background: #1a1a2e; color: #ccc; }
</style>
