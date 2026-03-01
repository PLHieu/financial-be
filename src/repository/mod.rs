//! Data access layer. All queries are scoped by user_id.

pub mod portfolio;
pub mod asset;
pub mod transaction;
pub mod net_worth_snapshot;
pub mod portfolio_snapshot;
pub mod asset_price_history;
pub mod coin_price_history;
pub mod transfer_transaction;
pub mod exchange_rate;
pub mod debt;
pub mod debt_transaction;
pub mod loan;
pub mod loan_transaction;
pub mod fund;
pub mod fund_transaction;
pub mod depreciating_item;
pub mod depreciating_item_transaction;
