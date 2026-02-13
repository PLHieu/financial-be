//! Domain models and API DTOs. All date/time stored as BSON DateTime (UTC).
//! Every user-scoped document has `user_id` for multi-tenant support.

use mongodb::bson::{oid::ObjectId, DateTime as BsonDateTime};
use serde::{Deserialize, Serialize};

// ---------- Enums ----------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Currency {
    #[serde(rename = "VND")]
    VND,
    #[serde(rename = "USD")]
    USD,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PortfolioType {
    Investment,
    #[serde(rename = "Savings/Goal")]
    SavingsGoal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssetType {
    Crypto,
    #[serde(rename = "Bank / Savings")]
    BankSavings,
    Cash,
    #[serde(rename = "Manual Asset")]
    ManualAsset,
    #[serde(rename = "Tiết kiệm linh hoạt")]
    TietKiemLinhHoat,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionType {
    Buy,
    Sell,
    Deposit,
    Withdraw,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssetStatus {
    #[serde(rename = "Active", alias = "ACTIVE", alias = "active")]
    Active,
    #[serde(rename = "Closed", alias = "CLOSED", alias = "closed")]
    Closed,
}

// ---------- Domain structs (BSON DateTime, for MongoDB) ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Portfolio {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub name: String,
    pub r#type: PortfolioType,
    pub base_currency: Currency,
    pub description: Option<String>,
    pub color: Option<String>,
    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub symbol: Option<String>,
    /// CoinGecko coin id (e.g. "bitcoin") for price scripts. Set when adding predefined crypto.
    #[serde(rename = "coingeckoId", skip_serializing_if = "Option::is_none")]
    pub coingecko_id: Option<String>,
    pub apy: Option<f64>,
    pub payout_frequency: Option<String>,
    pub monthly_contribution: Option<f64>,
    pub start_date: Option<String>,
    pub interest_rate_periods: Option<Vec<InterestRatePeriod>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterestRatePeriod {
    pub rate: f64,
    pub start_date: String,
    pub end_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    /// None = global asset (top-level). Some(pid) = legacy asset scoped to portfolio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub portfolio_id: Option<ObjectId>,
    pub name: String,
    pub r#type: AssetType,
    pub currency: Currency,
    pub status: AssetStatus,
    pub metadata: Option<AssetMetadata>,
    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    /// Portfolio this transaction belongs to (when creating from portfolio context).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub portfolio_id: Option<ObjectId>,
    pub asset_id: ObjectId,
    pub r#type: TransactionType,
    pub amount: f64,
    pub price: Option<f64>,
    pub quantity: Option<f64>,
    pub note: Option<String>,
    pub date: BsonDateTime,
    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetWorthSnapshot {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub date: BsonDateTime,
    pub total_net_worth: f64,
    pub created_at: BsonDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioSnapshot {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub date: BsonDateTime,
    pub portfolio_id: ObjectId,
    /// Total deposits (in portfolio base currency) up to this date. For chart: deposit line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_deposit: Option<f64>,
    /// Inventory at this date: e.g. {"USD": 10.0, "BTC": 0.5, "ETH": 2.0}. For display only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inventory: Option<serde_json::Value>,
    /// Net worth at this date in portfolio base currency. For chart: net worth line.
    pub total_value: f64,
    pub created_at: BsonDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPriceHistory {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub asset_id: ObjectId,
    pub date: BsonDateTime,
    pub price: f64,
    pub currency: String,
    pub created_at: BsonDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferTransaction {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub from_portfolio_id: ObjectId,
    pub to_portfolio_id: ObjectId,
    pub amount: f64,
    pub from_currency: String,
    pub to_currency: String,
    pub exchange_rate_id: Option<ObjectId>,
    pub converted_amount: Option<f64>,
    pub date: BsonDateTime,
    pub note: Option<String>,
    pub created_at: BsonDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeRate {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub from_currency: String,
    pub to_currency: String,
    pub rate: f64,
    pub date: BsonDateTime,
    pub source: Option<String>,
    pub created_at: BsonDateTime,
}

/// Price history by CoinGecko coin id. Used as cache from CoinGecko API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinPriceHistory {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub coin_id: String,
    pub date: BsonDateTime,
    pub price: f64,
    pub currency: String,
    pub created_at: BsonDateTime,
}

// ---------- API DTOs (camelCase, string IDs, date as ISO string) ----------

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioDto {
    pub id: String,
    pub name: String,
    pub r#type: String,
    pub base_currency: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetDto {
    pub id: String,
    pub portfolio_id: Option<String>,
    pub name: String,
    pub r#type: String,
    pub currency: String,
    pub status: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionDto {
    pub id: String,
    pub portfolio_id: Option<String>,
    pub asset_id: String,
    pub r#type: String,
    pub amount: f64,
    pub price: Option<f64>,
    pub quantity: Option<f64>,
    pub note: Option<String>,
    pub date: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetWorthSnapshotDto {
    pub id: String,
    pub date: String,
    pub total_net_worth: f64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioSnapshotDto {
    pub id: String,
    pub date: String,
    pub portfolio_id: String,
    pub total_deposit: Option<f64>,
    pub inventory: Option<serde_json::Value>,
    pub total_value: f64,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetPriceHistoryDto {
    pub id: String,
    pub asset_id: String,
    pub date: String,
    pub price: f64,
    pub currency: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferTransactionDto {
    pub id: String,
    pub from_portfolio_id: String,
    pub to_portfolio_id: String,
    pub amount: f64,
    pub from_currency: String,
    pub to_currency: String,
    pub exchange_rate_id: Option<String>,
    pub converted_amount: Option<f64>,
    pub date: String,
    pub note: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeRateDto {
    pub id: String,
    pub from_currency: String,
    pub to_currency: String,
    pub rate: f64,
    pub date: String,
    pub source: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoinPriceHistoryDto {
    pub id: String,
    pub coin_id: String,
    pub date: String,
    pub price: f64,
    pub currency: String,
    pub created_at: String,
}

// ---------- Request bodies (no id, no user_id) ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePortfolioRequest {
    pub name: String,
    pub r#type: String,
    pub base_currency: String,
    pub description: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAssetRequest {
    pub portfolio_id: String,
    pub name: String,
    pub r#type: String,
    pub currency: String,
    pub status: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Create global asset (top-level). No portfolio_id.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGlobalAssetRequest {
    pub name: String,
    pub r#type: String,
    pub currency: String,
    pub status: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAssetRequest {
    pub name: String,
    pub r#type: String,
    pub currency: String,
    pub status: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTransactionRequest {
    pub asset_id: String,
    pub r#type: String,
    pub amount: f64,
    pub price: Option<f64>,
    pub quantity: Option<f64>,
    pub note: Option<String>,
    pub date: String,
}

/// Create transaction in a portfolio (select asset from global list). portfolio_id from path.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePortfolioTransactionRequest {
    pub asset_id: String,
    pub r#type: String,
    pub amount: f64,
    pub price: Option<f64>,
    pub quantity: Option<f64>,
    pub note: Option<String>,
    pub date: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertNetWorthSnapshotRequest {
    pub date: String,
    pub total_net_worth: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertPortfolioSnapshotRequest {
    pub date: String,
    pub portfolio_id: String,
    pub total_deposit: Option<f64>,
    pub inventory: Option<serde_json::Value>,
    pub total_value: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertAssetPriceRequest {
    pub asset_id: String,
    pub date: String,
    pub price: f64,
    pub currency: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTransferRequest {
    pub from_portfolio_id: String,
    pub to_portfolio_id: String,
    pub amount: f64,
    pub from_currency: String,
    pub to_currency: String,
    pub exchange_rate_id: Option<String>,
    pub converted_amount: Option<f64>,
    pub date: String,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateExchangeRateRequest {
    pub from_currency: String,
    pub to_currency: String,
    pub rate: f64,
    pub date: String,
    pub source: Option<String>,
}
