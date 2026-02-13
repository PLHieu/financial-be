//! Predefined crypto assets (BTC, ETH, ...) for "add from list" when creating an asset.

use axum::Json;

/// One predefined asset: name, symbol (e.g. BTCUSD), CoinGecko id for prices.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PredefinedAssetDto {
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub coingecko_id: String,
}

fn predefined_list() -> Vec<PredefinedAssetDto> {
    vec![
        PredefinedAssetDto {
            id: "btc".to_string(),
            name: "Bitcoin".to_string(),
            symbol: "BTCUSD".to_string(),
            coingecko_id: "bitcoin".to_string(),
        },
        PredefinedAssetDto {
            id: "eth".to_string(),
            name: "Ethereum".to_string(),
            symbol: "ETHUSD".to_string(),
            coingecko_id: "ethereum".to_string(),
        },
        PredefinedAssetDto {
            id: "bnb".to_string(),
            name: "BNB".to_string(),
            symbol: "BNBUSD".to_string(),
            coingecko_id: "binancecoin".to_string(),
        },
        PredefinedAssetDto {
            id: "sol".to_string(),
            name: "Solana".to_string(),
            symbol: "SOLUSD".to_string(),
            coingecko_id: "solana".to_string(),
        },
        PredefinedAssetDto {
            id: "xrp".to_string(),
            name: "XRP".to_string(),
            symbol: "XRPUSD".to_string(),
            coingecko_id: "ripple".to_string(),
        },
        PredefinedAssetDto {
            id: "doge".to_string(),
            name: "Dogecoin".to_string(),
            symbol: "DOGEUSD".to_string(),
            coingecko_id: "dogecoin".to_string(),
        },
    ]
}

/// GET /predefined-assets – list of predefined crypto assets for quick-add in portfolio.
pub async fn list() -> Json<Vec<PredefinedAssetDto>> {
    Json(predefined_list())
}
