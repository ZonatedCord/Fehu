<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
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

  let tipo = $state<'income' | 'expense'>('expense');
  let metodo = $state('contanti');
  let importo = $state(0);
  let data = $state('');
  let descrizione = $state('');
  let categoriaId = $state<number | null>(null);

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

  onMount(async () => {
    categories = await api.listCategories().catch(() => []);
    const unDrag   = await listen('tauri://drag',           () => { dragging = true; });
    const unLeave  = await listen('tauri://drag-leave',     () => { dragging = false; });
    const unCancel = await listen('tauri://drag-cancelled', () => { dragging = false; });
    const unDrop   = await listen('tauri://drag-drop', async (e: any) => {
      dragging = false;
      const paths: string[] = e.payload.paths ?? [];
      const img = paths.find((p: string) => IMAGE_EXTS.some(ext => p.toLowerCase().endsWith('.' + ext)));
      if (img) await setImage(img);
    });
    return () => { unDrag(); unLeave(); unCancel(); unDrop(); };
  });

  async function setImage(path: string) {
    imagePath = path; previewUrl = ''; receipt = null; error = '';
    try { previewUrl = await invoke<string>('read_image_base64', { path }); } catch { previewUrl = ''; }
  }

  async function scegliImmagine() {
    const path = await open({ filters: [{ name: 'Immagini', extensions: IMAGE_EXTS }] });
    if (path && typeof path === 'string') await setImage(path);
  }

  async function analizza() {
    if (!imagePath) return;
    analyzing = true; error = ''; receipt = null;
    try {
      const result: ReceiptData = await invoke('analyze_receipt', { imagePath });
      receipt = result;
      if (result.importo) importo = result.importo;
      if (result.data) {
        const parts = result.data.split(/[-/]/);
        if (parts.length === 3) {
          data = parts[0].length === 4 ? result.data
            : `${parts[2]}-${parts[1].padStart(2,'0')}-${parts[0].padStart(2,'0')}`;
        }
      }
      if (result.descrizione) descrizione = result.descrizione;
      if (result.categoria) {
        const match = categories.find(c => c.name.toLowerCase() === result.categoria!.toLowerCase());
        if (match) {
          categoriaId = match.id;
        } else {
          // crea categoria al volo se non esiste
          try {
            const cat = await api.createCategory(result.categoria!, '#6366f1', 'package');
            categories = [...categories, cat].sort((a, b) => a.name.localeCompare(b.name));
            categoriaId = cat.id;
          } catch { /* ignora errori duplicati */ }
        }
      }
    } catch (e: any) { error = e.message ?? String(e); }
    finally { analyzing = false; }
  }

  async function salva() {
    if (importo <= 0) { error = 'Inserisci un importo valido'; return; }
    saving = true;
    try {
      await api.createTransaction({
        amount: importo, type: tipo, category_id: categoriaId,
        date: data || new Date().toISOString().slice(0, 10),
        description: descrizione, notes: '', metodo,
      });
      imagePath = ''; previewUrl = ''; receipt = null;
      importo = 0; data = ''; descrizione = ''; categoriaId = null; metodo = 'contanti';
      currentPage.set('transactions');
    } catch (e: any) { error = e.message ?? String(e); }
    finally { saving = false; }
  }
</script>

