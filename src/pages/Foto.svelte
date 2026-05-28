<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { onMount } from 'svelte';
  import { api } from '../lib/api';
  import { currentPage } from '../lib/stores';
  import type { Category, ReceiptData } from '../lib/types';

  const IMAGE_EXTS = ['jpg', 'jpeg', 'png', 'webp', 'heic', 'avif', 'bmp'];

  let imagePath = $state('');
  let previewUrl = $state('');
  let analyzing = $state(false);
  let dragging = $state(false);
  let receipt = $state<ReceiptData | null>(null);
  let categories = $state<Category[]>([]);
  let error = $state('');
  let saving = $state(false);
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
      categoriaId = cat.id;
      newCatOpen = false;
      newCatName = '';
    } catch (e: any) { error = e.message ?? String(e); }
    finally { newCatSaving = false; }
  }

  let tipo = $state<'income' | 'expense'>('expense');
  let importo = $state(0);
  let data = $state('');
  let descrizione = $state('');
  let categoriaId = $state<number | null>(null);

  onMount(async () => {
    categories = await api.listCategories().catch(() => []);

    const unDrag = await listen('tauri://drag', () => { dragging = true; });
    const unLeave = await listen('tauri://drag-leave', () => { dragging = false; });
    const unCancel = await listen('tauri://drag-cancelled', () => { dragging = false; });
    const unDrop = await listen<{ paths: string[] }>('tauri://drag-drop', (e) => {
      dragging = false;
      const img = e.payload.paths.find(p =>
        IMAGE_EXTS.some(ext => p.toLowerCase().endsWith('.' + ext))
      );
      if (img) setImage(img);
    });

    return () => { unDrag(); unLeave(); unCancel(); unDrop(); };
  });

  function setImage(path: string) {
    imagePath = path;
    previewUrl = convertFileSrc(path);
    receipt = null;
    error = '';
  }

  async function scegliImmagine() {
    const path = await open({
      filters: [{ name: 'Immagini', extensions: IMAGE_EXTS }],
    });
    if (path && typeof path === 'string') setImage(path);
  }

  async function analizza() {
    if (!imagePath) return;
    analyzing = true;
    error = '';
    receipt = null;
    try {
      const result: ReceiptData = await invoke('analyze_receipt', { imagePath });
      receipt = result;
      if (result.importo) importo = result.importo;
      if (result.data) {
        const parts = result.data.split(/[-/]/);
        if (parts.length === 3) {
          data = parts[0].length === 4
            ? result.data
            : `${parts[2]}-${parts[1].padStart(2,'0')}-${parts[0].padStart(2,'0')}`;
        }
      }
      if (result.descrizione) descrizione = result.descrizione;
      if (result.categoria) {
        const match = categories.find(c =>
          c.name.toLowerCase() === result.categoria!.toLowerCase()
        );
        if (match) categoriaId = match.id;
      }
    } catch (e: any) {
      error = e.message ?? String(e);
    } finally {
      analyzing = false;
    }
  }

  async function salva() {
    if (importo <= 0) { error = 'Inserisci un importo valido'; return; }
    saving = true;
    try {
      await api.createTransaction({
        amount: importo,
        type: tipo,
        category_id: categoriaId,
        date: data || new Date().toISOString().slice(0, 10),
        description: descrizione,
        notes: '',
      });
      imagePath = ''; previewUrl = ''; receipt = null;
      importo = 0; data = ''; descrizione = ''; categoriaId = null;
      currentPage.set('transactions');
    } catch (e: any) {
      error = e.message ?? String(e);
    } finally {
      saving = false;
    }
  }
</script>

