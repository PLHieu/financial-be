#!/usr/bin/env python3
"""
1) Fetch trades from Binance API → upsert into binance_transactions (raw). Fetch deposits from API (not stored in binance_transactions).
2) Import into the app's transactions collection: trades from binance_transactions, deposits directly from API (inserted into transactions only).

All quote amounts from Binance (USDT/USDC) are treated as USD (1:1) and stored in the app as USD.
App uses only VND and USD; no USDT/USDC. Binance API pair names (e.g. BTCUSDT) are used only for the API call; internally we map to assets with symbol BTCUSD.

You must pass --portfolio-id. Assets need metadata.symbol (e.g. BTCUSD). Binance pairs (BTCUSDT) and deposit coins (BTC) map to that asset.

Env:
  BINANCE_API_KEY      – Binance API key (required)
  BINANCE_API_SECRET   – Binance API secret
  MONGODB_URI          – MongoDB connection string (default: mongodb://localhost:27017)
  BINANCE_SYMBOLS      – Comma-separated Binance pair symbols (default: BTCUSDT,ETHUSDT; exchange uses USDT pairs, we store as USD)
  BINANCE_DEPOSITS     – Set to 1 to fetch deposits (default: 1)

Usage:
  pip install -r scripts/requirements.txt
  export BINANCE_API_KEY=... BINANCE_API_SECRET=...
  python scripts/sync_binance.py --portfolio-id <portfolio_id>
"""

import argparse
import hashlib
import hmac
import os
import sys
import time
from datetime import datetime, timezone
from typing import Any
from urllib.parse import urlencode

try:
    import requests
except ImportError:
    print("Install dependencies: pip install -r scripts/requirements.txt", file=sys.stderr)
    sys.exit(1)

try:
    from pymongo import MongoClient
    from pymongo.database import Database
    from bson import ObjectId
except ImportError:
    print("Install dependencies: pip install -r scripts/requirements.txt", file=sys.stderr)
    sys.exit(1)

# -----------------------------------------------------------------------------
# Config
# -----------------------------------------------------------------------------

def load_env() -> None:
    script_dir = os.path.dirname(os.path.abspath(__file__))
    env_path = os.path.join(script_dir, "..", ".env")
    if os.path.isfile(env_path):
        with open(env_path) as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith("#") and "=" in line:
                    k, v = line.split("=", 1)
                    os.environ.setdefault(k.strip(), v.strip())

load_env()

BINANCE_API_KEY = (os.environ.get("BINANCE_API_KEY") or "").strip()
BINANCE_API_SECRET = (os.environ.get("BINANCE_API_SECRET") or "").strip()
MONGODB_URI = os.environ.get("MONGODB_URI", "mongodb://localhost:27017").strip()
BINANCE_SYMBOLS_STR = os.environ.get("BINANCE_SYMBOLS", "BTCUSDT,ETHUSDT").strip()
BINANCE_DEPOSITS = os.environ.get("BINANCE_DEPOSITS", "1").strip().lower() in ("1", "true", "yes")

DB_NAME = "financial"
BINANCE_COLLECTION = "binance_transactions"
TRANSACTIONS_COLLECTION = "transactions"
PORTFOLIOS_COLLECTION = "portfolios"
ASSETS_COLLECTION = "assets"
BINANCE_SPOT_BASE = "https://api.binance.com"
BINANCE_SAPI_BASE = "https://api.binance.com"

# -----------------------------------------------------------------------------
# Binance API
# -----------------------------------------------------------------------------

def sign(params: dict, secret: str) -> str:
    return hmac.new(secret.encode("utf-8"), urlencode(params).encode("utf-8"), hashlib.sha256).hexdigest()


def signed_get(base: str, path: str, params: dict) -> list | dict:
    params["timestamp"] = int(time.time() * 1000)
    params["signature"] = sign(params, BINANCE_API_SECRET)
    url = f"{base.rstrip('/')}{path}?{urlencode(params)}"
    r = requests.get(url, headers={"X-MBX-APIKEY": BINANCE_API_KEY}, timeout=30)
    r.raise_for_status()
    return r.json()