<div class="page">
  <div class="page-header">
    <div>
      <h1>Foto / Spesa</h1>
      <p class="hint">Trascina o scegli un'immagine — scontrino, fattura, screenshot.</p>
    </div>
  </div>

  <div class="two-col">
    <!-- SINISTRA: zona drop + preview -->
    <div class="left-col">
      <div class="drop-zone" class:dragging>
        {#if previewUrl}
          <img src={previewUrl} alt="Anteprima" class="preview" />
        {:else}
          <div class="drop-placeholder">
            <span class="drop-icon">↓</span>
            <span>{dragging ? 'Rilascia qui' : 'Trascina un\'immagine'}</span>
          </div>
        {/if}
      </div>

      <div class="pick-area">
        <button class="btn-secondary" onclick={scegliImmagine}>Scegli file</button>
        {#if imagePath}
          <button class="btn-primary" onclick={analizza} disabled={analyzing}>
            {analyzing ? 'Analisi…' : 'Analizza'}
          </button>
          <button class="btn-ghost" onclick={() => { imagePath=''; previewUrl=''; receipt=null; error=''; importo=0; data=''; descrizione=''; }}>
            Rimuovi
          </button>
        {/if}
      </div>

      {#if error}
        <p class="error">{error}</p>
        {#if error.includes('Tesseract')}
          <p class="hint-error">Esegui: <code>brew install tesseract tesseract-lang</code></p>
        {/if}
      {/if}
    </div>

    <!-- DESTRA: form -->
    <div class="right-col">
      {#if imagePath}
        <div class="form-section">
          <h2>
            {#if analyzing}Estrazione in corso…
            {:else if receipt}
              Dati estratti — verifica e salva
              {#if receipt.categoria_source === 'keyword'}
                <span class="badge-offline">categoria stimata (offline)</span>
              {/if}
            {:else}Compila o premi Analizza{/if}
          </h2>

          <div class="tipo-toggle">
            <button class:active={tipo === 'expense'} onclick={() => tipo = 'expense'}>Uscita</button>
            <button class:active={tipo === 'income'} onclick={() => tipo = 'income'}>Entrata</button>
          </div>

          <form onsubmit={(e) => { e.preventDefault(); salva(); }}>
            <label>Importo (€)
              <input type="number" bind:value={importo} min="0.01" step="0.01" required />
            </label>
            <label>Data
              <input type="date" bind:value={data} />
            </label>
            <label>Descrizione
              <input bind:value={descrizione} placeholder="es. Caffè al bar" />
            </label>
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
                  <input bind:value={newCatName} placeholder="Nome categoria"
                    onkeydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); createCategoryInline(); } }} />
                  <input type="color" bind:value={newCatColor} />
                  <button type="button" class="btn-primary btn-sm" onclick={createCategoryInline}
                    disabled={newCatSaving || !newCatName.trim()}>
                    {newCatSaving ? '…' : 'Crea'}
                  </button>
                </div>
              {/if}
            </label>
            <label>Metodo
              <select bind:value={metodo}>
                <option value="contanti">Contanti</option>
                <option value="carta">Carta</option>
                <option value="altro">Altro</option>
              </select>
            </label>
            <button type="submit" class="btn-primary full-width" disabled={saving}>
              {saving ? 'Salvataggio…' : 'Salva transazione'}
            </button>
          </form>
        </div>
      {:else}
        <div class="right-empty">
          <span>Carica un'immagine per iniziare</span>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .page { max-width: 960px; }
  .page-header { margin-bottom: 1.25rem; }
  h1 { margin: 0 0 0.2rem; font-size: 1.5rem; }
  h2 { margin: 0 0 1rem; font-size: 0.95rem; color: #aaa; font-weight: 500; display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap; }
  .badge-offline { font-size: 0.72rem; background: #1a1a2e; border: 1px solid #2e2e4e; color: #555; padding: 0.1rem 0.45rem; border-radius: 4px; font-weight: 400; }
  .hint { color: #666; font-size: 0.85rem; margin: 0; }

  .two-col { display: grid; grid-template-columns: 1fr 1fr; gap: 1.5rem; align-items: start; }

  /* Drop zone */
  .drop-zone {
    min-height: 340px; border: 2px dashed #2e2e4e; border-radius: 10px;
    display: flex; align-items: center; justify-content: center;
    margin-bottom: 0.75rem; transition: border-color 0.15s, background 0.15s; overflow: hidden;
  }
  .drop-zone.dragging { border-color: #6366f1; background: #1e1e3e; }
  .drop-placeholder { display: flex; flex-direction: column; align-items: center; gap: 0.5rem; color: #444; font-size: 0.9rem; pointer-events: none; }
  .drop-icon { font-size: 2.5rem; color: #2e2e4e; }
  .drop-zone.dragging .drop-icon { color: #6366f1; }
  .preview { max-width: 100%; max-height: 400px; object-fit: contain; }

  .pick-area { display: flex; gap: 0.5rem; flex-wrap: wrap; margin-bottom: 0.75rem; }
  .error { color: #f87171; font-size: 0.85rem; margin: 0.25rem 0; }
  .hint-error { color: #888; font-size: 0.8rem; margin: 0; }
  code { background: #1a1a2e; padding: 0.15rem 0.4rem; border-radius: 3px; }

  /* Right col */
  .right-col { position: sticky; top: 1rem; }
  .right-empty { display: flex; align-items: center; justify-content: center; min-height: 340px; color: #333; font-size: 0.9rem; border: 2px dashed #1e1e2e; border-radius: 10px; }
  .form-section { background: #1a1a2e; border-radius: 10px; padding: 1.25rem; }
  .tipo-toggle { display: flex; margin-bottom: 1rem; border-radius: 6px; overflow: hidden; border: 1px solid #2e2e4e; }
  .tipo-toggle button { flex: 1; padding: 0.5rem; border: none; background: #0f0f1a; color: #888; cursor: pointer; font-size: 0.9rem; }
  .tipo-toggle button.active { background: #6366f1; color: #fff; }

  form { display: flex; flex-direction: column; gap: 0.75rem; }
  label { display: flex; flex-direction: column; gap: 0.25rem; font-size: 0.85rem; color: #aaa; }
  input, select { padding: 0.5rem 0.75rem; border: 1px solid #2e2e4e; border-radius: 6px; background: #0f0f1a; color: #e0e0f0; font-size: 0.9rem; }
  .btn-primary { background: #6366f1; color: #fff; border: none; padding: 0.55rem 1.1rem; border-radius: 6px; cursor: pointer; font-size: 0.9rem; }
  .btn-primary:disabled { opacity: 0.6; cursor: not-allowed; }
  .btn-secondary { background: #1a1a2e; border: 1px solid #2e2e4e; color: #ccc; padding: 0.55rem 1.1rem; border-radius: 6px; cursor: pointer; font-size: 0.9rem; }
  .btn-ghost { background: none; border: none; color: #555; padding: 0.55rem 0.5rem; cursor: pointer; font-size: 0.85rem; }
  .btn-ghost:hover { color: #f87171; }
  .full-width { width: 100%; margin-top: 0.25rem; }
  .cat-row { display: flex; gap: 0.4rem; align-items: center; }
  .cat-row select { flex: 1; }
  .btn-new-cat { white-space: nowrap; background: #2a2a3e; border: 1px solid #3e3e5e; color: #a5b4fc; padding: 0.4rem 0.6rem; border-radius: 6px; cursor: pointer; font-size: 0.78rem; }
  .new-cat-inline { display: flex; gap: 0.4rem; margin-top: 0.4rem; align-items: center; }
  .new-cat-inline input:not([type="color"]) { flex: 1; }
  .new-cat-inline input[type="color"] { width: 36px; height: 32px; padding: 0.1rem; cursor: pointer; flex-shrink: 0; }
  .btn-sm { padding: 0.4rem 0.75rem; font-size: 0.82rem; }
</style>
