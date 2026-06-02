<div align="center">

# ᚠ Fehu

**Local-first personal finance for people who don't trust the cloud.**

[![License: MIT](https://img.shields.io/badge/license-MIT-6366f1?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-lightgrey?style=flat-square&logo=apple)](../../releases)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-24C8D8?style=flat-square&logo=tauri)](https://tauri.app)
[![Svelte 5](https://img.shields.io/badge/Svelte-5-FF3E00?style=flat-square&logo=svelte&logoColor=white)](https://svelte.dev)
[![Release](https://img.shields.io/github/v/release/ZonatedCord/Fehu?style=flat-square&color=6366f1)](../../releases/latest)
[![Built with Claude Code](https://img.shields.io/badge/built%20with-Claude%20Code-D97706?style=flat-square&logo=anthropic&logoColor=white)](https://claude.ai/code)

*Named after the Elder Futhark rune ᚠ — Old Norse for wealth.*

[**Download**](../../releases/latest) · [**Build from source**](#build-from-source) · [**Stack**](#stack)

</div>

---

## Why Fehu exists

I needed a personal finance app that was **free**, **local** (no cloud, no account), and complete enough to handle categories, recurring payments, P.IVA tax, and receipt scanning.

Nothing checked all three boxes. Free apps were too basic. Powerful apps were paywalled or cloud-only.

Nobody had built it the right way. So I built it myself.

---

## What is Fehu?

Fehu is a desktop app for tracking personal finances — income, expenses, savings, receipts, recurring payments. Everything runs locally: **no account, no sync, no subscription, no servers**. Your data is a single SQLite file on your disk.

---

## Features

### Core
| | |
|---|---|
| **Dashboard** | KPI cards, monthly bar chart, expense donut by category, Sankey money-flow chart, budget alerts |
| **Transactions** | Table and calendar views, inline category creation, file attachments, cash / card badges, multi-currency |
| **Recurring** | Templates that auto-insert at the scheduled date — daily, weekly, monthly, yearly |
| **Categories** | Icon + color picker, optional monthly budget cap with dashboard alert |
| **Goals** | Savings funds with progress bars — contributions create linked transactions, reduce saldo, appear in all charts. Per-goal history view. Dashboard overview card. |

### Power features
| | |
|---|---|
| **OCR receipts** | Drag-drop an image → Tesseract 5 extracts amount, date, merchant → local LLM categorizes automatically |
| **P.IVA calculator** | Italian freelance tax: Forfettario / Ordinario / Semplificato, INPS variants, ±20% projections. Rates auto-fetched from GitHub (2026 INPS + IRPEF). |
| **Telegram bot** | Add expenses and receipts from Telegram; desktop notifications via local bot |
| **Import / Export** | Import from *Registro Lavoro* XLSX, export filtered CSV and XLSX |
| **Backup & Restore** | One-click full SQLite database export and restore |
| **Dark / Light theme** | Persistent, floating card layout |
| **Native menu bar** | macOS native menu with keyboard shortcuts (⌘1–7, ⌘N, ⌘F, ⌘E, ⌘,) |

---

## Requirements

| Dependency | Required | Purpose |
|---|---|---|
| [Tesseract 5](https://github.com/tesseract-ocr/tesseract) | **Yes** | OCR engine for receipt scanning |
| [Ollama](https://ollama.com) | No | Local LLM auto-categorization (keyword fallback when offline) |

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
| Windows | `Fehu_*_x64_en-US.msi` (recommended) or `Fehu_*_x64-setup.exe` |

> **macOS — Gatekeeper:** if the app is blocked on first launch, run:
> ```bash
> xattr -dr com.apple.quarantine /Applications/Fehu.app
> ```
> Or use the `Rimuovi_Quarantine.command` script included in the DMG.

> **Windows — SmartScreen:** the app is not commercially signed (open source, free). If blocked:
> 1. Click **More info**
> 2. Click **Run anyway**
>
> The `.msi` installer is less likely to trigger this warning.

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
| LLM | Ollama HTTP API (model configurable) |
| Charts | Chart.js 4 + custom Sankey |
| Icons | @lucide/svelte |
| Bot | Python aiogram 3 sidecar |

---

## License

MIT — see [LICENSE](LICENSE).

---

<div align="center">

Built by [Marco Barlera](https://marcobarlera.com) with [Claude Code](https://claude.ai/code)

</div>
