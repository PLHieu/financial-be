//! Domain models and API DTOs. All date/time stored as BSON DateTime (UTC).
//! Every user-scoped document has `user_id` for multi-tenant support.

use mongodb::bson::{oid::ObjectId, DateTime as BsonDateTime};
use serde::{Deserialize, Serialize};

// ---------- Enums ----------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Currency {
    VND,
    USD,
    USDT,
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
    Active,
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
    pub portfolio_id: ObjectId,
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
    pub total_value: f64,
    pub pnl: f64,
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
    pub portfolio_id: String,
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
    pub total_value: f64,
    pub pnl: f64,
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
    pub total_value: f64,
    pub pnl: f64,
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
