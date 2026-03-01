mod dto;
mod parse;
mod portfolios;
mod assets;
mod predefined_assets;
mod transactions;
mod snapshots;
mod portfolio_snapshots;
mod asset_price_history;
mod coin_price_history;
mod transfer_transactions;
mod exchange_rates;
mod debts;
mod loans;
mod funds;
mod depreciating_items;

use axum::{
    routing::{delete, get, patch, post, put},
    Router,
};

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/portfolios", get(portfolios::list).post(portfolios::create))
        .route(
            "/portfolios/:id/performance",
            get(portfolios::performance),
        )
        .route(
            "/portfolios/:id",
            get(portfolios::get).put(portfolios::update).delete(portfolios::delete),
        )
        .route("/predefined-assets", get(predefined_assets::list))
        .route("/assets", get(assets::list_all).post(assets::create_global))
        .route(
            "/portfolios/:portfolio_id/assets",
            get(assets::list_by_portfolio).post(assets::create),
        )
        .route(
            "/assets/:id",
            get(assets::get).put(assets::update).patch(assets::update_status).delete(assets::delete),
        )
        .route(
            "/portfolios/:portfolio_id/transactions",
            get(transactions::list_by_portfolio).post(transactions::create_for_portfolio),
        )
        .route(
            "/assets/:asset_id/transactions",
            get(transactions::list_by_asset).post(transactions::create),
        )
        .route("/transactions", get(transactions::list_all))
        .route("/transactions/:id", delete(transactions::delete))
        .route(
            "/net-worth/performance",
            get(snapshots::net_worth_performance),
        )
        .route(
            "/net-worth-snapshots",
            get(snapshots::list_net_worth).post(snapshots::upsert_net_worth),
        )
        .route(
            "/portfolio-snapshots",
            get(portfolio_snapshots::list).post(portfolio_snapshots::upsert),
        )
        .route(
            "/asset-price-history",
            post(asset_price_history::upsert),
        )
        .route(
            "/assets/:asset_id/price-history",
            get(asset_price_history::list_by_asset),
        )
        .route(
            "/coin-price-history/fetch",
            post(coin_price_history::fetch_and_upsert),
        )
        .route(
            "/coin-price-history",
            get(coin_price_history::list),
        )
        .route(
            "/transfer-transactions",
            get(transfer_transactions::list).post(transfer_transactions::create),
        )
        .route(
            "/exchange-rates",
            get(exchange_rates::list).post(exchange_rates::create),
        )
        .route("/debts", get(debts::list).post(debts::create))
        .route("/debts/:id/performance", get(debts::performance))
        .route("/debts/:id", get(debts::get).delete(debts::delete))
        .route(
            "/debts/:id/transactions",
            get(debts::list_transactions).post(debts::create_transaction),
        )
        .route("/loans", get(loans::list).post(loans::create))
        .route("/loans/:id/performance", get(loans::performance))
        .route("/loans/:id", get(loans::get).delete(loans::delete))
        .route(
            "/loans/:id/transactions",
            get(loans::list_transactions).post(loans::create_transaction),
        )
        .route("/funds", get(funds::list).post(funds::create))
        .route("/funds/:id/performance", get(funds::performance))
        .route("/funds/:id", get(funds::get).delete(funds::delete))
        .route(
            "/funds/:id/transactions",
            get(funds::list_transactions).post(funds::create_transaction),
        )
        .route(
            "/depreciating-items",
            get(depreciating_items::list).post(depreciating_items::create),
        )
        .route(
            "/depreciating-items/:id/performance",
            get(depreciating_items::performance),
        )
        .route("/depreciating-items/:id", get(depreciating_items::get).delete(depreciating_items::delete))
        .route(
            "/depreciating-items/:id/transactions",
            get(depreciating_items::list_transactions)
                .post(depreciating_items::create_transaction),
        )
        .with_state(state)
    // UserContext is extracted per-route in each handler
}
