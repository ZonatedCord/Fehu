<script lang="ts">
  import { Upload, Download } from '@lucide/svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { api } from '../lib/api';

  let startDate = $state('');
  let endDate = $state('');
  let exporting = $state(false);
  let importing = $state(false);
  let importResult = $state('');
  let error = $state('');
  let success = $state('');

  async function doImport() {
    const path = await open({ filters: [{ name: 'Excel', extensions: ['xlsx'] }] });
    if (!path || typeof path !== 'string') return;
    importing = true; error = ''; importResult = '';
    try {
      const count: number = await invoke('import_xlsx', { filePath: path });
      importResult = `Importate ${count} transazioni.`;
    } catch (e: any) { error = e.message ?? String(e); }
    finally { importing = false; }
  }

  async function doExport() {
    exporting = true; error = ''; success = '';
    try {
      const csv = await api.exportCsv({ start_date: startDate || undefined, end_date: endDate || undefined });
      const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url; a.download = `fehu-${new Date().toISOString().slice(0,10)}.csv`; a.click();
      URL.revokeObjectURL(url);
      success = 'Scaricato.';
    } catch (e: any) { error = e.message ?? String(e); }
    finally { exporting = false; }
  }
</script>

<div class="page">
  <h1>Dati</h1>

  <section>
    <h2>Importa registro lavoro (.xlsx)</h2>
    <p class="hint">Legge il foglio "Registro Lavoro". Colonne: Data, Lavoro, Tipo, Incasso €, Pagato.</p>
    <button class="btn-secondary" onclick={doImport} disabled={importing}>
      <Upload size={14} />
      {importing ? 'Importazione…' : 'Scegli file .xlsx'}
    </button>
    {#if importResult}<p class="success">{importResult}</p>{/if}
  </section>

  <hr />

  <section>
    <h2>Esporta CSV</h2>
    <p class="hint">Lascia le date vuote per esportare tutto.</p>
    <form onsubmit={(e) => { e.preventDefault(); doExport(); }}>
      <label>Dal<input type="date" bind:value={startDate} /></label>
      <label>Al<input type="date" bind:value={endDate} /></label>
      <button type="submit" class="btn-primary" disabled={exporting}>
        <Download size={14} />
        {exporting ? 'Esportazione…' : 'Esporta CSV'}
      </button>
    </form>
    {#if success}<p class="success">{success}</p>{/if}
  </section>

  {#if error}<p class="error">{error}</p>{/if}
</div>

<style>
  .page { max-width: 480px; }
  h1 { margin: 0 0 1.5rem; font-size: 1.5rem; }
  h2 { font-size: 1rem; color: #aaa; margin: 0 0 0.5rem; font-weight: 500; }
  section { margin-bottom: 1.5rem; }
  hr { border: none; border-top: 1px solid #2e2e4e; margin: 1.5rem 0; }
  .hint { color: #666; font-size: 0.8rem; margin-bottom: 0.75rem; line-height: 1.5; }
  form { display: flex; flex-direction: column; gap: 0.75rem; }
  label { display: flex; flex-direction: column; gap: 0.25rem; font-size: 0.85rem; color: #aaa; }
  input[type="date"] { padding: 0.5rem 0.75rem; border: 1px solid #2e2e4e; border-radius: 6px; background: #0f0f1a; color: #e0e0f0; font-size: 0.9rem; }
  .btn-primary, .btn-secondary {
    display: inline-flex; align-items: center; gap: 0.4rem;
    border: none; padding: 0.55rem 1.1rem; border-radius: 6px; cursor: pointer; font-size: 0.9rem;
  }
  .btn-primary { background: #6366f1; color: #fff; align-self: flex-start; margin-top: 0.25rem; }
  .btn-secondary { background: #1a1a2e; border: 1px solid #2e2e4e; color: #ccc; }
  .btn-primary:disabled, .btn-secondary:disabled { opacity: 0.6; cursor: not-allowed; }
  .success { color: #4ade80; margin-top: 0.75rem; }
  .error { color: #f87171; margin-top: 0.75rem; }
</style>
