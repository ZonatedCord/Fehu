#!/usr/bin/env python3
"""
Fehu Telegram Bot
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
from datetime import date

try:
    import aiosqlite
    from aiogram import Bot, Dispatcher, F
    from aiogram.filters import Command
    from aiogram.fsm.context import FSMContext
    from aiogram.fsm.state import State, StatesGroup
    from aiogram.fsm.storage.memory import MemoryStorage
    from aiogram.types import Message, InlineKeyboardMarkup, InlineKeyboardButton, CallbackQuery
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


class FotoStates(StatesGroup):
    waiting_confirm = State()


class AggiungiStates(StatesGroup):
    waiting_category = State()
    waiting_metodo   = State()
    waiting_note     = State()
    waiting_confirm  = State()


def fmt_eur(amount: float) -> str:
    return f"{amount:,.2f} €".replace(",", "X").replace(".", ",").replace("X", ".")


def confirm_keyboard(amount: float, description: str) -> InlineKeyboardMarkup:
    return InlineKeyboardMarkup(inline_keyboard=[[
        InlineKeyboardButton(text="Salva", callback_data="foto_ok"),
        InlineKeyboardButton(text="Annulla", callback_data="foto_no"),
    ]])


def categories_keyboard(cats: list[dict]) -> InlineKeyboardMarkup:
    rows = []
    row = []
    for i, cat in enumerate(cats):
        row.append(InlineKeyboardButton(text=cat["name"], callback_data=f"aggiungi_cat_{cat['id']}"))
        if len(row) == 3:
            rows.append(row)
            row = []
    if row:
        rows.append(row)
    return InlineKeyboardMarkup(inline_keyboard=rows)


def metodo_keyboard() -> InlineKeyboardMarkup:
    return InlineKeyboardMarkup(inline_keyboard=[[
        InlineKeyboardButton(text="Contanti", callback_data="aggiungi_metodo_contanti"),
        InlineKeyboardButton(text="Carta",    callback_data="aggiungi_metodo_carta"),
    ]])


def note_keyboard() -> InlineKeyboardMarkup:
    return InlineKeyboardMarkup(inline_keyboard=[[
        InlineKeyboardButton(text="Salta", callback_data="aggiungi_note_skip"),
    ]])


def confirm_aggiungi_keyboard() -> InlineKeyboardMarkup:
    return InlineKeyboardMarkup(inline_keyboard=[[
        InlineKeyboardButton(text="Salva",   callback_data="aggiungi_ok"),
        InlineKeyboardButton(text="Annulla", callback_data="aggiungi_no"),
    ]])


_SESSIONE_SCADUTA = "Sessione scaduta. Riprendi con /aggiungi."

def build_summary(data: dict) -> str:
    label = "Entrata" if data.get("tx_type", "expense") == "income" else "Spesa"
    note_str = data.get("notes") or "—"
    return (
        f"*Riepilogo*\n\n"
        f"Importo: `{fmt_eur(data.get('amount', 0))}` ({label})\n"
        f"Descrizione: {data.get('description', '—')}\n"
        f"Categoria: {data.get('category_name', '—')}\n"
        f"Metodo: {data.get('metodo', '—').capitalize()}\n"
        f"Note: {note_str}"
    )


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


async def get_last_transactions(n: int = 5, category: str | None = None) -> list[dict]:
    async with aiosqlite.connect(DB_PATH) as db:
        if category:
            async with db.execute("""
                SELECT t.date, t.description, t.amount, t.type, c.name
                FROM transactions t LEFT JOIN categories c ON c.id = t.category_id
                WHERE LOWER(c.name) LIKE LOWER(?)
                ORDER BY t.date DESC, t.id DESC LIMIT ?
            """, (f"%{category}%", n)) as cur:
                rows = await cur.fetchall()
        else:
            async with db.execute("""
                SELECT t.date, t.description, t.amount, t.type, c.name
                FROM transactions t LEFT JOIN categories c ON c.id = t.category_id
                ORDER BY t.date DESC, t.id DESC LIMIT ?
            """, (n,)) as cur:
                rows = await cur.fetchall()
    return [{"date": r[0], "desc": r[1], "amount": r[2], "type": r[3], "cat": r[4]} for r in rows]


async def get_monthly_report(month: str) -> dict:
    async with aiosqlite.connect(DB_PATH) as db:
        async with db.execute("""
            SELECT c.name, SUM(t.amount) AS spent, c.budget_limit
            FROM transactions t
            LEFT JOIN categories c ON c.id = t.category_id
            WHERE t.type = 'expense' AND strftime('%Y-%m', t.date) = ?
            GROUP BY t.category_id ORDER BY spent DESC
        """, (month,)) as cur:
            rows = await cur.fetchall()
        async with db.execute(
            "SELECT COALESCE(SUM(amount),0) FROM transactions WHERE type='income' AND strftime('%Y-%m',date)=?", (month,)
        ) as cur:
            total_in = (await cur.fetchone())[0]
        async with db.execute(
            "SELECT COALESCE(SUM(amount),0) FROM transactions WHERE type='expense' AND strftime('%Y-%m',date)=?", (month,)
        ) as cur:
            total_out = (await cur.fetchone())[0]
    return {"rows": [{"cat": r[0] or "—", "spent": r[1], "budget": r[2]} for r in rows],
            "income": total_in, "expense": total_out}


async def get_goals() -> list[dict]:
    async with aiosqlite.connect(DB_PATH) as db:
        async with db.execute("SELECT name, target, saved, color FROM goals ORDER BY id") as cur:
            rows = await cur.fetchall()
    return [{"name": r[0], "target": r[1], "saved": r[2]} for r in rows]


async def get_categories_with_totals() -> list[dict]:
    month = f"{date.today().year}-{date.today().month:02d}"
    async with aiosqlite.connect(DB_PATH) as db:
        async with db.execute("""
            SELECT c.name, c.budget_limit,
                   COALESCE(SUM(CASE WHEN t.type='expense' AND strftime('%Y-%m',t.date)=? THEN t.amount ELSE 0 END),0)
            FROM categories c
            LEFT JOIN transactions t ON t.category_id = c.id
            GROUP BY c.id ORDER BY c.name
        """, (month,)) as cur:
            rows = await cur.fetchall()
    return [{"name": r[0], "budget": r[1], "spent": r[2]} for r in rows]


async def get_categories() -> list[dict]:
    async with aiosqlite.connect(DB_PATH) as db:
        async with db.execute("SELECT id, name FROM categories ORDER BY name") as cur:
            rows = await cur.fetchall()
    return [{"id": r[0], "name": r[1]} for r in rows]


async def add_transaction(amount: float, description: str, tx_type: str = "expense",
                          metodo: str = "carta", tx_date: str | None = None,
                          category_id: int | None = None, notes: str = "") -> None:
    today = tx_date or date.today().isoformat()
    async with aiosqlite.connect(DB_PATH) as db:
        if category_id is None:
            async with db.execute("SELECT id FROM categories WHERE name='Altro' LIMIT 1") as cur:
                row = await cur.fetchone()
                category_id = row[0] if row else None
        await db.execute(
            "INSERT INTO transactions (amount,type,category_id,date,description,notes,source,metodo,currency)"
            " VALUES (?, ?, ?, ?, ?, ?, 'telegram', ?, 'EUR')",
            (amount, tx_type, category_id, today, description, notes, metodo),
        )
        label = "entrata" if tx_type == "income" else "spesa"
        await db.execute(
            "INSERT INTO bot_notifications (title, body) VALUES (?, ?)",
            (f"Nuova {label} via Telegram", f"{fmt_eur(amount)} — {description}"),
        )
        await db.commit()


async def find_tesseract() -> str | None:
    async with aiosqlite.connect(DB_PATH) as db:
        async with db.execute("SELECT value FROM settings WHERE key='tesseract_path'") as cur:
            row = await cur.fetchone()
            override = row[0] if row else ""
    if override and os.path.exists(override):
        return override
    for p in ["/opt/homebrew/bin/tesseract", "/usr/local/bin/tesseract", "/usr/bin/tesseract",
              r"C:\Program Files\Tesseract-OCR\tesseract.exe"]:
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
        "*Fehu Bot*\n\n"
        "Comandi disponibili:\n"
        "• `/saldo` — patrimonio attuale\n"
        "• `/spese [N]` — ultime N transazioni\n"
        "• `/spese @categoria` — filtra per categoria\n"
        "• `/aggiungi <importo> <descrizione>` — aggiungi spesa (form guidato)\n"
        "• `/aggiungi +<importo> <descrizione>` — aggiungi entrata (form guidato)\n"
        "• `/report [mese]` — report mensile\n"
        "• `/obiettivi` — fondi risparmio\n"
        "• `/categorie` — categorie con spese del mese\n"
        "• Manda una *foto* — OCR scontrino automatico",
        parse_mode="Markdown",
    )


async def cmd_saldo(message: Message):
    try:
        p = await get_patrimonio()
        await message.answer(
            "*Saldo attuale*\n\n"
            f"Contanti: `{fmt_eur(p['contanti'])}`\n"
            f"Carta:    `{fmt_eur(p['carta'])}`\n"
            f"──────────────\n"
            f"*Totale: `{fmt_eur(p['totale'])}`*",
            parse_mode="Markdown",
        )
    except Exception as e:
        await message.answer(f"Errore: {e}")


async def cmd_spese(message: Message):
    parts = (message.text or "").split()
    n, cat_filter = 5, None
    for part in parts[1:]:
        if part.startswith("@"):
            cat_filter = part[1:]
        else:
            try:
                n = max(1, min(20, int(part)))
            except ValueError:
                pass
    try:
        txs = await get_last_transactions(n, cat_filter)
        if not txs:
            await message.answer("Nessuna transazione trovata.")
            return
        header = f"*Ultime {n} transazioni*" + (f" in _{cat_filter}_" if cat_filter else "")
        lines = [header + "\n"]
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
        found = MESI_IT.get(arg)
        if found:
            month = f"{today.year}-{found}"
        elif re.match(r"\d{4}-\d{2}", arg):
            month = arg
    try:
        data = await get_monthly_report(month)
        rows, income, expense = data["rows"], data["income"], data["expense"]
        lines = [f"*Report {month}*\n",
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


async def cmd_aggiungi(message: Message, state: FSMContext):
    parts = (message.text or "").split(maxsplit=2)
    if len(parts) < 3:
        await message.answer(
            "Uso:\n`/aggiungi 12.50 Caffè` — spesa\n`/aggiungi +50 Stipendio` — entrata",
            parse_mode="Markdown",
        )
        return
    raw_amount = parts[1]
    tx_type = "income" if raw_amount.startswith("+") else "expense"
    try:
        amount = float(raw_amount.lstrip("+").replace(",", "."))
        if amount <= 0:
            raise ValueError
        description = parts[2]
    except ValueError:
        await message.answer("Importo non valido.")
        return

    try:
        cats = await get_categories()
    except Exception as e:
        await message.answer(f"Errore categorie: {e}")
        return

    await state.set_state(AggiungiStates.waiting_category)
    await state.update_data(amount=amount, description=description, tx_type=tx_type)

    label = "Entrata" if tx_type == "income" else "Spesa"
    await message.answer(
        f"*{label}*: `{fmt_eur(amount)}` — {description}\n\nScegli categoria:",
        parse_mode="Markdown",
        reply_markup=categories_keyboard(cats),
    )


async def _expired(query: CallbackQuery, state: FSMContext) -> bool:
    """Return True and notify if FSM state is stale (bot was restarted)."""
    data = await state.get_data()
    if not data.get("amount"):
        await query.answer()
        await query.message.edit_text(_SESSIONE_SCADUTA)
        await state.clear()
        return True
    return False


async def callback_aggiungi_cat(query: CallbackQuery, state: FSMContext):
    if await _expired(query, state): return
    await query.answer()
    cat_id = int(query.data.split("_", 2)[2])
    cats = await get_categories()
    cat_name = next((c["name"] for c in cats if c["id"] == cat_id), "—")
    await state.update_data(category_id=cat_id, category_name=cat_name)
    await state.set_state(AggiungiStates.waiting_metodo)
    await query.message.edit_text(
        f"Categoria: *{cat_name}*\n\nMetodo di pagamento:",
        parse_mode="Markdown",
        reply_markup=metodo_keyboard(),
    )


async def callback_aggiungi_metodo(query: CallbackQuery, state: FSMContext):
    if await _expired(query, state): return
    await query.answer()
    metodo = query.data.split("_", 2)[2]  # "contanti" or "carta"
    await state.update_data(metodo=metodo)
    await state.set_state(AggiungiStates.waiting_note)
    await query.message.edit_text(
        f"Metodo: *{metodo.capitalize()}*\n\nNote aggiuntive? Digita o salta:",
        parse_mode="Markdown",
        reply_markup=note_keyboard(),
    )


async def handle_aggiungi_note(message: Message, state: FSMContext):
    data = await state.get_data()
    if not data.get("amount"):
        await message.answer(_SESSIONE_SCADUTA)
        await state.clear()
        return
    await state.update_data(notes=message.text)
    await state.set_state(AggiungiStates.waiting_confirm)
    data = await state.get_data()
    await message.answer(
        build_summary(data),
        parse_mode="Markdown",
        reply_markup=confirm_aggiungi_keyboard(),
    )


async def callback_aggiungi_note_skip(query: CallbackQuery, state: FSMContext):
    if await _expired(query, state): return
    await query.answer()
    await state.update_data(notes="")
    await state.set_state(AggiungiStates.waiting_confirm)
    data = await state.get_data()
    await query.message.edit_text(
        build_summary(data),
        parse_mode="Markdown",
        reply_markup=confirm_aggiungi_keyboard(),
    )


async def callback_aggiungi_ok(query: CallbackQuery, state: FSMContext):
    if await _expired(query, state): return
    await query.answer()
    data = await state.get_data()
    try:
        await add_transaction(
            amount=data["amount"],
            description=data["description"],
            tx_type=data.get("tx_type", "expense"),
            metodo=data.get("metodo", "carta"),
            category_id=data.get("category_id"),
            notes=data.get("notes", ""),
        )
        label = "Entrata" if data.get("tx_type", "expense") == "income" else "Spesa"
        await query.message.edit_text(
            f"*{label} salvata*: `{fmt_eur(data['amount'])}` — {data['description']}",
            parse_mode="Markdown",
        )
    except Exception as e:
        await query.message.edit_text(f"Errore salvataggio: {e}")
    await state.clear()


async def callback_aggiungi_no(query: CallbackQuery, state: FSMContext):
    await query.answer()
    await query.message.edit_text("Annullato.")
    await state.clear()


async def cmd_obiettivi(message: Message):
    try:
        goals = await get_goals()
        if not goals:
            await message.answer("Nessun fondo risparmio.")
            return
        lines = ["*Fondi risparmio*\n"]
        for g in goals:
            pct = min(100, round(g["saved"] / g["target"] * 100)) if g["target"] > 0 else 0
            bar_filled = round(pct / 10)
            bar = "█" * bar_filled + "░" * (10 - bar_filled)
            lines.append(f"*{g['name']}*\n`{bar}` {pct}%\n{fmt_eur(g['saved'])} / {fmt_eur(g['target'])}\n")
        await message.answer("\n".join(lines), parse_mode="Markdown")
    except Exception as e:
        await message.answer(f"Errore: {e}")


async def cmd_categorie(message: Message):
    try:
        cats = await get_categories_with_totals()
        if not cats:
            await message.answer("Nessuna categoria.")
            return
        month = f"{date.today().year}-{date.today().month:02d}"
        lines = [f"*Categorie — {month}*\n"]
        for c in cats:
            budget_str = f" / {fmt_eur(c['budget'])}" if c["budget"] else ""
            over = " ⚠️" if c["budget"] and c["spent"] > c["budget"] else ""
            if c["spent"] > 0 or c["budget"]:
                lines.append(f"• {c['name']}: `{fmt_eur(c['spent'])}{budget_str}`{over}")
        await message.answer("\n".join(lines) if len(lines) > 1 else "*Nessuna spesa questo mese.*",
                             parse_mode="Markdown")
    except Exception as e:
        await message.answer(f"Errore: {e}")


async def handle_photo(message: Message, bot: Bot, state: FSMContext):
    tess = await find_tesseract()
    if not tess:
        await message.answer("Tesseract non trovato.\nmacOS: `brew install tesseract tesseract-lang`",
                             parse_mode="Markdown")
        return

    photo = message.photo[-1]
    await message.answer("Analisi immagine in corso…")

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
        await message.answer("Timeout OCR. Riprova con immagine più nitida.")
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
            f"Importo non trovato.\n\nTesto estratto:\n```\n{raw[:300]}\n```\n\n"
            "Usa `/aggiungi <importo> <descrizione>` per inserire manualmente.",
            parse_mode="Markdown",
        )
        return

    preview = f"*{fmt_eur(amount)}*" + (f" — {merchant}" if merchant else "")
    await state.set_state(FotoStates.waiting_confirm)
    await state.update_data(amount=amount, description=merchant or "Scontrino")
    await message.answer(
        f"Trovato: {preview}\n\nSalvo?",
        parse_mode="Markdown",
        reply_markup=confirm_keyboard(amount, merchant),
    )


async def callback_foto(query: CallbackQuery, state: FSMContext):
    await query.answer()
    if query.data == "foto_ok":
        data = await state.get_data()
        try:
            await add_transaction(data["amount"], data["description"])
            await query.message.edit_text(
                f"Salvato: *{fmt_eur(data['amount'])}* — {data['description']}",
                parse_mode="Markdown",
            )
        except Exception as e:
            await query.message.edit_text(f"Errore salvataggio: {e}")
    else:
        await query.message.edit_text("Annullato.")
    await state.clear()


# ─── Main ────────────────────────────────────────────────────────────────────

async def main(token: str, db_path: str):
    global DB_PATH
    DB_PATH = db_path

    bot = Bot(token=token)
    dp = Dispatcher(storage=MemoryStorage())

    dp.message.register(cmd_start,      Command("start", "aiuta", "help"))
    dp.message.register(cmd_saldo,      Command("saldo"))
    dp.message.register(cmd_spese,      Command("spese"))
    dp.message.register(cmd_report,     Command("report"))
    dp.message.register(cmd_aggiungi,   Command("aggiungi"))
    dp.message.register(cmd_obiettivi,  Command("obiettivi"))
    dp.message.register(cmd_categorie,  Command("categorie"))

    dp.message.register(handle_aggiungi_note, AggiungiStates.waiting_note)

    dp.callback_query.register(callback_foto,               F.data.in_({"foto_ok", "foto_no"}))
    dp.callback_query.register(callback_aggiungi_cat,       F.data.startswith("aggiungi_cat_"))
    dp.callback_query.register(callback_aggiungi_metodo,    F.data.startswith("aggiungi_metodo_"))
    dp.callback_query.register(callback_aggiungi_note_skip, F.data == "aggiungi_note_skip")
    dp.callback_query.register(callback_aggiungi_ok,        F.data == "aggiungi_ok")
    dp.callback_query.register(callback_aggiungi_no,        F.data == "aggiungi_no")

    dp.message.register(lambda m, s: handle_photo(m, bot, s), F.photo)

    logger.info("Fehu bot avviato. DB: %s", db_path)
    await dp.start_polling(bot)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Fehu Telegram Bot")
    parser.add_argument("--token",   required=True)
    parser.add_argument("--db-path", required=True)
    args = parser.parse_args()
    asyncio.run(main(args.token, args.db_path))
