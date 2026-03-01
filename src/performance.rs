//! Performance series (cumulative capital + current value per day) for portfolios, funds, debts, loans, depreciating items, and net worth.

use std::collections::HashMap;

use chrono::{DateTime, Datelike, TimeZone, Utc};
use mongodb::bson::oid::ObjectId;
use mongodb::bson::DateTime as BsonDateTime;

use crate::error::AppError;
use crate::models::{
    Currency, DebtTransactionType, DepreciatingItemTransactionType, FundTransactionType,
    LoanTransactionType, PerformanceResponseDto, PerformanceSeriesPoint,
};
use crate::repository;
use crate::snapshot::{compute_portfolio_snapshots, depreciating_item_multiplier, fund_value_multiplier, value_in_base};

/// Build list of day-start DateTimes in [from_iso, to_iso] (inclusive). from/to are "YYYY-MM-DD".
fn dates_in_range(from_iso: &str, to_iso: &str) -> Result<Vec<DateTime<Utc>>, AppError> {
    let from_dt = parse_date_iso(from_iso).ok_or_else(|| AppError::bad_request())?;
    let to_dt = parse_date_iso(to_iso).ok_or_else(|| AppError::bad_request())?;
    if from_dt > to_dt {
        return Ok(vec![]);
    }
    let mut out = Vec::new();
    let mut current = from_dt;
    while current <= to_dt {
        out.push(current);
        current = current + chrono::Duration::days(1);
    }
    Ok(out)
}

fn parse_date_iso(s: &str) -> Option<DateTime<Utc>> {
    let naive = chrono::NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").ok()?;
    let naive_dt = naive.and_hms_opt(0, 0, 0)?;
    Some(Utc.from_utc_datetime(&naive_dt))
}

fn date_to_iso(dt: DateTime<Utc>) -> String {
    format!("{:04}-{:02}-{:02}", dt.year(), dt.month(), dt.day())
}

/// Earliest transaction date as "YYYY-MM-DD". Returns None if no dates.
fn first_tx_date_iso(dates: &[BsonDateTime]) -> Option<String> {
    let min_ts = dates.iter().map(|d| d.timestamp_millis()).min()?;
    let dt = DateTime::from_timestamp_millis(min_ts).unwrap_or_default();
    Some(date_to_iso(dt))
}

fn currency_str(c: &Currency) -> &'static str {
    match c {
        Currency::VND => "VND",
        Currency::USD => "USD",
    }
}

/// Portfolio performance: cumulative capital = total_deposit, current value = total_value per day.
/// When from_date is empty, use the first day with data (from snapshots).
pub async fn portfolio_performance(
    db: &mongodb::Database,
    user_id: ObjectId,
    portfolio_id: ObjectId,
    from_date: &str,
    to_date: &str,
) -> Result<PerformanceResponseDto, AppError> {
    let portfolio = repository::portfolio::get_by_id(db, user_id, portfolio_id)
        .await
        .map_err(AppError::internal)?
        .ok_or_else(|| AppError::not_found())?;
    // "Toàn bộ": from_date empty -> use old start so snapshot returns from first tx to today
    let start_bson = if from_date.trim().is_empty() {
        parse_date_iso("2000-01-01").map(|d| BsonDateTime::from_millis(d.timestamp_millis()))
    } else {
        parse_date_iso(from_date).map(|d| BsonDateTime::from_millis(d.timestamp_millis()))
    };
    let dtos = compute_portfolio_snapshots(db, user_id, Some(portfolio_id), start_bson).await?;
    let currency = currency_str(&portfolio.base_currency).to_string();

    let by_date: HashMap<String, (f64, f64)> = dtos
        .iter()
        .filter(|d| d.date.len() >= 10)
        .map(|d| {
            let key = d.date[..10].to_string();
            let cap = d.total_deposit.unwrap_or(0.0);
            let val = d.total_value;
            (key, (cap, val))
        })
        .collect();

    let effective_from = if from_date.trim().is_empty() {
        by_date.keys().min().map(String::as_str).unwrap_or(to_date)
    } else {
        from_date
    };
    let days = dates_in_range(effective_from, to_date)?;
    let mut last = (0.0, 0.0);
    let mut cap_series = Vec::with_capacity(days.len());
    let mut val_series = Vec::with_capacity(days.len());
    for d in &days {
        let key = date_to_iso(*d);
        let (cap, val) = by_date.get(&key).copied().unwrap_or(last);
        last = (cap, val);
        cap_series.push(PerformanceSeriesPoint {
            date: key.clone(),
            value: (cap * 100.0).round() / 100.0,
        });
        val_series.push(PerformanceSeriesPoint {
            date: key,
            value: (val * 100.0).round() / 100.0,
        });
    }

    Ok(PerformanceResponseDto {
        cumulative_capital_series: cap_series,
        current_value_series: val_series,
        currency,
    })
}

