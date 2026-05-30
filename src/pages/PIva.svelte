<script lang="ts">
  // ── Types ────────────────────────────────────────────────────────────────
  type Regime = 'forfettario' | 'ordinario' | 'semplificato';
  type InpsType = 'gestione_separata' | 'artigiani' | 'commercianti';

  interface Scenario {
    label: string;
    fatturato: number;
    baseImponibile: number;
    contributiInps: number;
    imposta: number;
    netto: number;
  }

  // ── ATECO coefficients ───────────────────────────────────────────────────
  const atecoOptions = [
    { label: 'Professioni (art. 54 TUIR) — avvocati, architetti, consulenti', coeff: 0.78 },
    { label: 'Servizi non professionali', coeff: 0.67 },
    { label: 'Intermediari del commercio', coeff: 0.62 },
    { label: 'Industria, artigianato e altri servizi', coeff: 0.40 },
    { label: 'Commercio (es. rivendita)', coeff: 0.40 },
    { label: 'Ristorazione e ospitalità', coeff: 0.40 },
    { label: 'Costruzioni e immobiliare', coeff: 0.86 },
  ];

  const inpsOptions: { label: string; value: InpsType }[] = [
    { label: 'Gestione Separata INPS (26.07%)', value: 'gestione_separata' },
    { label: 'Artigiani IVS (24% + minimale ~3.900€)', value: 'artigiani' },
    { label: 'Commercianti IVS (24.09% + minimale ~4.000€)', value: 'commercianti' },
  ];

  // ── State ────────────────────────────────────────────────────────────────
  let activeTab = $state<Regime>('forfettario');

  // Forfettario
  let fatturato = $state(30000);
  let atecoIdx = $state(0);
  let anniAttivita = $state(6);
  let inpsType = $state<InpsType>('gestione_separata');

  // Ordinario / Semplificato
  let fatturatoOrd = $state(40000);
  let speseOrd = $state(10000);
  let inpsTypeOrd = $state<InpsType>('gestione_separata');
  let addRegionale = $state(1.5);

  // ── Helpers ──────────────────────────────────────────────────────────────
  function calcInps(base: number, type: InpsType): number {
    if (type === 'gestione_separata') return base * 0.2607;
    if (type === 'artigiani') return Math.max(base * 0.24, 3900);
    return Math.max(base * 0.2409, 4000); // commercianti
  }

  function calcIrpef(reddito: number): number {
    if (reddito <= 0) return 0;
    let tax = 0;
    const r = reddito;
    if (r <= 28000) return r * 0.23;
    tax += 28000 * 0.23;
    if (r <= 50000) return tax + (r - 28000) * 0.35;
    tax += 22000 * 0.35;
    return tax + (r - 50000) * 0.43;
  }

  function calcForfettario(fat: number): { base: number; inps: number; imposta: number; netto: number } {
    const coeff = atecoOptions[atecoIdx].coeff;
    const aliquotaImposta = anniAttivita <= 5 ? 0.05 : 0.15;
    const base = fat * coeff;
    const inps = calcInps(base, inpsType);
    const baseDeducibile = Math.max(base - inps, 0);
    const imposta = baseDeducibile * aliquotaImposta;
    const netto = fat - inps - imposta;
    return { base, inps, imposta, netto };
  }

  function calcOrdinario(fat: number, spese: number, type: InpsType, addReg: number): {
    base: number; inps: number; imposta: number; netto: number;
  } {
    const reddito = fat - spese;
    const inps = calcInps(Math.max(reddito, 0), type);
    const baseIrpef = Math.max(reddito - inps, 0);
    const irpef = calcIrpef(baseIrpef);
    const addizionale = baseIrpef * (addReg / 100);
    const imposta = irpef + addizionale;
    const netto = fat - spese - inps - imposta;
    return { base: baseIrpef, inps, imposta, netto };
  }

  // ── Computed scenarios ───────────────────────────────────────────────────
  let scenarios = $derived.by(() => {
    const multipliers = [
      { label: 'Pessimista  −20%', factor: 0.8 },
      { label: 'Base', factor: 1.0 },
      { label: 'Ottimista  +20%', factor: 1.2 },
    ];

    return multipliers.map(({ label, factor }) => {
      if (activeTab === 'forfettario') {
        const fat = fatturato * factor;
        const { base, inps, imposta, netto } = calcForfettario(fat);
        return { label, fatturato: fat, baseImponibile: base, contributiInps: inps, imposta, netto };
      } else {
        const fat = fatturatoOrd * factor;
        const spese = speseOrd * factor;
        const { base, inps, imposta, netto } = calcOrdinario(fat, spese, inpsTypeOrd, addRegionale);
        return { label, fatturato: fat, baseImponibile: base, contributiInps: inps, imposta, netto };
      }
    }) as Scenario[];
  });

  function fmt(n: number): string {
    return new Intl.NumberFormat('it-IT', {
      style: 'currency', currency: 'EUR', maximumFractionDigits: 0,
    }).format(n);
  }
