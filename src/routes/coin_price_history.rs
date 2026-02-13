//! Fetch historical price from CoinGecko and upsert into MongoDB.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use mongodb::bson::DateTime as BsonDateTime;
use serde::Deserialize;
use url::Url;

use crate::config;
use crate::context::UserContext;
use crate::convert;
use crate::repository;
use crate::routes::dto;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchQuery {
    /// CoinGecko coin id (e.g. "bitcoin", "ethereum")
    pub coin_id: String,
    /// Target currency (e.g. "usd", "vnd"). Default "usd"
    #[serde(default = "default_currency")]
    pub currency: String,
}

fn default_currency() -> String {
    "usd".to_string()
}

/// Response for fetch-and-upsert: number of records upserted.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchResponse {
    pub coin_id: String,
    pub currency: String,
    pub points_upserted: u64,
}

/// CoinGecko market_chart response (only the fields we use).
#[derive(Debug, serde::Deserialize)]
struct CoinGeckoMarketChart {
    prices: Vec<[f64; 2]>,
}

/// POST /coin-price-history/fetch?coin_id=bitcoin&currency=usd
/// Fetches full history from CoinGecko and upserts into MongoDB (one row per day, last price of day).
pub async fn fetch_and_upsert(
    State(state): State<AppState>,
    ctx: UserContext,
    Query(q): Query<FetchQuery>,
) -> Result<Json<FetchResponse>, StatusCode> {
    let coin_id = q.coin_id.trim().to_lowercase();
    if coin_id.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let mut currency = q.currency.trim().to_lowercase();
    if currency.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    // Normalize stablecoin to USD: app uses only VND and USD
    if currency == "usdt" || currency == "usdc" || currency == "dai" || currency == "busd" {
        currency = "usd".to_string();
    }

    let base_url = config::coingecko_api_base();
    let base = base_url.trim_end_matches('/');
    let path = format!("{}/coins/{}/market_chart", base, coin_id);
    let mut url = Url::parse_with_params(
        &path,
        &[
            ("vs_currency", currency.as_str()),
            ("days", "365"),
        ],
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if let Some(key) = config::coingecko_api_key() {
        url.query_pairs_mut().append_pair("x_cg_demo_api_key", &key);
    }
    let url = url.to_string();

    let client = reqwest::Client::builder()
        .build()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let res = client
        .get(&url)
        .header("User-Agent", "financial-be/1.0 (personal finance app)")
        .send()
        .await
        .map_err(|e| {
            tracing::warn!("CoinGecko request failed: {}", e);
            StatusCode::BAD_GATEWAY
        })?;
    if !res.status().is_success() {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        tracing::warn!("CoinGecko error {}: {}", status, body);
        return Err(StatusCode::BAD_GATEWAY);
    }
    let body: CoinGeckoMarketChart = res.json().await.map_err(|e| {
        tracing::warn!("CoinGecko parse error: {}", e);
        StatusCode::BAD_GATEWAY
    })?;

    // Group by day (UTC): day_ms = (ts_ms / 86400000) * 86400000, keep last price per day
    const MS_PER_DAY: i64 = 86400 * 1000;
    let mut by_day: std::collections::BTreeMap<i64, f64> = std::collections::BTreeMap::new();
    for [ts_ms, price] in body.prices {
        let ts_ms = ts_ms as i64;
        let day_ms = (ts_ms / MS_PER_DAY) * MS_PER_DAY;
        by_day.insert(day_ms, price);
    }
    let points: Vec<(BsonDateTime, f64)> = by_day
        .into_iter()
        .map(|(ms, price)| (BsonDateTime::from_millis(ms), price))
        .collect();

    let now = convert::now_bson();
    let upserted = repository::coin_price_history::upsert_many(
        &state.db,
        ctx.user_id,
        &coin_id,
        &currency,
        &points,
        now,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(FetchResponse {
        coin_id,
        currency,
        points_upserted: upserted,
    }))
}

/// GET /coin-price-history?coin_id=bitcoin&currency=usd - list stored history for a coin (optional time range later).
pub async fn list(
    State(state): State<AppState>,
    ctx: UserContext,
    Query(q): Query<FetchQuery>,
) -> Result<Json<Vec<crate::models::CoinPriceHistoryDto>>, StatusCode> {
    let coin_id = q.coin_id.trim();
    if coin_id.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let currency: Option<String> = if q.currency.is_empty() {
        None
    } else {
        let c = q.currency.trim().to_lowercase();
        let normalized = if c == "usdt" || c == "usdc" || c == "dai" || c == "busd" {
            "usd"
        } else {
            c.as_str()
        };
        Some(normalized.to_string())
    };
    let list = repository::coin_price_history::get_by_coin(
        &state.db,
        ctx.user_id,
        coin_id,
        currency.as_deref(),
        None,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let dtos: Vec<_> = list.iter().map(dto::coin_price_history_to_dto).collect();
    Ok(Json(dtos))
}
