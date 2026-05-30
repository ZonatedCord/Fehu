#!/usr/bin/env python3
"""
Fehu Telegram Bot — Plan C
Requires: pip install aiogram aiosqlite
Usage: python3 fehu_bot.py --token BOT_TOKEN --db-path /path/to/fehu.db
"""

import argparse
import asyncio
import os
import re
import shutil
import subprocess
import sys
import tempfile
import logging
from calendar import month_name
from datetime import date

try:
    import aiosqlite
    from aiogram import Bot, Dispatcher, F
    from aiogram.filters import Command
    from aiogram.fsm.context import FSMContext
    from aiogram.fsm.state import State, StatesGroup
    from aiogram.fsm.storage.memory import MemoryStorage
    from aiogram.types import Message
except ImportError as e:
    print(f"Dipendenze mancanti: {e}\nInstalla con: pip install aiogram aiosqlite", file=sys.stderr)
    sys.exit(1)

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
logger = logging.getLogger(__name__)

DB_PATH: str = ""

MESI_IT = {
    "gennaio": "01", "febbraio": "02", "marzo": "03", "aprile": "04",
    "maggio": "05", "giugno": "06", "luglio": "07", "agosto": "08",
    "settembre": "09", "ottobre": "10", "novembre": "11", "dicembre": "12",
}


# ─── FSM States ──────────────────────────────────────────────────────────────

class FotoStates(StatesGroup):
    waiting_confirm = State()


# ─── Helpers ─────────────────────────────────────────────────────────────────

def fmt_eur(amount: float) -> str:
    return f"{amount:,.2f} €".replace(",", "X").replace(".", ",").replace("X", ".")


async def get_patrimonio() -> dict:
    async with aiosqlite.connect(DB_PATH) as db:
        async with db.execute("""
            SELECT
                COALESCE(SUM(CASE WHEN type='income'  AND metodo='contanti' THEN amount
                                  WHEN type='expense' AND metodo='contanti' THEN -amount ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN type='income'  AND metodo='carta' THEN amount
                                  WHEN type='expense' AND metodo='carta' THEN -amount ELSE 0 END), 0)
            FROM transactions
        """) as cur:
            row = await cur.fetchone()
            contanti, carta = row if row else (0, 0)
        async with db.execute("""
            SELECT
                COALESCE(SUM(CASE WHEN metodo='contanti' THEN amount ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN metodo='carta' THEN amount ELSE 0 END), 0)
            FROM balance_adjustments
        """) as cur:
            row = await cur.fetchone()
            adj_c, adj_k = row if row else (0, 0)
    return {"contanti": contanti + adj_c, "carta": carta + adj_k,
            "totale": contanti + adj_c + carta + adj_k}


async def get_last_transactions(n: int = 5) -> list[dict]:
    async with aiosqlite.connect(DB_PATH) as db:
        async with db.execute("""
            SELECT t.date, t.description, t.amount, t.type, c.name
            FROM transactions t LEFT JOIN categories c ON c.id = t.category_id
            ORDER BY t.date DESC, t.id DESC LIMIT ?
        """, (n,)) as cur:
            rows = await cur.fetchall()
    return [{"date": r[0], "desc": r[1], "amount": r[2], "type": r[3], "cat": r[4]} for r in rows]


async def get_monthly_report(month: str) -> list[dict]:
    """month: YYYY-MM"""
    async with aiosqlite.connect(DB_PATH) as db:
        async with db.execute("""
            SELECT c.name, SUM(t.amount) AS spent, c.budget_limit
            FROM transactions t
            LEFT JOIN categories c ON c.id = t.category_id
            WHERE t.type = 'expense' AND strftime('%Y-%m', t.date) = ?
            GROUP BY t.category_id
            ORDER BY spent DESC
        """, (month,)) as cur:
            rows = await cur.fetchall()
        async with db.execute("""
            SELECT COALESCE(SUM(amount), 0) FROM transactions
            WHERE type='income' AND strftime('%Y-%m', date) = ?
        """, (month,)) as cur:
            total_in = (await cur.fetchone())[0]
        async with db.execute("""
            SELECT COALESCE(SUM(amount), 0) FROM transactions
            WHERE type='expense' AND strftime('%Y-%m', date) = ?
        """, (month,)) as cur:
            total_out = (await cur.fetchone())[0]
    return {"rows": [{"cat": r[0] or "—", "spent": r[1], "budget": r[2]} for r in rows],
            "income": total_in, "expense": total_out}