<div class="page">
  <h1>Foto / Spesa</h1>
  <p class="hint">Carica o trascina qualsiasi immagine di spesa — scontrino, fattura, screenshot — i dati vengono estratti automaticamente.</p>

  <!-- Drop zone -->
  <div class="drop-zone" class:dragging>
    {#if previewUrl}
      <img src={previewUrl} alt="Anteprima" class="preview" />
    {:else}
      <div class="drop-placeholder">
        <span class="drop-icon">↓</span>
        <span>{dragging ? 'Rilascia qui' : 'Trascina qui un\'immagine'}</span>
      </div>
    {/if}
  </div>

  <div class="pick-area">
    <button class="btn-secondary" onclick={scegliImmagine}>Scegli file</button>
    {#if imagePath}
      <button class="btn-primary" onclick={analizza} disabled={analyzing}>
        {analyzing ? 'Analisi in corso…' : 'Analizza'}
      </button>
      {#if previewUrl}
        <button class="btn-ghost" onclick={() => { imagePath=''; previewUrl=''; receipt=null; }}>Rimuovi</button>
      {/if}
    {/if}
  </div>

  {#if error}
    <p class="error">{error}</p>
    {#if error.includes('moondream')}
      <p class="hint-error">Modello non installato. Esegui: <code>ollama pull moondream</code></p>
    {/if}
  {/if}

  {#if receipt !== null || (imagePath && !analyzing)}
    <div class="form-section">
      <h2>Dati transazione</h2>
      <div class="tipo-toggle">
        <button class:active={tipo === 'expense'} onclick={() => tipo = 'expense'}>Uscita</button>
        <button class:active={tipo === 'income'} onclick={() => tipo = 'income'}>Entrata</button>
      </div>
      <form onsubmit={(e) => { e.preventDefault(); salva(); }}>
        <label>Importo (€)<input type="number" bind:value={importo} min="0.01" step="0.01" required /></label>
        <label>Data<input type="date" bind:value={data} /></label>
        <label>Descrizione<input bind:value={descrizione} placeholder="es. Caffè al bar" /></label>
        <label>Categoria
          <div class="cat-row">
            <select bind:value={categoriaId}>
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
              <input type="color" bind:value={newCatColor} />
              <button type="button" class="btn-primary btn-sm" onclick={createCategoryInline} disabled={newCatSaving || !newCatName.trim()}>
                {newCatSaving ? '…' : 'Crea'}
              </button>
            </div>
          {/if}
        </label>
        <button type="submit" class="btn-primary full-width" disabled={saving}>
          {saving ? 'Salvataggio…' : 'Salva transazione'}
        </button>
      </form>
    </div>
  {/if}
</div>

<style>
  .page { max-width: 520px; }
  h1 { margin: 0 0 0.25rem; font-size: 1.5rem; }
  h2 { margin: 0 0 1rem; font-size: 1rem; color: #aaa; font-weight: 500; }
  .hint { color: #666; font-size: 0.85rem; margin-bottom: 1.25rem; }

  .drop-zone {
    min-height: 180px;
    border: 2px dashed #2e2e4e;
    border-radius: 10px;
    display: flex; align-items: center; justify-content: center;
    margin-bottom: 1rem;
    transition: border-color 0.15s, background 0.15s;
    overflow: hidden;
  }
  .drop-zone.dragging {
    border-color: #6366f1;
    background: #1e1e3e;
  }
  .drop-placeholder {
    display: flex; flex-direction: column; align-items: center; gap: 0.5rem;
    color: #444; font-size: 0.9rem; pointer-events: none;
  }
  .drop-icon { font-size: 2rem; color: #2e2e4e; }
  .drop-zone.dragging .drop-icon { color: #6366f1; }
  .preview { max-width: 100%; max-height: 280px; object-fit: contain; }

  .pick-area { display: flex; gap: 0.75rem; margin-bottom: 1rem; flex-wrap: wrap; }
  .form-section { background: #1a1a2e; border-radius: 10px; padding: 1.25rem; margin-top: 1rem; }
  .tipo-toggle { display: flex; margin-bottom: 1rem; border-radius: 6px; overflow: hidden; border: 1px solid #2e2e4e; }
  .tipo-toggle button { flex: 1; padding: 0.5rem; border: none; background: #0f0f1a; color: #888; cursor: pointer; font-size: 0.9rem; }
  .tipo-toggle button.active { background: #6366f1; color: #fff; }
  form { display: flex; flex-direction: column; gap: 0.75rem; }
  label { display: flex; flex-direction: column; gap: 0.25rem; font-size: 0.85rem; color: #aaa; }
  input, select { padding: 0.5rem 0.75rem; border: 1px solid #2e2e4e; border-radius: 6px; background: #0f0f1a; color: #e0e0f0; font-size: 0.9rem; }
  .btn-primary { background: #6366f1; color: #fff; border: none; padding: 0.55rem 1.1rem; border-radius: 6px; cursor: pointer; font-size: 0.9rem; }
  .btn-primary:disabled { opacity: 0.6; cursor: not-allowed; }
  .btn-secondary { background: #1a1a2e; border: 1px solid #2e2e4e; color: #ccc; padding: 0.55rem 1.1rem; border-radius: 6px; cursor: pointer; font-size: 0.9rem; }
  .btn-ghost { background: none; border: none; color: #555; padding: 0.55rem 0.75rem; border-radius: 6px; cursor: pointer; font-size: 0.85rem; }
  .btn-ghost:hover { color: #f87171; }
  .full-width { width: 100%; margin-top: 0.25rem; }
  .error { color: #f87171; font-size: 0.85rem; margin-bottom: 0.25rem; }
  .hint-error { color: #888; font-size: 0.8rem; }
  code { background: #1a1a2e; padding: 0.15rem 0.4rem; border-radius: 3px; }
  .cat-row { display: flex; gap: 0.4rem; align-items: center; }
  .cat-row select { flex: 1; }
  .btn-new-cat { white-space: nowrap; background: #2a2a3e; border: 1px solid #3e3e5e; color: #a5b4fc; padding: 0.4rem 0.6rem; border-radius: 6px; cursor: pointer; font-size: 0.78rem; }
  .new-cat-inline { display: flex; gap: 0.4rem; margin-top: 0.4rem; align-items: center; }
  .new-cat-inline input:not([type="color"]) { flex: 1; }
  .new-cat-inline input[type="color"] { width: 36px; height: 32px; padding: 0.1rem; cursor: pointer; flex-shrink: 0; }
  .btn-sm { padding: 0.4rem 0.75rem; font-size: 0.82rem; }
</style>
