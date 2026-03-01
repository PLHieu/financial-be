//! Compute portfolio snapshots on-the-fly from transactions, assets, price history, and exchange rates.
//! Does not read or write the portfolio_snapshots collection.

use std::collections::HashMap;

use chrono::{DateTime, Datelike, TimeZone, Utc};
use mongodb::bson::oid::ObjectId;
use mongodb::bson::DateTime as BsonDateTime;

use crate::error::AppError;
use crate::models::{
    Asset, AssetStatus, Currency, DebtTransactionType, DepreciatingItem,
    DepreciatingItemTransactionType, Fund, FundTransactionType, LoanTransactionType, Portfolio,
    PortfolioSnapshotDto, Transaction, TransactionType,
};
use crate::repository;
use crate::convert;

const USD_TO_VND_FALLBACK: f64 = 26_000.0;

/// Build portfolio snapshot DTOs for a user (and optionally a single portfolio) from start_date to today.
pub async fn compute_portfolio_snapshots(
    db: &mongodb::Database,
    user_id: ObjectId,
    portfolio_id: Option<ObjectId>,
    start_date: Option<BsonDateTime>,
) -> Result<Vec<PortfolioSnapshotDto>, AppError> {
    let portfolios: Vec<Portfolio> = if let Some(pid) = portfolio_id {
        let p = repository::portfolio::get_by_id(db, user_id, pid)
            .await
            .map_err(AppError::internal)?;
        match p {
            Some(port) => vec![port],
            None => return Ok(vec![]),
        }
    } else {
        repository::portfolio::get_all(db, user_id)
            .await
            .map_err(AppError::internal)?
    };

    let assets = repository::asset::get_all(db, user_id)
        .await
        .map_err(AppError::internal)?;
    let assets_map: HashMap<ObjectId, Asset> = assets.into_iter().filter_map(|a| a.id.map(|id| (id, a))).collect();

    let start_bson = start_date.unwrap_or_else(|| {
        let t = Utc::now() - chrono::Duration::days(365);
        BsonDateTime::from_millis(t.timestamp_millis())
    });

    let price_history = repository::asset_price_history::get_all_since(db, start_bson)
        .await
        .map_err(AppError::internal)?;
    let mut prices_by_asset: HashMap<ObjectId, Vec<(BsonDateTime, f64)>> = HashMap::new();
    for r in price_history {
        prices_by_asset
            .entry(r.asset_id)
            .or_default()
            .push((r.date, r.price));
    }

    let exchange_rates = repository::exchange_rate::get_all(db, Some(start_bson))
        .await
        .map_err(AppError::internal)?;
    let usd_to_vnd: Vec<(BsonDateTime, f64)> = exchange_rates
        .into_iter()
        .filter(|r| r.from_currency == "USD" && r.to_currency == "VND")
        .map(|r| (r.date, r.rate))
        .collect();

    let debts = repository::debt::get_all(db, user_id)
        .await
        .map_err(AppError::internal)?;
    let debt_txs = repository::debt_transaction::get_all_by_user(db, user_id)
        .await
        .map_err(AppError::internal)?;
    let loans = repository::loan::get_all(db, user_id)
        .await
        .map_err(AppError::internal)?;
    let loan_txs = repository::loan_transaction::get_all_by_user(db, user_id)
        .await
        .map_err(AppError::internal)?;
    let funds = repository::fund::get_all(db, user_id)
        .await
        .map_err(AppError::internal)?;
    let fund_txs = repository::fund_transaction::get_all_by_user(db, user_id)
        .await
        .map_err(AppError::internal)?;
    let depreciating_items = repository::depreciating_item::get_all(db, user_id)
        .await
        .map_err(AppError::internal)?;
    let depreciating_txs = repository::depreciating_item_transaction::get_all_by_user(db, user_id)
        .await
        .map_err(AppError::internal)?;

    let mut all_dtos = Vec::new();

    for portfolio in portfolios {
        let pid = portfolio.id.ok_or_else(|| AppError::internal("portfolio missing id"))?;
        if portfolio.user_id != user_id {
            continue;
        }

        let txs = repository::transaction::get_by_portfolio(db, user_id, pid)
            .await
            .map_err(AppError::internal)?;
        let mut txs_sorted = txs;
        txs_sorted.sort_by(|a, b| a.date.cmp(&b.date));

        if txs_sorted.is_empty() {
            continue;
        }

        let first_tx_date = txs_sorted[0].date;
        let range_start = day_start(min_bson(first_tx_date, start_bson));
        let now = Utc::now();
        let today_start = Utc
            .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
            .single()
            .unwrap_or(now);
        let range_end = today_start + chrono::Duration::days(1);

        let base_currency_str = match portfolio.base_currency {
            Currency::VND => "VND",
            Currency::USD => "USD",
        };

        let mut current = range_start;
        while current < range_end {
            let end_of_day = current + chrono::Duration::days(1);
            let end_of_day_bson = BsonDateTime::from_millis(end_of_day.timestamp_millis());

            let day_txs: Vec<&Transaction> = txs_sorted
                .iter()
                .filter(|t| t.date < end_of_day_bson)
                .collect();

            let usd_asset_id = find_usd_asset_id(&day_txs, &assets_map, user_id);

            let mut balance_by_asset: HashMap<ObjectId, f64> = HashMap::new();
            let mut total_deposit_base = 0.0f64;

            for t in &day_txs {
                let aid = t.asset_id;
                let typ = &t.r#type;
                let amount = t.amount;
                let quantity = t.quantity.unwrap_or(0.0);

                match typ {
                    TransactionType::Deposit => {
                        *balance_by_asset.entry(aid).or_insert(0.0) += amount;
                        let asset = assets_map.get(&aid);
                        if let Some(a) = asset {
                            if is_usd_asset(a) {
                                total_deposit_base += usd_to_base(amount, base_currency_str, current, &usd_to_vnd);
                            } else {
                                let price = get_price_for_date(
                                    prices_by_asset.get(&aid).map(|v| v.as_slice()).unwrap_or(&[]),
                                    current,
                                );
                                total_deposit_base += usd_to_base(
                                    amount * price.unwrap_or(1.0),
                                    base_currency_str,
                                    current,
                                    &usd_to_vnd,
                                );
                            }
                        } else {
                            total_deposit_base += usd_to_base(amount, base_currency_str, current, &usd_to_vnd);
                        }
                    }
                    TransactionType::Withdraw => {
                        *balance_by_asset.entry(aid).or_insert(0.0) -= amount;
                    }
                    TransactionType::Buy => {
                        *balance_by_asset.entry(aid).or_insert(0.0) += quantity;
                        if let Some(usd_id) = usd_asset_id {
                            *balance_by_asset.entry(usd_id).or_insert(0.0) -= amount;
                        }
                    }
                    TransactionType::Sell => {
                        *balance_by_asset.entry(aid).or_insert(0.0) -= quantity;
                        if let Some(usd_id) = usd_asset_id {
                            *balance_by_asset.entry(usd_id).or_insert(0.0) += amount;
                        }
                    }
                }
            }

            let inventory: HashMap<String, f64> = balance_by_asset
                .iter()
                .filter(|(_, &b)| b != 0.0)
                .filter_map(|(aid, &bal)| {
                    let asset = assets_map.get(aid)?;
                    let key = asset_balance_key(asset);
                    Some((key, (bal * 1e8).round() / 1e8))
                })
                .collect();
            let inventory_value = serde_json::to_value(inventory).unwrap_or(serde_json::Value::Object(Default::default()));

            let mut net_worth_base = 0.0;
            for (aid, &bal) in &balance_by_asset {
                if bal == 0.0 {
                    continue;
                }
                let Some(asset) = assets_map.get(aid) else {
                    continue;
                };
                if is_usd_asset(asset) {
                    net_worth_base += usd_to_base(bal, base_currency_str, current, &usd_to_vnd);
                } else {
                    let prices = prices_by_asset.get(aid).map(|v| v.as_slice()).unwrap_or(&[]);
                    if let Some(price) = get_price_for_date(prices, current) {
                        net_worth_base += usd_to_base(bal * price, base_currency_str, current, &usd_to_vnd);
                    }
                }
            }

            let total_debt: f64 = debts
                .iter()
                .filter(|d| d.portfolio_id == Some(pid))
                .filter_map(|debt| {
                    let did = debt.id?;
                    let balance = debt_txs
                        .iter()
                        .filter(|t| t.debt_id == did)
                        .filter(|t| t.date < end_of_day_bson)
                        .map(|t| match t.r#type {
                            DebtTransactionType::Borrow => t.amount,
                            DebtTransactionType::Repay => -t.amount,
                        })
                        .sum::<f64>();
                    Some(value_in_base(balance, &debt.currency, base_currency_str, current, &usd_to_vnd))
                })
                .sum();

            let total_loans_receivable: f64 = loans
                .iter()
                .filter(|l| l.portfolio_id == Some(pid))
                .filter_map(|loan| {
                    let lid = loan.id?;
                    let balance = loan_txs
                        .iter()
                        .filter(|t| t.loan_id == lid)
                        .filter(|t| t.date < end_of_day_bson)
                        .map(|t| match t.r#type {
                            LoanTransactionType::Lend => t.amount,
                            LoanTransactionType::Repay => -t.amount,
                        })
                        .sum::<f64>();
                    Some(value_in_base(balance, &loan.currency, base_currency_str, current, &usd_to_vnd))
                })
                .sum();

            let total_funds_value: f64 = funds
                .iter()
                .filter(|f| f.portfolio_id == Some(pid))
                .filter(|f| matches!(f.status, AssetStatus::Active))
                .filter_map(|fund| {
                    let fid = fund.id?;
                    let balance = fund_txs
                        .iter()
                        .filter(|t| t.fund_id == fid)
                        .filter(|t| t.date < end_of_day_bson)
                        .map(|t| match t.r#type {
                            FundTransactionType::Deposit => t.amount,
                            FundTransactionType::Withdraw => -t.amount,
                        })
                        .sum::<f64>();
                    let mult = fund_value_multiplier(fund, current);
                    Some(value_in_base(balance * mult, &fund.currency, base_currency_str, current, &usd_to_vnd))
                })
                .sum();

            let total_depreciating_value: f64 = depreciating_items
                .iter()
                .filter(|i| i.portfolio_id == Some(pid))
                .filter(|i| matches!(i.status, AssetStatus::Active))
                .filter_map(|item| {
                    let iid = item.id?;
                    let balance = depreciating_txs
                        .iter()
                        .filter(|t| t.depreciating_item_id == iid)
                        .filter(|t| t.date < end_of_day_bson)
                        .map(|t| match t.r#type {
                            DepreciatingItemTransactionType::Deposit => t.amount,
                            DepreciatingItemTransactionType::Withdraw => -t.amount,
                        })
                        .sum::<f64>();
                    let mult = depreciating_item_multiplier(&item.purchase_date, &item.depreciation_curve, current);
                    Some(value_in_base(balance * mult, &item.currency, base_currency_str, current, &usd_to_vnd))
                })
                .sum();

            let total_value = net_worth_base - total_debt + total_loans_receivable + total_funds_value + total_depreciating_value;

            let date_bson = BsonDateTime::from_millis(current.timestamp_millis());
            all_dtos.push(PortfolioSnapshotDto {
                id: String::new(),
                date: convert::bson_dt_to_rfc3339(&date_bson),
                portfolio_id: pid.to_hex(),
                total_deposit: Some((total_deposit_base * 100.0).round() / 100.0),
                inventory: Some(inventory_value),
                total_value: (total_value * 100.0).round() / 100.0,
                created_at: convert::bson_dt_to_rfc3339(&date_bson),
            });

            current = current + chrono::Duration::days(1);
        }
    }

    // When computing for all portfolios, add one snapshot per day for "global" items (portfolio_id = null)
    // so the frontend can sum portfolio snapshots + global once without double-counting.
    if portfolio_id.is_none() {
        let global_txs = repository::transaction::get_by_portfolio_global(db, user_id)
            .await
            .map_err(AppError::internal)?;
        let mut g_txs = global_txs;
        g_txs.sort_by(|a, b| a.date.cmp(&b.date));
        if !g_txs.is_empty() {
            let first_tx_date = g_txs[0].date;
            let range_start = day_start(min_bson(first_tx_date, start_bson));
            let now = Utc::now();
            let today_start = Utc
                .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
                .single()
                .unwrap_or(now);
            let range_end = today_start + chrono::Duration::days(1);
            let base_currency_str = "VND";
            let mut current = range_start;
            while current < range_end {
                let end_of_day = current + chrono::Duration::days(1);
                let end_of_day_bson = BsonDateTime::from_millis(end_of_day.timestamp_millis());
                let day_txs: Vec<&Transaction> = g_txs
                    .iter()
                    .filter(|t| t.date < end_of_day_bson)
                    .collect();
                let usd_asset_id = find_usd_asset_id(&day_txs, &assets_map, user_id);
                let mut balance_by_asset: HashMap<ObjectId, f64> = HashMap::new();
                let mut total_deposit_base = 0.0f64;
                for t in &day_txs {
                    let aid = t.asset_id;
                    let typ = &t.r#type;
                    let amount = t.amount;
                    let quantity = t.quantity.unwrap_or(0.0);
                    match typ {
                        TransactionType::Deposit => {
                            *balance_by_asset.entry(aid).or_insert(0.0) += amount;
                            let asset = assets_map.get(&aid);
                            if let Some(a) = asset {
                                if is_usd_asset(a) {
                                    total_deposit_base += usd_to_base(amount, base_currency_str, current, &usd_to_vnd);
                                } else {
                                    let price = get_price_for_date(
                                        prices_by_asset.get(&aid).map(|v| v.as_slice()).unwrap_or(&[]),
                                        current,
                                    );
                                    total_deposit_base += usd_to_base(
                                        amount * price.unwrap_or(1.0),
                                        base_currency_str,
                                        current,
                                        &usd_to_vnd,
                                    );
                                }
                            } else {
                                total_deposit_base += usd_to_base(amount, base_currency_str, current, &usd_to_vnd);
                            }
                        }
                        TransactionType::Withdraw => {
                            *balance_by_asset.entry(aid).or_insert(0.0) -= amount;
                        }
                        TransactionType::Buy => {
                            *balance_by_asset.entry(aid).or_insert(0.0) += quantity;
                            if let Some(usd_id) = usd_asset_id {
                                *balance_by_asset.entry(usd_id).or_insert(0.0) -= amount;
                            }
                        }
                        TransactionType::Sell => {
                            *balance_by_asset.entry(aid).or_insert(0.0) -= quantity;
                            if let Some(usd_id) = usd_asset_id {
                                *balance_by_asset.entry(usd_id).or_insert(0.0) += amount;
                            }
                        }
                    }
                }
                let mut net_worth_base = 0.0;
                for (aid, &bal) in &balance_by_asset {
                    if bal == 0.0 {
                        continue;
                    }
                    let Some(asset) = assets_map.get(aid) else {
                        continue;
                    };
                    if is_usd_asset(asset) {
                        net_worth_base += usd_to_base(bal, base_currency_str, current, &usd_to_vnd);
                    } else {
                        let prices = prices_by_asset.get(aid).map(|v| v.as_slice()).unwrap_or(&[]);
                        if let Some(price) = get_price_for_date(prices, current) {
                            net_worth_base += usd_to_base(bal * price, base_currency_str, current, &usd_to_vnd);
                        }
                    }
                }
                let total_debt: f64 = debts
                    .iter()
                    .filter(|d| d.portfolio_id.is_none())
                    .filter_map(|debt| {
                        let did = debt.id?;
                        let balance = debt_txs
                            .iter()
                            .filter(|t| t.debt_id == did)
                            .filter(|t| t.date < end_of_day_bson)
                            .map(|t| match t.r#type {
                                DebtTransactionType::Borrow => t.amount,
                                DebtTransactionType::Repay => -t.amount,
                            })
                            .sum::<f64>();
                        Some(value_in_base(balance, &debt.currency, base_currency_str, current, &usd_to_vnd))
                    })
                    .sum();
                let total_loans_receivable: f64 = loans
                    .iter()
                    .filter(|l| l.portfolio_id.is_none())
                    .filter_map(|loan| {
                        let lid = loan.id?;
                        let balance = loan_txs
                            .iter()
                            .filter(|t| t.loan_id == lid)
                            .filter(|t| t.date < end_of_day_bson)
                            .map(|t| match t.r#type {
                                LoanTransactionType::Lend => t.amount,
                                LoanTransactionType::Repay => -t.amount,
                            })
                            .sum::<f64>();
                        Some(value_in_base(balance, &loan.currency, base_currency_str, current, &usd_to_vnd))
                    })
                    .sum();
                let total_funds_value: f64 = funds
                    .iter()
                    .filter(|f| f.portfolio_id.is_none())
                    .filter(|f| matches!(f.status, AssetStatus::Active))
                    .filter_map(|fund| {
                        let fid = fund.id?;
                        let balance = fund_txs
                            .iter()
                            .filter(|t| t.fund_id == fid)
                            .filter(|t| t.date < end_of_day_bson)
                            .map(|t| match t.r#type {
                                FundTransactionType::Deposit => t.amount,
                                FundTransactionType::Withdraw => -t.amount,
                            })
                            .sum::<f64>();
                        let mult = fund_value_multiplier(fund, current);
                        Some(value_in_base(balance * mult, &fund.currency, base_currency_str, current, &usd_to_vnd))
                    })
                    .sum();
                let total_depreciating_value: f64 = depreciating_items
                    .iter()
                    .filter(|i| i.portfolio_id.is_none())
                    .filter(|i| matches!(i.status, AssetStatus::Active))
                    .filter_map(|item| {
                        let iid = item.id?;
                        let balance = depreciating_txs
                            .iter()
                            .filter(|t| t.depreciating_item_id == iid)
                            .filter(|t| t.date < end_of_day_bson)
                            .map(|t| match t.r#type {
                                DepreciatingItemTransactionType::Deposit => t.amount,
                                DepreciatingItemTransactionType::Withdraw => -t.amount,
                            })
                            .sum::<f64>();
                        let mult = depreciating_item_multiplier(&item.purchase_date, &item.depreciation_curve, current);
                        Some(value_in_base(balance * mult, &item.currency, base_currency_str, current, &usd_to_vnd))
                    })
                    .sum();
                let total_value = net_worth_base - total_debt + total_loans_receivable + total_funds_value + total_depreciating_value;
                let date_bson = BsonDateTime::from_millis(current.timestamp_millis());
                all_dtos.push(PortfolioSnapshotDto {
                    id: String::new(),
                    date: convert::bson_dt_to_rfc3339(&date_bson),
                    portfolio_id: "global".to_string(),
                    total_deposit: Some((total_deposit_base * 100.0).round() / 100.0),
                    inventory: None,
                    total_value: (total_value * 100.0).round() / 100.0,
                    created_at: convert::bson_dt_to_rfc3339(&date_bson),
                });
                current = current + chrono::Duration::days(1);
            }
        }
    }

    all_dtos.sort_by(|a, b| a.date.cmp(&b.date));
    Ok(all_dtos)
}