def fetch_spot_trades(symbol: str) -> list[dict]:
    path = "/api/v3/myTrades"
    all_trades: list[dict] = []
    from_id: int | None = None
    while True:
        params: dict[str, Any] = {"symbol": symbol, "limit": 1000}
        if from_id is not None:
            params["fromId"] = from_id
        data = signed_get(BINANCE_SPOT_BASE, path, params)
        if not data:
            break
        all_trades.extend(data)
        if len(data) < 1000:
            break
        from_id = data[-1]["id"] + 1
    return all_trades


def fetch_deposit_history(limit: int = 1000) -> list[dict]:
    path = "/sapi/v1/capital/deposit/hisrec"
    data = signed_get(BINANCE_SAPI_BASE, path, {"limit": limit})
    return data if isinstance(data, list) else []


# -----------------------------------------------------------------------------
# MongoDB – binance_transactions
# -----------------------------------------------------------------------------

def get_db() -> Database:
    return MongoClient(MONGODB_URI)[DB_NAME]


def upsert_trades(coll, trades: list[dict], symbol: str) -> int:
    n = 0
    for t in trades:
        doc = {**t, "source": "binance", "kind": "trade", "symbol": symbol}
        coll.update_one({"symbol": symbol, "id": t["id"]}, {"$set": doc}, upsert=True)
        n += 1
    return n


def upsert_deposits(coll, deposits: list[dict]) -> int:
    n = 0
    for d in deposits:
        doc = {**d, "source": "binance", "kind": "deposit"}
        filt = {"kind": "deposit", "coin": d.get("coin"), "txId": d.get("txId"), "insertTime": d.get("insertTime")}
        coll.update_one(filt, {"$set": doc}, upsert=True)
        n += 1
    return n


# -----------------------------------------------------------------------------
# Import into transactions (portfolio)
# -----------------------------------------------------------------------------

def get_portfolio(db: Database, portfolio_id: str) -> dict | None:
    try:
        return db[PORTFOLIOS_COLLECTION].find_one({"_id": ObjectId(portfolio_id)})
    except Exception:
        return None


def get_assets_for_import(db: Database, user_id: ObjectId, portfolio_id: ObjectId) -> list[dict]:
    """Assets usable for this portfolio: assets in this portfolio + global assets (no portfolio_id)."""
    return list(
        db[ASSETS_COLLECTION].find(
            {
                "user_id": user_id,
                "$or": [
                    {"portfolio_id": portfolio_id},
                    {"portfolio_id": None},
                    {"portfolio_id": {"$exists": False}},
                ],
            }
        )
    )


def build_symbol_to_asset(assets: list[dict]) -> dict[str, ObjectId]:
    out: dict[str, ObjectId] = {}
    for a in assets:
        sym = (a.get("metadata") or {}).get("symbol")
        if sym and isinstance(sym, str):
            s = sym.strip().upper()
            out[s] = a["_id"]
            if s.endswith("USD") and len(s) > 3:
                out[s[:-3] + "USDT"] = a["_id"]
    return out


# Binance deposit "coin" values: always treat as USD (no separate USDT/USDC in app).
STABLECOIN_ALIASES = ("USDT", "USDC", "BUSD", "DAI")


def get_or_create_usd_asset(db: Database, user_id: ObjectId) -> ObjectId:
    """Ensure a global USD asset exists for this user; return its _id."""
    now = datetime.now(timezone.utc)
    doc = db[ASSETS_COLLECTION].find_one(
        {"user_id": user_id, "metadata.symbol": "USD"},
        projection={"_id": 1},
    )
    if doc:
        return doc["_id"]
    res = db[ASSETS_COLLECTION].insert_one(
        {
            "user_id": user_id,
            "name": "USD",
            "type": "Manual Asset",
            "currency": "USD",
            "status": "Active",
            "metadata": {"symbol": "USD"},
            "created_at": now,
            "updated_at": now,
        }
    )
    return res.inserted_id


