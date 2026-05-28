<script lang="ts">
  import { onMount } from 'svelte';
  import { PiggyBank, Plus, Trash2 } from '@lucide/svelte';
  import { api } from '../lib/api';
  import type { Goal } from '../lib/types';

  let goals = $state<Goal[]>([]);
  let error = $state('');
  let creating = $state(false);
  let newName = $state('');
  let newTarget = $state(0);
  let newColor = $state('#6366f1');
  let saving = $state(false);

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
    const newSaved = Math.min(goal.saved + amount, goal.target);
    try {
      await api.updateGoalSaved(goal.id, newSaved);
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
          <span class="goal-name">{goal.name}</span>
          <button class="btn-del" onclick={() => rimuovi(goal.id)} title="Elimina">
            <Trash2 size={13} />
          </button>
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
  .btn-primary { display: inline-flex; align-items: center; gap: 0.3rem; background: #6366f1; color: #fff; border: none; padding: 0.5rem 1rem; border-radius: 6px; cursor: pointer; font-size: 0.9rem; }
  .btn-primary:disabled { opacity: 0.6; cursor: not-allowed; }
  .btn-ghost { background: none; border: none; color: #666; cursor: pointer; padding: 0.5rem; }
  .btn-del { background: none; border: none; color: #555; cursor: pointer; padding: 0.2rem; opacity: 0.6; }
  .btn-del:hover { color: #f87171; opacity: 1; }

  .create-form { display: flex; gap: 0.5rem; align-items: flex-end; flex-wrap: wrap; background: #1a1a2e; border-radius: 8px; padding: 1rem; margin-bottom: 1.25rem; }
  .create-form input { padding: 0.5rem 0.75rem; border: 1px solid #2e2e4e; border-radius: 6px; background: #0f0f1a; color: #e0e0f0; font-size: 0.9rem; }
  .create-form input:first-child { flex: 1; min-width: 180px; }
  .target-label { display: flex; flex-direction: column; gap: 0.25rem; font-size: 0.8rem; color: #888; }
  .target-label input { width: 100px; }
  input[type="color"] { width: 36px; height: 36px; padding: 0.1rem; border: 1px solid #2e2e4e; border-radius: 6px; cursor: pointer; background: #0f0f1a; }
  .error { color: #f87171; }
  .empty-state { display: flex; flex-direction: column; align-items: center; gap: 1rem; padding: 3rem; color: #444; text-align: center; }

  .goals-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 1rem; }
  .goal-card { background: #1a1a2e; border-radius: 10px; padding: 1.25rem; border-top: 3px solid var(--c, #6366f1); }
  .goal-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 0.75rem; }
  .goal-name { font-weight: 600; font-size: 1rem; }
  .amounts { display: flex; align-items: baseline; gap: 0.25rem; margin-bottom: 0.5rem; }
  .saved { font-size: 1.4rem; font-weight: 700; color: var(--c, #6366f1); }
  .sep, .target { color: #555; font-size: 0.9rem; }
  .progress-bar { background: #0f0f1a; border-radius: 4px; height: 6px; margin-bottom: 0.4rem; overflow: hidden; }
  .progress-fill { height: 100%; border-radius: 4px; transition: width 0.3s; }
  .goal-footer { display: flex; justify-content: space-between; font-size: 0.78rem; color: #666; margin-bottom: 0.75rem; }
  .pct { color: var(--c, #6366f1); font-weight: 600; }
  .quick-add { display: flex; gap: 0.3rem; align-items: center; flex-wrap: wrap; }
  .btn-add { background: #0f0f1a; border: 1px solid #2e2e4e; color: #aaa; padding: 0.25rem 0.5rem; border-radius: 4px; cursor: pointer; font-size: 0.78rem; }
  .btn-add:hover { border-color: var(--c, #6366f1); color: var(--c, #6366f1); }
  .quick-add input { width: 70px; padding: 0.25rem 0.4rem; border: 1px solid #2e2e4e; border-radius: 4px; background: #0f0f1a; color: #e0e0f0; font-size: 0.78rem; }
</style>