fn min_bson(a: BsonDateTime, b: BsonDateTime) -> BsonDateTime {
    if a.timestamp_millis() <= b.timestamp_millis() {
        a
    } else {
        b
    }
}

fn day_start(dt: BsonDateTime) -> DateTime<Utc> {
    let millis = dt.timestamp_millis();
    let chrono_dt = DateTime::from_timestamp_millis(millis).unwrap_or_default();
    Utc.with_ymd_and_hms(chrono_dt.year(), chrono_dt.month(), chrono_dt.day(), 0, 0, 0)
        .single()
        .unwrap_or(chrono_dt)
}

fn is_usd_asset(asset: &Asset) -> bool {
    let sym = asset
        .metadata
        .as_ref()
        .and_then(|m| m.symbol.as_deref())
        .unwrap_or("")
        .trim()
        .to_uppercase();
    let name = asset.name.trim().to_uppercase();
    sym == "USD" || name == "USD"
}

fn asset_balance_key(asset: &Asset) -> String {
    let sym = asset
        .metadata
        .as_ref()
        .and_then(|m| m.symbol.as_deref())
        .map(|s| s.trim().to_uppercase())
        .unwrap_or_else(|| asset.name.trim().to_uppercase());
    if sym.is_empty() {
        format!("asset_{}", asset.id.as_ref().map(|id| id.to_hex()).unwrap_or_default())
    } else if sym == "USD" {
        "USD".to_string()
    } else if sym.ends_with("USD") && sym.len() > 3 {
        sym[..sym.len() - 3].to_string()
    } else {
        sym
    }
}

