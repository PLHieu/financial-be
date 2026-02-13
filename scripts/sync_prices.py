#!/usr/bin/env python3
"""
1) Fetch prices from CoinGecko API and upsert into coin_price_history.
2) Fill asset_price_history for assets (from --coin-asset-map or from DB assets by symbol/coingeckoId).

Optional map for BTC/ETH (or any coin): you create assets in DB, then pass coin_id=asset_id.
Prices are fetched for each coin and written to both coin_price_history and asset_price_history for the given asset.

Env:
  MONGODB_URI         – MongoDB connection string (default: mongodb://localhost:27017)
  DEFAULT_USER_ID     – 24-char hex ObjectId (default: 000000000000000000000001)
  COINGECKO_API_BASE  – (optional)
  COINGECKO_API_KEY   – (optional)
  COIN_ASSET_MAP      – Optional. Format: coin_id=asset_oid_hex,coin_id2=asset_oid_hex (e.g. bitcoin=507f...,ethereum=507f...)

Usage:
  python scripts/sync_prices.py --coin-asset-map "bitcoin=<your_btc_asset_oid>,ethereum=<your_eth_asset_oid>"
  python scripts/sync_prices.py --coin-asset-map "bitcoin=698ea4a24e3c4ee180021c0b,ethereum=698ea4b54e3c4ee180021c0c" --days 90
  python scripts/sync_prices.py --portfolio-id <id> --days 90
  python scripts/sync_prices.py --coin-ids bitcoin,ethereum --currency usd
"""

import argparse
import os
import sys
from datetime import datetime, timezone

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

MONGODB_URI = os.environ.get("MONGODB_URI", "mongodb://localhost:27017").strip()
DEFAULT_USER_ID_HEX = os.environ.get("DEFAULT_USER_ID", "000000000000000000000001").strip()
COINGECKO_BASE = os.environ.get("COINGECKO_API_BASE", "https://pro-api.coingecko.com/api/v3").strip().rstrip("/")
COINGECKO_KEY = os.environ.get("COINGECKO_API_KEY", "").strip()

DB_NAME = "financial"
ASSETS_COLLECTION = "assets"
ASSET_PRICE_HISTORY_COLLECTION = "asset_price_history"
COIN_PRICE_HISTORY_COLLECTION = "coin_price_history"
PORTFOLIOS_COLLECTION = "portfolios"

# All prices fetched in USD only. No USDT/USDC in system.
ASSET_TO_COINGECKO: dict[str, str] = {
    "bitcoin": "bitcoin", "btc": "bitcoin",
    "ethereum": "ethereum", "eth": "ethereum",
    "binancecoin": "binancecoin", "bnb": "binancecoin",
    "solana": "solana", "sol": "solana",
    "ripple": "ripple", "xrp": "ripple",
    "dogecoin": "dogecoin", "doge": "dogecoin",
}

# -----------------------------------------------------------------------------
# CoinGecko API
# -----------------------------------------------------------------------------

