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

## Run with Docker

From the `financial-be` directory:

```bash
docker compose up --build
```

(or `docker-compose up --build` if you use the older CLI). This starts MongoDB and the API. Then open:

- **Health:** http://localhost:3001/health  
- **API:** http://localhost:3001/api/...

Optional: copy `.env.example` to `.env` and set variables; the API service will load them via `env_file: .env`.

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
| POST | `/api/coin-price-history/fetch?coin_id=&currency=` | Fetch CoinGecko history for a coin and upsert into MongoDB |
| GET | `/api/coin-price-history?coin_id=&currency=` | List stored coin price history for a coin |
| GET | `/api/transfer-transactions` | List transfer transactions |
| POST | `/api/transfer-transactions` | Create transfer (base currency only) |
| GET | `/api/exchange-rates` | List exchange rates |
| POST | `/api/exchange-rates` | Create exchange rate |

Indexes are created automatically on first run.

## Daily price cron (23:59:59 UTC)

If **CRON_COIN_IDS** is set, a background task runs **once per day at 23:59:59 UTC**. It fetches today’s price for each listed CoinGecko coin and upserts into `coin_price_history` for the default user.

- **CRON_COIN_IDS** – Comma-separated CoinGecko coin IDs (e.g. `bitcoin,ethereum,tether`). If unset or empty, the cron is not started.
- **CRON_CURRENCY** – Target currency for prices (default `usd`).
- **COINGECKO_API_BASE** / **COINGECKO_API_KEY** – Same as for the `/api/coin-price-history/fetch` endpoint.

## API examples (Postman / cURL)

- **Postman:** Import [docs/financial-be-postman-collection.json](docs/financial-be-postman-collection.json) in Postman (Import → Upload Files). Set collection variables `baseUrl` (default `http://localhost:3001`), and optionally `portfolioId` / `assetId` after creating resources.
- **cURL:** See [docs/api-curl-examples.md](docs/api-curl-examples.md) for copy-paste curl commands for every endpoint. Optional header: `X-User-Id: <24-char-hex-ObjectId>`.

## React Native app

The React Native app is not updated in this repo; you can point it at this backend later (env var for API URL, use string IDs, and call these endpoints).
