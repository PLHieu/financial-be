//! Parse API string values to domain enums.

use crate::models::{AssetStatus, AssetType, Currency, PortfolioType, TransactionType};

pub fn parse_portfolio_type(s: &str) -> Result<PortfolioType, &'static str> {
    match s {
        "Investment" => Ok(PortfolioType::Investment),
        "Savings/Goal" => Ok(PortfolioType::SavingsGoal),
        _ => Err("invalid portfolio type"),
    }
}

pub fn parse_currency(s: &str) -> Result<Currency, &'static str> {
    match s {
        "VND" => Ok(Currency::VND),
        "USD" => Ok(Currency::USD),
        "USDT" => Ok(Currency::USDT),
        _ => Err("invalid currency"),
    }
}

pub fn parse_asset_type(s: &str) -> Result<AssetType, &'static str> {
    match s {
        "Crypto" => Ok(AssetType::Crypto),
        "Bank / Savings" => Ok(AssetType::BankSavings),
        "Cash" => Ok(AssetType::Cash),
        "Manual Asset" => Ok(AssetType::ManualAsset),
        "Tiết kiệm linh hoạt" => Ok(AssetType::TietKiemLinhHoat),
        _ => Err("invalid asset type"),
    }
}

pub fn parse_asset_status(s: &str) -> Result<AssetStatus, &'static str> {
    match s {
        "Active" => Ok(AssetStatus::Active),
        "Closed" => Ok(AssetStatus::Closed),
        _ => Err("invalid asset status"),
    }
}

pub fn parse_transaction_type(s: &str) -> Result<TransactionType, &'static str> {
    match s {
        "Buy" => Ok(TransactionType::Buy),
        "Sell" => Ok(TransactionType::Sell),
        "Deposit" => Ok(TransactionType::Deposit),
        "Withdraw" => Ok(TransactionType::Withdraw),
        _ => Err("invalid transaction type"),
    }
}