def fetch_market_chart(coin_id: str, currency: str, days: int) -> list[tuple[datetime, float]]:
    path = f"{COINGECKO_BASE}/coins/{coin_id}/market_chart"
    params = {"vs_currency": currency, "days": str(days)}
    if COINGECKO_KEY:
        params["x_cg_pro_api_key"] = COINGECKO_KEY
    r = requests.get(path, params=params, headers={"User-Agent": "financial-be/1.0"}, timeout=60)
    r.raise_for_status()
    data = r.json()
    prices = data.get("prices") or []
    ms_per_day = 86400 * 1000
    by_day: dict[int, float] = {}
    for [ts_ms, price] in prices:
        day_ms = (int(ts_ms) // ms_per_day) * ms_per_day
        by_day[day_ms] = float(price)
    out = [(datetime.fromtimestamp(ms / 1000.0, tz=timezone.utc), p) for ms, p in sorted(by_day.items())]
    return out


# -----------------------------------------------------------------------------
# DB
# -----------------------------------------------------------------------------

def get_db() -> Database:
    return MongoClient(MONGODB_URI)[DB_NAME]


def get_portfolio(db: Database, portfolio_id: str) -> dict | None:
    try:
        return db[PORTFOLIOS_COLLECTION].find_one({"_id": ObjectId(portfolio_id)})
    except Exception:
        return None


def get_assets(db: Database, portfolio_id: ObjectId | None) -> list[dict]:
    if portfolio_id is None:
        return list(db[ASSETS_COLLECTION].find({}))
    return list(db[ASSETS_COLLECTION].find({"portfolio_id": portfolio_id}))


def asset_to_coingecko_id(asset: dict) -> str | None:
    meta = asset.get("metadata") or {}
    cg = meta.get("coingeckoId") or meta.get("coingecko_id")
    if isinstance(cg, str) and cg.strip():
        return cg.strip().lower()
    name = (asset.get("name") or "").strip().lower()
    symbol = (meta.get("symbol") or "").strip().lower() if isinstance(meta.get("symbol"), str) else ""
    for key in (symbol, name):
        if not key:
            continue
        if key in ASSET_TO_COINGECKO:
            return ASSET_TO_COINGECKO[key]
        if key.endswith("usd") and len(key) > 3:
            base = key[:-3]
            if base in ASSET_TO_COINGECKO:
                return ASSET_TO_COINGECKO[base]
    return None


def upsert_coin_price_history(
    db: Database, user_id: ObjectId, coin_id: str, currency: str, points: list[tuple[datetime, float]]
) -> int:
    coll = db[COIN_PRICE_HISTORY_COLLECTION]
    now = datetime.now(timezone.utc)
    n = 0
    for date, price in points:
        doc = {
            "user_id": user_id,
            "coin_id": coin_id,
            "date": date,
            "price": price,
            "currency": currency,
            "created_at": now,
        }
        coll.update_one(
            {"user_id": user_id, "coin_id": coin_id, "date": date, "currency": currency},
            {"$set": doc},
            upsert=True,
        )
        n += 1
    return n


def get_coin_price_history(
    db: Database, user_id: ObjectId, coin_id: str, currency: str | None
) -> list[dict]:
    q: dict = {"user_id": user_id, "coin_id": coin_id}
    if currency:
        q["currency"] = currency
    return list(db[COIN_PRICE_HISTORY_COLLECTION].find(q).sort("date", 1))


def upsert_asset_price(
    db: Database, asset_id: ObjectId, date: datetime, price: float, currency: str
) -> None:
    now = datetime.now(timezone.utc)
    doc = {"asset_id": asset_id, "date": date, "price": price, "currency": currency, "created_at": now}
    db[ASSET_PRICE_HISTORY_COLLECTION].update_one(
        {"asset_id": asset_id, "date": date},
        {"$set": doc},
        upsert=True,
    )


# -----------------------------------------------------------------------------
# Main
# -----------------------------------------------------------------------------

def parse_coin_asset_map(s: str) -> list[tuple[str, ObjectId]]:
    """Parse COIN_ASSET_MAP or --coin-asset-map: 'bitcoin=oid_hex,ethereum=oid_hex'. Returns [(coin_id, asset_id), ...]."""
    out: list[tuple[str, ObjectId]] = []
    for part in (s or "").strip().split(","):
        part = part.strip()
        if not part or "=" not in part:
            continue
        coin_id, oid_hex = part.split("=", 1)
        coin_id = coin_id.strip().lower()
        oid_hex = oid_hex.strip()
        if not coin_id or not oid_hex:
            continue
        try:
            out.append((coin_id, ObjectId(oid_hex)))
        except Exception:
            continue
    return out


def main() -> int:
    parser = argparse.ArgumentParser(description="Fetch CoinGecko prices → coin_price_history → asset_price_history.")
    parser.add_argument("--portfolio-id", help="Only consider assets in this portfolio (optional).")
    parser.add_argument("--coin-ids", help="Comma-separated CoinGecko ids (e.g. bitcoin,ethereum). If omitted, derived from assets or from --coin-asset-map.")
    parser.add_argument(
        "--coin-asset-map",
        help="Map coin_id to asset ObjectId: 'bitcoin=oid_hex,ethereum=oid_hex'. Use after creating BTC/ETH assets in DB. Env: COIN_ASSET_MAP.",
    )
    parser.add_argument("--currency", default="usd", help="Currency for prices (default: usd).")
    parser.add_argument("--days", type=int, default=365, help="Days of history to fetch (default: 365).")
    args = parser.parse_args()

    try:
        user_id = ObjectId(DEFAULT_USER_ID_HEX)
    except Exception:
        print("Invalid DEFAULT_USER_ID (must be 24-char hex)", file=sys.stderr)
        return 1

    db = get_db()
    portfolio_id: ObjectId | None = None
    if args.portfolio_id:
        p = get_portfolio(db, args.portfolio_id)
        if not p:
            print(f"Portfolio not found: {args.portfolio_id}", file=sys.stderr)
            return 1
        portfolio_id = p["_id"]

    # Optional: explicit map coin_id -> asset_id (e.g. bitcoin=oid,ethereum=oid)
    coin_asset_map_str = (args.coin_asset_map or os.environ.get("COIN_ASSET_MAP") or "").strip()
    coin_asset_map = parse_coin_asset_map(coin_asset_map_str) if coin_asset_map_str else []

    # Step 1: Determine coin_ids and (coin_id -> asset_id) for map mode
    coin_ids: list[str] = []
    if coin_asset_map:
        coin_ids = [cid for cid, _ in coin_asset_map]
        print(f"Using coin-asset map: {len(coin_asset_map)} entries (e.g. {coin_ids[:3]})")
    elif args.coin_ids:
        coin_ids = [s.strip().lower() for s in args.coin_ids.split(",") if s.strip()]
    else:
        assets = get_assets(db, portfolio_id)
        seen: set[str] = set()
        for a in assets:
            cid = asset_to_coingecko_id(a)
            if cid and cid not in seen:
                seen.add(cid)
                coin_ids.append(cid)
    if not coin_ids:
        print("No coin IDs. Set --coin-asset-map 'bitcoin=<asset_oid>,ethereum=<asset_oid>' or --coin-ids or add assets with symbol/coingeckoId.")
        return 0

    currency = args.currency.lower()

    # Step 2: Fetch from CoinGecko and upsert coin_price_history
    total_points = 0
    for coin_id in coin_ids:
        try:
            points = fetch_market_chart(coin_id, currency, args.days)
            n = upsert_coin_price_history(db, user_id, coin_id, currency, points)
            total_points += n
            print(f"  {coin_id}: {n} points → coin_price_history")
        except Exception as e:
            print(f"  {coin_id}: error – {e}", file=sys.stderr)

    print(f"Coin price history: {total_points} points upserted.")

    # Step 3: Fill asset_price_history
    updated = 0
    if coin_asset_map:
        # Map mode: for each (coin_id, asset_id) copy coin_price_history → asset_price_history
        for coin_id, asset_id in coin_asset_map:
            rows = get_coin_price_history(db, user_id, coin_id, currency)
            for row in rows:
                d = row.get("date")
                if d is None:
                    continue
                price = row.get("price")
                if price is None:
                    continue
                cur = (row.get("currency") or "usd").lower()
                upsert_asset_price(db, asset_id, d, float(price), cur)
                updated += 1
            if rows:
                print(f"  {coin_id} → asset {asset_id}: {len(rows)} dates → asset_price_history")
    else:
        # Discover assets from DB (by portfolio or all)
        assets = get_assets(db, portfolio_id)
        asset_coin: list[tuple[dict, str]] = []
        for a in assets:
            cid = asset_to_coingecko_id(a)
            if cid:
                asset_coin.append((a, cid))
        for asset, coin_id in asset_coin:
            rows = get_coin_price_history(db, user_id, coin_id, currency)
            for row in rows:
                d = row.get("date")
                if d is None:
                    continue
                price = row.get("price")
                if price is None:
                    continue
                cur = (row.get("currency") or "usd").lower()
                upsert_asset_price(db, asset["_id"], d, float(price), cur)
                updated += 1
            if rows:
                print(f"  {asset.get('name', '?')} ({coin_id}): {len(rows)} dates → asset_price_history")
    print(f"Asset price history: {updated} row(s) upserted.")
    print("Done.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
