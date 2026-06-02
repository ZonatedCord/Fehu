<script lang="ts">
  import { CheckCircle, XCircle, Circle } from '@lucide/svelte';
  import { api } from '../lib/api';
  import type { DepsStatus } from '../lib/types';

  let { onComplete }: { onComplete: () => void } = $props();

  let step = $state(1);
  let deps = $state<DepsStatus | null>(null);
  let checking = $state(false);
  let installing = $state(false);
  let installMsg = $state('');
  let copied = $state(false);

  const OLLAMA_PROMPT = "Devo configurare Ollama sul mio computer per farlo funzionare con un'app desktop chiamata Fehu (tracker finanziario locale) che lo usa per categorizzare automaticamente le spese. Aiutami a: 1) installare Ollama, 2) scegliere un modello leggero per classificazione testuale (llama3.2, qwen2.5, mistral o phi4-mini), 3) verificare che funzioni. Dimmi prima il mio sistema operativo.";

  async function goToStep2() {
    step = 2;
    checking = true;
    try {
      deps = await api.checkDependencies();
    } catch {
      deps = { tesseract: false, ollama: false, tesseract_version: null };
    } finally {
      checking = false;
    }
  }

  async function installTesseract() {
    installing = true; installMsg = 'Installazione Tesseract…';
    try {
      const r = await api.installDependency('tesseract');
      installMsg = r.success ? '✓ Tesseract installato.' : r.output.slice(0, 200);
      if (r.success) deps = await api.checkDependencies();
    } catch (e: any) { installMsg = String(e); }
    finally { installing = false; }
  }

  function copyOllamaPrompt() {
    navigator.clipboard.writeText(OLLAMA_PROMPT).then(() => { copied = true; setTimeout(() => copied = false, 1500); });
  }

  async function complete() {
    try { await api.setSetting('onboarded', 'true'); } catch {}
    onComplete();
  }
</script>

