//! Parse API string values to domain enums.

use crate::models::{
    AssetStatus, AssetType, Currency, DebtTransactionType, DepreciatingItemTransactionType,
    FundTransactionType, LoanTransactionType, PortfolioType, TransactionType,
};

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
        "Depreciating Asset" => Ok(AssetType::DepreciatingAsset),
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

pub fn parse_debt_transaction_type(s: &str) -> Result<DebtTransactionType, &'static str> {
    match s {
        "Borrow" => Ok(DebtTransactionType::Borrow),
        "Repay" => Ok(DebtTransactionType::Repay),
        _ => Err("invalid debt transaction type"),
    }
}

pub fn parse_loan_transaction_type(s: &str) -> Result<LoanTransactionType, &'static str> {
    match s {
        "Lend" => Ok(LoanTransactionType::Lend),
        "Repay" => Ok(LoanTransactionType::Repay),
        _ => Err("invalid loan transaction type"),
    }
}

pub fn parse_fund_transaction_type(s: &str) -> Result<FundTransactionType, &'static str> {
    match s {
        "Deposit" => Ok(FundTransactionType::Deposit),
        "Withdraw" => Ok(FundTransactionType::Withdraw),
        _ => Err("invalid fund transaction type"),
    }
}

pub fn parse_depreciating_item_transaction_type(
    s: &str,
) -> Result<DepreciatingItemTransactionType, &'static str> {
    match s {
        "Deposit" => Ok(DepreciatingItemTransactionType::Deposit),
        "Withdraw" => Ok(DepreciatingItemTransactionType::Withdraw),
        _ => Err("invalid depreciating item transaction type"),
    }
}
