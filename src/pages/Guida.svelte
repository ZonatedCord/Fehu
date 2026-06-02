<div class="page">
  <h1>Guida</h1>

  <div class="sections">

    <section>
      <h2>Scorciatoie da tastiera</h2>
      <table class="shortcuts">
        <thead><tr><th>Shortcut</th><th>Azione</th></tr></thead>
        <tbody>
          <tr><td><kbd>⌘ 1</kbd></td><td>Dashboard</td></tr>
          <tr><td><kbd>⌘ 2</kbd></td><td>Transazioni</td></tr>
          <tr><td><kbd>⌘ 3</kbd></td><td>Categorie</td></tr>
          <tr><td><kbd>⌘ 4</kbd></td><td>Foto / OCR</td></tr>
          <tr><td><kbd>⌘ 5</kbd></td><td>Obiettivi</td></tr>
          <tr><td><kbd>⌘ 6</kbd></td><td>Ricorrenti</td></tr>
          <tr><td><kbd>⌘ 7</kbd></td><td>P.IVA</td></tr>
          <tr><td><kbd>⌘ N</kbd></td><td>Nuova transazione</td></tr>
          <tr><td><kbd>⌘ F</kbd></td><td>Cerca transazioni</td></tr>
          <tr><td><kbd>⌘ E</kbd></td><td>Esporta / Backup</td></tr>
          <tr><td><kbd>⌘ ,</kbd></td><td>Impostazioni</td></tr>
          <tr><td><kbd>⌘ C / V / X</kbd></td><td>Copia / Incolla / Taglia</td></tr>
        </tbody>
      </table>
    </section>

    <section>
      <h2>OCR scontrini</h2>
      <ol class="steps">
        <li>Vai su <strong>Foto</strong> (⌘4)</li>
        <li>Trascina o seleziona un'immagine dello scontrino</li>
        <li>Fehu esegue Tesseract localmente ed estrae importo, data e negozio</li>
        <li>Se Ollama è attivo, la spesa viene categorizzata automaticamente</li>
        <li>Rivedi i dati e salva</li>
      </ol>
      <p class="note">Senza Ollama: la categorizzazione usa regole per parole chiave (es. "Esselunga" → Cibo).</p>
    </section>

    <section>
      <h2>Bot Telegram</h2>
      <ol class="steps">
        <li>Vai su <strong>Impostazioni</strong> → sezione Telegram Bot → "Come configurare"</li>
        <li>Crea il bot con @BotFather su Telegram</li>
        <li>Incolla il token e clicca <strong>Avvia bot</strong></li>
        <li>Usa i comandi nel bot:
          <ul>
            <li><code>/saldo</code> — patrimonio attuale</li>
            <li><code>/spese [N]</code> — ultime N transazioni</li>
            <li><code>/aggiungi 12.50 Caffè</code> — spesa rapida</li>
            <li><code>/aggiungi +500 Fattura</code> — entrata</li>
            <li><code>/report [mese]</code> — report mensile</li>
            <li><code>/obiettivi</code> — fondi risparmio</li>
            <li><code>/categorie</code> — spese per categoria</li>
            <li>Foto → OCR automatico con bottoni di conferma</li>
          </ul>
        </li>
      </ol>
    </section>

    <section>
      <h2>Auto-aggiornamento</h2>
      <p>Fehu controlla gli aggiornamenti automaticamente 3 secondi dopo l'avvio.</p>
      <ul>
        <li>Se disponibile: appare un popup in alto a destra con il bottone <strong>Installa ora</strong></li>
        <li>Un punto arancione compare sull'icona Impostazioni nella sidebar</li>
        <li>Puoi anche controllare manualmente da <strong>Impostazioni → Aggiornamenti</strong></li>
      </ul>
    </section>

    <section>
      <h2>Domande frequenti</h2>

      <div class="faq">
        <div class="faq-item">
          <p class="faq-q">macOS blocca l'app al primo avvio (Gatekeeper)</p>
          <p class="faq-a">L'app non ha firma commerciale Apple. Apri il Terminale e digita:<br>
          <code>xattr -dr com.apple.quarantine /Applications/Fehu.app</code><br>
          Oppure usa lo script <em>Rimuovi_Quarantine.command</em> nel DMG.</p>
        </div>

        <div class="faq-item">
          <p class="faq-q">Windows SmartScreen blocca l'installer</p>
          <p class="faq-a">Clicca <strong>Ulteriori informazioni</strong> → <strong>Esegui comunque</strong>. In alternativa usa il file <code>.msi</code>.</p>
        </div>

        <div class="faq-item">
          <p class="faq-q">I miei dati sono al sicuro?</p>
          <p class="faq-a">Sì. Fehu non invia nulla a internet. Tutti i dati sono in un file SQLite sul tuo computer:
          <code>~/Library/Application Support/dev.fehu.app/fehu.db</code> (macOS).</p>
        </div>

        <div class="faq-item">
          <p class="faq-q">Come faccio il backup?</p>
          <p class="faq-a">Vai su <strong>Dati → Backup & Ripristino → Esporta backup .db</strong>. Puoi ripristinare dallo stesso schermo.</p>
        </div>

        <div class="faq-item">
          <p class="faq-q">Ollama è obbligatorio?</p>
          <p class="faq-a">No. Senza Ollama la categorizzazione usa regole per parole chiave. Funziona offline e senza dipendenze aggiuntive.</p>
        </div>
      </div>
    </section>

  </div>
