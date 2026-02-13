//! Daily cron: at 23:59:59 UTC, fetch today's price for configured coins and upsert to MongoDB.

use chrono::{TimeZone, Utc};
use mongodb::bson::DateTime as BsonDateTime;
use std::sync::Arc;
use std::time::Duration;
use url::Url;

use crate::config;
use crate::convert;
use crate::repository;

/// CoinGecko market_chart response (only the fields we use).
#[derive(Debug, serde::Deserialize)]
struct CoinGeckoMarketChart {
    prices: Vec<[f64; 2]>,
}

/// Returns the next instant at 23:59:59 UTC (today or tomorrow).
fn next_235959_utc() -> chrono::DateTime<Utc> {
    let now = Utc::now();
    let today_2359 = now
        .date_naive()
        .and_hms_opt(23, 59, 59)
        .unwrap()
        .and_utc();
    if now < today_2359 {
        today_2359
    } else {
        let tomorrow = now
            .date_naive()
            .succ_opt()
            .unwrap_or(now.date_naive());
        tomorrow
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_utc()
    }
}

/// Fetch today's (UTC) last price for one coin from CoinGecko and return (date_00:00_utc, price).
async fn fetch_today_price(
    coin_id: &str,
    currency: &str,
) -> Result<Option<(BsonDateTime, f64)>, String> {
    let base_url = config::coingecko_api_base();
    let base = base_url.trim_end_matches('/');
    let path = format!("{}/coins/{}/market_chart", base, coin_id);
    let mut url = Url::parse_with_params(
        &path,
        &[("vs_currency", currency), ("days", "2")],
    )
    .map_err(|e| e.to_string())?;
    if let Some(key) = config::coingecko_api_key() {
        url.query_pairs_mut().append_pair("x_cg_demo_api_key", &key);
    }
    let url = url.to_string();

    let client = reqwest::Client::builder()
        .build()
        .map_err(|e| e.to_string())?;
    let res = client
        .get(&url)
        .header("User-Agent", "financial-be/1.0 (personal finance app)")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !res.status().is_success() {
        return Err(format!("CoinGecko HTTP {}", res.status()));
    }
    let body: CoinGeckoMarketChart = res.json().await.map_err(|e| e.to_string())?;

    // Last point = latest price; its day (UTC) is the date we store
    const MS_PER_DAY: i64 = 86400 * 1000;
    let last = body.prices.last().copied();
    let Some([ts_ms, price]) = last else {
        return Ok(None);
    };
    let ts_ms = ts_ms as i64;
    let day_ms = (ts_ms / MS_PER_DAY) * MS_PER_DAY;
    let date = BsonDateTime::from_millis(day_ms);
    Ok(Some((date, price)))
}

/// Run the daily price job once: for each CRON_COIN_IDS coin, fetch today's price and upsert.
async fn run_daily_price_job(db: &mongodb::Database) -> Result<(), String> {
    let coin_ids = config::cron_coin_ids();
    if coin_ids.is_empty() {
        tracing::debug!("CRON_COIN_IDS empty, skipping daily price job");
        return Ok(());
    }

    let user_id = mongodb::bson::oid::ObjectId::parse_str(config::default_user_id_hex().as_str())
        .map_err(|e| e.to_string())?;
    let currency = config::cron_currency();
    let now = convert::now_bson();

    let mut total_upserted = 0u64;
    for coin_id in &coin_ids {
        match fetch_today_price(coin_id, &currency).await {
            Ok(Some((date, price))) => {
                let n = repository::coin_price_history::upsert_many(
                    db,
                    user_id,
                    coin_id,
                    &currency,
                    &[(date, price)],
                    now,
                )
                .await?;
                total_upserted += n;
                tracing::info!("Cron: upserted {} point(s) for {} ({}), date {:?}", n, coin_id, currency, date);
            }
            Ok(None) => {
                tracing::warn!("Cron: no price data for {}", coin_id);
            }
            Err(e) => {
                tracing::warn!("Cron: fetch failed for {}: {}", coin_id, e);
            }
        }
    }
    tracing::info!("Cron: daily price job finished, {} points upserted", total_upserted);
    Ok(())
}

/// Loop: sleep until next 23:59:59 UTC, run job, repeat.
pub async fn cron_loop(db: Arc<mongodb::Database>) {
    loop {
        let next = next_235959_utc();
        let now = Utc::now();
        let duration = (next - now).to_std().unwrap_or(Duration::ZERO);
        if duration > Duration::ZERO {
            tracing::info!("Cron: next run at {} UTC (in {:?})", next, duration);
            tokio::time::sleep(duration).await;
        }

        if let Err(e) = run_daily_price_job(db.as_ref()).await {
            tracing::error!("Cron: daily price job error: {}", e);
        }
    }
}