/// Fund performance: balance (deposit - withdraw) and value = balance * fund_value_multiplier per day.
/// When from_date is empty ("Toàn bộ"), use first transaction date.
pub async fn fund_performance(
    db: &mongodb::Database,
    user_id: ObjectId,
    fund_id: ObjectId,
    from_date: &str,
    to_date: &str,
) -> Result<PerformanceResponseDto, AppError> {
    let fund = repository::fund::get_by_id(db, user_id, fund_id)
        .await
        .map_err(AppError::internal)?
        .ok_or_else(|| AppError::not_found())?;
    let txs = repository::fund_transaction::get_by_fund(db, user_id, fund_id)
        .await
        .map_err(AppError::internal)?;

    let tx_dates: Vec<BsonDateTime> = txs.iter().map(|t| t.date).collect();
    let effective_from = if from_date.trim().is_empty() {
        first_tx_date_iso(&tx_dates).unwrap_or_else(|| to_date.to_string())
    } else {
        from_date.to_string()
    };
    let days = dates_in_range(&effective_from, to_date)?;
    let currency = currency_str(&fund.currency).to_string();
    let mut cap_series = Vec::with_capacity(days.len());
    let mut val_series = Vec::with_capacity(days.len());

    for day_start in &days {
        let end_of_day = *day_start + chrono::Duration::days(1);
        let end_bson = BsonDateTime::from_millis(end_of_day.timestamp_millis());
        let balance: f64 = txs
            .iter()
            .filter(|t| t.date < end_bson)
            .map(|t| match t.r#type {
                FundTransactionType::Deposit => t.amount,
                FundTransactionType::Withdraw => -t.amount,
            })
            .sum();
        let mult = fund_value_multiplier(&fund, *day_start);
        let value = balance * mult;
        let key = date_to_iso(*day_start);
        cap_series.push(PerformanceSeriesPoint {
            date: key.clone(),
            value: (balance * 100.0).round() / 100.0,
        });
        val_series.push(PerformanceSeriesPoint {
            date: key,
            value: (value * 100.0).round() / 100.0,
        });
    }

    Ok(PerformanceResponseDto {
        cumulative_capital_series: cap_series,
        current_value_series: val_series,
        currency,
    })
}

