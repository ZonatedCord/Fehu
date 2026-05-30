# Fehu ᚠ

**Fehu** is a local-first personal finance desktop app. The name comes from the Elder Futhark rune ᚠ — Old Norse for *wealth* and *cattle*.

No cloud. No subscriptions. Your data stays on your machine.

---

## Features

- **Dashboard** — KPI cards (income / expenses / balance), monthly bar chart, expense donut by category, Sankey flow chart, budget alerts
- **Transactions** — table and calendar views, CRUD with inline category creation, file attachments, cash/card metodo badges, multi-currency support
- **Recurring transactions** — templates that auto-insert at the scheduled date (daily / weekly / monthly / yearly)
- **Categories** — icon + color picker, optional monthly budget limit with alert on Dashboard
- **Goals / Savings funds** — progress bars, quick-add buttons, versamento creates an expense transaction
- **OCR receipts** — drag-drop an image → Tesseract 5 extracts amount/date/merchant → qwen2.5-coder:7b categorizes (keyword fallback when Ollama offline), payment method auto-detected from receipt text
- **P.IVA calculator** — Forfettario / Ordinario / Semplificato regimes, INPS variants, 3 scenarios (−20% / base / +20%)
- **Multi-currency** — per-transaction currency (EUR, USD, GBP, CHF, JPY)
- **Backup & Restore** — export / restore the full SQLite database
- **Import / Export** — import from "Registro Lavoro" XLSX, export CSV and XLSX with date filters

---

## Requirements

| Dependency | Required | Purpose |
|---|---|---|
| [Tesseract 5](https://github.com/tesseract-ocr/tesseract) | **Yes** | OCR engine for receipt scanning |
| [Ollama](https://ollama.com) + `qwen2.5-coder:7b` | No | LLM categorization (keyword fallback used when unavailable) |

### macOS

```bash
brew install tesseract tesseract-lang
```

### Windows

Download the Tesseract installer from [UB Mannheim](https://github.com/UB-Mannheim/tesseract/wiki). Default install path `C:\Program Files\Tesseract-OCR\` is auto-detected.

---

## Install

Download the latest release for your platform from [Releases](../../releases):

| Platform | File |
|---|---|
| macOS Apple Silicon | `Fehu_*_aarch64.dmg` |
| macOS Intel | `Fehu_*_x86_64.dmg` |
| Windows | `Fehu_*_x64-setup.exe` |

**macOS note:** if Gatekeeper blocks the app on first launch, run:
```bash
xattr -dr com.apple.quarantine /Applications/Fehu.app
```

---

## Build from source

### Prerequisites

- [Node.js 20+](https://nodejs.org) + [pnpm](https://pnpm.io)
- [Rust stable](https://rustup.rs)
- Xcode Command Line Tools (macOS) or Visual Studio Build Tools (Windows)

### Steps

```bash
git clone https://github.com/marcobarlera/fehu
cd fehu
pnpm install
pnpm tauri build
```

The `.dmg` / `.msi` is produced in `src-tauri/target/release/bundle/`.

### Dev server

```bash
pnpm tauri dev
```

---

## Stack

| Layer | Technology |
|---|---|
| Desktop shell | [Tauri 2](https://tauri.app) |
| Frontend | [Svelte 5](https://svelte.dev) + TypeScript + Vite |
| Database | SQLite via rusqlite (WAL mode) |
| OCR | Tesseract 5 (local subprocess) |
| LLM | Ollama HTTP API — `qwen2.5-coder:7b` |
| Charts | Chart.js 4 + custom Sankey |
| Icons | @lucide/svelte |

---

## License

MIT — see [LICENSE](LICENSE).

---

*Built by [Marco Barlera](https://marcobarlera.it)*
