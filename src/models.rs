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
    #[serde(rename = "Depreciating Asset")]
    DepreciatingAsset,
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
pub struct DepreciationCurveYear {
    pub year: u32,
    #[serde(rename = "ratePercent")]
    pub rate_percent: f64,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DebtTransactionType {
    Borrow,
    Repay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Debt {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub portfolio_id: Option<ObjectId>,
    pub name: String,
    pub currency: Currency,
    pub status: AssetStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebtTransaction {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    #[serde(rename = "debtId")]
    pub debt_id: ObjectId,
    pub r#type: DebtTransactionType,
    pub amount: f64,
    pub date: BsonDateTime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LoanTransactionType {
    Lend,
    Repay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Loan {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub portfolio_id: Option<ObjectId>,
    pub name: String,
    pub currency: Currency,
    pub status: AssetStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoanTransaction {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    #[serde(rename = "loanId")]
    pub loan_id: ObjectId,
    pub r#type: LoanTransactionType,
    pub amount: f64,
    pub date: BsonDateTime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FundTransactionType {
    Deposit,
    Withdraw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fund {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub portfolio_id: Option<ObjectId>,
    pub name: String,
    pub currency: Currency,
    #[serde(rename = "fundType")]
    pub fund_type: String,
    #[serde(rename = "startDate")]
    pub start_date: Option<String>,
    pub status: AssetStatus,
    /// Interest rate % per year, or interest_rate_periods for flexible savings style.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// Inflation rate % per year when fund has no interest (user-entered at creation).
    #[serde(rename = "inflationRate", skip_serializing_if = "Option::is_none")]
    pub inflation_rate: Option<f64>,
    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundTransaction {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    #[serde(rename = "fundId")]
    pub fund_id: ObjectId,
    pub r#type: FundTransactionType,
    pub amount: f64,
    pub date: BsonDateTime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DepreciatingItemTransactionType {
    Deposit,
    Withdraw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepreciatingItem {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub portfolio_id: Option<ObjectId>,
    pub name: String,
    pub currency: Currency,
    #[serde(rename = "purchaseDate")]
    pub purchase_date: String,
    #[serde(rename = "depreciationCurve")]
    pub depreciation_curve: Vec<DepreciationCurveYear>,
    pub status: AssetStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepreciatingItemTransaction {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    #[serde(rename = "depreciatingItemId")]
    pub depreciating_item_id: ObjectId,
    pub r#type: DepreciatingItemTransactionType,
    pub amount: f64,
    pub date: BsonDateTime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    pub created_at: BsonDateTime,
    pub updated_at: BsonDateTime,
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

/// One point in a performance series (date, value). Used for both cumulative capital and current value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceSeriesPoint {
    pub date: String,
    pub value: f64,
}

/// Response for performance and net-worth performance APIs: two series + currency.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceResponseDto {
    pub cumulative_capital_series: Vec<PerformanceSeriesPoint>,
    pub current_value_series: Vec<PerformanceSeriesPoint>,
    pub currency: String,
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
pub struct DebtDto {
    pub id: String,
    pub portfolio_id: Option<String>,
    pub name: String,
    pub currency: String,
    pub status: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebtTransactionDto {
    pub id: String,
    pub debt_id: String,
    pub r#type: String,
    pub amount: f64,
    pub date: String,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoanDto {
    pub id: String,
    pub portfolio_id: Option<String>,
    pub name: String,
    pub currency: String,
    pub status: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoanTransactionDto {
    pub id: String,
    pub loan_id: String,
    pub r#type: String,
    pub amount: f64,
    pub date: String,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundDto {
    pub id: String,
    pub portfolio_id: Option<String>,
    pub name: String,
    pub currency: String,
    pub fund_type: String,
    pub start_date: Option<String>,
    pub status: String,
    pub metadata: Option<serde_json::Value>,
    pub inflation_rate: Option<f64>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FundTransactionDto {
    pub id: String,
    pub fund_id: String,
    pub r#type: String,
    pub amount: f64,
    pub date: String,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepreciatingItemDto {
    pub id: String,
    pub portfolio_id: Option<String>,
    pub name: String,
    pub currency: String,
    pub purchase_date: String,
    pub depreciation_curve: Vec<DepreciationCurveYear>,
    pub status: String,
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DepreciatingItemTransactionDto {
    pub id: String,
    pub depreciating_item_id: String,
    pub r#type: String,
    pub amount: f64,
    pub date: String,
    pub note: Option<String>,
    pub created_at: String,
    pub updated_at: String,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDebtRequest {
    pub portfolio_id: Option<String>,
    pub name: String,
    pub currency: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDebtTransactionRequest {
    pub r#type: String,
    pub amount: f64,
    pub date: String,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLoanRequest {
    pub portfolio_id: Option<String>,
    pub name: String,
    pub currency: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLoanTransactionRequest {
    pub r#type: String,
    pub amount: f64,
    pub date: String,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFundRequest {
    pub portfolio_id: Option<String>,
    pub name: String,
    pub currency: String,
    pub fund_type: String,
    pub start_date: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub inflation_rate: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFundTransactionRequest {
    pub r#type: String,
    pub amount: f64,
    pub date: String,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDepreciatingItemRequest {
    pub portfolio_id: Option<String>,
    pub name: String,
    pub currency: String,
    pub purchase_date: String,
    pub depreciation_curve: Vec<DepreciationCurveYear>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDepreciatingItemTransactionRequest {
    pub r#type: String,
    pub amount: f64,
    pub date: String,
    pub note: Option<String>,
}
