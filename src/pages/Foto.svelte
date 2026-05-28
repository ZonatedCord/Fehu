<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { api } from '../lib/api';
  import { currentPage } from '../lib/stores';
  import type { Category, ReceiptData } from '../lib/types';

  let imagePath = $state('');
  let previewUrl = $state('');
  let analyzing = $state(false);
  let receipt = $state<ReceiptData | null>(null);
  let categories = $state<Category[]>([]);
  let error = $state('');
  let saving = $state(false);

  // Form fields (pre-filled from OCR, editable by user)
  let tipo = $state<'income' | 'expense'>('expense');
  let importo = $state(0);
  let data = $state('');
  let descrizione = $state('');
  let categoriaId = $state<number | null>(null);

  onMount(async () => {
    categories = await api.listCategories().catch(() => []);
  });

  async function scegliImmagine() {
    const path = await open({
      filters: [{ name: 'Immagini', extensions: ['jpg', 'jpeg', 'png', 'webp', 'heic'] }],
    });
    if (!path || typeof path !== 'string') return;
    imagePath = path;
    previewUrl = convertFileSrc(path);
    receipt = null;
    error = '';
  }

  async function analizza() {
    if (!imagePath) return;
    analyzing = true;
    error = '';
    receipt = null;
    try {
      const result: ReceiptData = await invoke('analyze_receipt', { imagePath });
      receipt = result;
      // Pre-fill form
      if (result.importo) importo = result.importo;
      if (result.data) {
        // Convert dd/MM/yyyy or yyyy-MM-dd to yyyy-MM-dd for input
        const parts = result.data.split(/[-/]/);
        if (parts.length === 3) {
          data = parts[0].length === 4
            ? result.data  // already YYYY-MM-DD
            : `${parts[2]}-${parts[1].padStart(2,'0')}-${parts[0].padStart(2,'0')}`;
        }
      }
      if (result.descrizione) descrizione = result.descrizione;
      // Try to match categoria to existing categories
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
      // Reset and go to transactions
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
  <h1>Foto scontrino</h1>
  <p class="hint">Scatta o scegli una foto dello scontrino — i dati vengono estratti automaticamente.</p>

  <div class="pick-area">
    <button class="btn-secondary" onclick={scegliImmagine}>
      Scegli foto
    </button>
    {#if imagePath}
      <button class="btn-primary" onclick={analizza} disabled={analyzing}>
        {analyzing ? 'Analisi in corso…' : 'Analizza scontrino'}
      </button>
    {/if}
  </div>

  {#if previewUrl}
    <img src={previewUrl} alt="Anteprima scontrino" class="preview" />
  {/if}

  {#if error}
    <p class="error">{error}</p>
    {#if error.includes('moondream')}
      <p class="hint-error">Esegui: <code>ollama pull moondream</code></p>
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
          <select bind:value={categoriaId}>
            <option value={null}>— nessuna —</option>
            {#each categories as cat (cat.id)}
              <option value={cat.id}>{cat.name}</option>
            {/each}
          </select>
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
  .hint { color: #666; font-size: 0.85rem; margin-bottom: 1.5rem; }
  .pick-area { display: flex; gap: 0.75rem; margin-bottom: 1rem; flex-wrap: wrap; }
  .preview { max-width: 100%; max-height: 280px; border-radius: 8px; object-fit: contain; margin-bottom: 1rem; border: 1px solid #2e2e4e; }
  .form-section { background: #1a1a2e; border-radius: 10px; padding: 1.25rem; margin-top: 1rem; }
  .tipo-toggle { display: flex; gap: 0; margin-bottom: 1rem; border-radius: 6px; overflow: hidden; border: 1px solid #2e2e4e; }
  .tipo-toggle button { flex: 1; padding: 0.5rem; border: none; background: #0f0f1a; color: #888; cursor: pointer; font-size: 0.9rem; }
  .tipo-toggle button.active { background: #6366f1; color: #fff; }
  form { display: flex; flex-direction: column; gap: 0.75rem; }
  label { display: flex; flex-direction: column; gap: 0.25rem; font-size: 0.85rem; color: #aaa; }
  input, select { padding: 0.5rem 0.75rem; border: 1px solid #2e2e4e; border-radius: 6px; background: #0f0f1a; color: #e0e0f0; font-size: 0.9rem; }
  .btn-primary { background: #6366f1; color: #fff; border: none; padding: 0.55rem 1.1rem; border-radius: 6px; cursor: pointer; font-size: 0.9rem; }
  .btn-primary:disabled { opacity: 0.6; cursor: not-allowed; }
  .btn-secondary { background: #1a1a2e; border: 1px solid #2e2e4e; color: #ccc; padding: 0.55rem 1.1rem; border-radius: 6px; cursor: pointer; font-size: 0.9rem; }
  .full-width { width: 100%; margin-top: 0.25rem; }
  .error { color: #f87171; font-size: 0.85rem; }
  .hint-error { color: #888; font-size: 0.8rem; margin-top: 0.25rem; }
  code { background: #1a1a2e; padding: 0.15rem 0.4rem; border-radius: 3px; font-size: 0.85rem; }
</style>