/// Debt performance: cumulative = sum(Borrow), value = sum(Borrow) - sum(Repay) per day.
/// When from_date is empty ("Toàn bộ"), use first transaction date.
pub async fn debt_performance(
    db: &mongodb::Database,
    user_id: ObjectId,
    debt_id: ObjectId,
    from_date: &str,
    to_date: &str,
) -> Result<PerformanceResponseDto, AppError> {
    let debt = repository::debt::get_by_id(db, user_id, debt_id)
        .await
        .map_err(AppError::internal)?
        .ok_or_else(|| AppError::not_found())?;
    let txs = repository::debt_transaction::get_by_debt(db, user_id, debt_id)
        .await
        .map_err(AppError::internal)?;

    let tx_dates: Vec<BsonDateTime> = txs.iter().map(|t| t.date).collect();
    let effective_from = if from_date.trim().is_empty() {
        first_tx_date_iso(&tx_dates).unwrap_or_else(|| to_date.to_string())
    } else {
        from_date.to_string()
    };
    let days = dates_in_range(&effective_from, to_date)?;
    let currency = currency_str(&debt.currency).to_string();
    let mut cap_series = Vec::with_capacity(days.len());
    let mut val_series = Vec::with_capacity(days.len());

    for day_start in &days {
        let end_of_day = *day_start + chrono::Duration::days(1);
        let end_bson = BsonDateTime::from_millis(end_of_day.timestamp_millis());
        let cumulative: f64 = txs
            .iter()
            .filter(|t| t.date < end_bson)
            .filter(|t| matches!(t.r#type, DebtTransactionType::Borrow))
            .map(|t| t.amount)
            .sum();
        let value: f64 = txs
            .iter()
            .filter(|t| t.date < end_bson)
            .map(|t| match t.r#type {
                DebtTransactionType::Borrow => t.amount,
                DebtTransactionType::Repay => -t.amount,
            })
            .sum();
        let key = date_to_iso(*day_start);
        cap_series.push(PerformanceSeriesPoint {
            date: key.clone(),
            value: (cumulative * 100.0).round() / 100.0,
        });
        val_series.push(PerformanceSeriesPoint {
            date: key,
            value: (value * 100.0).round() / 100.0,
        });
    }

    Ok(PerformanceResponseDto {
        cumulative_capital_series: cap_series,
        current_value_series: val_series,
        currency,
    })
}

/// Loan performance: cumulative = sum(Lend), value = sum(Lend) - sum(Repay) per day.
/// When from_date is empty ("Toàn bộ"), use first transaction date.
pub async fn loan_performance(
    db: &mongodb::Database,
    user_id: ObjectId,
    loan_id: ObjectId,
    from_date: &str,
    to_date: &str,
) -> Result<PerformanceResponseDto, AppError> {
    let loan = repository::loan::get_by_id(db, user_id, loan_id)
        .await
        .map_err(AppError::internal)?
        .ok_or_else(|| AppError::not_found())?;
    let txs = repository::loan_transaction::get_by_loan(db, user_id, loan_id)
        .await
        .map_err(AppError::internal)?;

    let tx_dates: Vec<BsonDateTime> = txs.iter().map(|t| t.date).collect();
    let effective_from = if from_date.trim().is_empty() {
        first_tx_date_iso(&tx_dates).unwrap_or_else(|| to_date.to_string())
    } else {
        from_date.to_string()
    };
    let days = dates_in_range(&effective_from, to_date)?;
    let currency = currency_str(&loan.currency).to_string();
    let mut cap_series = Vec::with_capacity(days.len());
    let mut val_series = Vec::with_capacity(days.len());

    for day_start in &days {
        let end_of_day = *day_start + chrono::Duration::days(1);
        let end_bson = BsonDateTime::from_millis(end_of_day.timestamp_millis());
        let cumulative: f64 = txs
            .iter()
            .filter(|t| t.date < end_bson)
            .filter(|t| matches!(t.r#type, LoanTransactionType::Lend))
            .map(|t| t.amount)
            .sum();
        let value: f64 = txs
            .iter()
            .filter(|t| t.date < end_bson)
            .map(|t| match t.r#type {
                LoanTransactionType::Lend => t.amount,
                LoanTransactionType::Repay => -t.amount,
            })
            .sum();
        let key = date_to_iso(*day_start);
        cap_series.push(PerformanceSeriesPoint {
            date: key.clone(),
            value: (cumulative * 100.0).round() / 100.0,
        });
        val_series.push(PerformanceSeriesPoint {
            date: key,
            value: (value * 100.0).round() / 100.0,
        });
    }

    Ok(PerformanceResponseDto {
        cumulative_capital_series: cap_series,
        current_value_series: val_series,
        currency,
    })
}

/// Depreciating item performance: balance and value = balance * depreciating_item_multiplier per day.
/// When from_date is empty ("Toàn bộ"), use first transaction date.
pub async fn depreciating_performance(
    db: &mongodb::Database,
    user_id: ObjectId,
    item_id: ObjectId,
    from_date: &str,
    to_date: &str,
) -> Result<PerformanceResponseDto, AppError> {
    let item = repository::depreciating_item::get_by_id(db, user_id, item_id)
        .await
        .map_err(AppError::internal)?
        .ok_or_else(|| AppError::not_found())?;
    let txs = repository::depreciating_item_transaction::get_by_item(db, user_id, item_id)
        .await
        .map_err(AppError::internal)?;

    let tx_dates: Vec<BsonDateTime> = txs.iter().map(|t| t.date).collect();
    let effective_from = if from_date.trim().is_empty() {
        first_tx_date_iso(&tx_dates).unwrap_or_else(|| to_date.to_string())
    } else {
        from_date.to_string()
    };
    let days = dates_in_range(&effective_from, to_date)?;
    let currency = currency_str(&item.currency).to_string();
    let mut cap_series = Vec::with_capacity(days.len());
    let mut val_series = Vec::with_capacity(days.len());

    for day_start in &days {
        let end_of_day = *day_start + chrono::Duration::days(1);
        let end_bson = BsonDateTime::from_millis(end_of_day.timestamp_millis());
        let balance: f64 = txs
            .iter()
            .filter(|t| t.date < end_bson)
            .map(|t| match t.r#type {
                DepreciatingItemTransactionType::Deposit => t.amount,
                DepreciatingItemTransactionType::Withdraw => -t.amount,
            })
            .sum();
        let mult = depreciating_item_multiplier(&item.purchase_date, &item.depreciation_curve, *day_start);
        let value = balance * mult;
        let key = date_to_iso(*day_start);
        cap_series.push(PerformanceSeriesPoint {
            date: key.clone(),
            value: (balance * 100.0).round() / 100.0,
        });
        val_series.push(PerformanceSeriesPoint {
            date: key,
            value: (value * 100.0).round() / 100.0,
        });
    }

    Ok(PerformanceResponseDto {
        cumulative_capital_series: cap_series,
        current_value_series: val_series,
        currency,
    })
}

/// Net worth performance: aggregate all portfolio snapshots per day, convert to base_currency.
/// When from_date is empty ("Toàn bộ"), use oldest date in snapshot data.
pub async fn net_worth_performance(
    db: &mongodb::Database,
    user_id: ObjectId,
    from_date: &str,
    to_date: &str,
    base_currency: &str,
) -> Result<PerformanceResponseDto, AppError> {
    let start_bson = if from_date.trim().is_empty() {
        parse_date_iso("2000-01-01").map(|d| BsonDateTime::from_millis(d.timestamp_millis()))
    } else {
        parse_date_iso(from_date).map(|d| BsonDateTime::from_millis(d.timestamp_millis()))
    };
    let dtos = compute_portfolio_snapshots(db, user_id, None, start_bson).await?;
    let portfolios = repository::portfolio::get_all(db, user_id)
        .await
        .map_err(AppError::internal)?;
    let pid_to_currency: HashMap<String, Currency> = portfolios
        .into_iter()
        .filter_map(|p| p.id.map(|id| (id.to_hex(), p.base_currency)))
        .collect();

    let exchange_rates = repository::exchange_rate::get_all(db, start_bson)
        .await
        .map_err(AppError::internal)?;
    let usd_to_vnd: Vec<(BsonDateTime, f64)> = exchange_rates
        .into_iter()
        .filter(|r| r.from_currency == "USD" && r.to_currency == "VND")
        .map(|r| (r.date, r.rate))
        .collect();

    // Group by date: for each date collect (portfolio_id, total_deposit, total_value)
    let mut by_date: HashMap<String, Vec<(String, f64, f64)>> = HashMap::new();
    for d in &dtos {
        if d.date.len() < 10 {
            continue;
        }
        let key = d.date[..10].to_string();
        let cap = d.total_deposit.unwrap_or(0.0);
        let val = d.total_value;
        by_date
            .entry(key)
            .or_default()
            .push((d.portfolio_id.clone(), cap, val));
    }

    let effective_from = if from_date.trim().is_empty() {
        by_date.keys().min().map(String::as_str).unwrap_or(to_date)
    } else {
        from_date
    };
    let days = dates_in_range(effective_from, to_date)?;
    let mut cap_series = Vec::with_capacity(days.len());
    let mut val_series = Vec::with_capacity(days.len());
    let mut last_cap = 0.0;
    let mut last_val = 0.0;

    for day_start in &days {
        let key = date_to_iso(*day_start);
        let day_dt = *day_start;
        let rows = by_date.get(&key).map(|v| v.as_slice()).unwrap_or(&[]);
        let mut total_cap = 0.0;
        let mut total_val = 0.0;
        for (pid, cap, val) in rows {
            let currency = pid_to_currency.get(pid).cloned().unwrap_or(Currency::VND);
            total_cap += value_in_base(*cap, &currency, base_currency, day_dt, &usd_to_vnd);
            total_val += value_in_base(*val, &currency, base_currency, day_dt, &usd_to_vnd);
        }
        if rows.is_empty() {
            total_cap = last_cap;
            total_val = last_val;
        } else {
            last_cap = total_cap;
            last_val = total_val;
        }
        cap_series.push(PerformanceSeriesPoint {
            date: key.clone(),
            value: (total_cap * 100.0).round() / 100.0,
        });
        val_series.push(PerformanceSeriesPoint {
            date: key,
            value: (total_val * 100.0).round() / 100.0,
        });
    }

    Ok(PerformanceResponseDto {
        cumulative_capital_series: cap_series,
        current_value_series: val_series,
        currency: base_currency.to_string(),
    })
}
