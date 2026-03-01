//! Domain -> API DTO conversion (enum to API string, DateTime to ISO string).

use crate::convert;
use crate::models::{
    Asset, AssetPriceHistory, CoinPriceHistory, Debt, DebtTransaction, DepreciatingItem,
    DepreciatingItemTransaction, ExchangeRate, Fund, FundTransaction, Loan, LoanTransaction,
    NetWorthSnapshot, Portfolio, PortfolioSnapshot, Transaction, TransferTransaction,
};
use crate::models::{
    AssetDto, AssetPriceHistoryDto, CoinPriceHistoryDto, DebtDto, DebtTransactionDto,
    DepreciatingItemDto, DepreciatingItemTransactionDto, ExchangeRateDto, FundDto,
    FundTransactionDto, LoanDto, LoanTransactionDto, NetWorthSnapshotDto, PortfolioDto,
    PortfolioSnapshotDto, TransactionDto, TransferTransactionDto,
};
use crate::models::{
    AssetStatus, AssetType, Currency, DebtTransactionType, DepreciatingItemTransactionType,
    FundTransactionType, LoanTransactionType, PortfolioType, TransactionType,
};

fn portfolio_type_str(t: &PortfolioType) -> &'static str {
    match t {
        PortfolioType::Investment => "Investment",
        PortfolioType::SavingsGoal => "Savings/Goal",
    }
}

fn currency_str(c: &Currency) -> &'static str {
    match c {
        Currency::VND => "VND",
        Currency::USD => "USD",
    }
}

fn asset_type_str(t: &AssetType) -> &'static str {
    match t {
        AssetType::Crypto => "Crypto",
        AssetType::BankSavings => "Bank / Savings",
        AssetType::Cash => "Cash",
        AssetType::ManualAsset => "Manual Asset",
        AssetType::TietKiemLinhHoat => "Tiết kiệm linh hoạt",
        AssetType::DepreciatingAsset => "Depreciating Asset",
    }
}

fn asset_status_str(s: &AssetStatus) -> &'static str {
    match s {
        AssetStatus::Active => "Active",
        AssetStatus::Closed => "Closed",
    }
}

fn transaction_type_str(t: &TransactionType) -> &'static str {
    match t {
        TransactionType::Buy => "Buy",
        TransactionType::Sell => "Sell",
        TransactionType::Deposit => "Deposit",
        TransactionType::Withdraw => "Withdraw",
    }
}

fn debt_transaction_type_str(t: &DebtTransactionType) -> &'static str {
    match t {
        DebtTransactionType::Borrow => "Borrow",
        DebtTransactionType::Repay => "Repay",
    }
}

fn loan_transaction_type_str(t: &LoanTransactionType) -> &'static str {
    match t {
        LoanTransactionType::Lend => "Lend",
        LoanTransactionType::Repay => "Repay",
    }
}

pub fn portfolio_to_dto(p: &Portfolio) -> PortfolioDto {
    PortfolioDto {
        id: p.id.as_ref().map(|id| id.to_hex()).unwrap_or_default(),
        name: p.name.clone(),
        r#type: portfolio_type_str(&p.r#type).to_string(),
        base_currency: currency_str(&p.base_currency).to_string(),
        description: p.description.clone(),
        color: p.color.clone(),
        created_at: convert::bson_dt_to_rfc3339(&p.created_at),
        updated_at: convert::bson_dt_to_rfc3339(&p.updated_at),
    }
}

pub fn asset_to_dto(a: &Asset) -> AssetDto {
    AssetDto {
        id: a.id.as_ref().map(|id| id.to_hex()).unwrap_or_default(),
        portfolio_id: a.portfolio_id.map(|id| id.to_hex()),
        name: a.name.clone(),
        r#type: asset_type_str(&a.r#type).to_string(),
        currency: currency_str(&a.currency).to_string(),
        status: asset_status_str(&a.status).to_string(),
        metadata: a.metadata.as_ref().map(|m| {
            serde_json::to_value(m).unwrap_or(serde_json::Value::Null)
        }),
        created_at: convert::bson_dt_to_rfc3339(&a.created_at),
        updated_at: convert::bson_dt_to_rfc3339(&a.updated_at),
    }
}

pub fn transaction_to_dto(t: &Transaction) -> TransactionDto {
    TransactionDto {
        id: t.id.as_ref().map(|id| id.to_hex()).unwrap_or_default(),
        portfolio_id: t.portfolio_id.map(|id| id.to_hex()),
        asset_id: t.asset_id.to_hex(),
        r#type: transaction_type_str(&t.r#type).to_string(),
        amount: t.amount,
        price: t.price,
        quantity: t.quantity,
        note: t.note.clone(),
        date: convert::bson_dt_to_rfc3339(&t.date),
        created_at: convert::bson_dt_to_rfc3339(&t.created_at),
        updated_at: convert::bson_dt_to_rfc3339(&t.updated_at),
    }
}

