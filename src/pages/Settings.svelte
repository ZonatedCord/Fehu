<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { getVersion } from '@tauri-apps/api/app';
  import { api } from '../lib/api';
  import { updateAvailable } from '../lib/stores';
  import type { AppSettings } from '../lib/types';

  let settings = $state<AppSettings>({
    ollama_url: 'http://localhost:11434',
    tesseract_path: '',
    currency_symbol: '€',
    onboarded: 'false',
    theme: 'dark',
  });
  let saving = $state(false);
  let saved = $state(false);
  let error = $state('');
  let botToken = $state('');
  let botRunning = $state(false);
  let botMsg = $state('');
  let botBusy = $state(false);
  let updateChecking = $state(false);
  let updateMsg = $state('');
  let updateReady = $state<any>(null);
  let currentVersion = $state('');
  let ollamaGuide = $state(false);
  let telegramGuide = $state(false);
  let ollamaPing = $state('');
  let copied = $state('');

  onMount(async () => {
    try {
      const [s, running, ver] = await Promise.all([
        api.getSettings(),
        api.getTelegramStatus(),
        getVersion(),
      ]);
      settings = s;
      botRunning = running;
      botToken = (s as any).telegram_token ?? '';
      currentVersion = ver;
    } catch (e: any) {
      error = e.message ?? String(e);
    }
  });

  async function save() {
    saving = true;
    saved = false;
    error = '';
    try {
      await api.setSetting('ollama_url', settings.ollama_url);
      await api.setSetting('tesseract_path', settings.tesseract_path);
      await api.setSetting('currency_symbol', settings.currency_symbol);
      if (botToken) await api.setSetting('telegram_token', botToken);
      saved = true;
      setTimeout(() => { saved = false; }, 2000);
    } catch (e: any) {
      error = e.message ?? String(e);
    } finally {
      saving = false;
    }
  }

  async function checkUpdate() {
    updateChecking = true; updateMsg = ''; updateReady = null;
    try {
      const { check } = await import('@tauri-apps/plugin-updater');
      const update = await check();
      if (update) {
        updateReady = update;
        updateAvailable.set(true);
        updateMsg = `v${update.version} disponibile`;
      } else {
        updateAvailable.set(false);
        updateMsg = 'Sei già alla versione più recente.';
      }
    } catch (e: any) { updateMsg = `Errore: ${e.message ?? String(e)}`; }
    finally { updateChecking = false; }
  }

  async function installUpdate() {
    if (!updateReady) return;
    updateChecking = true;
    updateMsg = 'Download in corso…';
    try {
      await updateReady.downloadAndInstall();
      updateMsg = 'Installato. Riavvia Fehu.';
      updateAvailable.set(false);
    } catch (e: any) { updateMsg = `Errore: ${e.message ?? String(e)}`; }
    finally { updateChecking = false; }
  }

  async function toggleBot() {
    botBusy = true; botMsg = ''; error = '';
    try {
      if (botRunning) {
        await api.stopTelegramBot();
        botRunning = false;
        botMsg = 'Bot fermato.';
      } else {
        if (!botToken.trim()) { error = 'Inserisci il token bot prima di avviare.'; return; }
        // Auto-save token so it persists across navigation
        await api.setSetting('telegram_token', botToken);
        const msg = await api.startTelegramBot(botToken);
        botRunning = true;
        botMsg = msg;
      }
    } catch (e: any) { error = e.message ?? String(e); }
    finally { botBusy = false; }
  }

  async function autoDetectTesseract() {
    settings.tesseract_path = '';
  }

  async function pingOllama() {
    ollamaPing = 'Connessione…';
    try {
      const res = await fetch(settings.ollama_url.replace(/\/$/, '') + '/api/tags', { signal: AbortSignal.timeout(4000) });
      if (res.ok) { const j = await res.json(); ollamaPing = `OK — ${j.models?.length ?? 0} modelli installati`; }
      else ollamaPing = `Errore HTTP ${res.status}`;
    } catch { ollamaPing = 'Nessuna risposta. Ollama non è in esecuzione?'; }
  }

  function copy(text: string, key: string) {
    navigator.clipboard.writeText(text).then(() => { copied = key; setTimeout(() => copied = '', 1500); });
  }
