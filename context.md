# Fehu — Session Context

Track progress across conversations. Update at end of each session.

## Stack
- Tauri 2 + Svelte 5 + TypeScript + Vite
- rusqlite (bundled-full) — SQLite at app data dir
- Ollama — local LLM (Plan B)
- Python aiogram sidecar — Telegram bot (Plan C)

## Stack notes
- SvelteKit with `@sveltejs/adapter-static` (required for Tauri) — app shell in `src/routes/+page.svelte`
- `@lucide/svelte` for icons (replaced deprecated `lucide-svelte`)
- calamine 0.35 for xlsx reading (API: `Data::DateTime`, `to_ymd_hms_milli()`)

## Plans
- [x] Plan A defined: Core desktop app
- [x] Plan A implemented
- [ ] Plan B defined: LLM pipeline (Ollama categorization + receipt OCR)
- [ ] Plan C defined: Telegram bot (Python aiogram sidecar)

## Session Log

| Date | What happened |
|------|--------------|
| 2026-05-28 | Project started, Plan A written |
| 2026-05-28 | Plan A fully implemented — 20 commits, all tasks complete |