pub fn net_worth_snapshot_to_dto(s: &NetWorthSnapshot) -> NetWorthSnapshotDto {
    NetWorthSnapshotDto {
        id: s.id.as_ref().map(|id| id.to_hex()).unwrap_or_default(),
        date: convert::bson_dt_to_rfc3339(&s.date),
        total_net_worth: s.total_net_worth,
        created_at: convert::bson_dt_to_rfc3339(&s.created_at),
    }
}

pub fn portfolio_snapshot_to_dto(s: &PortfolioSnapshot) -> PortfolioSnapshotDto {
    PortfolioSnapshotDto {
        id: s.id.as_ref().map(|id| id.to_hex()).unwrap_or_default(),
        date: convert::bson_dt_to_rfc3339(&s.date),
        portfolio_id: s.portfolio_id.to_hex(),
        total_deposit: s.total_deposit,
        inventory: s.inventory.clone(),
        total_value: s.total_value,
        created_at: convert::bson_dt_to_rfc3339(&s.created_at),
    }
}

pub fn asset_price_history_to_dto(r: &AssetPriceHistory) -> AssetPriceHistoryDto {
    AssetPriceHistoryDto {
        id: r.id.as_ref().map(|id| id.to_hex()).unwrap_or_default(),
        asset_id: r.asset_id.to_hex(),
        date: convert::bson_dt_to_rfc3339(&r.date),
        price: r.price,
        currency: r.currency.clone(),
        created_at: convert::bson_dt_to_rfc3339(&r.created_at),
    }
}

pub fn transfer_transaction_to_dto(t: &TransferTransaction) -> TransferTransactionDto {
    TransferTransactionDto {
        id: t.id.as_ref().map(|id| id.to_hex()).unwrap_or_default(),
        from_portfolio_id: t.from_portfolio_id.to_hex(),
        to_portfolio_id: t.to_portfolio_id.to_hex(),
        amount: t.amount,
        from_currency: t.from_currency.clone(),
        to_currency: t.to_currency.clone(),
        exchange_rate_id: t.exchange_rate_id.map(|id| id.to_hex()),
        converted_amount: t.converted_amount,
        date: convert::bson_dt_to_rfc3339(&t.date),
        note: t.note.clone(),
        created_at: convert::bson_dt_to_rfc3339(&t.created_at),
    }
}

pub fn exchange_rate_to_dto(r: &ExchangeRate) -> ExchangeRateDto {
    ExchangeRateDto {
        id: r.id.as_ref().map(|id| id.to_hex()).unwrap_or_default(),
        from_currency: r.from_currency.clone(),
        to_currency: r.to_currency.clone(),
        rate: r.rate,
        date: convert::bson_dt_to_rfc3339(&r.date),
        source: r.source.clone(),
        created_at: convert::bson_dt_to_rfc3339(&r.created_at),
    }
}

pub fn coin_price_history_to_dto(r: &CoinPriceHistory) -> CoinPriceHistoryDto {
    CoinPriceHistoryDto {
        id: r.id.as_ref().map(|id| id.to_hex()).unwrap_or_default(),
        coin_id: r.coin_id.clone(),
        date: convert::bson_dt_to_rfc3339(&r.date),
        price: r.price,
        currency: r.currency.clone(),
        created_at: convert::bson_dt_to_rfc3339(&r.created_at),
    }
}

pub fn debt_to_dto(d: &Debt) -> DebtDto {
    DebtDto {
        id: d.id.as_ref().map(|id| id.to_hex()).unwrap_or_default(),
        portfolio_id: d.portfolio_id.map(|id| id.to_hex()),
        name: d.name.clone(),
        currency: currency_str(&d.currency).to_string(),
        status: asset_status_str(&d.status).to_string(),
        metadata: d.metadata.clone(),
        created_at: convert::bson_dt_to_rfc3339(&d.created_at),
        updated_at: convert::bson_dt_to_rfc3339(&d.updated_at),
    }
}

pub fn debt_transaction_to_dto(t: &DebtTransaction) -> DebtTransactionDto {
    DebtTransactionDto {
        id: t.id.as_ref().map(|id| id.to_hex()).unwrap_or_default(),
        debt_id: t.debt_id.to_hex(),
        r#type: debt_transaction_type_str(&t.r#type).to_string(),
        amount: t.amount,
        date: convert::bson_dt_to_rfc3339(&t.date),
        note: t.note.clone(),
        created_at: convert::bson_dt_to_rfc3339(&t.created_at),
        updated_at: convert::bson_dt_to_rfc3339(&t.updated_at),
    }
}