</div>

<style>
  .page { max-width: 680px; }
  h1 { margin: 0 0 1.5rem; font-size: 1.5rem; }
  .sections { display: flex; flex-direction: column; gap: 1.25rem; }
  section { background: var(--bg-card); border-radius: 16px; padding: 1.25rem 1.5rem; box-shadow: 0 1px 3px rgba(0,0,0,0.06); }
  h2 { font-size: 0.8rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.08em; color: var(--text-dim); margin: 0 0 1rem; }

  .shortcuts { width: 100%; border-collapse: collapse; font-size: 0.875rem; }
  .shortcuts th { text-align: left; color: var(--text-dim); font-weight: 500; font-size: 0.75rem; padding: 0.4rem 0.75rem; border-bottom: 1px solid var(--border); }
  .shortcuts td { padding: 0.45rem 0.75rem; border-bottom: 1px solid var(--border2); }
  .shortcuts tbody tr:last-child td { border-bottom: none; }
  kbd { background: var(--bg-elevated); border: 1px solid var(--border); border-radius: 5px; padding: 0.1rem 0.4rem; font-size: 0.8rem; font-family: monospace; }

  .steps { margin: 0; padding-left: 1.25rem; display: flex; flex-direction: column; gap: 0.4rem; font-size: 0.875rem; color: var(--text-muted); line-height: 1.5; }
  .steps li { padding-left: 0.25rem; }
  .steps ul { margin: 0.4rem 0 0; padding-left: 1.25rem; display: flex; flex-direction: column; gap: 0.25rem; }
  .note { margin: 0.75rem 0 0; font-size: 0.8rem; color: var(--text-dim); font-style: italic; }
  p { margin: 0 0 0.5rem; font-size: 0.875rem; color: var(--text-muted); line-height: 1.5; }
  ul { margin: 0.25rem 0; padding-left: 1.25rem; font-size: 0.875rem; color: var(--text-muted); }
  li { margin-bottom: 0.2rem; }
  code { background: var(--bg-elevated); padding: 0.1rem 0.35rem; border-radius: 4px; font-size: 0.8rem; }

  .faq { display: flex; flex-direction: column; gap: 1rem; }
  .faq-item { border-left: 3px solid var(--border); padding-left: 0.875rem; }
  .faq-q { font-weight: 600; color: var(--text); font-size: 0.875rem; margin: 0 0 0.25rem; }
  .faq-a { font-size: 0.82rem; color: var(--text-muted); margin: 0; line-height: 1.55; }
</style>
