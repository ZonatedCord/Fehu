<div align="center">

# ᚠ Fehu

**Local-first personal finance for people who don't trust the cloud.**

[![License: MIT](https://img.shields.io/badge/license-MIT-6366f1?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-lightgrey?style=flat-square&logo=apple)](../../releases)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-24C8D8?style=flat-square&logo=tauri)](https://tauri.app)
[![Svelte 5](https://img.shields.io/badge/Svelte-5-FF3E00?style=flat-square&logo=svelte&logoColor=white)](https://svelte.dev)
[![Release](https://img.shields.io/github/v/release/ZonatedCord/Fehu?style=flat-square&color=6366f1)](../../releases/latest)

*Named after the Elder Futhark rune ᚠ — Old Norse for wealth.*

[**Download**](../../releases/latest) · [**Build from source**](#build-from-source) · [**Stack**](#stack)

</div>

---

## Why Fehu exists

I wanted a personal finance app that was:

- **free** — no subscription, no freemium limits
- **local** — no cloud account, no sync, no servers
- **complete** — categories, recurring payments, receipts with OCR, P.IVA calculator, Telegram bot

I couldn't find anything that hit all three. Free apps were too basic. Powerful apps were paywalled or cloud-only. So I built it myself.

## What is Fehu?

Fehu is a desktop app for tracking personal finances — income, expenses, savings, receipts, recurring payments. Everything runs locally: **no account, no sync, no subscription, no servers**. Your data is an SQLite file on your disk.

---

## Features

### Core
| | |
|---|---|
| **Dashboard** | KPI cards, monthly bar chart, expense donut by category, Sankey money-flow chart, budget limit alerts |
| **Transactions** | Table and calendar views, inline category creation, file attachments, cash / card badges, multi-currency |
| **Recurring** | Templates that auto-insert at the scheduled date — daily, weekly, monthly, yearly |
| **Categories** | Icon + color picker, optional monthly budget cap with dashboard alert |
| **Goals** | Savings funds with progress bars — topping up creates a linked expense transaction |

### Power features
| | |
|---|---|
| **OCR receipts** | Drag-drop an image → Tesseract 5 extracts amount, date, merchant → local LLM categorizes automatically |
| **P.IVA calculator** | Italian freelance tax scenarios: Forfettario / Ordinario / Semplificato, INPS variants, ±20% projections |
| **Import / Export** | Import from *Registro Lavoro* XLSX, export filtered CSV and XLSX |
| **Backup & Restore** | One-click full SQLite database export and restore |
| **Dark / Light theme** | System-aware, persistent |

---

## Requirements

| Dependency | Required | Purpose |
|---|---|---|
| [Tesseract 5](https://github.com/tesseract-ocr/tesseract) | **Yes** | OCR engine for receipt scanning |
| [Ollama](https://ollama.com) + `qwen2.5-coder:7b` | No | LLM auto-categorization (keyword fallback when offline) |

**macOS:**
```bash
brew install tesseract tesseract-lang
```

**Windows:** download the installer from [UB Mannheim](https://github.com/UB-Mannheim/tesseract/wiki). Default path `C:\Program Files\Tesseract-OCR\` is auto-detected.

---

## Install

Download the latest release from [**Releases**](../../releases/latest):

| Platform | File |
|---|---|
| macOS Apple Silicon | `Fehu_*_aarch64.dmg` |
| macOS Intel | `Fehu_*_x86_64.dmg` |
| Windows | `Fehu_*_x64-setup.exe` |

> **macOS — Gatekeeper:** if the app is blocked on first launch, run:
> ```bash
> xattr -dr com.apple.quarantine /Applications/Fehu.app
> ```

---

## Build from source

**Prerequisites:** Node.js 20+, [pnpm](https://pnpm.io), [Rust stable](https://rustup.rs), Xcode CLT (macOS) or VS Build Tools (Windows).

```bash
git clone https://github.com/ZonatedCord/Fehu
cd Fehu
pnpm install
pnpm tauri build        # → src-tauri/target/release/bundle/
```

```bash
pnpm tauri dev          # dev server with hot-reload
```

---

## Stack

| Layer | Technology |
|---|---|
| Desktop shell | [Tauri 2](https://tauri.app) (Rust backend) |
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

<div align="center">

Built by [Marco Barlera](https://marcobarlera.it)

</div>