<div class="overlay">
  <div class="card">
    <!-- Step indicator -->
    <div class="steps">
      {#each [1, 2, 3] as s}
        <div class="step-dot" class:active={step === s} class:done={step > s}></div>
      {/each}
    </div>

    {#if step === 1}
      <div class="rune">ᚠ</div>
      <h1>Benvenuto in Fehu</h1>
      <p class="body">
        Fehu è un tracker finanziario personale che gira interamente sul tuo dispositivo.
        Nessun dato viene inviato a server esterni — tutto rimane locale.
      </p>
      <ul class="feature-list">
        <li>Tieni traccia di entrate e uscite</li>
        <li>Analisi scontrini con OCR locale (Tesseract)</li>
        <li>Categorizzazione con LLM locale (Ollama — opzionale)</li>
        <li>Obiettivi di risparmio e gestione patrimonio</li>
        <li>Calcolatore P.IVA per freelancer italiani</li>
      </ul>
      <button class="btn-primary" onclick={goToStep2}>Inizia setup →</button>

    {:else if step === 2}
      <h1>Controllo dipendenze</h1>
      <p class="body">Fehu usa strumenti locali per OCR e categorizzazione. Entrambi sono opzionali.</p>

      {#if checking}
        <div class="dep-row">
          <Circle size={20} class="checking" />
          <span>Verifica in corso…</span>
        </div>
      {:else if deps}
        <div class="deps">
          <div class="dep-row">
            {#if deps.tesseract}
              <CheckCircle size={20} color="#4ade80" />
              <div>
                <span class="dep-name">Tesseract OCR</span>
                {#if deps.tesseract_version}<span class="dep-version">{deps.tesseract_version}</span>{/if}
              </div>
            {:else}
              <XCircle size={20} color="#f87171" />
              <div>
                <span class="dep-name">Tesseract OCR — non trovato</span>
                <div class="dep-actions">
                  <button class="btn-install-dep" onclick={installTesseract} disabled={installing}>
                    {installing ? 'Installazione…' : '↓ Installa (macOS)'}
                  </button>
                  <code class="install-cmd">Windows: github.com/UB-Mannheim/tesseract</code>
                </div>
                {#if installMsg}<span class="dep-install-msg" class:ok={installMsg.startsWith('✓')}>{installMsg}</span>{/if}
              </div>
            {/if}
          </div>

          <div class="dep-row">
            {#if deps.ollama}
              <CheckCircle size={20} color="#4ade80" />
              <span class="dep-name">Ollama — in esecuzione</span>
            {:else}
              <XCircle size={20} color="#f87171" />
              <div>
                <span class="dep-name">Ollama — non trovato <span class="optional">(opzionale)</span></span>
                <span class="dep-hint">Senza Ollama la categoria viene stimata da parole chiave.</span>
                <button class="btn-install-dep" onclick={copyOllamaPrompt} style="margin-top:0.25rem;">
                  {copied ? '✓ Copiato' : 'Copia prompt AI per installazione guidata'}
                </button>
              </div>
            {/if}
          </div>
        </div>
      {/if}

      <div class="row">
        <button class="btn-ghost" onclick={() => { step = 1; }}>← Indietro</button>
        <button class="btn-primary" onclick={() => { step = 3; }} disabled={checking}>
          Continua →
        </button>
      </div>

    {:else}
      <div class="rune success">✓</div>
      <h1>Tutto pronto</h1>
      <p class="body">
        Puoi iniziare ad usare Fehu. Se Tesseract o Ollama non erano presenti,
        puoi installarli in qualsiasi momento e riprendere a usare le funzionalità di analisi.
      </p>
      <p class="body-small">
        Vai in <strong>Impostazioni</strong> per personalizzare l'URL di Ollama,
        il percorso di Tesseract e il simbolo di valuta.
      </p>
      <button class="btn-primary" onclick={complete}>Apri Fehu</button>
    {/if}
  </div>
</div>

<style>
  .overlay {
    position: fixed; inset: 0;
    background: rgba(0, 0, 0, 0.75);
    display: flex; align-items: center; justify-content: center;
    z-index: 100; backdrop-filter: blur(4px);
  }
  .card {
    background: var(--bg-card); border: 1px solid var(--border);
    border-radius: 14px; padding: 2.5rem 2rem;
    width: 100%; max-width: 480px;
    display: flex; flex-direction: column; gap: 1.25rem;
  }

  .steps { display: flex; gap: 0.4rem; }
  .step-dot {
    width: 6px; height: 6px; border-radius: 50%;
    background: var(--border); transition: background 0.2s;
  }
  .step-dot.active { background: var(--accent); }
  .step-dot.done { background: var(--income); }

  .rune { font-size: 2.5rem; color: var(--accent-lt); }
  .rune.success { color: var(--income); }
  h1 { margin: 0; font-size: 1.4rem; }
  .body { margin: 0; color: var(--text-muted); font-size: 0.9rem; line-height: 1.6; }
  .body-small { margin: 0; color: var(--text-dim); font-size: 0.82rem; line-height: 1.6; }

  .feature-list {
    margin: 0; padding: 0 0 0 1.2rem;
    color: var(--text-muted); font-size: 0.875rem;
    display: flex; flex-direction: column; gap: 0.35rem;
  }

  .deps { display: flex; flex-direction: column; gap: 1rem; }
  .dep-row {
    display: flex; align-items: flex-start; gap: 0.75rem;
    font-size: 0.875rem;
  }
  .dep-name { color: var(--text); display: block; }
  .optional { color: var(--text-dim); font-size: 0.78rem; font-weight: 400; }
  .dep-version { color: var(--text-dim); font-size: 0.78rem; display: block; margin-top: 0.15rem; }
  .dep-hint { color: var(--text-dim); font-size: 0.78rem; display: block; margin-top: 0.15rem; }
  .dep-actions { display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap; margin-top: 0.3rem; }
  .install-cmd {
    background: var(--bg-elevated); color: var(--accent-lt);
    padding: 0.2rem 0.5rem; border-radius: 4px;
    font-size: 0.78rem; font-family: monospace;
  }
  .btn-install-dep {
    background: var(--accent); color: #fff; border: none;
    border-radius: 7px; padding: 0.35rem 0.75rem;
    font-size: 0.78rem; font-weight: 600; cursor: pointer;
  }
  .btn-install-dep:disabled { opacity: 0.6; cursor: not-allowed; }
  .dep-install-msg { font-size: 0.75rem; color: var(--expense); display: block; margin-top: 0.25rem; }
  .dep-install-msg.ok { color: var(--income); }

  .row { display: flex; gap: 0.75rem; justify-content: flex-end; }
  .btn-primary {
    background: var(--accent); color: #fff; border: none;
    padding: 0.65rem 1.4rem; border-radius: 10px;
    cursor: pointer; font-size: 0.9rem; font-weight: 600;
  }
  .btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
  .btn-ghost {
    background: none; border: 1px solid var(--border); color: var(--text-dim);
    padding: 0.65rem 1rem; border-radius: 10px; cursor: pointer; font-size: 0.9rem;
  }
  .btn-ghost:hover { color: var(--text-muted); }
</style>