fn get_price_for_date(
    prices: &[(BsonDateTime, f64)],
    target: DateTime<Utc>,
) -> Option<f64> {
    let target_ms = target.timestamp_millis();
    let mut out = None;
    for (dt, p) in prices {
        if dt.timestamp_millis() <= target_ms {
            out = Some(*p);
        } else {
            break;
        }
    }
    out
}

fn usd_to_base(
    usd_value: f64,
    base_currency: &str,
    at_date: DateTime<Utc>,
    usd_to_vnd: &[(BsonDateTime, f64)],
) -> f64 {
    if base_currency != "VND" {
        return usd_value;
    }
    let at_ms = at_date.timestamp_millis();
    let rate = usd_to_vnd
        .iter()
        .rev()
        .find(|(dt, _)| dt.timestamp_millis() <= at_ms)
        .map(|(_, r)| *r)
        .unwrap_or(USD_TO_VND_FALLBACK);
    usd_value * rate
}

/// Convert amount in given currency to base currency. Public for net-worth performance aggregation.
pub fn value_in_base(
    amount: f64,
    currency: &Currency,
    base_currency_str: &str,
    at_date: DateTime<Utc>,
    usd_to_vnd: &[(BsonDateTime, f64)],
) -> f64 {
    let at_ms = at_date.timestamp_millis();
    let rate = usd_to_vnd
        .iter()
        .rev()
        .find(|(dt, _)| dt.timestamp_millis() <= at_ms)
        .map(|(_, r)| *r)
        .unwrap_or(USD_TO_VND_FALLBACK);
    let is_usd = matches!(currency, Currency::USD);
    if base_currency_str == "VND" {
        if is_usd {
            amount * rate
        } else {
            amount
        }
    } else {
        if is_usd {
            amount
        } else {
            amount / rate
        }
    }
}

