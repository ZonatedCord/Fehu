# Fehu — Session Context

Track progress across conversations. Update at end of each session.

---

## Stack

| Layer | Tech |
|-------|------|
| Desktop | Tauri 2 + Svelte 5 + TypeScript + Vite |
| Backend | Rust (lib.rs + commands/) |
| Storage | SQLite via rusqlite (bundled-full), WAL mode |
| OCR | Tesseract 5 (`brew install tesseract tesseract-lang`) |
| LLM | Ollama HTTP API — `qwen2.5-coder:7b` — opzionale |
| Charting | Chart.js 4 + SankeyChart custom |
| Icons | `@lucide/svelte` v1.x |
| URL open | `@tauri-apps/plugin-opener` (`openUrl`, `openPath`) |
| File pick | `@tauri-apps/plugin-dialog` (`open`) |

**Note:** `@tauri-apps/plugin-shell` NON è installato. Usare `plugin-opener`.  
**Note:** Lucide non ha `Github` (brand icon rimosso). Usare `ExternalLink`.

---

## Struttura progetto

```
src/
  routes/+page.svelte        # App shell: layout + router (currentPage store) + Onboarding overlay
  lib/
    types.ts                 # Tutti i tipi TS (Page, Transaction, Category, Goal, ...)
    api.ts                   # Wrapper invoke() per tutti i comandi Tauri
    stores.ts                # currentPage writable store
  components/
    Sidebar.svelte           # Nav: mainNav (7 voci) + separatore + utilityNav (2 voci)
    Onboarding.svelte        # Wizard primo avvio (3 step), overlay fullscreen
    Modal.svelte             # Modal generico riusabile
    SankeyChart.svelte       # Sankey flow chart
    CategoryIcon.svelte      # Icona categoria
    IconPicker.svelte        # Picker icone
  pages/
    Dashboard.svelte         # KPI, charts (bar, doughnut, sankey)
    Transactions.svelte      # Lista + calendario + modal CRUD + allegati file
    Categories.svelte        # CRUD categorie + icon/color picker
    Export.svelte            # Import XLSX, export CSV/XLSX
    Foto.svelte              # OCR pipeline: drag-drop → analizza → salva tx
    Obiettivi.svelte         # Goals con progress bar, versamenti
    Settings.svelte          # Impostazioni (ollama_url, tesseract_path, currency_symbol)
    About.svelte             # Info app, nome runa, autore, stack, link
    PIva.svelte              # Calcolatore P.IVA (forfettario/ordinario/semplificato)

src-tauri/src/
  lib.rs                     # Tauri builder + setup DB + invoke_handler (tutti i comandi)
  models.rs                  # Struct Rust: Category, Transaction, Goal, ReceiptData, ...
  error.rs                   # AppError enum (Db, NotFound, Validation) + AppResult<T>
  db/mod.rs                  # open(), migrate() — schema SQLite completo
  commands/
    categories.rs            # list/create/update/delete_category
    transactions.rs          # list/create/update/delete_transaction
    goals.rs                 # list/create/update_saved/delete/contribute_to_goal
    stats.rs                 # get_dashboard_stats
    export.rs                # export_csv, export_xlsx
    import.rs                # import_xlsx
    patrimonio.rs            # get_patrimonio, list/create/delete_balance_adjustment
    vision.rs                # analyze_receipt (OCR + LLM + keyword fallback), read_image_base64
    settings.rs              # get_settings, set_setting
    check.rs                 # check_dependencies → { tesseract: bool, ollama: bool, version }
    files.rs                 # attach_file, list_attachments, delete_attachment
```

---

## DB Schema (corrente)

```sql
categories         (id, name, color, icon)
transactions       (id, amount, type, category_id→categories, date, description, notes, source, metodo, created_at)
goals              (id, name, target, saved, color, icon, created_at)
balance_adjustments(id, metodo, amount, note, date, created_at)
settings           (key TEXT PK, value TEXT)
transaction_files  (id, transaction_id→transactions ON DELETE CASCADE, file_name, file_path, created_at)
```

**Allegati**: copiati in `{app_data_dir}/attachments/{transaction_id}/{ts}_{filename}`

---

## Comandi Tauri registrati (lib.rs)

