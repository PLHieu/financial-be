use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use mongodb::bson::oid::ObjectId;

use crate::context::UserContext;
use crate::convert;
use crate::error::AppError;
use crate::models::{CreatePortfolioTransactionRequest, CreateTransactionRequest, Transaction};
use crate::repository;
use crate::routes::{dto, parse};
use crate::state::AppState;

#[derive(serde::Deserialize)]
pub struct ListTransactionsQuery {
    pub limit: Option<u64>,
}

pub async fn list_by_asset(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(asset_id): Path<String>,
) -> Result<Json<Vec<crate::models::TransactionDto>>, AppError> {
    let aid = ObjectId::parse_str(&asset_id).map_err(|_| AppError::bad_request())?;
    let list = repository::transaction::get_by_asset(&state.db, ctx.user_id, aid)
        .await
        .map_err(AppError::internal)?;
    let dtos: Vec<_> = list.iter().map(dto::transaction_to_dto).collect();
    Ok(Json(dtos))
}

pub async fn create(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(asset_id): Path<String>,
    Json(req): Json<CreateTransactionRequest>,
) -> Result<Json<crate::models::TransactionDto>, AppError> {
    let aid = ObjectId::parse_str(&asset_id).map_err(|_| AppError::bad_request())?;
    let _ = repository::asset::get_by_id(&state.db, ctx.user_id, aid)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::not_found())?;
    let r#type = parse::parse_transaction_type(&req.r#type).map_err(|_| AppError::bad_request())?;
    let date = convert::parse_iso_to_bson(&req.date).map_err(|_| AppError::bad_request())?;
    let now = convert::now_bson();
    let transaction = Transaction {
        id: None,
        user_id: ctx.user_id,
        portfolio_id: None,
        asset_id: aid,
        r#type,
        amount: req.amount,
        price: req.price,
        quantity: req.quantity,
        note: req.note,
        date,
        created_at: now,
        updated_at: now,
    };
    let id = repository::transaction::create(&state.db, transaction)
        .await
        .map_err(AppError::internal)?;
    let t = repository::transaction::get_by_asset(&state.db, ctx.user_id, aid)
        .await
        .map_err(AppError::internal)?;
    let t = t.into_iter().find(|x| x.id == Some(id)).ok_or_else(|| AppError::internal("created transaction not found"))?;
    Ok(Json(dto::transaction_to_dto(&t)))
}

/// GET /portfolios/:portfolio_id/transactions – list transactions in this portfolio.
pub async fn list_by_portfolio(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(portfolio_id): Path<String>,
) -> Result<Json<Vec<crate::models::TransactionDto>>, AppError> {
    let pid = ObjectId::parse_str(&portfolio_id).map_err(|_| AppError::bad_request())?;
    let list = repository::transaction::get_by_portfolio(&state.db, ctx.user_id, pid)
        .await
        .map_err(AppError::internal)?;
    let dtos: Vec<_> = list.iter().map(dto::transaction_to_dto).collect();
    Ok(Json(dtos))
}

/// POST /portfolios/:portfolio_id/transactions – create transaction in portfolio (select asset from global list).
pub async fn create_for_portfolio(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(portfolio_id): Path<String>,
    Json(req): Json<CreatePortfolioTransactionRequest>,
) -> Result<Json<crate::models::TransactionDto>, AppError> {
    let pid = ObjectId::parse_str(&portfolio_id).map_err(|_| AppError::bad_request())?;
    let aid = ObjectId::parse_str(&req.asset_id).map_err(|_| AppError::bad_request())?;
    let _ = repository::portfolio::get_by_id(&state.db, ctx.user_id, pid)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::not_found())?;
    let _ = repository::asset::get_by_id(&state.db, ctx.user_id, aid)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::not_found())?;
    let r#type = parse::parse_transaction_type(&req.r#type).map_err(|_| AppError::bad_request())?;
    let date = convert::parse_iso_to_bson(&req.date).map_err(|_| AppError::bad_request())?;
    let now = convert::now_bson();
    let transaction = Transaction {
        id: None,
        user_id: ctx.user_id,
        portfolio_id: Some(pid),
        asset_id: aid,
        r#type,
        amount: req.amount,
        price: req.price,
        quantity: req.quantity,
        note: req.note,
        date,
        created_at: now,
        updated_at: now,
    };
    let id = repository::transaction::create(&state.db, transaction)
        .await
        .map_err(AppError::internal)?;
    let list = repository::transaction::get_by_portfolio(&state.db, ctx.user_id, pid)
        .await
        .map_err(AppError::internal)?;
    let t = list.into_iter().find(|x| x.id == Some(id)).ok_or_else(|| AppError::internal("created transaction not found"))?;
    Ok(Json(dto::transaction_to_dto(&t)))
}

pub async fn list_all(
    State(state): State<AppState>,
    ctx: UserContext,
    Query(q): Query<ListTransactionsQuery>,
) -> Result<Json<Vec<crate::models::TransactionDto>>, AppError> {
    let list = repository::transaction::get_all(&state.db, ctx.user_id, q.limit)
        .await
        .map_err(AppError::internal)?;
    let dtos: Vec<_> = list.iter().map(dto::transaction_to_dto).collect();
    Ok(Json(dtos))
}

pub async fn delete(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let oid = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let deleted = repository::transaction::delete_by_id(&state.db, ctx.user_id, oid)
        .await
        .map_err(AppError::internal)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found())
    }
}
