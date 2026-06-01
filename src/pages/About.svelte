<script lang="ts">
  import { getVersion } from '@tauri-apps/api/app';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { Globe, ExternalLink } from '@lucide/svelte';
  import { onMount } from 'svelte';

  let version = $state('');

  onMount(async () => {
    try { version = await getVersion(); } catch { version = '—'; }
  });

  async function navigate(url: string) {
    try { await openUrl(url); } catch {}
  }

  const stack = [
    { label: 'Tauri 2', desc: 'Desktop app framework' },
    { label: 'Svelte 5', desc: 'Reactive UI' },
    { label: 'Rust', desc: 'Backend & comandi' },
    { label: 'SQLite', desc: 'Storage locale (rusqlite)' },
    { label: 'Tesseract 5', desc: 'OCR locale' },
    { label: 'Ollama', desc: 'LLM locale (opzionale)' },
  ];
</script>

<div class="page">
  <div class="hero">
    <div class="rune">ᚠ</div>
    <div>
      <h1>Fehu</h1>
      <p class="tagline">Tracker finanziario locale. Nessun cloud, nessuna API esterna.</p>
      <p class="etymology"><em>ᚠ Fehu</em> — runa norrena, significa «ricchezza, bestiame». Prima runa del futhark antico, simbolo di prosperità e abbondanza.</p>
      {#if version}
        <span class="version">v{version}</span>
      {/if}
    </div>
  </div>

  <div class="sections">
    <section class="why-section">
      <h2>Perché esiste</h2>
      <p class="why-text">Cercavo un'app per le finanze personali che fosse <strong>gratuita</strong>, <strong>locale</strong> (niente cloud, niente account) e abbastanza completa da gestire categorie, ricorrenti, P.IVA e foto ricevute.</p>
      <p class="why-text">Non ho trovato nulla che soddisfacesse tutti e tre i requisiti. I software gratuiti erano troppo basilari. Quelli potenti erano a pagamento o basati su cloud.</p>
      <p class="why-text why-conclusion">Nessuno l'aveva costruito nel modo giusto. Allora l'ho costruito io.</p>
    </section>

    <section>
      <h2>Autore</h2>
      <div class="links">
        <button class="link-btn" onclick={() => navigate('https://marcobarlera.com')}>
          <Globe size={15} />
          marcobarlera.com
        </button>
        <button class="link-btn" onclick={() => navigate('https://github.com/marcobarlera')}>
          <ExternalLink size={15} />
          GitHub
        </button>
      </div>
    </section>

    <section>
      <h2>Stack</h2>
      <ul class="stack-list">
        {#each stack as item}
          <li>
            <span class="stack-label">{item.label}</span>
            <span class="stack-desc">{item.desc}</span>
          </li>
        {/each}
      </ul>
    </section>

    <section>
      <h2>Principi</h2>
      <ul class="principles">
        <li>Dati finanziari 100% locali — solo nel tuo dispositivo</li>
        <li>Zero dipendenze cloud o API esterne a pagamento</li>
        <li>OCR e LLM eseguiti in locale (Tesseract + Ollama)</li>
        <li>Open source — codice ispezionabile e modificabile</li>
      </ul>
    </section>
  </div>
</div>

<style>
  .page { max-width: 560px; }

  .hero {
    display: flex; align-items: center; gap: 1.25rem;
    margin-bottom: 2.5rem; padding-bottom: 2rem;
    border-bottom: 1px solid var(--border);
  }
  .rune {
    font-size: 3.5rem; line-height: 1;
    color: var(--accent-lt); flex-shrink: 0;
  }
  h1 { margin: 0 0 0.25rem; font-size: 2rem; letter-spacing: 0.04em; }
  .tagline { margin: 0 0 0.3rem; color: var(--text-dim); font-size: 0.9rem; }
  .etymology { margin: 0 0 0.5rem; color: var(--text-muted); font-size: 0.78rem; line-height: 1.5; }
  .version {
    display: inline-block; background: var(--bg-card);
    color: var(--text-dim); font-size: 0.78rem; padding: 0.15rem 0.5rem;
    border-radius: 4px; border: 1px solid var(--border);
  }

  .sections { display: flex; flex-direction: column; gap: 1.5rem; }
  section { background: var(--bg-card); border-radius: 16px; padding: 1.25rem 1.5rem; box-shadow: 0 1px 3px rgba(0,0,0,0.06); }
  .why-section { border-left: 3px solid var(--accent); }
  .why-text { margin: 0 0 0.6rem; font-size: 0.9rem; color: var(--text-muted); line-height: 1.6; }
  .why-text:last-child { margin-bottom: 0; }
  .why-conclusion { color: var(--text); font-weight: 600; font-style: italic; }
  h2 {
    margin: 0 0 1rem; font-size: 0.78rem; font-weight: 600;
    text-transform: uppercase; letter-spacing: 0.08em; color: var(--text-dim);
  }

  .links { display: flex; gap: 0.75rem; flex-wrap: wrap; }
  .link-btn {
    display: flex; align-items: center; gap: 0.4rem;
    background: var(--bg-base); border: 1px solid var(--border);
    color: var(--accent-lt); padding: 0.5rem 0.9rem;
    border-radius: 10px; cursor: pointer; font-size: 0.875rem;
    transition: border-color 0.15s;
  }
  .link-btn:hover { border-color: var(--accent); }

  .stack-list {
    list-style: none; margin: 0; padding: 0;
    display: flex; flex-direction: column; gap: 0.5rem;
  }
  .stack-list li {
    display: flex; justify-content: space-between; align-items: baseline;
    font-size: 0.875rem;
  }
  .stack-label { color: var(--text); font-weight: 500; }
  .stack-desc { color: var(--text-dim); font-size: 0.8rem; }

  .principles {
    margin: 0; padding: 0; list-style: none;
    display: flex; flex-direction: column; gap: 0.6rem;
  }
  .principles li {
    font-size: 0.875rem; color: var(--text-muted); padding-left: 1rem;
    position: relative;
  }
  .principles li::before {
    content: '—'; position: absolute; left: 0; color: var(--text-dim);
  }
</style>