async def add_transaction(amount: float, description: str, tx_date: str | None = None) -> None:
    today = tx_date or date.today().isoformat()
    async with aiosqlite.connect(DB_PATH) as db:
        async with db.execute("SELECT id FROM categories WHERE name='Altro' LIMIT 1") as cur:
            row = await cur.fetchone()
            cat_id = row[0] if row else None
        await db.execute(
            "INSERT INTO transactions (amount,type,category_id,date,description,notes,source,metodo,currency)"
            " VALUES (?, 'expense', ?, ?, ?, '', 'telegram', 'carta', 'EUR')",
            (amount, cat_id, today, description),
        )
        # Notify Fehu desktop app
        await db.execute(
            "INSERT INTO bot_notifications (title, body) VALUES (?, ?)",
            ("Nuova spesa via Telegram", f"{fmt_eur(amount)} — {description}"),
        )
        await db.commit()


async def find_tesseract() -> str | None:
    async with aiosqlite.connect(DB_PATH) as db:
        async with db.execute("SELECT value FROM settings WHERE key='tesseract_path'") as cur:
            row = await cur.fetchone()
            override = row[0] if row else ""
    if override and os.path.exists(override):
        return override
    candidates = [
        "/opt/homebrew/bin/tesseract", "/usr/local/bin/tesseract", "/usr/bin/tesseract",
        r"C:\Program Files\Tesseract-OCR\tesseract.exe",
    ]
    for p in candidates:
        if os.path.exists(p):
            return p
    return shutil.which("tesseract")


def parse_amount(text: str) -> float | None:
    m = re.search(r"-?(\d+)[,. ](\d{2})\s*[€$]", text)
    if m:
        return float(f"{m.group(1)}.{m.group(2)}")
    return None


def parse_merchant(text: str) -> str:
    for line in text.splitlines():
        t = line.strip()
        if len(t) < 3:
            continue
        letters = sum(1 for c in t if c.isalpha())
        uppers = sum(1 for c in t if c.isupper())
        if letters > 2 and uppers * 100 // max(letters, 1) >= 60:
            return t[:40]
    return ""


# ─── Handlers ────────────────────────────────────────────────────────────────

async def cmd_start(message: Message):
    await message.answer(
        "👋 *Fehu Bot*\n\n"
        "Comandi:\n"
        "• `/saldo` — patrimonio attuale\n"
        "• `/spese [N]` — ultime N transazioni\n"
        "• `/aggiungi <importo> <descrizione>` — aggiungi spesa\n"
        "• `/report [mese]` — report mensile (es. `/report maggio`)\n"
        "• Manda una *foto* — OCR scontrino → conferma → salva\n"
        "• `/aiuta` — questo messaggio",
        parse_mode="Markdown",
    )


async def cmd_saldo(message: Message):
    try:
        p = await get_patrimonio()
        await message.answer(
            "💰 *Saldo attuale*\n\n"
            f"Contanti: `{fmt_eur(p['contanti'])}`\n"
            f"Carta:    `{fmt_eur(p['carta'])}`\n"
            f"──────────────\n"
            f"*Totale:  `{fmt_eur(p['totale'])}`*",
            parse_mode="Markdown",
        )
    except Exception as e:
        await message.answer(f"Errore: {e}")


async def cmd_spese(message: Message):
    parts = (message.text or "").split()
    n = 5
    if len(parts) >= 2:
        try:
            n = max(1, min(20, int(parts[1])))
        except ValueError:
            pass
    try:
        txs = await get_last_transactions(n)
        if not txs:
            await message.answer("Nessuna transazione.")
            return
        lines = [f"📋 *Ultime {n} transazioni*\n"]
        for t in txs:
            sign = "+" if t["type"] == "income" else "-"
            cat = f" [{t['cat']}]" if t["cat"] else ""
            lines.append(f"`{t['date']}` {sign}{fmt_eur(t['amount'])} — {t['desc'] or '—'}{cat}")
        await message.answer("\n".join(lines), parse_mode="Markdown")
    except Exception as e:
        await message.answer(f"Errore: {e}")


async def cmd_report(message: Message):
    parts = (message.text or "").split(maxsplit=1)
    today = date.today()
    month = f"{today.year}-{today.month:02d}"

    if len(parts) >= 2:
        arg = parts[1].strip().lower()
        # Try "maggio" / "may" → YYYY-MM current year
        found = MESI_IT.get(arg)
        if found:
            month = f"{today.year}-{found}"
        # Try "YYYY-MM"
        elif re.match(r"\d{4}-\d{2}", arg):
            month = arg

    try:
        data = await get_monthly_report(month)
        rows = data["rows"]
        income, expense = data["income"], data["expense"]

        label = month
        lines = [f"📊 *Report {label}*\n",
                 f"Entrate: `{fmt_eur(income)}`  Uscite: `{fmt_eur(expense)}`\n"]
        for r in rows:
            budget_str = f" / {fmt_eur(r['budget'])}" if r["budget"] else ""
            over = " ⚠️" if r["budget"] and r["spent"] > r["budget"] else ""
            lines.append(f"• {r['cat']}: `{fmt_eur(r['spent'])}{budget_str}`{over}")

        if not rows:
            lines.append("Nessuna spesa questo mese.")

        await message.answer("\n".join(lines), parse_mode="Markdown")
    except Exception as e:
        await message.answer(f"Errore: {e}")