def build_coin_to_asset(
    assets: list[dict],
    usd_asset_id_override: ObjectId | None = None,
) -> dict[str, ObjectId]:
    out: dict[str, ObjectId] = {}
    symbol_to_asset = build_symbol_to_asset(assets)
    usd_asset_id = usd_asset_id_override or symbol_to_asset.get("USD")
    for sym, aid in symbol_to_asset.items():
        if sym.endswith("USD") and len(sym) > 3:
            base = sym[:-3]
            if base and base not in out:
                out[base] = aid
        if sym.endswith("USDT") and len(sym) > 4:
            base = sym[:-4]
            if base and base not in out:
                out[base] = aid
        if sym not in out:
            out[sym] = aid
    # USDT/USDC/etc. are always treated as USD: use existing USD asset or override.
    if usd_asset_id:
        for alias in STABLECOIN_ALIASES:
            if alias not in out:
                out[alias] = usd_asset_id
    return out


def ms_to_datetime(ms: int | None) -> datetime:
    if ms is None:
        return datetime.now(timezone.utc)
    return datetime.fromtimestamp(ms / 1000.0, tz=timezone.utc)


def trade_to_transaction(
    trade: dict,
    user_id: ObjectId,
    asset_id: ObjectId,
    portfolio_id: ObjectId,
    now: datetime,
) -> dict:
    is_buy = trade.get("isBuyer", True)
    if "side" in trade:
        is_buy = str(trade["side"]).upper() == "BUY"
    tx_type = "BUY" if is_buy else "SELL"
    qty = float(trade.get("qty", 0) or 0)
    price = float(trade.get("price", 0) or 0)
    quote_qty = float(trade.get("quoteQty", 0) or 0)
    if quote_qty == 0 and qty and price:
        quote_qty = qty * price
    return {
        "user_id": user_id,
        "portfolio_id": portfolio_id,
        "asset_id": asset_id,
        "type": tx_type,
        "amount": quote_qty,
        "price": price,
        "quantity": qty,
        "note": f"[Binance trade {trade.get('id', '')}]",
        "date": ms_to_datetime(trade.get("time")),
        "created_at": now,
        "updated_at": now,
    }


def deposit_to_transaction(
    deposit: dict,
    user_id: ObjectId,
    asset_id: ObjectId,
    portfolio_id: ObjectId,
    now: datetime,
) -> dict:
    amount = float(deposit.get("amount", 0) or 0)
    return {
        "user_id": user_id,
        "portfolio_id": portfolio_id,
        "asset_id": asset_id,
        "type": "DEPOSIT",
        "amount": amount,
        "price": None,
        "quantity": amount,
        "note": f"[Binance deposit {deposit.get('txId', '')}]",
        "date": ms_to_datetime(deposit.get("insertTime")),
        "created_at": now,
        "updated_at": now,
    }


def existing_transaction_with_note(db: Database, user_id: ObjectId, asset_id: ObjectId, note: str) -> bool:
    return db[TRANSACTIONS_COLLECTION].find_one({"user_id": user_id, "asset_id": asset_id, "note": note}) is not None


# -----------------------------------------------------------------------------
# Main
# -----------------------------------------------------------------------------