fn find_usd_asset_id(
    day_txs: &[&Transaction],
    assets_map: &HashMap<ObjectId, Asset>,
    user_id: ObjectId,
) -> Option<ObjectId> {
    for t in day_txs {
        if let Some(a) = assets_map.get(&t.asset_id) {
            if a.user_id == user_id && is_usd_asset(a) {
                return Some(t.asset_id);
            }
        }
    }
    for (_, a) in assets_map {
        if a.user_id == user_id && is_usd_asset(a) {
            if let Some(id) = a.id {
                return Some(id);
            }
        }
    }
    None
}

/// Depreciation multiplier for a depreciating item: decreases smoothly day-by-day using fractional years.
/// Completed full years: product of (1 + rate_i/100). Current partial year: (1 + rate/100)^fraction
/// so the chart shows gradual decline each day instead of a single drop per year.
/// Public for use by performance module.
pub fn depreciating_item_multiplier(
    purchase_date: &str,
    curve: &[crate::models::DepreciationCurveYear],
    as_of: DateTime<Utc>,
) -> f64 {
    if curve.is_empty() {
        return 1.0;
    }
    let naive_date = match chrono::NaiveDate::parse_from_str(purchase_date, "%Y-%m-%d") {
        Ok(d) => d,
        _ => return 1.0,
    };
    let naive_dt = match naive_date.and_hms_opt(0, 0, 0) {
        Some(dt) => dt,
        None => return 1.0,
    };
    let purchase = Utc.from_utc_datetime(&naive_dt);
    let duration = as_of.signed_duration_since(purchase);
    let days_since = duration.num_days();
    if days_since <= 0 {
        return 1.0;
    }
    let years_f = days_since as f64 / 365.25;
    let full_years = years_f.floor() as usize;
    let frac = years_f - full_years as f64;

    let mut mult = 1.0;
    for i in 0..full_years {
        let rate = curve
            .get(i)
            .or_else(|| curve.last())
            .map(|y| y.rate_percent)
            .unwrap_or(0.0);
        mult *= 1.0 + rate / 100.0;
    }
    if frac > 1e-9 {
        let rate = curve
            .get(full_years)
            .or_else(|| curve.last())
            .map(|y| y.rate_percent)
            .unwrap_or(0.0);
        mult *= (1.0 + rate / 100.0).powf(frac);
    }
    mult
}