async def cmd_aggiungi(message: Message):
    parts = (message.text or "").split(maxsplit=2)
    if len(parts) < 3:
        await message.answer(
            "Uso: `/aggiungi <importo> <descrizione>`\nEs: `/aggiungi 12.50 Caffè al bar`",
            parse_mode="Markdown",
        )
        return
    try:
        amount = float(parts[1].replace(",", "."))
        if amount <= 0:
            raise ValueError("deve essere positivo")
        description = parts[2]
    except ValueError as e:
        await message.answer(f"Importo non valido: {e}")
        return
    try:
        await add_transaction(amount, description)
        await message.answer(f"✅ *{fmt_eur(amount)}* — {description}", parse_mode="Markdown")
    except Exception as e:
        await message.answer(f"Errore salvataggio: {e}")


async def cmd_aiuta(message: Message):
    await cmd_start(message)


async def handle_photo(message: Message, bot: Bot, state: FSMContext):
    """OCR flow: photo → tesseract → parse → ask confirm."""
    tess = await find_tesseract()
    if not tess:
        await message.answer("Tesseract non trovato. Installa con:\nmacOS: brew install tesseract tesseract-lang\nWindows: github.com/UB-Mannheim/tesseract")
        return

    photo = message.photo[-1]  # highest resolution
    await message.answer("Analisi immagine…")

    with tempfile.NamedTemporaryFile(suffix=".jpg", delete=False) as tmp:
        tmp_path = tmp.name

    try:
        await bot.download(photo, destination=tmp_path)
        result = subprocess.run(
            [tess, tmp_path, "stdout", "-l", "ita+eng", "--psm", "6", "--dpi", "150"],
            capture_output=True, text=True, timeout=30,
        )
        raw = result.stdout.strip()
    except subprocess.TimeoutExpired:
        await message.answer("Timeout OCR. Riprova con un'immagine più nitida.")
        return
    except Exception as e:
        await message.answer(f"Errore OCR: {e}")
        return
    finally:
        try:
            os.unlink(tmp_path)
        except OSError:
            pass

    if not raw:
        await message.answer("Nessun testo trovato nell'immagine.")
        return

    amount = parse_amount(raw)
    merchant = parse_merchant(raw)

    if not amount:
        await message.answer(
            f"Non ho trovato un importo nell'immagine.\n\nTesto estratto:\n```\n{raw[:400]}\n```\n\n"
            "Usa `/aggiungi <importo> <descrizione>` per aggiungere manualmente.",
            parse_mode="Markdown",
        )
        return

    preview = f"*{fmt_eur(amount)}*" + (f" — {merchant}" if merchant else "")
    await state.set_state(FotoStates.waiting_confirm)
    await state.update_data(amount=amount, description=merchant or "Scontrino")
    await message.answer(
        f"Ho trovato: {preview}\n\nConfermi? Rispondi *sì* o *no*.",
        parse_mode="Markdown",
    )


async def handle_foto_confirm(message: Message, state: FSMContext):
    text = (message.text or "").strip().lower()
    if text in ("sì", "si", "yes", "s", "y", "ok", "1"):
        data = await state.get_data()
        try:
            await add_transaction(data["amount"], data["description"])
            await message.answer(
                f"✅ Salvato: *{fmt_eur(data['amount'])}* — {data['description']}",
                parse_mode="Markdown",
            )
        except Exception as e:
            await message.answer(f"Errore salvataggio: {e}")
    else:
        await message.answer("Annullato. Usa `/aggiungi` per inserire manualmente.")
    await state.clear()


# ─── Main ────────────────────────────────────────────────────────────────────

async def main(token: str, db_path: str):
    global DB_PATH
    DB_PATH = db_path

    bot = Bot(token=token)
    storage = MemoryStorage()
    dp = Dispatcher(storage=storage)

    dp.message.register(cmd_start,    Command("start"))
    dp.message.register(cmd_saldo,    Command("saldo"))
    dp.message.register(cmd_spese,    Command("spese"))
    dp.message.register(cmd_report,   Command("report"))
    dp.message.register(cmd_aggiungi, Command("aggiungi"))
    dp.message.register(cmd_aiuta,    Command("aiuta", "help"))
    # FSM: photo confirmation
    dp.message.register(handle_foto_confirm, FotoStates.waiting_confirm, F.text)
    # Photo handler (must be last)
    dp.message.register(lambda m, s: handle_photo(m, bot, s), F.photo)

    logger.info("Fehu bot avviato. DB: %s", db_path)
    await dp.start_polling(bot)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Fehu Telegram Bot")
    parser.add_argument("--token",   required=True, help="Token da @BotFather")
    parser.add_argument("--db-path", required=True, help="Percorso fehu.db")
    args = parser.parse_args()
    asyncio.run(main(args.token, args.db_path))
