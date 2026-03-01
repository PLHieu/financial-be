use axum::{
    extract::{Path, Query, State},
    Json,
};
use mongodb::bson::oid::ObjectId;

use crate::context::UserContext;
use crate::convert;
use crate::error::AppError;
use crate::models::{CreateFundRequest, CreateFundTransactionRequest, Fund, FundTransaction};
use crate::performance;
use crate::repository;
use crate::routes::{dto, parse};
use crate::state::AppState;

#[derive(serde::Deserialize)]
pub struct PerformanceQuery {
    pub from_date: String,
    pub to_date: String,
}

#[derive(serde::Deserialize)]
pub struct FundsQuery {
    pub portfolio_id: Option<String>,
}

/// GET /funds – list all funds for user, optionally filter by portfolio_id.
pub async fn list(
    State(state): State<AppState>,
    ctx: UserContext,
    Query(q): Query<FundsQuery>,
) -> Result<Json<Vec<crate::models::FundDto>>, AppError> {
    let list = if let Some(pid_str) = &q.portfolio_id {
        let pid = ObjectId::parse_str(pid_str).map_err(|_| AppError::bad_request())?;
        repository::fund::get_by_portfolio(state.db.as_ref(), ctx.user_id, pid)
            .await
            .map_err(AppError::internal)?
    } else {
        repository::fund::get_all(state.db.as_ref(), ctx.user_id)
            .await
            .map_err(AppError::internal)?
    };
    let dtos: Vec<_> = list.iter().map(dto::fund_to_dto).collect();
    Ok(Json(dtos))
}

/// POST /funds – create a fund.
pub async fn create(
    State(state): State<AppState>,
    ctx: UserContext,
    Json(req): Json<CreateFundRequest>,
) -> Result<Json<crate::models::FundDto>, AppError> {
    let currency = parse::parse_currency(&req.currency).map_err(|_| AppError::bad_request())?;
    let portfolio_id = req
        .portfolio_id
        .as_ref()
        .and_then(|s| ObjectId::parse_str(s).ok());
    let now = convert::now_bson();
    let fund = Fund {
        id: None,
        user_id: ctx.user_id,
        portfolio_id,
        name: req.name,
        currency,
        fund_type: req.fund_type,
        start_date: req.start_date,
        status: crate::models::AssetStatus::Active,
        metadata: req.metadata,
        inflation_rate: req.inflation_rate,
        created_at: now,
        updated_at: now,
    };
    let id = repository::fund::create(state.db.as_ref(), fund)
        .await
        .map_err(AppError::internal)?;
    let f = repository::fund::get_by_id(state.db.as_ref(), ctx.user_id, id)
        .await
        .map_err(AppError::internal)?
        .ok_or_else(|| AppError::internal("created fund not found"))?;
    Ok(Json(dto::fund_to_dto(&f)))
}

/// GET /funds/:id – get one fund.
pub async fn get(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
) -> Result<Json<crate::models::FundDto>, AppError> {
    let oid = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let f = repository::fund::get_by_id(state.db.as_ref(), ctx.user_id, oid)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::not_found())?;
    Ok(Json(dto::fund_to_dto(&f)))
}

/// GET /funds/:id/transactions – list transactions for this fund.
pub async fn list_transactions(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
) -> Result<Json<Vec<crate::models::FundTransactionDto>>, AppError> {
    let fund_id = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let _ = repository::fund::get_by_id(state.db.as_ref(), ctx.user_id, fund_id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::not_found())?;
    let list = repository::fund_transaction::get_by_fund(state.db.as_ref(), ctx.user_id, fund_id)
        .await
        .map_err(AppError::internal)?;
    let dtos: Vec<_> = list.iter().map(dto::fund_transaction_to_dto).collect();
    Ok(Json(dtos))
}

/// POST /funds/:id/transactions – create a fund transaction (Deposit or Withdraw).
pub async fn create_transaction(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
    Json(req): Json<CreateFundTransactionRequest>,
) -> Result<Json<crate::models::FundTransactionDto>, AppError> {
    let fund_id = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let _ = repository::fund::get_by_id(state.db.as_ref(), ctx.user_id, fund_id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::not_found())?;
    let r#type = parse::parse_fund_transaction_type(&req.r#type).map_err(|_| AppError::bad_request())?;
    let date = convert::parse_iso_to_bson(&req.date).map_err(|_| AppError::bad_request())?;
    let now = convert::now_bson();
    let tx = FundTransaction {
        id: None,
        user_id: ctx.user_id,
        fund_id,
        r#type,
        amount: req.amount,
        date,
        note: req.note,
        created_at: now,
        updated_at: now,
    };
    let _ = repository::fund_transaction::create(state.db.as_ref(), tx)
        .await
        .map_err(AppError::internal)?;
    let list = repository::fund_transaction::get_by_fund(state.db.as_ref(), ctx.user_id, fund_id)
        .await
        .map_err(AppError::internal)?;
    let t = list
        .last()
        .ok_or_else(|| AppError::internal("created transaction not found"))?;
    Ok(Json(dto::fund_transaction_to_dto(t)))
}

/// GET /funds/:id/performance?from_date=&to_date=
pub async fn performance(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
    Query(q): Query<PerformanceQuery>,
) -> Result<Json<crate::models::PerformanceResponseDto>, AppError> {
    let oid = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let out = performance::fund_performance(state.db.as_ref(), ctx.user_id, oid, &q.from_date, &q.to_date).await?;
    Ok(Json(out))
}

/// DELETE /funds/:id – delete fund and all its transactions.
pub async fn delete(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
) -> Result<axum::http::StatusCode, AppError> {
    use axum::http::StatusCode;
    let oid = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let _ = repository::fund::get_by_id(state.db.as_ref(), ctx.user_id, oid)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::not_found())?;
    let _ = repository::fund_transaction::delete_by_fund(state.db.as_ref(), ctx.user_id, oid)
        .await
        .map_err(AppError::internal)?;
    let deleted = repository::fund::delete(state.db.as_ref(), ctx.user_id, oid)
        .await
        .map_err(AppError::internal)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found())
    }
}
