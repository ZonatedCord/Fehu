<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';
  import type { AppSettings } from '../lib/types';

  let settings = $state<AppSettings>({
    ollama_url: 'http://localhost:11434',
    tesseract_path: '',
    currency_symbol: '€',
    onboarded: 'false',
  });
  let saving = $state(false);
  let saved = $state(false);
  let error = $state('');
  let botToken = $state('');
  let botRunning = $state(false);
  let botMsg = $state('');
  let botBusy = $state(false);

  onMount(async () => {
    try {
      const [s, running, token] = await Promise.all([
        api.getSettings(),
        api.getTelegramStatus(),
        api.getSettings().then(s => (s as any).telegram_token ?? ''),
      ]);
      settings = s;
      botRunning = running;
      botToken = token;
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

  async function toggleBot() {
    botBusy = true; botMsg = ''; error = '';
    try {
      if (botRunning) {
        await api.stopTelegramBot();
        botRunning = false;
        botMsg = 'Bot fermato.';
      } else {
        if (!botToken.trim()) { error = 'Inserisci il token bot prima di avviare.'; return; }
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
      <h2>LLM</h2>
      <label>
        <span>URL Ollama</span>
        <p class="field-hint">Indirizzo del server Ollama locale. Default: http://localhost:11434</p>
        <input bind:value={settings.ollama_url} placeholder="http://localhost:11434" />
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
      <h2>Telegram Bot</h2>
      <label>
        <span>Token bot</span>
        <p class="field-hint">Ottieni il token da @BotFather su Telegram. Richiede Python 3 + <code>pip install aiogram aiosqlite</code>.</p>
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
  .subtitle { color: #666; font-size: 0.875rem; margin: 0; }

  .sections { display: flex; flex-direction: column; gap: 1.75rem; }
  section { background: #1a1a2e; border-radius: 10px; padding: 1.25rem; display: flex; flex-direction: column; gap: 1rem; }
  h2 { margin: 0 0 0.25rem; font-size: 0.8rem; font-weight: 600; text-transform: uppercase; letter-spacing: 0.08em; color: #666; }

  label { display: flex; flex-direction: column; gap: 0.3rem; font-size: 0.9rem; color: #ccc; }
  .field-hint { font-size: 0.78rem; color: #555; margin: 0; line-height: 1.4; }
  .input-row { display: flex; gap: 0.5rem; align-items: center; }
  .input-row input { flex: 1; }

  input {
    padding: 0.5rem 0.75rem; border: 1px solid #2e2e4e;
    border-radius: 6px; background: #0f0f1a; color: #e0e0f0;
    font-size: 0.9rem; width: 100%;
  }
  input:focus { outline: 1px solid #6366f1; }

  .actions { display: flex; flex-direction: column; gap: 0.5rem; }
  .btn-primary {
    background: #6366f1; color: #fff; border: none;
    padding: 0.6rem 1.25rem; border-radius: 6px;
    cursor: pointer; font-size: 0.9rem; align-self: flex-start;
  }
  .btn-primary:disabled { opacity: 0.6; cursor: not-allowed; }
  .btn-ghost {
    background: none; border: 1px solid #2e2e4e; color: #666;
    padding: 0.5rem 0.75rem; border-radius: 6px; cursor: pointer;
    font-size: 0.82rem; white-space: nowrap;
  }
  .btn-ghost:hover { color: #aaa; border-color: #3e3e5e; }
  .error { color: #f87171; font-size: 0.85rem; margin: 0; }
  code { background: #0f0f1a; padding: 0.1rem 0.35rem; border-radius: 3px; font-size: 0.8rem; }
  .bot-row { display: flex; align-items: center; gap: 0.75rem; }
  .btn-bot { background: #1a2e1a; border: 1px solid #166534; color: #4ade80; padding: 0.5rem 1rem; border-radius: 6px; cursor: pointer; font-size: 0.9rem; }
  .btn-bot.running { background: #2e1a1a; border-color: #7f1d1d; color: #f87171; }
  .btn-bot:disabled { opacity: 0.6; cursor: not-allowed; }
  .bot-status { font-size: 0.78rem; color: #555; }
  .bot-status.on { color: #4ade80; }
  .bot-msg { color: #4ade80; font-size: 0.82rem; margin: 0; }
</style>
