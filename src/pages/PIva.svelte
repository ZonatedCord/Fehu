<script lang="ts">
  import { onMount } from 'svelte';
  import { openUrl } from '@tauri-apps/plugin-opener';

  type Regime = 'forfettario' | 'ordinario' | 'semplificato';
  type InpsType = 'gestione_separata' | 'artigiani' | 'commercianti';

  interface Scenario {
    label: string;
    fatturato: number;
    baseImponibile: number;
    contributiInps: number;
    detrazione: number;
    imposta: number;
    netto: number;
  }

  interface RataInps {
    label: string;
    scadenza: string;
    importo: number;
  }

  const RATES_URL = 'https://raw.githubusercontent.com/ZonatedCord/Fehu/main/docs/piva-rates.json';
  const CACHE_KEY = 'fehu_piva_rates';
  const CACHE_TTL = 24 * 60 * 60 * 1000; // 24h

  // Defaults (fallback se offline)
  let atecoOptions = $state([
    { label: 'Professioni sanitarie, insegnamento, assistenza sociale', coeff: 0.78 },
    { label: 'Attività professionali (avvocati, architetti, consulenti art. 54)', coeff: 0.78 },
    { label: 'Costruzioni e attività immobiliari', coeff: 0.86 },
    { label: 'Intermediari del commercio', coeff: 0.62 },
    { label: 'Servizi non professionali, attività sportive', coeff: 0.67 },
    { label: 'Industria, artigianato, altri servizi', coeff: 0.40 },
    { label: 'Commercio al dettaglio / ambulante / ristorazione', coeff: 0.40 },
  ]);

  let INPS_RATES = $state({
    gestione_separata: { aliquota: 0.2607, minimale: 0, massimale: 122295, reddito_minimale: 0 },
    artigiani:         { aliquota: 0.24,   minimale: 4521, massimale: 56224, reddito_minimale: 18808 },
    commercianti:      { aliquota: 0.2448,  minimale: 4612, massimale: 56224, reddito_minimale: 18808 },
  });

  let ratesAnno = $state(2026);
  let ratesLoading = $state(false);
  let ratesCustom = $state(false);
  let showRatesEditor = $state(false);

  // Editable overrides (mirror of INPS_RATES for the form)
  let editGs    = $state({ aliquota: 26.07, massimale: 122295 });
  let editArt   = $state({ aliquota: 24.00, minimale: 4521, massimale: 56224 });
  let editComm  = $state({ aliquota: 24.48, minimale: 4612, massimale: 56224 });
  let editAnno  = $state(2026);

  onMount(async () => {
    // Check for user override first
    const override = localStorage.getItem(CACHE_KEY + '_override');
    if (override) {
      applyRates(JSON.parse(override));
      syncEditorFromRates();
      ratesCustom = true;
      return;
    }
    await fetchRates();
  });

  async function fetchRates(force = false) {
    try {
      if (!force) {
        const cached = localStorage.getItem(CACHE_KEY);
        if (cached) {
          const { data, ts } = JSON.parse(cached);
          const annoOk = !data._anno || data._anno >= 2026;
          if (annoOk && Date.now() - ts < CACHE_TTL) { applyRates(data); syncEditorFromRates(); return; }
        }
      }
      ratesLoading = true;
      const res = await fetch(RATES_URL, { signal: AbortSignal.timeout(5000) });
      if (res.ok) {
        const data = await res.json();
        localStorage.setItem(CACHE_KEY, JSON.stringify({ data, ts: Date.now() }));
        applyRates(data);
        syncEditorFromRates();
      }
    } catch {} finally { ratesLoading = false; }
  }

  function applyRates(data: any) {
    if (data.ateco) atecoOptions = data.ateco;
    if (data.inps) {
      INPS_RATES = {
        gestione_separata: { ...INPS_RATES.gestione_separata, ...data.inps.gestione_separata },
        artigiani:         { ...INPS_RATES.artigiani, ...data.inps.artigiani },
        commercianti:      { ...INPS_RATES.commercianti, ...data.inps.commercianti },
      };
    }
    if (data._anno) ratesAnno = data._anno;
  }

  function syncEditorFromRates() {
    editGs   = { aliquota: +(INPS_RATES.gestione_separata.aliquota * 100).toFixed(4), massimale: INPS_RATES.gestione_separata.massimale };
    editArt  = { aliquota: +(INPS_RATES.artigiani.aliquota * 100).toFixed(4), minimale: INPS_RATES.artigiani.minimale, massimale: INPS_RATES.artigiani.massimale };
    editComm = { aliquota: +(INPS_RATES.commercianti.aliquota * 100).toFixed(4), minimale: INPS_RATES.commercianti.minimale, massimale: INPS_RATES.commercianti.massimale };
    editAnno = ratesAnno;
  }

  function saveCustomRates() {
    const override = {
      _anno: editAnno,
      inps: {
        gestione_separata: { aliquota: editGs.aliquota / 100, massimale: editGs.massimale, minimale: 0 },
        artigiani:         { aliquota: editArt.aliquota / 100, minimale: editArt.minimale, massimale: editArt.massimale, reddito_minimale: 17504 },
        commercianti:      { aliquota: editComm.aliquota / 100, minimale: editComm.minimale, massimale: editComm.massimale, reddito_minimale: 17504 },
      }
    };
    localStorage.setItem(CACHE_KEY + '_override', JSON.stringify(override));
    applyRates(override);
    ratesCustom = true;
    showRatesEditor = false;
  }

  function resetRates() {
    localStorage.removeItem(CACHE_KEY + '_override');
    localStorage.removeItem(CACHE_KEY);
    ratesCustom = false;
    fetchRates(true);
  }

  const inpsOptions: { label: string; value: InpsType }[] = [
    { label: 'Gestione Separata INPS', value: 'gestione_separata' },
    { label: 'Artigiani IVS', value: 'artigiani' },
    { label: 'Commercianti IVS', value: 'commercianti' },
  ];

  // State
  let activeTab = $state<Regime>('forfettario');
  let fatturato = $state(30000);
  let atecoIdx = $state(1);
  let anniAttivita = $state(6);
  let inpsType = $state<InpsType>('gestione_separata');
  let fatturatoOrd = $state(40000);
  let speseOrd = $state(10000);
  let inpsTypeOrd = $state<InpsType>('gestione_separata');
  let addRegionale = $state(1.5);

  // Calcoli
  function calcInps(base: number, type: InpsType): number {
    const r = INPS_RATES[type];
    if (type === 'gestione_separata') {
      return Math.min(base, r.massimale) * r.aliquota;
    }
    const variabile = Math.max(base - r.reddito_minimale, 0);
    return r.minimale + Math.min(variabile, r.massimale - r.reddito_minimale) * r.aliquota;
  }

  function calcIrpef(reddito: number): number {
    if (reddito <= 0) return 0;
    let tax = 0;
    if (reddito <= 28000) return reddito * 0.23;
    tax += 28000 * 0.23;
    if (reddito <= 50000) return tax + (reddito - 28000) * 0.33;
    tax += 22000 * 0.33;
    return tax + (reddito - 50000) * 0.43;
  }

  function calcDetrazioneAuto(reddito: number): number {
    if (reddito <= 0) return 0;
    if (reddito <= 5500) return 1265;
    if (reddito <= 28000) return 1265 * (28000 - reddito) / 22500;
    return 0;
  }

  function calcForfettario(fat: number) {
    const coeff = atecoOptions[atecoIdx].coeff;
    const aliq = anniAttivita <= 5 ? 0.05 : 0.15;
    const base = fat * coeff;
    const inps = calcInps(base, inpsType);
    const imponibile = Math.max(base - inps, 0);
    const imposta = imponibile * aliq;
    return { base, inps, detrazione: 0, imposta, netto: fat - inps - imposta };
  }

  function calcOrdinario(fat: number, spese: number, type: InpsType, addReg: number) {
    const reddito = Math.max(fat - spese, 0);
    const inps = calcInps(reddito, type);
    const baseIrpef = Math.max(reddito - inps, 0);
    const detr = calcDetrazioneAuto(baseIrpef);
    const irpef = Math.max(calcIrpef(baseIrpef) - detr, 0);
    const addizionale = baseIrpef * (addReg / 100);
    const imposta = irpef + addizionale;
    return { base: baseIrpef, inps, detrazione: detr, imposta, netto: fat - spese - inps - imposta };
  }

  function calcRateInps(inps: number, type: InpsType, fat: number): RataInps[] {
    if (type === 'gestione_separata') {
      const acc1 = inps * 0.4;
      const acc2 = inps * 0.6;
      return [
        { label: '1° Acconto (40%)', scadenza: '30 giugno', importo: acc1 },
        { label: '2° Acconto (60%)', scadenza: '30 novembre', importo: acc2 },
      ];
    }
    // Artigiani / Commercianti: quota fissa in 4 rate + quota variabile in 2 rate
    const r = INPS_RATES[type];
    const fissa = r.minimale;
    const variabile = Math.max(inps - fissa, 0);
    const rata = fissa / 4;
    return [
      { label: 'Quota fissa 1/4', scadenza: '16 maggio', importo: rata },
      { label: 'Quota fissa 2/4', scadenza: '20 agosto', importo: rata },
      { label: 'Quota fissa 3/4 + acc. variabile', scadenza: '16 novembre', importo: rata + variabile * 0.4 },
      { label: 'Quota fissa 4/4 + saldo variabile', scadenza: '16 febbraio', importo: rata + variabile * 0.6 },
    ];
  }

  let scenarios = $derived.by(() => {
    return [
      { label: 'Pessimista −20%', factor: 0.8 },
      { label: 'Base',            factor: 1.0 },
      { label: 'Ottimista +20%',  factor: 1.2 },
    ].map(({ label, factor }) => {
      if (activeTab === 'forfettario') {
        const fat = fatturato * factor;
        const { base, inps, detrazione, imposta, netto } = calcForfettario(fat);
        return { label, fatturato: fat, baseImponibile: base, contributiInps: inps, detrazione, imposta, netto } as Scenario;
      } else {
        const fat = fatturatoOrd * factor;
        const spese = speseOrd * factor;
        const { base, inps, detrazione, imposta, netto } = calcOrdinario(fat, spese, activeTab === 'ordinario' ? inpsTypeOrd : inpsTypeOrd, addRegionale);
        return { label, fatturato: fat, baseImponibile: base, contributiInps: inps, detrazione, imposta, netto } as Scenario;
      }
    });
  });

  let rateInps = $derived.by(() => {
    const base = activeTab === 'forfettario' ? scenarios[1] : scenarios[1];
    const type = activeTab === 'forfettario' ? inpsType : inpsTypeOrd;
    return calcRateInps(base.contributiInps, type, base.fatturato);
  });

  function fmt(n: number): string {
    return new Intl.NumberFormat('it-IT', { style: 'currency', currency: 'EUR', maximumFractionDigits: 0 }).format(n);
  }