</script>

<div class="page">
  <div class="page-header">
    <h1>Impostazioni</h1>
    <p class="subtitle">Configurazione dell'app. Le modifiche vengono salvate nel database locale.</p>
  </div>

  <div class="sections">
    <section>
      <h2>OCR</h2>
      <label>
        <span>Percorso Tesseract</span>
        <p class="field-hint">Lascia vuoto per rilevamento automatico. Usa un percorso assoluto se Tesseract non viene trovato.</p>
        <div class="input-row">
          <input
            bind:value={settings.tesseract_path}
            placeholder="Auto-rilevamento (lascia vuoto)"
          />
          <button class="btn-ghost" onclick={autoDetectTesseract} type="button">Reimposta</button>
        </div>
      </label>
    </section>

    <section>
      <div class="section-header">
        <h2>LLM — Ollama</h2>
        <button class="btn-guide-toggle" onclick={() => ollamaGuide = !ollamaGuide}>
          {ollamaGuide ? 'Chiudi guida' : 'Come configurare'}
        </button>
      </div>

      {#if ollamaGuide}
        <div class="guide-box">
          <p class="guide-step"><span class="step-num">1</span> Scarica e installa Ollama da <strong>ollama.com</strong></p>
          <p class="guide-step"><span class="step-num">2</span> Scarica il modello (terminale):</p>
          <div class="code-row">
            <code>ollama pull qwen2.5-coder:7b</code>
            <button class="btn-copy" onclick={() => copy('ollama pull qwen2.5-coder:7b', 'ollama-pull')}>
              {copied === 'ollama-pull' ? '✓' : 'Copia'}
            </button>
          </div>
          <p class="guide-step"><span class="step-num">3</span> Ollama parte in automatico. L'URL default è già corretto.</p>
          <p class="guide-note">Senza Ollama, Fehu usa categorizzazione automatica per parole chiave.</p>
        </div>
      {/if}

      <label>
        <span>URL Ollama</span>
        <div class="input-row">
          <input bind:value={settings.ollama_url} placeholder="http://localhost:11434" />
          <button class="btn-ghost" onclick={pingOllama} type="button">Testa</button>
        </div>
        {#if ollamaPing}<p class="ping-msg" class:ok={ollamaPing.startsWith('OK')}>{ollamaPing}</p>{/if}
      </label>
    </section>

    <section>
      <h2>Valuta</h2>
      <label>
        <span>Simbolo valuta</span>
        <p class="field-hint">Usato per la visualizzazione degli importi.</p>
        <input bind:value={settings.currency_symbol} placeholder="€" style="max-width: 80px;" />
      </label>
    </section>

    <section>
      <h2>Aggiornamenti</h2>
      <div class="update-row">
        <div class="update-info">
          <span class="update-version">Versione corrente: <strong>v{currentVersion || '…'}</strong></span>
          {#if $updateAvailable && updateReady}
            <span class="update-badge">v{updateReady.version} disponibile</span>
          {/if}
        </div>
        <div class="update-actions">
          <button class="btn-ghost" onclick={checkUpdate} disabled={updateChecking}>
            {updateChecking ? 'Controllo…' : 'Controlla'}
          </button>
          {#if updateReady}
            <button class="btn-primary" onclick={installUpdate} disabled={updateChecking}>
              Installa v{updateReady.version}
            </button>
          {/if}
        </div>
      </div>
      {#if updateMsg}<p class="update-msg" class:ok={!updateReady && updateMsg.includes('recente')}>{updateMsg}</p>{/if}
    </section>

    <section>
      <div class="section-header">
        <h2>Telegram Bot</h2>
        <button class="btn-guide-toggle" onclick={() => telegramGuide = !telegramGuide}>
          {telegramGuide ? 'Chiudi guida' : 'Come configurare'}
        </button>
      </div>

      {#if telegramGuide}
        <div class="guide-box">
          <p class="guide-step"><span class="step-num">1</span> Apri Telegram e cerca <strong>@BotFather</strong></p>
          <p class="guide-step"><span class="step-num">2</span> Invia questo prompt a BotFather (copia e incolla):</p>
          <div class="code-row">
            <code>/newbot</code>
            <button class="btn-copy" onclick={() => copy('/newbot', 'newbot')}>
              {copied === 'newbot' ? '✓' : 'Copia'}
            </button>
          </div>
          <p class="guide-step"><span class="step-num">3</span> Scegli un nome per il bot (es. <em>Fehu Finance</em>) e uno username che finisce in <code>bot</code></p>
          <p class="guide-step"><span class="step-num">4</span> BotFather ti manda il token — incollalo qui sotto</p>
          <p class="guide-step"><span class="step-num">5</span> Assicurati di avere Python 3 installato:</p>
          <div class="code-row">
            <code>pip3 install aiogram aiosqlite</code>
            <button class="btn-copy" onclick={() => copy('pip3 install aiogram aiosqlite', 'pip')}>
              {copied === 'pip' ? '✓' : 'Copia'}
            </button>
          </div>
          <p class="guide-note">Il bot gira in locale sul tuo Mac — nessun server esterno.</p>
        </div>
      {/if}

      <label>
        <span>Token bot</span>
        <input
          type="password"
          bind:value={botToken}
          placeholder="1234567890:AAF..."
          autocomplete="off"
        />
      </label>
      <div class="bot-row">
        <button
          class="btn-bot"
          class:running={botRunning}
          onclick={toggleBot}
          disabled={botBusy}
        >
          {botBusy ? '…' : botRunning ? 'Ferma bot' : 'Avvia bot'}
        </button>
        <span class="bot-status" class:on={botRunning}>{botRunning ? 'in esecuzione' : 'fermo'}</span>
      </div>
      {#if botMsg}<p class="bot-msg">{botMsg}</p>{/if}
    </section>

    <div class="actions">
      <button class="btn-primary" onclick={save} disabled={saving}>
        {saving ? 'Salvataggio…' : saved ? 'Salvato' : 'Salva impostazioni'}
      </button>
      {#if error}
        <p class="error">{error}</p>
      {/if}
    </div>
  </div>
</div>

<style>
  .page { max-width: 600px; }
  .page-header { margin-bottom: 2rem; }
  h1 { margin: 0 0 0.4rem; font-size: 1.5rem; }
  .subtitle { color: var(--text-dim); font-size: 0.875rem; margin: 0; }

  .sections { display: flex; flex-direction: column; gap: 1.75rem; }
  section { background: var(--bg-card); border-radius: 16px; padding: 1.25rem 1.5rem; display: flex; flex-direction: column; gap: 1rem; box-shadow: 0 1px 3px rgba(0,0,0,0.06); }
  h2 { margin: 0 0 0.25rem; font-size: 0.8rem; font-weight: 600; text-transform: uppercase; letter-spacing: 0.08em; color: var(--text-dim); }

  label { display: flex; flex-direction: column; gap: 0.3rem; font-size: 0.9rem; color: var(--text); }
  .field-hint { font-size: 0.78rem; color: var(--text-dim); margin: 0; line-height: 1.4; }
  .input-row { display: flex; gap: 0.5rem; align-items: center; }
  .input-row input { flex: 1; }

  input {
    padding: 0.5rem 0.75rem; border: 1px solid var(--border);
    border-radius: 6px; background: var(--bg-base); color: var(--text);
    font-size: 0.9rem; width: 100%;
  }
  input:focus { outline: 1px solid #6366f1; }

  .actions { display: flex; flex-direction: column; gap: 0.5rem; }
  .btn-primary {
    background: var(--accent); color: #fff; border: none;
    padding: 0.6rem 1.25rem; border-radius: 6px;
    cursor: pointer; font-size: 0.9rem; align-self: flex-start;
  }
  .btn-primary:disabled { opacity: 0.6; cursor: not-allowed; }
  .btn-ghost {
    background: none; border: 1px solid var(--border); color: var(--text-dim);
    padding: 0.5rem 0.75rem; border-radius: 6px; cursor: pointer;
    font-size: 0.82rem; white-space: nowrap;
  }
  .btn-ghost:hover { color: #aaa; border-color: #3e3e5e; }
  .error { color: var(--expense); font-size: 0.85rem; margin: 0; white-space: pre-wrap; user-select: text; }
  code { background: var(--bg-elevated); padding: 0.1rem 0.35rem; border-radius: 4px; font-size: 0.8rem; }
  .bot-row { display: flex; align-items: center; gap: 0.75rem; }
  .btn-bot { background: color-mix(in srgb, var(--income) 12%, transparent); border: 1px solid color-mix(in srgb, var(--income) 40%, transparent); color: var(--income); padding: 0.5rem 1rem; border-radius: 10px; cursor: pointer; font-size: 0.9rem; }
  .btn-bot.running { background: color-mix(in srgb, var(--expense) 12%, transparent); border-color: color-mix(in srgb, var(--expense) 40%, transparent); color: var(--expense); }
  .btn-bot:disabled { opacity: 0.6; cursor: not-allowed; }
  .bot-status { font-size: 0.78rem; color: var(--text-dim); }
  .bot-status.on { color: var(--income); }
  .bot-msg { color: var(--income); font-size: 0.82rem; margin: 0; }
  .update-row { display: flex; align-items: center; justify-content: space-between; gap: 1rem; flex-wrap: wrap; }
  .update-info { display: flex; flex-direction: column; gap: 0.3rem; }
  .update-version { font-size: 0.85rem; color: var(--text-muted); }
  .update-badge { font-size: 0.75rem; font-weight: 600; color: #f97316; background: color-mix(in srgb, #f97316 12%, transparent); border: 1px solid color-mix(in srgb, #f97316 35%, transparent); padding: 0.15rem 0.5rem; border-radius: 6px; width: fit-content; }
  .update-actions { display: flex; gap: 0.5rem; align-items: center; flex-shrink: 0; }
  .update-msg { font-size: 0.82rem; margin: 0.5rem 0 0; color: var(--text-muted); }
  .update-msg.ok { color: var(--income); }

  .section-header { display: flex; align-items: center; justify-content: space-between; }
  .section-header h2 { margin: 0; }
  .btn-guide-toggle { background: none; border: none; color: var(--accent-lt); font-size: 0.78rem; cursor: pointer; padding: 0; text-decoration: underline; }
  .btn-guide-toggle:hover { color: var(--accent); }

  .guide-box { background: var(--bg-base); border: 1px solid var(--border2); border-radius: 10px; padding: 1rem 1.1rem; display: flex; flex-direction: column; gap: 0.6rem; }
  .guide-step { margin: 0; font-size: 0.84rem; color: var(--text-muted); display: flex; align-items: baseline; gap: 0.5rem; }
  .step-num { background: var(--accent); color: #fff; font-size: 0.7rem; font-weight: 700; width: 18px; height: 18px; border-radius: 50%; display: inline-flex; align-items: center; justify-content: center; flex-shrink: 0; }
  .guide-note { margin: 0; font-size: 0.78rem; color: var(--text-dim); font-style: italic; }
  .code-row { display: flex; align-items: center; gap: 0.5rem; background: var(--bg-elevated); border-radius: 7px; padding: 0.4rem 0.75rem; }
  .code-row code { background: none; padding: 0; flex: 1; font-size: 0.82rem; color: var(--text); }
  .btn-copy { background: none; border: 1px solid var(--border); border-radius: 5px; color: var(--text-dim); font-size: 0.72rem; padding: 0.15rem 0.5rem; cursor: pointer; white-space: nowrap; flex-shrink: 0; }
  .btn-copy:hover { color: var(--text); }
  .ping-msg { font-size: 0.8rem; margin: 0.25rem 0 0; color: var(--text-muted); }
  .ping-msg.ok { color: var(--income); }
</style>
