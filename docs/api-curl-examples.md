# API cURL examples

Base URL: `http://localhost:3001/api`  
Optional header: `X-User-Id: <24-char-hex-ObjectId>` (omit to use default user).

Replace `:id`, `:portfolio_id`, `:asset_id` with real ObjectId hex strings after creating resources.

---

## Health (no /api prefix)

```bash
curl -X GET http://localhost:3001/health
```

---

## Portfolios

### List portfolios
```bash
curl -X GET http://localhost:3001/api/portfolios \
  -H "Content-Type: application/json"
```

### Create portfolio
```bash
curl -X POST http://localhost:3001/api/portfolios \
  -H "Content-Type: application/json" \
  -d '{"name":"My Portfolio","type":"Investment","baseCurrency":"VND","description":null,"color":"#136dec"}'
```

### Get portfolio
```bash
curl -X GET http://localhost:3001/api/portfolios/:id \
  -H "Content-Type: application/json"
```

### Update portfolio
```bash
curl -X PUT http://localhost:3001/api/portfolios/:id \
  -H "Content-Type: application/json" \
  -d '{"name":"Updated Name","type":"Investment","baseCurrency":"VND","description":null,"color":null}'
```

### Delete portfolio
```bash
curl -X DELETE http://localhost:3001/api/portfolios/:id
```

---

## Assets

### List assets by portfolio
```bash
curl -X GET http://localhost:3001/api/portfolios/:portfolio_id/assets \
  -H "Content-Type: application/json"
```

### Create asset
```bash
curl -X POST http://localhost:3001/api/portfolios/:portfolio_id/assets \
  -H "Content-Type: application/json" \
  -d '{"portfolioId":"<portfolio_id>","name":"BTC","type":"Crypto","currency":"USD","status":"Active","metadata":{"symbol":"BTCUSD"}}'
```

### Get asset
```bash
curl -X GET http://localhost:3001/api/assets/:id \
  -H "Content-Type: application/json"
```

### Update asset status (PATCH)
```bash
curl -X PATCH http://localhost:3001/api/assets/:id \
  -H "Content-Type: application/json" \
  -d '{"status":"Closed"}'
```

---

## Transactions

### List transactions by asset
```bash
curl -X GET http://localhost:3001/api/assets/:asset_id/transactions \
  -H "Content-Type: application/json"
```

### Create transaction
```bash
curl -X POST http://localhost:3001/api/assets/:asset_id/transactions \
  -H "Content-Type: application/json" \
  -d '{"assetId":"<asset_id>","type":"Buy","amount":1000,"price":50000,"quantity":0.02,"note":null,"date":"2025-01-15T00:00:00Z"}'
```

### List all transactions
```bash
curl -X GET "http://localhost:3001/api/transactions?limit=50" \
  -H "Content-Type: application/json"
```

---

## Net worth snapshots

### Upsert net worth snapshot
```bash
curl -X POST http://localhost:3001/api/net-worth-snapshots \
  -H "Content-Type: application/json" \
  -d '{"date":"2025-01-15","totalNetWorth":100000000}'
```

### List net worth snapshots
```bash
curl -X GET "http://localhost:3001/api/net-worth-snapshots?time_range=1M" \
  -H "Content-Type: application/json"
```
(time_range: 1M | 3M | 6M | 1Y | ALL)

---

## Portfolio snapshots

### Upsert portfolio snapshot
```bash
curl -X POST http://localhost:3001/api/portfolio-snapshots \
  -H "Content-Type: application/json" \
  -d '{"date":"2025-01-15","portfolioId":"<portfolio_id>","totalValue":50000000}'
```

### List portfolio snapshots
```bash
curl -X GET "http://localhost:3001/api/portfolio-snapshots?time_range=1M&portfolio_id=<portfolio_id>" \
  -H "Content-Type: application/json"
```

---

## Coin price history (CoinGecko → DB)

### Fetch from CoinGecko and upsert into coin_price_history
Fetches 365 days of history from CoinGecko `market_chart`, one row per day, and upserts into `coin_price_history`. Optional header `X-User-Id` (or default user from env).

```bash
# Bitcoin, USD
curl -X POST "http://localhost:3001/api/coin-price-history/fetch?coinId=bitcoin&currency=usd" \
  -H "Content-Type: application/json"

# Ethereum, USD (optional: add X-User-Id if you use multiple users)
curl -X POST "http://localhost:3001/api/coin-price-history/fetch?coinId=ethereum&currency=usd" \
  -H "Content-Type: application/json" \
  -H "X-User-Id: YOUR_24_CHAR_USER_OBJECTID_HEX"
```

Query params: `coinId` (CoinGecko id: bitcoin, ethereum, …), `currency` (e.g. usd, vnd). Response: `{ "coinId": "bitcoin", "currency": "usd", "pointsUpserted": 365 }`.

### List coin price history
```bash
curl -X GET "http://localhost:3001/api/coin-price-history?coinId=bitcoin&currency=usd" \
  -H "Content-Type: application/json"
```

---

## Asset price history

### Upsert asset price
```bash
curl -X POST http://localhost:3001/api/asset-price-history \
  -H "Content-Type: application/json" \
  -d '{"assetId":"<asset_id>","date":"2025-01-15","price":50000,"currency":"USD"}'
```

### List asset price history
```bash
curl -X GET "http://localhost:3001/api/assets/:asset_id/price-history?time_range=1M" \
  -H "Content-Type: application/json"
```

---

## Transfer transactions

### List transfer transactions
```bash
curl -X GET "http://localhost:3001/api/transfer-transactions?from_portfolio_id=&to_portfolio_id=" \
  -H "Content-Type: application/json"
```

### Create transfer
```bash
curl -X POST http://localhost:3001/api/transfer-transactions \
  -H "Content-Type: application/json" \
  -d '{"fromPortfolioId":"<from_id>","toPortfolioId":"<to_id>","amount":1000000,"fromCurrency":"VND","toCurrency":"USD","date":"2025-01-15T00:00:00Z","note":null}'
```

---

## Exchange rates

### List exchange rates
```bash
curl -X GET "http://localhost:3001/api/exchange-rates?date=2025-01-15&from_currency=VND&to_currency=USD" \
  -H "Content-Type: application/json"
```

### Create exchange rate
```bash
curl -X POST http://localhost:3001/api/exchange-rates \
  -H "Content-Type: application/json" \
  -d '{"fromCurrency":"VND","toCurrency":"USD","rate":0.00004,"date":"2025-01-15T00:00:00Z","source":"manual"}'
```
