# financial-be

Backend for the Financial app: **Rust + MongoDB**, multi-tenant ready. All data is scoped by `user_id`; auth is not implemented yet (use optional `X-User-Id` header or the default user).

## Prerequisites

- **Rust** (stable): [rustup](https://rustup.rs/)
- **MongoDB** (local or remote): the app uses database name `financial`

## How to run

1. **Copy env and set MongoDB**

   ```bash
   cp .env.example .env
   ```

   Edit `.env` and set `MONGODB_URI` if MongoDB is not on `localhost:27017`.

2. **Start the server**

   ```bash
   cargo run
   ```

   The API listens on **http://0.0.0.0:3001** (override with `PORT` in `.env`).

3. **Check health**

   ```bash
   curl http://localhost:3001/health
   ```

   Response: `OK`

## API base and auth

- **Base URL:** `http://localhost:3001/api`
- **User scope:** Send header `X-User-Id` with a 24-char hex ObjectId to act as that user. If omitted, the default user from `DEFAULT_USER_ID` is used.
- **IDs:** All resource IDs are string (ObjectId hex). Request/response bodies use **camelCase**.

## Main endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/portfolios` | List portfolios |
| POST | `/api/portfolios` | Create portfolio |
| GET | `/api/portfolios/:id` | Get portfolio |
| PUT | `/api/portfolios/:id` | Update portfolio |
| DELETE | `/api/portfolios/:id` | Delete portfolio |
| GET | `/api/portfolios/:portfolio_id/assets` | List assets |
| POST | `/api/portfolios/:portfolio_id/assets` | Create asset |
| GET | `/api/assets/:id` | Get asset |
| PATCH | `/api/assets/:id` | Update asset (e.g. status) |
| GET | `/api/assets/:asset_id/transactions` | List transactions |
| POST | `/api/assets/:asset_id/transactions` | Create transaction |
| GET | `/api/transactions` | List all transactions (`?limit=` optional) |
| POST | `/api/net-worth-snapshots` | Upsert net worth snapshot |
| GET | `/api/net-worth-snapshots?time_range=1M\|3M\|6M\|1Y\|ALL` | List net worth snapshots |
| POST | `/api/portfolio-snapshots` | Upsert portfolio snapshot |
| GET | `/api/portfolio-snapshots?time_range=&portfolio_id=` | List portfolio snapshots |
| POST | `/api/asset-price-history` | Upsert asset price |
| GET | `/api/assets/:asset_id/price-history?time_range=` | List asset price history |
| GET | `/api/transfer-transactions` | List transfer transactions |
| POST | `/api/transfer-transactions` | Create transfer (base currency only) |
| GET | `/api/exchange-rates` | List exchange rates |
| POST | `/api/exchange-rates` | Create exchange rate |

Indexes are created automatically on first run.

## React Native app

The React Native app is not updated in this repo; you can point it at this backend later (env var for API URL, use string IDs, and call these endpoints).
