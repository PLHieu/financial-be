#!/usr/bin/env python3
"""
Compute portfolio_snapshots from transactions and asset_price_history.

For each portfolio and each day (from first transaction to last/today):
  - total_deposit: sum of Deposit transactions (converted to portfolio base currency) up to that day
  - inventory: JSON e.g. {"USD": 10, "BTC": 0.5, "ETH": 2.0} (balances per asset symbol)
  - total_value (net worth): sum(inventory[asset] * price_at_day) in base currency; USD = 1:1

Result is used by the app API and chart: each portfolio has 2 lines — total deposit and net worth.

Env:
  MONGODB_URI  – MongoDB connection (default: mongodb://localhost:27017)

Usage:
  pip install -r scripts/requirements.txt
  python scripts/build_portfolio_snapshots.py [--portfolio-id ID] [--days 365]
"""

import argparse
import os
import sys
from collections import defaultdict
from datetime import datetime, timezone, timedelta

try:
    from pymongo import MongoClient
    from pymongo.database import Database
    from bson import ObjectId
except ImportError:
    print("Install dependencies: pip install -r scripts/requirements.txt", file=sys.stderr)
    sys.exit(1)


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

MONGODB_URI = os.environ.get("MONGODB_URI", "mongodb://localhost:27017").strip()
DB_NAME = "financial"
COLL_TRANSACTIONS = "transactions"
COLL_ASSETS = "assets"
COLL_PORTFOLIOS = "portfolios"
COLL_ASSET_PRICE_HISTORY = "asset_price_history"
COLL_PORTFOLIO_SNAPSHOTS = "portfolio_snapshots"
COLL_EXCHANGE_RATES = "exchange_rates"


def get_db() -> Database:
    return MongoClient(MONGODB_URI)[DB_NAME]


def asset_balance_key(asset: dict) -> str:
    """Symbol for inventory key: BTCUSD -> BTC, USD -> USD."""
    sym = (asset.get("metadata") or {}).get("symbol") or asset.get("name") or ""
    sym = (sym or "").strip().upper()
    if not sym:
        return f"asset_{asset['_id']}"
    if sym == "USD":
        return "USD"
    if sym.endswith("USD") and len(sym) > 3:
        return sym[:-3]  # BTCUSD -> BTC
    return sym


def is_usd_asset(asset: dict) -> bool:
    """True only for the actual USD cash asset (symbol/name USD), not for crypto quoted in USD (e.g. BTCUSD)."""
    sym = ((asset.get("metadata") or {}).get("symbol") or "").strip().upper()
    name = (asset.get("name") or "").strip().upper()
    return sym == "USD" or name == "USD"


def as_utc(dt: datetime) -> datetime:
    """Ensure datetime is timezone-aware UTC (for comparison)."""
    if dt.tzinfo is None:
        return dt.replace(tzinfo=timezone.utc)
    return dt


def get_price_for_date(prices_by_date: list[tuple[datetime, float]], target: datetime) -> float | None:
    """Get price at or most recent before target. prices_by_date sorted by date asc."""
    if not prices_by_date:
        return None
    target = as_utc(target)
    out = None
    for d, p in prices_by_date:
        if as_utc(d) <= target:
            out = p
        else:
            break
    return out


