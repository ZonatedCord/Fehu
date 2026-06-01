# Fehu — Session Context

Track progress across conversations. Update at end of each session.

---

## Stack

| Layer | Tech |
|-------|------|
| Desktop | Tauri 2 + Svelte 5 + TypeScript + Vite |
| Backend | Rust (lib.rs + commands/) |
| Storage | SQLite via rusqlite (bundled-full), WAL mode, PRAGMA user_version migrations |
| OCR | Tesseract 5 (`brew install tesseract tesseract-lang`) |
| LLM | Ollama HTTP API — `qwen2.5-coder:7b` — opzionale |
| Charting | Chart.js 4 + SankeyChart custom |
| Icons | `@lucide/svelte` v1.x |
| URL open | `@tauri-apps/plugin-opener` (`openUrl`, `openPath`) |
| File pick | `@tauri-apps/plugin-dialog` (`open`) |
| Notifications | `tauri-plugin-notification` |
| Auto-update | `tauri-plugin-updater` (keypair da generare — vedi sotto) |
| Telegram bot | Python aiogram 3.x sidecar — `sidecar/fehu_bot.py` |

**Note:** `@tauri-apps/plugin-shell` NON installato. Usare `plugin-opener`.
**Note:** Lucide non ha `Github` (brand icon rimosso). Usare `ExternalLink`.
**Note:** Python venv per bot in `sidecar/.venv/` — aiogram + aiosqlite installati su questa macchina.

---

## Struttura progetto

```
src/
  routes/+page.svelte        # App shell: layout + router (currentPage store) + Onboarding overlay
                             # CSS vars sistema tema; keyboard shortcuts Cmd+N/F/,; fade 110ms
  lib/
    types.ts                 # Page, Transaction, Category, Goal, BudgetAlert, RecurringTemplate, ...
    api.ts                   # Wrapper invoke() per tutti i comandi Tauri
    stores.ts                # currentPage, keyboardAction, theme, updateAvailable writable stores
  components/
    Sidebar.svelte           # mainNav (8) + utilityNav (2) + "Tema chiaro/scuro" toggle nav item
    Onboarding.svelte        # Wizard primo avvio
    Modal.svelte             # Modal generico riusabile
    SankeyChart.svelte
    CategoryIcon.svelte
    IconPicker.svelte
  pages/
    Dashboard.svelte         # KPI, charts, rettifiche, budget alerts
    Transactions.svelte      # Lista + calendario + CRUD + allegati + search + date range filter
    Categories.svelte        # CRUD + budget_limit per categoria
    Export.svelte            # Import XLSX, export CSV/XLSX, backup/restore DB
    Foto.svelte              # OCR pipeline
    Obiettivi.svelte         # Goals con progress bar
    Ricorrenti.svelte        # Template ricorrenti + auto-insert
    Settings.svelte          # Config + Telegram bot toggle + check aggiornamenti
    About.svelte
    PIva.svelte              # Calcolatore forfettario/ordinario/semplificato

sidecar/
  fehu_bot.py               # Telegram bot: /start, /report [mese], /foto (OCR aiogram FSM)
  requirements.txt
  .venv/                    # virtualenv con aiogram 3.28 + aiosqlite (gitignored)

src-tauri/src/
  lib.rs                     # setup + native menu bar (Fehu/File/Modifica/Visualizza) + background thread notifiche
  models.rs
  error.rs
  db/mod.rs                  # MIGRATIONS[6] + PRAGMA user_version
  commands/
    categories.rs            # CRUD + budget_limit
    transactions.rs          # CRUD + search_text, currency, attachment_count subquery
    goals.rs
    stats.rs                 # get_dashboard_stats, get_budget_alerts
    export.rs / import.rs
    patrimonio.rs
    vision.rs                # OCR + Ollama + keyword fallback + metodo auto
    settings.rs
    check.rs
    files.rs                 # attach/list/delete_attachment + cascade disk delete
    backup.rs                # export_database (WAL checkpoint), restore_database (integrity_check + swap)
    recurring.rs             # CRUD + toggle + check_and_insert_recurring
    telegram.rs              # start/stop/status — usa sidecar/.venv/bin/python3 se disponibile
```

---

## DB Schema (PRAGMA user_version = 6)

```sql
categories          (id, name, color, icon, budget_limit REAL)
transactions        (id, amount, type, category_id, date, description, notes, source,
                     metodo TEXT DEFAULT 'carta', currency TEXT DEFAULT 'EUR', created_at)
goals               (id, name, target, saved, color, icon, created_at)
balance_adjustments (id, metodo, amount, note, date, created_at)
settings            (key TEXT PK, value TEXT)
transaction_files   (id, transaction_id CASCADE, file_name, file_path, created_at)
recurring_templates (id, description, amount, type, category_id, frequency, next_date, active, created_at)
bot_notifications   (id, title, body, shown INT DEFAULT 0, created_at)
```