</script>

<div class="page">
  <div class="page-header">
    <h1>Calcolatore P.IVA</h1>
    <p class="subtitle">Stima del carico fiscale per regime forfettario, ordinario e semplificato.</p>
  </div>

  <!-- Tab selector -->
  <div class="tabs">
    {#each (['forfettario', 'ordinario', 'semplificato'] as Regime[]) as tab}
      <button class:active={activeTab === tab} onclick={() => { activeTab = tab; }}>
        {tab.charAt(0).toUpperCase() + tab.slice(1)}
      </button>
    {/each}
  </div>

  <div class="content">
    <!-- ── FORFETTARIO ─────────────────────────────────────────────── -->
    {#if activeTab === 'forfettario'}
      <div class="inputs">
        <label>
          <span>Fatturato annuo previsto</span>
          <div class="input-row">
            <input type="number" bind:value={fatturato} min="0" step="1000" />
            <span class="unit">€</span>
          </div>
        </label>

        <label>
          <span>Categoria ATECO</span>
          <select bind:value={atecoIdx}>
            {#each atecoOptions as opt, i}
              <option value={i}>{opt.label} — coeff. {(opt.coeff * 100).toFixed(0)}%</option>
            {/each}
          </select>
        </label>

        <label>
          <span>Anni di attività</span>
          <div class="input-row">
            <input type="number" bind:value={anniAttivita} min="1" max="50" />
            <span class="unit-hint">{anniAttivita <= 5 ? 'Aliquota agevolata 5%' : 'Aliquota ordinaria 15%'}</span>
          </div>
        </label>

        <label>
          <span>Tipo INPS</span>
          <select bind:value={inpsType}>
            {#each inpsOptions as opt}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </label>

        <div class="formula-box">
          <p class="formula-title">Formula applicata</p>
          <p class="formula">
            Base imponibile = fatturato × {(atecoOptions[atecoIdx].coeff * 100).toFixed(0)}%<br>
            INPS deduc. dalla base → imposta {anniAttivita <= 5 ? '5%' : '15%'} sulla base ridotta
          </p>
        </div>
      </div>

    <!-- ── ORDINARIO / SEMPLIFICATO ──────────────────────────────── -->
    {:else}
      <div class="inputs">
        {#if activeTab === 'semplificato'}
          <div class="regime-note">
            Regime semplificato — stessa tassazione IRPEF del regime ordinario.
            Limite ricavi: 400.000€ (servizi) / 700.000€ (altri).
          </div>
        {/if}

        <label>
          <span>Fatturato annuo previsto</span>
          <div class="input-row">
            <input type="number" bind:value={fatturatoOrd} min="0" step="1000" />
            <span class="unit">€</span>
          </div>
        </label>

        <label>
          <span>Spese deducibili annue</span>
          <div class="input-row">
            <input type="number" bind:value={speseOrd} min="0" step="500" />
            <span class="unit">€</span>
          </div>
        </label>

        <label>
          <span>Tipo INPS</span>
          <select bind:value={inpsTypeOrd}>
            {#each inpsOptions as opt}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </label>

        <label>
          <span>Addizionale regionale (%)</span>
          <div class="input-row">
            <input type="number" bind:value={addRegionale} min="0" max="4" step="0.1" style="max-width: 100px;" />
            <span class="unit-hint">Media nazionale ~1.5%</span>
          </div>
        </label>

        <div class="formula-box">
          <p class="formula-title">Scaglioni IRPEF 2024</p>
          <p class="formula">
            0 – 28.000€ → 23% &nbsp;|&nbsp; 28.001 – 50.000€ → 35% &nbsp;|&nbsp; &gt;50.000€ → 43%<br>
            + Addizionale regionale {addRegionale}% + Addizionale comunale (non inclusa)
          </p>
        </div>
      </div>
    {/if}

    <!-- ── SCENARI ─────────────────────────────────────────────────── -->
    <div class="scenarios">
      {#each scenarios as sc, i}
        <div class="scenario-card" class:base={i === 1}>
          <div class="scenario-label">{sc.label}</div>
          <div class="scenario-fatturato">{fmt(sc.fatturato)}</div>

          <div class="scenario-rows">
            <div class="sc-row">
              <span>Base imponibile</span>
              <span>{fmt(sc.baseImponibile)}</span>
            </div>
            <div class="sc-row">
              <span>Contributi INPS</span>
              <span class="negative">{fmt(sc.contributiInps)}</span>
            </div>
            <div class="sc-row">
              <span>Imposta</span>
              <span class="negative">{fmt(sc.imposta)}</span>
            </div>
            <div class="sc-row netto">
              <span>Netto stimato</span>
              <span class:positive={sc.netto > 0} class:negative-val={sc.netto < 0}>
                {fmt(sc.netto)}
              </span>
            </div>
          </div>
        </div>
      {/each}
    </div>

    <p class="disclaimer">
      Stime indicative basate su aliquote 2024. Non includono addizionale comunale,
      detrazioni d'imposta, contribuenti minimi, o casistiche particolari.
      Consulta un commercialista per una pianificazione fiscale accurata.
    </p>
  </div>
</div>

<style>
  .page { max-width: 900px; }
  .page-header { margin-bottom: 1.5rem; }
  h1 { margin: 0 0 0.3rem; font-size: 1.5rem; }
  .subtitle { color: var(--text-dim); font-size: 0.875rem; margin: 0; }

  /* Tabs */
  .tabs {
    display: flex; gap: 0; margin-bottom: 1.75rem;
    background: var(--bg-card); border-radius: 8px; padding: 0.25rem;
    width: fit-content;
  }
  .tabs button {
    background: none; border: none; color: var(--text-dim);
    padding: 0.5rem 1.1rem; cursor: pointer; border-radius: 6px;
    font-size: 0.875rem; transition: background 0.15s, color 0.15s;
  }
  .tabs button.active { background: #2e2e4e; color: #a5b4fc; }

  .content { display: grid; grid-template-columns: 320px 1fr; gap: 1.5rem; align-items: start; }

  /* Inputs */
  .inputs {
    background: var(--bg-card); border-radius: 10px; padding: 1.25rem;
    display: flex; flex-direction: column; gap: 1rem;
  }
  label { display: flex; flex-direction: column; gap: 0.3rem; font-size: 0.85rem; color: #aaa; }
  label span { color: var(--text-muted); }
  .input-row { display: flex; align-items: center; gap: 0.5rem; }
  .unit { color: var(--text-dim); font-size: 0.85rem; }
  .unit-hint { color: #4ade80; font-size: 0.78rem; white-space: nowrap; }

  input[type="number"], select {
    padding: 0.5rem 0.75rem; border: 1px solid var(--border);
    border-radius: 6px; background: var(--bg-base); color: var(--text);
    font-size: 0.9rem; width: 100%;
  }
  input[type="number"] { width: 140px; }
  input:focus, select:focus { outline: 1px solid #6366f1; }

  .formula-box {
    background: var(--bg-base); border: 1px solid var(--border2);
    border-radius: 6px; padding: 0.75rem; margin-top: 0.25rem;
  }
  .formula-title { font-size: 0.72rem; color: var(--text-dim); margin: 0 0 0.35rem; text-transform: uppercase; letter-spacing: 0.06em; }
  .formula { font-size: 0.78rem; color: var(--text-dim); margin: 0; line-height: 1.6; }

  .regime-note {
    background: #12122a; border: 1px solid var(--border); border-radius: 6px;
    padding: 0.65rem 0.85rem; font-size: 0.8rem; color: var(--text-dim); line-height: 1.5;
  }

  /* Scenarios */
  .scenarios {
    display: flex; flex-direction: column; gap: 0.75rem;
    align-self: start;
  }
  .scenario-card {
    background: var(--bg-card); border: 1px solid var(--border);
    border-radius: 10px; padding: 1.1rem;
  }
  .scenario-card.base { border-color: #4e4e8e; }

  .scenario-label { font-size: 0.75rem; color: var(--text-dim); text-transform: uppercase; letter-spacing: 0.07em; margin-bottom: 0.3rem; }
  .scenario-fatturato { font-size: 1.35rem; font-weight: 700; color: var(--text); margin-bottom: 0.85rem; }

  .scenario-rows { display: flex; flex-direction: column; gap: 0.45rem; }
  .sc-row {
    display: flex; justify-content: space-between;
    font-size: 0.85rem; color: var(--text-muted);
  }
  .sc-row.netto { border-top: 1px solid #2e2e4e; padding-top: 0.45rem; margin-top: 0.1rem; color: #ccc; font-weight: 600; }
  .negative { color: #f87171; }
  .positive { color: #4ade80; }
  .negative-val { color: #f87171; }

  .disclaimer {
    grid-column: 1 / -1; font-size: 0.75rem; color: #444;
    margin: 0; line-height: 1.6; border-top: 1px solid #1e1e2e; padding-top: 1rem;
  }

  @media (max-width: 700px) {
    .content { grid-template-columns: 1fr; }
    .scenarios { flex-direction: column; }
  }
</style>
