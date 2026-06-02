<script lang="ts">
  import { onMount } from 'svelte';
  import { PiggyBank, Plus, Trash2, Pencil, Check, X } from '@lucide/svelte';
  import { api } from '../lib/api';
  import type { Goal } from '../lib/types';

  let goals = $state<Goal[]>([]);
  let error = $state('');
  let creating = $state(false);
  let newName = $state('');
  let newTarget = $state(0);
  let newColor = $state('#6366f1');
  let saving = $state(false);
  let goalMetodo = $state<Record<number, 'contanti' | 'carta'>>({});
  let editingId = $state<number | null>(null);
  let editName = $state('');
  let editTarget = $state(0);
  let editColor = $state('#6366f1');

  function getMetodo(goalId: number): 'contanti' | 'carta' {
    return goalMetodo[goalId] ?? 'carta';
  }

  function setMetodo(goalId: number, m: 'contanti' | 'carta') {
    goalMetodo = { ...goalMetodo, [goalId]: m };
  }

  async function load() {
    try { goals = await api.listGoals(); }
    catch (e: any) { error = e.message ?? String(e); }
  }
  onMount(load);

  async function creaObbiettivo() {
    if (!newName.trim() || newTarget <= 0) return;
    saving = true;
    try {
      await api.createGoal(newName.trim(), newTarget, newColor);
      newName = ''; newTarget = 0; creating = false;
      await load();
    } catch (e: any) { error = e.message ?? String(e); }
    finally { saving = false; }
  }

  async function aggiungi(goal: Goal, amount: number) {
    if (amount <= 0) return;
    const metodo = getMetodo(goal.id);
    const date = new Date().toISOString().slice(0, 10);
    try {
      await api.contributeToGoal(goal.id, amount, metodo, date);
      await load();
    } catch (e: any) { error = e.message ?? String(e); }
  }

  function startEdit(goal: Goal) {
    editingId = goal.id;
    editName = goal.name;
    editTarget = goal.target;
    editColor = goal.color;
  }

  async function saveEdit(id: number) {
    if (!editName.trim() || editTarget <= 0) return;
    try {
      await api.updateGoal(id, editName.trim(), editTarget, editColor);
      editingId = null;
      await load();
    } catch (e: any) { error = e.message ?? String(e); }
  }

  async function rimuovi(id: number) {
    if (!confirm('Eliminare questo obiettivo?')) return;
    try { await api.deleteGoal(id); await load(); }
    catch (e: any) { error = e.message ?? String(e); }
  }

  function fmt(n: number) {
    return new Intl.NumberFormat('it-IT', { style: 'currency', currency: 'EUR' }).format(n);
  }

  function pct(goal: Goal) {
    return Math.min(Math.round((goal.saved / goal.target) * 100), 100);
  }
</script>