pub fn loan_to_dto(l: &Loan) -> LoanDto {
    LoanDto {
        id: l.id.as_ref().map(|id| id.to_hex()).unwrap_or_default(),
        portfolio_id: l.portfolio_id.map(|id| id.to_hex()),
        name: l.name.clone(),
        currency: currency_str(&l.currency).to_string(),
        status: asset_status_str(&l.status).to_string(),
        metadata: l.metadata.clone(),
        created_at: convert::bson_dt_to_rfc3339(&l.created_at),
        updated_at: convert::bson_dt_to_rfc3339(&l.updated_at),
    }
}

pub fn loan_transaction_to_dto(t: &LoanTransaction) -> LoanTransactionDto {
    LoanTransactionDto {
        id: t.id.as_ref().map(|id| id.to_hex()).unwrap_or_default(),
        loan_id: t.loan_id.to_hex(),
        r#type: loan_transaction_type_str(&t.r#type).to_string(),
        amount: t.amount,
        date: convert::bson_dt_to_rfc3339(&t.date),
        note: t.note.clone(),
        created_at: convert::bson_dt_to_rfc3339(&t.created_at),
        updated_at: convert::bson_dt_to_rfc3339(&t.updated_at),
    }
}

fn fund_transaction_type_str(t: &FundTransactionType) -> &'static str {
    match t {
        FundTransactionType::Deposit => "Deposit",
        FundTransactionType::Withdraw => "Withdraw",
    }
}

pub fn fund_to_dto(f: &Fund) -> FundDto {
    FundDto {
        id: f.id.as_ref().map(|id| id.to_hex()).unwrap_or_default(),
        portfolio_id: f.portfolio_id.map(|id| id.to_hex()),
        name: f.name.clone(),
        currency: currency_str(&f.currency).to_string(),
        fund_type: f.fund_type.clone(),
        start_date: f.start_date.clone(),
        status: asset_status_str(&f.status).to_string(),
        metadata: f.metadata.clone(),
        inflation_rate: f.inflation_rate,
        created_at: convert::bson_dt_to_rfc3339(&f.created_at),
        updated_at: convert::bson_dt_to_rfc3339(&f.updated_at),
    }
}

pub fn fund_transaction_to_dto(t: &FundTransaction) -> FundTransactionDto {
    FundTransactionDto {
        id: t.id.as_ref().map(|id| id.to_hex()).unwrap_or_default(),
        fund_id: t.fund_id.to_hex(),
        r#type: fund_transaction_type_str(&t.r#type).to_string(),
        amount: t.amount,
        date: convert::bson_dt_to_rfc3339(&t.date),
        note: t.note.clone(),
        created_at: convert::bson_dt_to_rfc3339(&t.created_at),
        updated_at: convert::bson_dt_to_rfc3339(&t.updated_at),
    }
}

fn depreciating_item_transaction_type_str(t: &DepreciatingItemTransactionType) -> &'static str {
    match t {
        DepreciatingItemTransactionType::Deposit => "Deposit",
        DepreciatingItemTransactionType::Withdraw => "Withdraw",
    }
}

pub fn depreciating_item_to_dto(i: &DepreciatingItem) -> DepreciatingItemDto {
    DepreciatingItemDto {
        id: i.id.as_ref().map(|id| id.to_hex()).unwrap_or_default(),
        portfolio_id: i.portfolio_id.map(|id| id.to_hex()),
        name: i.name.clone(),
        currency: currency_str(&i.currency).to_string(),
        purchase_date: i.purchase_date.clone(),
        depreciation_curve: i.depreciation_curve.clone(),
        status: asset_status_str(&i.status).to_string(),
        metadata: i.metadata.clone(),
        created_at: convert::bson_dt_to_rfc3339(&i.created_at),
        updated_at: convert::bson_dt_to_rfc3339(&i.updated_at),
    }
}

pub fn depreciating_item_transaction_to_dto(
    t: &DepreciatingItemTransaction,
) -> DepreciatingItemTransactionDto {
    DepreciatingItemTransactionDto {
        id: t.id.as_ref().map(|id| id.to_hex()).unwrap_or_default(),
        depreciating_item_id: t.depreciating_item_id.to_hex(),
        r#type: depreciating_item_transaction_type_str(&t.r#type).to_string(),
        amount: t.amount,
        date: convert::bson_dt_to_rfc3339(&t.date),
        note: t.note.clone(),
        created_at: convert::bson_dt_to_rfc3339(&t.created_at),
        updated_at: convert::bson_dt_to_rfc3339(&t.updated_at),
    }
}
