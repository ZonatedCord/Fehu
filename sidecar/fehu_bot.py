#!/usr/bin/env python3
"""
Fehu Telegram Bot — Plan C
Requires: pip install aiogram aiosqlite
Usage: python3 fehu_bot.py --token BOT_TOKEN --db-path /path/to/fehu.db
"""

import argparse
import asyncio
import sys
import logging
from datetime import datetime, date

try:
    import aiosqlite
    from aiogram import Bot, Dispatcher, types
    from aiogram.filters import Command
    from aiogram.types import Message
except ImportError as e:
    print(f"Dipendenze mancanti: {e}\nInstalla con: pip install aiogram aiosqlite", file=sys.stderr)
    sys.exit(1)

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(levelname)s %(message)s")
logger = logging.getLogger(__name__)

# --- Globals set at startup ---
DB_PATH: str = ""


# ─── Helpers ────────────────────────────────────────────────────────────────

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
            adj_contanti, adj_carta = row if row else (0, 0)

    return {
        "contanti": contanti + adj_contanti,
        "carta": carta + adj_carta,
        "totale": contanti + adj_contanti + carta + adj_carta,
    }


async def get_last_transactions(n: int = 5) -> list[dict]:
    async with aiosqlite.connect(DB_PATH) as db:
        async with db.execute("""
            SELECT t.date, t.description, t.amount, t.type, c.name
            FROM transactions t LEFT JOIN categories c ON c.id = t.category_id
            ORDER BY t.date DESC, t.id DESC
            LIMIT ?
        """, (n,)) as cur:
            rows = await cur.fetchall()
    return [{"date": r[0], "desc": r[1], "amount": r[2], "type": r[3], "cat": r[4]} for r in rows]


async def add_transaction(amount: float, description: str) -> int:
    today = date.today().isoformat()
    async with aiosqlite.connect(DB_PATH) as db:
        # Find or use "Altro" category
        async with db.execute("SELECT id FROM categories WHERE name='Altro' LIMIT 1") as cur:
            row = await cur.fetchone()
            cat_id = row[0] if row else None

        await db.execute(
            "INSERT INTO transactions (amount, type, category_id, date, description, notes, source, metodo, currency) "
            "VALUES (?, 'expense', ?, ?, ?, '', 'telegram', 'carta', 'EUR')",
            (amount, cat_id, today, description),
        )
        await db.commit()
        return await db.execute("SELECT last_insert_rowid()") and 0


# ─── Handlers ───────────────────────────────────────────────────────────────

async def cmd_start(message: Message):
    await message.answer(
        "👋 *Fehu Bot*\n\n"
        "Comandi disponibili:\n"
        "• `/saldo` — patrimonio attuale\n"
        "• `/spese [N]` — ultime N spese (default 5)\n"
        "• `/aggiungi <importo> <descrizione>` — aggiungi spesa\n"
        "• `/aiuta` — questo messaggio",
        parse_mode="Markdown",
    )


async def cmd_saldo(message: Message):
    try:
        p = await get_patrimonio()
        text = (
            "💰 *Saldo attuale*\n\n"
            f"Contanti: `{fmt_eur(p['contanti'])}`\n"
            f"Carta:    `{fmt_eur(p['carta'])}`\n"
            f"──────────────\n"
            f"*Totale:  `{fmt_eur(p['totale'])}`*"
        )
        await message.answer(text, parse_mode="Markdown")
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


async def cmd_aggiungi(message: Message):
    parts = (message.text or "").split(maxsplit=2)
    if len(parts) < 3:
        await message.answer("Uso: `/aggiungi <importo> <descrizione>`\nEsempio: `/aggiungi 12.50 Caffè al bar`", parse_mode="Markdown")
        return

    try:
        amount = float(parts[1].replace(",", "."))
        if amount <= 0:
            raise ValueError("importo deve essere positivo")
        description = parts[2]
    except ValueError as e:
        await message.answer(f"Importo non valido: {e}")
        return

    try:
        await add_transaction(amount, description)
        await message.answer(f"✅ Spesa aggiunta: *{fmt_eur(amount)}* — {description}", parse_mode="Markdown")
    except Exception as e:
        await message.answer(f"Errore salvataggio: {e}")


async def cmd_aiuta(message: Message):
    await cmd_start(message)


# ─── Main ───────────────────────────────────────────────────────────────────

async def main(token: str, db_path: str):
    global DB_PATH
    DB_PATH = db_path

    bot = Bot(token=token)
    dp = Dispatcher()

    dp.message.register(cmd_start,    Command("start"))
    dp.message.register(cmd_saldo,    Command("saldo"))
    dp.message.register(cmd_spese,    Command("spese"))
    dp.message.register(cmd_aggiungi, Command("aggiungi"))
    dp.message.register(cmd_aiuta,    Command("aiuta", "help"))

    logger.info("Fehu bot avviato. DB: %s", db_path)
    await dp.start_polling(bot)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Fehu Telegram Bot")
    parser.add_argument("--token",   required=True, help="Telegram bot token da @BotFather")
    parser.add_argument("--db-path", required=True, help="Percorso al file fehu.db")
    args = parser.parse_args()

    asyncio.run(main(args.token, args.db_path))