<div class="page">
  <div class="page-header">
    <h1>Fondi risparmio</h1>
    <button class="btn-primary" onclick={() => creating = !creating}>
      <Plus size={14} /> Nuovo obiettivo
    </button>
  </div>

  {#if creating}
    <div class="create-form">
      <input bind:value={newName} placeholder="Nome fondo (es. Vacanze estive)" />
      <label class="target-label">
        Obiettivo (€)
        <input type="number" bind:value={newTarget} min="1" step="10" />
      </label>
      <input type="color" bind:value={newColor} title="Colore" />
      <button class="btn-primary" onclick={creaObbiettivo} disabled={saving || !newName.trim() || newTarget <= 0}>
        {saving ? '…' : 'Crea'}
      </button>
      <button class="btn-ghost" onclick={() => creating = false}>Annulla</button>
    </div>
  {/if}

  {#if error}<p class="error">{error}</p>{/if}

  {#if goals.length === 0 && !creating}
    <div class="empty-state">
      <PiggyBank size={48} />
      <p>Nessun fondo ancora. Crea il tuo primo obiettivo di risparmio.</p>
    </div>
  {/if}

  <div class="goals-grid">
    {#each goals as goal (goal.id)}
      {@const p = pct(goal)}
      <div class="goal-card" style="--c: {goal.color}">
        <div class="goal-header">
          {#if editingId === goal.id}
            <input class="edit-name" bind:value={editName} />
            <input class="edit-target" type="number" bind:value={editTarget} min="1" step="10" />
            <input type="color" bind:value={editColor} title="Colore" class="edit-color" />
            <button class="btn-icon" onclick={() => saveEdit(goal.id)} title="Salva"><Check size={13} /></button>
            <button class="btn-icon" onclick={() => editingId = null} title="Annulla"><X size={13} /></button>
          {:else}
            <span class="goal-name">{goal.name}</span>
            <div class="goal-actions">
              <button class="btn-del" onclick={() => startEdit(goal)} title="Modifica"><Pencil size={13} /></button>
              <button class="btn-del" onclick={() => rimuovi(goal.id)} title="Elimina"><Trash2 size={13} /></button>
            </div>
          {/if}
        </div>
        <div class="amounts">
          <span class="saved">{fmt(goal.saved)}</span>
          <span class="sep">/</span>
          <span class="target">{fmt(goal.target)}</span>
        </div>
        <div class="progress-bar">
          <div class="progress-fill" style="width: {p}%; background: {goal.color}"></div>
        </div>
        <div class="goal-footer">
          <span class="pct">{p}%</span>
          <span class="mancano">Mancano {fmt(goal.target - goal.saved)}</span>
        </div>
        <div class="quick-add">
          <div class="metodo-toggle">
            <button class="metodo-btn" class:active={getMetodo(goal.id) === 'contanti'} onclick={() => setMetodo(goal.id, 'contanti')}>Contanti</button>
            <button class="metodo-btn" class:active={getMetodo(goal.id) === 'carta'} onclick={() => setMetodo(goal.id, 'carta')}>Carta</button>
          </div>
          {#each [10, 50, 100] as amt}
            <button class="btn-add" onclick={() => aggiungi(goal, amt)}>+{amt}€</button>
          {/each}
          <input type="number" placeholder="altro" min="1" step="1"
            onkeydown={(e) => { if (e.key === 'Enter') { aggiungi(goal, +(e.target as HTMLInputElement).value); (e.target as HTMLInputElement).value = ''; } }} />
        </div>
      </div>
    {/each}
  </div>
</div>

<style>
  .page { max-width: 900px; }
  .page-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 1.25rem; }
  h1 { margin: 0; font-size: 1.5rem; }
  .btn-primary { display: inline-flex; align-items: center; gap: 0.3rem; background: var(--accent); color: #fff; border: none; padding: 0.5rem 1rem; border-radius: 6px; cursor: pointer; font-size: 0.9rem; }
  .btn-primary:disabled { opacity: 0.6; cursor: not-allowed; }
  .btn-ghost { background: none; border: none; color: var(--text-dim); cursor: pointer; padding: 0.5rem; }
  .btn-del { background: none; border: none; color: var(--text-dim); cursor: pointer; padding: 0.2rem; opacity: 0.6; }
  .btn-del:hover { color: var(--expense); opacity: 1; }

  .create-form { display: flex; gap: 0.5rem; align-items: flex-end; flex-wrap: wrap; background: var(--bg-card); border-radius: 8px; padding: 1rem; margin-bottom: 1.25rem; }
  .create-form input { padding: 0.5rem 0.75rem; border: 1px solid var(--border); border-radius: 6px; background: var(--bg-base); color: var(--text); font-size: 0.9rem; }
  .create-form input:first-child { flex: 1; min-width: 180px; }
  .target-label { display: flex; flex-direction: column; gap: 0.25rem; font-size: 0.8rem; color: var(--text-muted); }
  .target-label input { width: 100px; }
  input[type="color"] { width: 36px; height: 36px; padding: 0.1rem; border: 1px solid var(--border); border-radius: 6px; cursor: pointer; background: var(--bg-base); }
  .error { color: var(--expense); }
  .empty-state { display: flex; flex-direction: column; align-items: center; gap: 1rem; padding: 3rem; color: var(--text-dim); text-align: center; }

  .goals-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 1rem; }
  .goal-card { background: var(--bg-card); border-radius: 14px; padding: 1.25rem; border: 1px solid var(--border); border-top: 3px solid var(--c, #6366f1); box-shadow: 0 1px 3px rgba(0,0,0,0.06); }
  .goal-header { display: flex; align-items: center; justify-content: space-between; gap: 0.4rem; margin-bottom: 0.75rem; flex-wrap: wrap; }
  .goal-actions { display: flex; gap: 0.2rem; }
  .btn-icon { background: none; border: none; color: var(--text-dim); cursor: pointer; padding: 0.2rem; opacity: 0.7; }
  .btn-icon:hover { opacity: 1; color: var(--accent); }
  .edit-name { flex: 1; min-width: 80px; padding: 0.2rem 0.4rem; border: 1px solid var(--border); border-radius: 4px; background: var(--bg-base); color: var(--text); font-size: 0.9rem; }
  .edit-target { width: 70px; padding: 0.2rem 0.4rem; border: 1px solid var(--border); border-radius: 4px; background: var(--bg-base); color: var(--text); font-size: 0.9rem; }
  .edit-color { width: 28px; height: 28px; padding: 0.1rem; border: 1px solid var(--border); border-radius: 4px; cursor: pointer; background: var(--bg-base); }
  .goal-name { font-weight: 600; font-size: 1rem; }
  .amounts { display: flex; align-items: baseline; gap: 0.25rem; margin-bottom: 0.5rem; }
  .saved { font-size: 1.4rem; font-weight: 700; color: var(--c, #6366f1); }
  .sep, .target { color: var(--text-dim); font-size: 0.9rem; }
  .progress-bar { background: var(--bg-elevated); border-radius: 4px; height: 8px; margin-bottom: 0.4rem; overflow: hidden; border: 1px solid var(--border2); }
  .progress-fill { height: 100%; border-radius: 4px; transition: width 0.3s; }
  .goal-footer { display: flex; justify-content: space-between; font-size: 0.78rem; color: var(--text-dim); margin-bottom: 0.75rem; }
  .pct { color: var(--c, #6366f1); font-weight: 600; }
  .quick-add { display: flex; gap: 0.3rem; align-items: center; flex-wrap: wrap; }
  .btn-add { background: var(--bg-base); border: 1px solid var(--border); color: var(--text-muted); padding: 0.25rem 0.5rem; border-radius: 4px; cursor: pointer; font-size: 0.78rem; }
  .btn-add:hover { border-color: var(--c, #6366f1); color: var(--c, #6366f1); }
  .quick-add input { width: 70px; padding: 0.25rem 0.4rem; border: 1px solid var(--border); border-radius: 4px; background: var(--bg-base); color: var(--text); font-size: 0.78rem; }
  .metodo-toggle { display: flex; gap: 0.2rem; width: 100%; margin-bottom: 0.3rem; }
  .metodo-btn { flex: 1; background: var(--bg-base); border: 1px solid var(--border); color: var(--text-dim); padding: 0.2rem 0.4rem; border-radius: 4px; cursor: pointer; font-size: 0.72rem; }
  .metodo-btn.active { border-color: var(--c, #6366f1); color: var(--c, #6366f1); }
</style>