Categories: `list_categories`, `create_category`, `update_category`, `delete_category`  
Transactions: `list_transactions`, `create_transaction`, `update_transaction`, `delete_transaction`  
Goals: `list_goals`, `create_goal`, `update_goal_saved`, `delete_goal`, `contribute_to_goal`  
Stats: `get_dashboard_stats`  
Export/Import: `export_csv`, `export_xlsx`, `import_xlsx`  
Vision: `analyze_receipt`, `read_image_base64`  
Patrimonio: `get_patrimonio`, `list_balance_adjustments`, `create_balance_adjustment`, `delete_balance_adjustment`  
Settings: `get_settings`, `set_setting`  
Check: `check_dependencies`  
Files: `attach_file`, `list_attachments`, `delete_attachment`

---

## OCR Pipeline (vision.rs)

1. `find_tesseract(override)` — cerca path da settings, poi `/opt/homebrew`, `/usr/local`, `/usr/bin`, Windows paths, PATH fallback
2. Tesseract subprocess: `ita+eng, --psm 6, --dpi 150` → raw_text
3. Regex Rust: importo (`-9,40€`), data (mesi italiani + ISO + DD/MM/YYYY), merchant (linee UPPERCASE)
4. Ollama `http://{ollama_url}/api/generate` — modello `qwen2.5-coder:7b` → 1 parola categoria
5. Se Ollama fallisce → `categorize_by_keywords(raw_text)` → categoria da dizionario IT
6. `ReceiptData.categoria_source`: `"ollama"` | `"keyword"` | null

---

## Settings Keys

| Key | Default | Uso |
|-----|---------|-----|
| `ollama_url` | `http://localhost:11434` | URL server Ollama |
| `tesseract_path` | `""` | Override path Tesseract (vuoto = auto) |
| `currency_symbol` | `€` | Visualizzazione importi |
| `onboarded` | `"false"` | Flag primo avvio (onboarding wizard) |

---

## Pagine Sidebar

**Main nav:** Dashboard, Transazioni, Categorie, Dati, Foto, Obiettivi, P.IVA  
**Separatore** visivo  
**Utility nav:** Impostazioni, About

---

## Calcolatore P.IVA (PIva.svelte)

Puro frontend, nessun comando Rust. Tre tab:
- **Forfettario**: fatturato × coeff ATECO → base imponibile → INPS → imposta 5%/15%
- **Ordinario**: fatturato - spese → IRPEF scaglioni (23/35/43%) + addizionale regionale
- **Semplificato**: identico a Ordinario, con nota su limiti ricavi

Output: 3 scenari (pessimista −20%, base, ottimista +20%) con breakdown completo.

---

## Backlog (prioritizzato)

### 🔴 Alta
- **DMG + GitHub Release CI** — `.github/workflows/release.yml` (arm64 + intel + windows)
- **README professionale** — screenshot, requisiti, install, build da sorgente
- **Logo SVG definitivo** — runa ᚠ come asset, non testo
- **Balance adjustments list** — dashboard manca lista/delete rettifiche esistenti

### 🟡 Media
- **Plan C: Telegram bot** — Python aiogram sidecar (non iniziato)
- **Metodo auto da OCR** — Postepay/Bancomat/POS → carta; contanti → contanti
- **Transazioni ricorrenti** — spese periodiche con auto-insert
- **Backup/restore** — export SQLite completo + restore da file
- **Attachment count in lista** — icona paperclip su righe tx con allegati
- **Cascade file delete** — quando si elimina tx, cancellare file da disco

### 🟢 Bassa
- **Categoria "Altro" default** — seed categorie standard al primo avvio
- **Budget per categoria** — alert soglia mensile
- **Valuta multipla** — per spese in viaggio
- **UI polish** — empty states, spaziatura, animazioni

---

## Session Log

| Data | Sessione |
|------|----------|
| 2026-05-28 | Progetto avviato, Plan A scritto |
| 2026-05-28 | Plan A implementato — 20 commit, tutte le feature core |
| 2026-05-29 | Plan B (OCR) implementato: Tesseract + Ollama + regex Rust per importo/data/store |
| 2026-05-29 | Fix pipeline OCR: keep_alive=0, categoria auto-creata, regex più robuste |
| 2026-05-29 | **Sessione corrente**: Settings, Onboarding, About, P.IVA, keyword fallback, Windows support, allegati file, separatore sidebar |