Seed: INSERT OR IGNORE categoria "Altro" (#9ca3af, help-circle).

---

## CSS Tema

Sistema basato su CSS custom properties in `+page.svelte` `:global(:root)` / `[data-theme="light"]`.

Vars principali: `--bg-base`, `--bg-card`, `--bg-card2`, `--bg-elevated`, `--border`, `--border2`, `--text`, `--text-muted`, `--text-dim`, `--accent`, `--accent-lt`, `--income`, `--expense`, `--sidebar-bg`.

Toggle: nav item "Tema chiaro/scuro" in Sidebar → setAttribute + api.setSetting('theme').
Persistenza: letto da settings in onMount di +page.svelte.

---

## Telegram Bot

Venv: `sidecar/.venv/bin/python3` — aiogram 3.28, aiosqlite installati.
Rust: usa venv python se esiste, fallback a `python3` di sistema.
Token: salvato in settings table (key `telegram_token`) — auto-saved al click "Avvia bot".
Comandi: `/start`, `/report [mese]`, `/foto` (aiogram FSM: download→OCR→confirm→INSERT).
Notifiche: bot scrive in `bot_notifications` → thread Rust→ notifica nativa OS ogni 3s.

---

## Auto-update ✅ configurato

- Keypair generato: `~/.tauri/fehu.key` (privata) / `~/.tauri/fehu.key.pub` (pubblica)
- `tauri.conf.json` → `plugins.updater.pubkey` settato, endpoint → `ZonatedCord/Fehu`, `dialog: false`
- GitHub Secrets → `TAURI_SIGNING_PRIVATE_KEY` + `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` settati
- CI `.github/workflows/release.yml` — permissions: `contents: write`
- Startup check: 3s delay → se update → `pendingUpdate` state → modale toast in alto a destra
- Badge arancione su icona Impostazioni sidebar se update disponibile

---

## Settings Keys

| Key | Default | Uso |
|-----|---------|-----|
| `ollama_url` | `http://localhost:11434` | URL Ollama |
| `tesseract_path` | `""` | Override path Tesseract |
| `currency_symbol` | `€` | Simbolo valuta UI |
| `onboarded` | `"false"` | Flag primo avvio |
| `theme` | `"dark"` | `"dark"` / `"light"` |
| `telegram_token` | `""` | Token bot Telegram |

---

## Comandi Tauri (tutti registrati in lib.rs)

```
list/create/update/delete_category
list/create/update/delete_transaction
list/create/update_goal_saved/delete/contribute_to_goal
get_dashboard_stats, get_budget_alerts
export_csv, export_xlsx, import_xlsx
analyze_receipt, read_image_base64
get_patrimonio, list/create/delete_balance_adjustment
get_settings, set_setting
check_dependencies
attach_file, list_attachments, delete_attachment
export_database, restore_database
list/create/update/delete/toggle_recurring, check_and_insert_recurring
start_telegram_bot, stop_telegram_bot, get_telegram_status
```

---

## Test

- Rust: 27 test `#[cfg(test)]` (categories, transactions, budget alerts, recurring advance_date, leap year, check_and_insert)
- Playwright: configurato (`playwright.config.ts`), test in `e2e/basic.spec.ts`

---

## Session Log

| Data | Sessione |
|------|----------|
| 2026-05-28 | Progetto avviato, Plan A implementato — core desktop app |
| 2026-05-29 | Plan B (OCR pipeline), Settings, Onboarding, About, P.IVA, allegati file |
| 2026-05-29 | Fix OCR: keep_alive=0, categoria auto-creata, regex migliorata |
| 2026-05-30 | Round 1 backlog: ricorrenti, budget, multi-currency, backup, telegram sidecar, OCR metodo, cascade delete, attachment count |
| 2026-05-30 | Round 2: search, date filter, shortcuts, fade, tema chiaro/scuro, notifiche native, migration versioning, test suite, auto-update, bot /report+/foto |
| 2026-05-31 | Fix: bot path (CARGO_MANIFEST_DIR), CSS vars complete (--bg-elevated), layout centrato, token persistence, theme toggle nav item, Python venv aiogram installato |
| 2026-06-01 | Release v0.1.0 su GitHub (ZonatedCord/Fehu). CI 3 piattaforme (aarch64 ✅ Windows ✅ Intel ✅). Icona runa Fehu indigo. |
| 2026-06-02 | UI overhaul light theme: floating card layout, sidebar chiara, contrasti migliorati. Auto-update UX (startup popup, badge sidebar, settings section). Native macOS menu bar (Fehu/File/Modifica/Visualizza) con Cmd+1-7, E, N, F, e copy-paste nativi. Guide AI-assisted per Ollama e Telegram in Settings. Badge metodo → icone Lucide Banknote/CreditCard. Leggenda badge transazioni. QA checklist in docs/. README italiano "Perché esiste". |