/// Fund value multiplier: with interest → (1 + r/100)^t; without → (1 - inflation_rate/100)^t.
/// For interest_rate_periods, compounds per period (supports camelCase or snake_case in JSON).
/// Public for use by performance module.
pub fn fund_value_multiplier(fund: &Fund, as_of: DateTime<Utc>) -> f64 {
    let fund_start_naive = fund
        .start_date
        .as_ref()
        .and_then(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").ok());
    let t_years = fund_start_naive
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .map(|naive| {
            let start = Utc.from_utc_datetime(&naive);
            let secs = (as_of - start).num_seconds() as f64;
            secs / (365.25 * 24.0 * 3600.0)
        })
        .unwrap_or(0.0);
    if t_years <= 0.0 {
        return 1.0;
    }
    if let Some(meta) = &fund.metadata {
        if let Some(r) = meta.get("interestRate").and_then(|v| v.as_f64()) {
            return (1.0 + r / 100.0).powf(t_years);
        }
        let periods_key = if meta.get("interest_rate_periods").is_some() {
            "interest_rate_periods"
        } else if meta.get("interestRatePeriods").is_some() {
            "interestRatePeriods"
        } else {
            ""
        };
        if !periods_key.is_empty() {
            if let Some(arr) = meta.get(periods_key).and_then(|v| v.as_array()) {
                let mut mult = 1.0_f64;
                let as_of_naive = as_of.date_naive();
                let fund_start = match &fund_start_naive {
                    Some(d) => *d,
                    None => return 1.0,
                };
                #[derive(Clone, Copy)]
                struct Period { rate: f64, start: chrono::NaiveDate, end: chrono::NaiveDate }
                let mut periods: Vec<Period> = Vec::new();
                for p in arr {
                    let rate = match p.get("rate").and_then(|v| v.as_f64()) {
                        Some(r) => r,
                        None => continue,
                    };
                    let start_s = p.get("start_date").or_else(|| p.get("startDate"))
                        .and_then(|v| v.as_str());
                    let end_s = p.get("end_date").or_else(|| p.get("endDate"))
                        .and_then(|v| v.as_str());
                    if let (Some(s), Some(e)) = (start_s, end_s) {
                        if let (Ok(start), Ok(end)) = (
                            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d"),
                            chrono::NaiveDate::parse_from_str(e, "%Y-%m-%d"),
                        ) {
                            periods.push(Period { rate, start, end });
                        }
                    }
                }
                periods.sort_by_key(|p| p.start);
                for period in periods {
                    let range_start = fund_start.max(period.start);
                    let range_end = as_of_naive.min(period.end);
                    if range_start < range_end {
                        let days = range_end.signed_duration_since(range_start).num_days() as f64;
                        let years = days / 365.25;
                        mult *= (1.0 + period.rate / 100.0).powf(years);
                    }
                }
                if mult != 1.0 {
                    return mult;
                }
            }
        }
    }
    if let Some(inf) = fund.inflation_rate {
        return (1.0 - inf / 100.0).powf(t_years);
    }
    1.0
}