</script>

<div class="page">
  <div class="page-header">
    <div class="header-row">
      <h1>Calcolatore P.IVA</h1>
      <span class="anno-badge" class:loading={ratesLoading} class:custom={ratesCustom}>
        {ratesLoading ? 'Aggiornamento…' : ratesCustom ? 'Aliquote personalizzate' : `Aliquote ${ratesAnno}`}
      </span>
      <button class="btn-rate-action" onclick={() => fetchRates(true)} disabled={ratesLoading} title="Aggiorna da GitHub">↻</button>
      <button class="btn-rate-action" onclick={() => showRatesEditor = !showRatesEditor} title="Modifica manuale">✎</button>
    </div>
    <p class="subtitle">Stima del carico fiscale per regime forfettario, ordinario e semplificato.</p>
  </div>

  {#if showRatesEditor}
    <div class="rates-editor">
      <div class="rates-editor-title">Modifica aliquote manuale</div>
      <div class="rates-grid">
        <div class="rate-group">
          <div class="rate-group-label">Gestione Separata</div>
          <label>Aliquota %<input type="number" bind:value={editGs.aliquota} step="0.01" min="0" max="50" /></label>
          <label>Massimale €<input type="number" bind:value={editGs.massimale} step="100" /></label>
        </div>
        <div class="rate-group">
          <div class="rate-group-label">Artigiani IVS</div>
          <label>Aliquota %<input type="number" bind:value={editArt.aliquota} step="0.001" min="0" max="50" /></label>
          <label>Minimale €<input type="number" bind:value={editArt.minimale} step="10" /></label>
          <label>Massimale €<input type="number" bind:value={editArt.massimale} step="100" /></label>
        </div>
        <div class="rate-group">
          <div class="rate-group-label">Commercianti IVS</div>
          <label>Aliquota %<input type="number" bind:value={editComm.aliquota} step="0.001" min="0" max="50" /></label>
          <label>Minimale €<input type="number" bind:value={editComm.minimale} step="10" /></label>
          <label>Massimale €<input type="number" bind:value={editComm.massimale} step="100" /></label>
        </div>
      </div>
      <label class="anno-label">Anno di riferimento<input type="number" bind:value={editAnno} min="2020" max="2030" style="width:80px" /></label>
      <div class="rates-actions">
        <button class="btn-save-rates" onclick={saveCustomRates}>Salva</button>
        {#if ratesCustom}<button class="btn-reset-rates" onclick={resetRates}>Ripristina da GitHub</button>{/if}
      </div>
      <div class="fonti-ufficiali">
        <span>Fonti ufficiali:</span>
        <button class="btn-fonte" onclick={() => openUrl('https://www.inps.it/it/it/inps-comunica/notizie/dettaglio-news-page.news.2026.02.gestioni-artigiani-e-commercianti-i-contributi-per-il-2026.html')}>INPS Artigiani/Commercianti</button>
        <button class="btn-fonte" onclick={() => openUrl('https://www.inps.it/it/it/inps-comunica/notizie/dettaglio-news-page.news.2026.02.gestione-separata-le-aliquote-contributive-per-il-2026.html')}>INPS Gestione Separata</button>
        <button class="btn-fonte" onclick={() => openUrl('https://www.agenziaentrate.gov.it/portale/imposta-sul-reddito-delle-persone-fisiche-irpef-/aliquote-e-calcolo-dell-irpef')}>Agenzia Entrate IRPEF</button>
      </div>
    </div>
  {/if}

  <div class="tabs">
    {#each (['forfettario', 'ordinario', 'semplificato'] as Regime[]) as tab}
      <button class:active={activeTab === tab} onclick={() => { activeTab = tab; }}>
        {tab.charAt(0).toUpperCase() + tab.slice(1)}
      </button>
    {/each}
  </div>

  <div class="layout">
    <!-- Inputs -->
    <div class="inputs-card">

      {#if activeTab === 'forfettario'}
        <label>
          <span>Fatturato annuo</span>
          <div class="input-row">
            <input type="number" bind:value={fatturato} min="0" step="1000" />
            <span class="unit">€</span>
          </div>
        </label>

        <label>
          <span>Categoria ATECO</span>
          <select bind:value={atecoIdx}>
            {#each atecoOptions as opt, i}
              <option value={i}>{opt.label} ({(opt.coeff * 100).toFixed(0)}%)</option>
            {/each}
          </select>
        </label>

        <label>
          <span>Anni di attività</span>
          <div class="input-row">
            <input type="number" bind:value={anniAttivita} min="1" max="50" style="width:80px" />
            <span class="unit-badge" class:agev={anniAttivita <= 5}>
              {anniAttivita <= 5 ? 'Aliquota 5% (start-up)' : 'Aliquota 15%'}
            </span>
          </div>
        </label>

        <label>
          <span>Tipo INPS</span>
          <select bind:value={inpsType}>
            {#each inpsOptions as opt}<option value={opt.value}>{opt.label}</option>{/each}
          </select>
        </label>

        <div class="info-box">
          <p class="info-title">Formula forfettario</p>
          <p class="info-body">
            Base = fatturato × {(atecoOptions[atecoIdx].coeff * 100).toFixed(0)}%<br>
            INPS deducibile dalla base → imposta sostitutiva {anniAttivita <= 5 ? '5%' : '15%'}
          </p>
        </div>

      {:else}
        {#if activeTab === 'semplificato'}
          <div class="regime-note">
            Regime semplificato: stessa IRPEF del regime ordinario.<br>
            Limite ricavi: 400.000€ (prestazioni di servizi) / 700.000€ (altre attività).
          </div>
        {/if}

        <label>
          <span>Fatturato annuo</span>
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
            {#each inpsOptions as opt}<option value={opt.value}>{opt.label}</option>{/each}
          </select>
        </label>

        <label>
          <span>Addizionale regionale (%)</span>
          <div class="input-row">
            <input type="number" bind:value={addRegionale} min="0" max="4" step="0.1" style="width:80px" />
            <span class="unit-badge">Media naz. ~1.5%</span>
          </div>
        </label>

        <div class="info-box">
          <p class="info-title">Scaglioni IRPEF 2024 + detrazioni</p>
          <p class="info-body">
            0–28.000€ → 23% &nbsp;|&nbsp; 28.001–50.000€ → 35% &nbsp;|&nbsp; &gt;50.000€ → 43%<br>
            + Addizionale regionale {addRegionale}%<br>
            Detrazione lavoro autonomo: fino a €1.265 (reddito ≤ €28.000)
          </p>
        </div>
      {/if}
    </div>

    <!-- Scenari -->
    <div class="right-col">
      <div class="scenarios">
        {#each scenarios as sc, i}
          <div class="scenario-card" class:base={i === 1}>
            <div class="sc-label">{sc.label}</div>
            <div class="sc-fatturato">{fmt(sc.fatturato)}</div>
            <div class="sc-rows">
              <div class="sc-row"><span>Base imponibile</span><span>{fmt(sc.baseImponibile)}</span></div>
              <div class="sc-row"><span>Contributi INPS</span><span class="neg">{fmt(sc.contributiInps)}</span></div>
              {#if sc.detrazione > 0}
                <div class="sc-row"><span>Detrazione lav. aut.</span><span class="pos">−{fmt(sc.detrazione)}</span></div>
              {/if}
              <div class="sc-row"><span>Imposta</span><span class="neg">{fmt(sc.imposta)}</span></div>
              <div class="sc-row netto">
                <span>Netto stimato</span>
                <span class:pos={sc.netto > 0} class:neg={sc.netto < 0}>{fmt(sc.netto)}</span>
              </div>
            </div>
          </div>
        {/each}
      </div>

      <!-- Rate INPS -->
      <div class="rate-card">
        <div class="rate-title">Rate INPS — scenario base</div>
        <div class="rate-rows">
          {#each rateInps as rata}
            <div class="rate-row">
              <div class="rate-label">{rata.label}</div>
              <div class="rate-scad">{rata.scadenza}</div>
              <div class="rate-importo">{fmt(rata.importo)}</div>
            </div>
          {/each}
          <div class="rate-row total">
            <div class="rate-label">Totale annuo</div>
            <div class="rate-scad"></div>
            <div class="rate-importo">{fmt(scenarios[1].contributiInps)}</div>
          </div>
        </div>
      </div>
    </div>
  </div>

  <p class="disclaimer">
    Stime indicative — aliquote INPS e IRPEF 2024. Non includono addizionale comunale, casse professionali,
    detrazioni familiari, o situazioni particolari. Consulta un commercialista per pianificazione accurata.
  </p>
</div>

<style>
  .page { max-width: 960px; }
  .page-header { margin-bottom: 1.25rem; }
  .header-row { display: flex; align-items: center; gap: 0.75rem; margin-bottom: 0.3rem; }
  h1 { margin: 0; font-size: 1.5rem; }
  .anno-badge { font-size: 0.75rem; background: color-mix(in srgb, var(--accent) 12%, transparent); color: var(--accent-lt); border: 1px solid color-mix(in srgb, var(--accent) 30%, transparent); padding: 0.15rem 0.6rem; border-radius: 6px; font-weight: 600; }
  .anno-badge.loading { color: var(--text-dim); background: var(--bg-elevated); border-color: var(--border); }
  .anno-badge.custom { background: color-mix(in srgb, #f97316 12%, transparent); color: #f97316; border-color: color-mix(in srgb, #f97316 30%, transparent); }
  .btn-rate-action { background: var(--bg-elevated); border: 1px solid var(--border); color: var(--text-muted); border-radius: 6px; padding: 0.15rem 0.5rem; cursor: pointer; font-size: 0.85rem; }
  .btn-rate-action:hover { color: var(--text); }
  .btn-rate-action:disabled { opacity: 0.5; cursor: not-allowed; }

  .rates-editor { background: var(--bg-card); border: 1px solid var(--border); border-radius: 14px; padding: 1.25rem 1.5rem; margin-bottom: 1.25rem; display: flex; flex-direction: column; gap: 1rem; box-shadow: 0 1px 3px rgba(0,0,0,0.06); }
  .rates-editor-title { font-size: 0.8rem; font-weight: 700; text-transform: uppercase; letter-spacing: 0.07em; color: var(--text-dim); }
  .rates-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 1rem; }
  .rate-group { display: flex; flex-direction: column; gap: 0.4rem; }
  .rate-group-label { font-size: 0.78rem; font-weight: 600; color: var(--text-muted); margin-bottom: 0.2rem; }
  .rate-group label, .anno-label { display: flex; flex-direction: column; gap: 0.15rem; font-size: 0.75rem; color: var(--text-dim); }
  .rate-group input, .anno-label input { padding: 0.35rem 0.5rem; border: 1px solid var(--border); border-radius: 6px; background: var(--bg-base); color: var(--text); font-size: 0.82rem; width: 100%; }
  .rates-actions { display: flex; gap: 0.5rem; }
  .btn-save-rates { background: var(--accent); color: #fff; border: none; border-radius: 8px; padding: 0.4rem 0.9rem; font-size: 0.82rem; font-weight: 600; cursor: pointer; }
  .btn-reset-rates { background: none; border: 1px solid var(--border); color: var(--text-muted); border-radius: 8px; padding: 0.4rem 0.9rem; font-size: 0.82rem; cursor: pointer; }
  .fonti-ufficiali { display: flex; align-items: center; gap: 0.5rem; flex-wrap: wrap; font-size: 0.75rem; color: var(--text-dim); padding-top: 0.5rem; border-top: 1px solid var(--border2); }
  .btn-fonte { background: none; border: none; color: var(--accent-lt); font-size: 0.75rem; cursor: pointer; text-decoration: underline; padding: 0; }
  .btn-fonte:hover { color: var(--accent); }
  .subtitle { color: var(--text-dim); font-size: 0.875rem; margin: 0; }

  .tabs {
    display: flex; gap: 0; margin-bottom: 1.5rem;
    background: var(--bg-card); border-radius: 10px; padding: 0.25rem; width: fit-content;
    box-shadow: 0 1px 3px rgba(0,0,0,0.06);
  }
  .tabs button {
    background: none; border: none; color: var(--text-dim);
    padding: 0.5rem 1.1rem; cursor: pointer; border-radius: 8px;
    font-size: 0.875rem; transition: background 0.15s, color 0.15s;
  }
  .tabs button.active {
    background: color-mix(in srgb, var(--accent) 15%, transparent);
    color: var(--accent-lt); font-weight: 600;
  }

  .layout { display: grid; grid-template-columns: 310px 1fr; gap: 1.25rem; align-items: start; }

  .inputs-card {
    background: var(--bg-card); border-radius: 14px; padding: 1.25rem 1.5rem;
    display: flex; flex-direction: column; gap: 1rem;
    box-shadow: 0 1px 3px rgba(0,0,0,0.06);
  }
  label { display: flex; flex-direction: column; gap: 0.3rem; font-size: 0.85rem; color: var(--text-muted); }
  .input-row { display: flex; align-items: center; gap: 0.5rem; }
  .unit { color: var(--text-dim); font-size: 0.85rem; }
  .unit-badge {
    font-size: 0.75rem; padding: 0.15rem 0.5rem; border-radius: 5px;
    background: var(--bg-elevated); color: var(--text-muted); white-space: nowrap;
  }
  .unit-badge.agev { background: color-mix(in srgb, var(--income) 12%, transparent); color: var(--income); }
  input[type="number"], select {
    padding: 0.5rem 0.75rem; border: 1px solid var(--border);
    border-radius: 10px; background: var(--bg-base); color: var(--text); font-size: 0.9rem; width: 100%;
  }
  input[type="number"] { width: 130px; }
  input:focus, select:focus { outline: 1px solid var(--accent); }
  .info-box {
    background: var(--bg-elevated); border: 1px solid var(--border2);
    border-radius: 8px; padding: 0.75rem;
  }
  .info-title { font-size: 0.7rem; color: var(--text-dim); margin: 0 0 0.3rem; text-transform: uppercase; letter-spacing: 0.06em; }
  .info-body { font-size: 0.78rem; color: var(--text-muted); margin: 0; line-height: 1.65; }
  .regime-note {
    background: color-mix(in srgb, var(--accent) 8%, transparent);
    border: 1px solid color-mix(in srgb, var(--accent) 25%, transparent);
    border-radius: 8px; padding: 0.65rem 0.85rem;
    font-size: 0.8rem; color: var(--text-muted); line-height: 1.5;
  }

  /* Right column */
  .right-col { display: flex; flex-direction: column; gap: 1rem; }
  .scenarios { display: flex; flex-direction: column; gap: 0.75rem; }
  .scenario-card {
    background: var(--bg-card); border: 1px solid var(--border);
    border-radius: 12px; padding: 1rem 1.1rem;
    box-shadow: 0 1px 3px rgba(0,0,0,0.04);
  }
  .scenario-card.base { border-color: var(--accent); }
  .sc-label { font-size: 0.72rem; color: var(--text-dim); text-transform: uppercase; letter-spacing: 0.07em; margin-bottom: 0.2rem; }
  .sc-fatturato { font-size: 1.3rem; font-weight: 700; color: var(--text); margin-bottom: 0.75rem; }
  .sc-rows { display: flex; flex-direction: column; gap: 0.4rem; }
  .sc-row { display: flex; justify-content: space-between; font-size: 0.85rem; color: var(--text-muted); }
  .sc-row.netto { border-top: 1px solid var(--border2); padding-top: 0.4rem; margin-top: 0.1rem; font-weight: 600; color: var(--text); }
  .neg { color: var(--expense); }
  .pos { color: var(--income); }

  /* Rate INPS */
  .rate-card {
    background: var(--bg-card); border-radius: 12px; padding: 1rem 1.1rem;
    box-shadow: 0 1px 3px rgba(0,0,0,0.04); border: 1px solid var(--border);
  }
  .rate-title { font-size: 0.72rem; text-transform: uppercase; letter-spacing: 0.07em; color: var(--text-dim); margin-bottom: 0.75rem; font-weight: 600; }
  .rate-rows { display: flex; flex-direction: column; gap: 0; }
  .rate-row { display: grid; grid-template-columns: 1fr auto auto; gap: 0.75rem; padding: 0.45rem 0; border-bottom: 1px solid var(--border2); font-size: 0.82rem; align-items: baseline; }
  .rate-row:last-child { border-bottom: none; }
  .rate-row.total { font-weight: 700; color: var(--text); padding-top: 0.6rem; border-top: 1px solid var(--border); border-bottom: none; }
  .rate-label { color: var(--text-muted); }
  .rate-scad { color: var(--text-dim); font-size: 0.75rem; white-space: nowrap; }
  .rate-importo { text-align: right; color: var(--expense); white-space: nowrap; }
  .rate-row.total .rate-importo { color: var(--text); }

  .disclaimer {
    font-size: 0.75rem; color: var(--text-dim); margin: 1rem 0 0;
    line-height: 1.6; border-top: 1px solid var(--border2); padding-top: 1rem;
  }

  @media (max-width: 700px) {
    .layout { grid-template-columns: 1fr; }
  }
</style>