def main() -> int:
    parser = argparse.ArgumentParser(
        description="Fetch Binance trades + deposits, then insert into app transactions collection.",
    )
    parser.add_argument(
        "--portfolio-id",
        required=True,
        help="Portfolio ObjectId. Trades and deposits are inserted into the transactions collection for this portfolio.",
    )
    args = parser.parse_args()

    if not BINANCE_API_KEY or not BINANCE_API_SECRET:
        print("Set BINANCE_API_KEY and BINANCE_API_SECRET (e.g. in .env)", file=sys.stderr)
        return 1

    symbols = [s.strip() for s in BINANCE_SYMBOLS_STR.split(",") if s.strip()]
    if not symbols:
        print("Set BINANCE_SYMBOLS (e.g. BTCUSDT,ETHUSDT)", file=sys.stderr)
        return 1

    db = get_db()
    coll = db[BINANCE_COLLECTION]

    # Step 1: Fetch from Binance API and upsert into binance_transactions (raw)
    total_trades = 0
    for symbol in symbols:
        try:
            trades = fetch_spot_trades(symbol)
            n = upsert_trades(coll, trades, symbol)
            total_trades += n
            print(f"  {symbol}: {n} trades")
        except Exception as e:
            print(f"  {symbol}: error – {e}", file=sys.stderr)
    print(f"Trades: {total_trades} upserted into {BINANCE_COLLECTION}")

    # Fetch deposits from API only; we insert them into transactions collection (not binance_transactions)
    deposits_from_api: list[dict] = []
    if BINANCE_DEPOSITS:
        try:
            deposits_from_api = fetch_deposit_history()
            print(f"Deposits: {len(deposits_from_api)} fetched from Binance API (will insert into transactions only)")
        except Exception as e:
            print(f"Deposits fetch error: {e}", file=sys.stderr)

    # Step 2: Import into app transactions collection (trades from binance_transactions, deposits from API)
    portfolio = get_portfolio(db, args.portfolio_id)
    if not portfolio:
        print(f"Portfolio not found: {args.portfolio_id}", file=sys.stderr)
        return 1

    user_id = portfolio["user_id"]
    portfolio_oid = portfolio["_id"]
    assets = get_assets_for_import(db, user_id, portfolio_oid)
    symbol_to_asset = build_symbol_to_asset(assets)
    # USDT/USDC deposits need a USD asset: use existing or create one globally for this user.
    usd_asset_id = symbol_to_asset.get("USD")
    if not usd_asset_id and any(
        (d.get("coin") or "").strip().upper() in STABLECOIN_ALIASES
        for d in deposits_from_api
    ):
        usd_asset_id = get_or_create_usd_asset(db, user_id)
    coin_to_asset = build_coin_to_asset(assets, usd_asset_id_override=usd_asset_id)
    now = datetime.now(timezone.utc)

    if not symbol_to_asset and not usd_asset_id:
        print("No assets found for this portfolio (include global assets with metadata.symbol e.g. BTCUSD).", file=sys.stderr)
    else:
        print(f"Assets for import: {list(symbol_to_asset.keys())}")

    trades = list(coll.find({"source": "binance", "kind": "trade", "symbol": {"$in": list(symbol_to_asset)}}).sort("time", 1))
    deposits = sorted(deposits_from_api, key=lambda d: d.get("insertTime") or 0)

    inserted_trades = 0
    for t in trades:
        sym = (t.get("symbol") or "").strip().upper()
        asset_id = symbol_to_asset.get(sym)
        if not asset_id:
            continue
        note = f"[Binance trade {t.get('id', '')}]"
        if existing_transaction_with_note(db, user_id, asset_id, note):
            continue
        db[TRANSACTIONS_COLLECTION].insert_one(
            trade_to_transaction(t, user_id, asset_id, portfolio_oid, now)
        )
        inserted_trades += 1

    inserted_deposits = 0
    deposit_coins_seen: dict[str, int] = {}
    for d in deposits:
        coin = (d.get("coin") or "").strip().upper()
        if not coin:
            coin = "(empty)"
        deposit_coins_seen[coin] = deposit_coins_seen.get(coin, 0) + 1
        asset_id = coin_to_asset.get(coin)
        if not asset_id:
            continue
        note = f"[Binance deposit {d.get('txId', '')}]"
        if existing_transaction_with_note(db, user_id, asset_id, note):
            continue
        db[TRANSACTIONS_COLLECTION].insert_one(
            deposit_to_transaction(d, user_id, asset_id, portfolio_oid, now)
        )
        inserted_deposits += 1

    print(f"Portfolio: {portfolio.get('name', args.portfolio_id)}")
    print(f"transactions collection: {inserted_trades} trades, {inserted_deposits} deposits inserted.")
    if deposits:
        print(f"Deposit coins from API: {dict(deposit_coins_seen)}. Mapped coins for assets: {list(coin_to_asset.keys())}.")
    if inserted_deposits == 0 and deposits:
        print("Tip: Deposits are only inserted when coin matches an asset (e.g. BTC->BTCUSD, ETH->ETHUSD). USDT/USDC are auto-mapped to USD (a USD asset is created if missing).")
    print("Done.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