def run(
    db: Database,
    portfolio_id: ObjectId | None,
    days: int,
) -> None:
    portfolios = list(
        db[COLL_PORTFOLIOS].find({} if portfolio_id is None else {"_id": portfolio_id})
    )
    if not portfolios:
        print("No portfolios found.", file=sys.stderr)
        return

    assets = {a["_id"]: a for a in db[COLL_ASSETS].find({})}
    # Load all asset price history: (asset_id, date) -> price (currency from doc, we assume USD or base)
    price_cursor = db[COLL_ASSET_PRICE_HISTORY].find({}).sort("date", 1)
    prices_by_asset: dict[ObjectId, list[tuple[datetime, float]]] = defaultdict(list)
    for doc in price_cursor:
        aid = doc["asset_id"]
        dt = doc["date"].as_datetime() if hasattr(doc["date"], "as_datetime") else doc["date"]
        prices_by_asset[aid].append((as_utc(dt), float(doc.get("price") or 0)))

    # USD/VND rates by date (for base_currency VND)
    rates_cursor = db[COLL_EXCHANGE_RATES].find(
        {"from_currency": "USD", "to_currency": "VND"}
    ).sort("date", 1)
    usd_to_vnd: list[tuple[datetime, float]] = []
    for doc in rates_cursor:
        dt = doc["date"].as_datetime() if hasattr(doc["date"], "as_datetime") else doc["date"]
        usd_to_vnd.append((as_utc(dt), float(doc.get("rate") or 0)))

    def usd_to_base(usd_value: float, base_currency: str, at_date: datetime) -> float:
        if (base_currency or "").upper() != "VND":
            return usd_value
        rate = get_price_for_date(usd_to_vnd, at_date) if usd_to_vnd else 25000.0
        return usd_value * (rate or 25000.0)

    now = datetime.now(timezone.utc)
    total_upserted = 0

    for portfolio in portfolios:
        pid = portfolio["_id"]
        user_id = portfolio["user_id"]
        base_currency = (portfolio.get("base_currency") or "USD").upper()
        if base_currency not in ("USD", "VND"):
            base_currency = "USD"

        # Transactions for this portfolio (with portfolio_id set)
        txs = list(
            db[COLL_TRANSACTIONS].find({"portfolio_id": pid}).sort("date", 1)
        )
        if not txs:
            print(f"  Portfolio {portfolio.get('name', pid)}: no transactions, skip.")
            continue

        # USD asset: for Buy we debit USD (amount spent), for Sell we credit USD (amount received)
        usd_asset_id = None
        for t in txs:
            a = assets.get(t["asset_id"])
            if a and is_usd_asset(a):
                usd_asset_id = t["asset_id"]
                break
        if usd_asset_id is None:
            for a in assets.values():
                if a.get("user_id") == user_id and is_usd_asset(a):
                    usd_asset_id = a["_id"]
                    break

        first_ts = txs[0]["date"]
        first_dt = as_utc(first_ts.as_datetime() if hasattr(first_ts, "as_datetime") else first_ts)
        start_date = first_dt.replace(hour=0, minute=0, second=0, microsecond=0)
        if start_date.tzinfo is None:
            start_date = start_date.replace(tzinfo=timezone.utc)
        # Always extend snapshots to today (so chart runs up to current date)
        today_end = datetime.now(timezone.utc).replace(hour=0, minute=0, second=0, microsecond=0) + timedelta(days=1)
        if today_end.tzinfo is None:
            today_end = today_end.replace(tzinfo=timezone.utc)
        end_date = today_end
        # Optional: limit to last `days`
        if days > 0:
            from_end = end_date - timedelta(days=days)
            if start_date < from_end:
                start_date = from_end

        current = start_date
        while current < end_date:
            end_of_day = current + timedelta(days=1)

            # Transactions up to end of this day (date < end_of_day)
            day_txs = [
                t
                for t in txs
                if as_utc(t["date"].as_datetime() if hasattr(t["date"], "as_datetime") else t["date"]) < end_of_day
            ]

            # Per-asset balance:
            # - Deposit/Withdraw: change in that asset by amount
            # - Buy(asset_id=A, amount=M, quantity=Q): A += Q, USD -= M (spend USD to get A)
            # - Sell(asset_id=A, amount=M, quantity=Q): A -= Q, USD += M (receive USD)
            balance_by_asset: dict[ObjectId, float] = defaultdict(float)
            total_deposit_base = 0.0

            for t in day_txs:
                aid = t["asset_id"]
                typ = (t.get("type") or "").upper()
                amount = float(t.get("amount") or 0)
                quantity = float(t.get("quantity") or 0)

                if typ == "DEPOSIT":
                    balance_by_asset[aid] += amount
                    # Convert deposit to base for total_deposit
                    asset = assets.get(aid)
                    if asset and is_usd_asset(asset):
                        total_deposit_base += usd_to_base(amount, base_currency, current)
                    else:
                        prices = prices_by_asset.get(aid, [])
                        price_usd = get_price_for_date(prices, current) if prices else None
                        if price_usd is not None:
                            total_deposit_base += usd_to_base(amount * price_usd, base_currency, current)
                        else:
                            total_deposit_base += usd_to_base(amount, base_currency, current)
                elif typ == "WITHDRAW":
                    balance_by_asset[aid] -= amount
                elif typ == "BUY":
                    balance_by_asset[aid] += quantity
                    if usd_asset_id is not None:
                        balance_by_asset[usd_asset_id] -= amount  # spent USD
                elif typ == "SELL":
                    balance_by_asset[aid] -= quantity
                    if usd_asset_id is not None:
                        balance_by_asset[usd_asset_id] += amount  # received USD

            # Inventory: asset_id -> symbol, then { symbol: balance } (include negative)
            inventory: dict[str, float] = {}
            for aid, bal in balance_by_asset.items():
                if bal == 0:
                    continue
                asset = assets.get(aid)
                if not asset:
                    continue
                key = asset_balance_key(asset)
                inventory[key] = round(bal, 8)

            # Net worth: sum(balance * price) in base currency (negative balance = negative value)
            net_worth_base = 0.0
            for aid, bal in balance_by_asset.items():
                if bal == 0:
                    continue
                asset = assets.get(aid)
                if not asset:
                    continue
                if is_usd_asset(asset):
                    net_worth_base += usd_to_base(bal, base_currency, current)
                else:
                    prices = prices_by_asset.get(aid, [])
                    price_usd = get_price_for_date(prices, current) if prices else None
                    if price_usd is not None:
                        net_worth_base += usd_to_base(bal * price_usd, base_currency, current)
                    # else skip or use 0

            # Snapshot date at start of day UTC (pymongo stores as BSON DateTime)
            date_utc = current.replace(hour=0, minute=0, second=0, microsecond=0)
            if date_utc.tzinfo is None:
                date_utc = date_utc.replace(tzinfo=timezone.utc)
            now_utc = datetime.now(timezone.utc)

            doc = {
                "user_id": user_id,
                "date": date_utc,
                "portfolio_id": pid,
                "total_deposit": round(total_deposit_base, 2),
                "inventory": inventory,
                "total_value": round(net_worth_base, 2),
                "created_at": now_utc,
            }
            db[COLL_PORTFOLIO_SNAPSHOTS].update_one(
                {"user_id": user_id, "date": date_utc, "portfolio_id": pid},
                {"$set": doc},
                upsert=True,
            )
            total_upserted += 1
            current += timedelta(days=1)

        print(f"  Portfolio {portfolio.get('name', pid)}: snapshots from {start_date.date()} to {(current - timedelta(days=1)).date()}.")

    print(f"Done. Upserted {total_upserted} portfolio snapshot(s).")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Build portfolio_snapshots from transactions and asset_price_history.",
    )
    parser.add_argument(
        "--portfolio-id",
        type=str,
        default=None,
        help="Build only for this portfolio ObjectId (hex). Default: all portfolios.",
    )
    parser.add_argument(
        "--days",
        type=int,
        default=0,
        help="Limit to last N days (0 = no limit). Default: 0.",
    )
    args = parser.parse_args()

    portfolio_id = None
    if args.portfolio_id:
        try:
            portfolio_id = ObjectId(args.portfolio_id)
        except Exception:
            print(f"Invalid --portfolio-id: {args.portfolio_id}", file=sys.stderr)
            return 1

    db = get_db()
    run(db, portfolio_id, args.days)
    return 0


if __name__ == "__main__":
    sys.exit(main())
