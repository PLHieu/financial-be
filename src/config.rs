//! App config from environment. No auth for now; optional X-User-Id header
//! can be used. Default user id is used when header is missing (single-tenant mode).

pub fn mongodb_uri() -> String {
    std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string())
}

pub fn port() -> u16 {
    std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3001)
}

/// Default user ID when no auth / no X-User-Id. Use a fixed ObjectId so all
/// data is attributed to one "anonymous" user until auth is added.
pub fn default_user_id_hex() -> String {
    std::env::var("DEFAULT_USER_ID").unwrap_or_else(|_| "000000000000000000000001".to_string())
}

pub fn allowed_origin() -> Option<String> {
    std::env::var("ALLOWED_ORIGIN").ok()
}

/// CoinGecko API base URL (no trailing slash).
pub fn coingecko_api_base() -> String {
    std::env::var("COINGECKO_API_BASE").unwrap_or_else(|_| "https://api.coingecko.com/api/v3".to_string())
}

/// Optional CoinGecko API key (e.g. x_cg_demo_api_key for demo tier).
pub fn coingecko_api_key() -> Option<String> {
    std::env::var("COINGECKO_API_KEY").ok()
}

/// Comma-separated CoinGecko coin IDs for the daily cron (e.g. "bitcoin,ethereum,tether").
/// If unset or empty, the daily price cron does nothing.
pub fn cron_coin_ids() -> Vec<String> {
    std::env::var("CRON_COIN_IDS")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Currency for the daily price cron. App uses only "usd" and "vnd" (no USDT/USDC).
pub fn cron_currency() -> String {
    let c = std::env::var("CRON_CURRENCY").unwrap_or_else(|_| "usd".to_string());
    let lower = c.to_lowercase();
    if lower == "usdt" || lower == "usdc" || lower == "dai" || lower == "busd" {
        "usd".to_string()
    } else {
        c
    }
}
