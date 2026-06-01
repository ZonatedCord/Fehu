<script lang="ts">
  import { onMount } from 'svelte';
  import { Plus, Tag } from '@lucide/svelte';
  import { api } from '../lib/api';
  import Modal from '../components/Modal.svelte';
  import IconPicker from '../components/IconPicker.svelte';
  import CategoryIcon from '../components/CategoryIcon.svelte';
  import type { Category } from '../lib/types';

  let categories = $state<Category[]>([]);
  let error = $state('');
  let modalOpen = $state(false);
  let editing = $state<Category | null>(null);
  let form = $state({ name: '', color: '#6366f1', icon: 'package', budget_limit: '' });
  let saving = $state(false);

  async function load() {
    try { categories = await api.listCategories(); }
    catch (e: any) { error = e.message ?? String(e); }
  }
  onMount(load);

  function openCreate() { editing = null; form = { name: '', color: '#6366f1', icon: 'package', budget_limit: '' }; modalOpen = true; }
  function openEdit(cat: Category) { editing = cat; form = { name: cat.name, color: cat.color, icon: cat.icon, budget_limit: cat.budget_limit != null ? String(cat.budget_limit) : '' }; modalOpen = true; }

  async function save() {
    if (!form.name.trim()) return;
    saving = true;
    const budgetVal = form.budget_limit.trim() ? parseFloat(form.budget_limit) : null;
    try {
      if (editing) await api.updateCategory(editing.id, form.name, form.color, form.icon, budgetVal);
      else await api.createCategory(form.name, form.color, form.icon, budgetVal);
      modalOpen = false;
      await load();
    } catch (e: any) { error = e.message ?? String(e); }
    finally { saving = false; }
  }

  async function remove(id: number) {
    if (!confirm('Eliminare questa categoria? Le transazioni mantengono i dati.')) return;
    try { await api.deleteCategory(id); await load(); }
    catch (e: any) { error = e.message ?? String(e); }
  }
</script>

<div class="page">
  <div class="page-header">
    <h1>Categorie</h1>
    <button class="btn-primary" onclick={openCreate}><Plus size={14} /> Nuova</button>
  </div>
  {#if error}<p class="error">{error}</p>{/if}
  {#if categories.length === 0}
    <div class="empty-state">
      <Tag size={44} class="empty-icon" />
      <p class="empty-title">Nessuna categoria</p>
      <p class="empty-hint">Crea categorie per organizzare le tue spese.</p>
    </div>
  {/if}
  <div class="grid">
    {#each categories as cat (cat.id)}
      <div class="card">
        <span class="swatch" style="background:{cat.color}"></span>
        <span class="cat-icon" style="color:{cat.color}">
          <CategoryIcon name={cat.icon} size={16} />
        </span>
        <span class="name">{cat.name}</span>
        {#if cat.budget_limit != null}<span class="budget-badge">€{cat.budget_limit}/m</span>{/if}
        <div class="actions">
          <button onclick={() => openEdit(cat)}>Modifica</button>
          <button class="danger" onclick={() => remove(cat.id)}>Elimina</button>
        </div>
      </div>
    {/each}
  </div>
</div>

<Modal title={editing ? 'Modifica categoria' : 'Nuova categoria'} open={modalOpen} onclose={() => modalOpen = false}>
  <form onsubmit={(e) => { e.preventDefault(); save(); }}>
    <label>Nome<input bind:value={form.name} placeholder="es. Cibo" required /></label>
    <label>Colore<input type="color" bind:value={form.color} /></label>
    <label>
      Icona
      <IconPicker value={form.icon} onselect={(n) => form.icon = n} />
    </label>
    <label>Budget mensile (€) — opzionale
      <input type="number" bind:value={form.budget_limit} placeholder="es. 300" min="0" step="0.01" />
    </label>
    <div class="form-actions">
      <button type="button" onclick={() => modalOpen = false}>Annulla</button>
      <button type="submit" class="btn-primary" disabled={saving}>{saving ? 'Salvataggio…' : 'Salva'}</button>
    </div>
  </form>
</Modal>

<style>
  .page { max-width: 800px; }
  .page-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1.5rem; }
  h1 { margin: 0; font-size: 1.5rem; }
  .btn-primary { display: inline-flex; align-items: center; gap: 0.3rem; background: var(--accent); color: #fff; border: none; padding: 0.5rem 1rem; border-radius: 6px; cursor: pointer; font-size: 0.9rem; }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 0.75rem; }
  .card { background: var(--bg-card); border-radius: 12px; padding: 0.75rem 1rem; display: flex; align-items: center; gap: 0.5rem; border: 1px solid var(--border); box-shadow: 0 1px 3px rgba(0,0,0,0.05); }
  .swatch { width: 10px; height: 10px; border-radius: 50%; flex-shrink: 0; }
  .cat-icon { display: flex; align-items: center; }
  .name { flex: 1; font-size: 0.9rem; }
  .actions { display: flex; gap: 0.25rem; }
  .actions button { font-size: 0.75rem; padding: 0.2rem 0.4rem; background: var(--bg-elevated); border: none; color: var(--text-muted); cursor: pointer; border-radius: 4px; }
  .actions .danger { color: var(--expense); }
  .budget-badge { font-size: 0.7rem; color: var(--text-muted); background: var(--bg-card2); border: 1px solid var(--border); padding: 0.1rem 0.35rem; border-radius: 4px; white-space: nowrap; }
  .error { color: var(--expense); }
  .empty-state { display: flex; flex-direction: column; align-items: center; padding: 4rem 2rem; gap: 0.5rem; }
  :global(.empty-icon) { color: var(--border); }
  .empty-title { color: var(--text-dim); font-size: 1rem; margin: 0; }
  .empty-hint { color: var(--text-dim); font-size: 0.85rem; margin: 0; }
  form { display: flex; flex-direction: column; gap: 0.75rem; }
  label { display: flex; flex-direction: column; gap: 0.4rem; font-size: 0.85rem; color: var(--text-muted); }
  input { padding: 0.5rem 0.75rem; border: 1px solid var(--border); border-radius: 6px; background: var(--bg-base); color: var(--text); font-size: 0.9rem; }
  input[type="color"] { padding: 0.1rem; height: 36px; cursor: pointer; }
  .form-actions { display: flex; justify-content: flex-end; gap: 0.5rem; margin-top: 0.5rem; }
</style>
