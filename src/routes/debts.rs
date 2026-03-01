use axum::{
    extract::{Path, Query, State},
    Json,
};
use mongodb::bson::oid::ObjectId;

use crate::context::UserContext;
use crate::convert;
use crate::error::AppError;
use crate::models::{CreateDebtRequest, CreateDebtTransactionRequest, Debt, DebtTransaction};
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
pub struct DebtsQuery {
    pub portfolio_id: Option<String>,
}

/// GET /debts – list all debts for user, optionally filter by portfolio_id.
pub async fn list(
    State(state): State<AppState>,
    ctx: UserContext,
    Query(q): Query<DebtsQuery>,
) -> Result<Json<Vec<crate::models::DebtDto>>, AppError> {
    let list = if let Some(pid_str) = &q.portfolio_id {
        let pid = ObjectId::parse_str(pid_str).map_err(|_| AppError::bad_request())?;
        repository::debt::get_by_portfolio(state.db.as_ref(), ctx.user_id, pid)
            .await
            .map_err(AppError::internal)?
    } else {
        repository::debt::get_all(state.db.as_ref(), ctx.user_id)
            .await
            .map_err(AppError::internal)?
    };
    let dtos: Vec<_> = list.iter().map(dto::debt_to_dto).collect();
    Ok(Json(dtos))
}

/// POST /debts – create a debt.
pub async fn create(
    State(state): State<AppState>,
    ctx: UserContext,
    Json(req): Json<CreateDebtRequest>,
) -> Result<Json<crate::models::DebtDto>, AppError> {
    let currency = parse::parse_currency(&req.currency).map_err(|_| AppError::bad_request())?;
    let portfolio_id = req
        .portfolio_id
        .as_ref()
        .and_then(|s| ObjectId::parse_str(s).ok());
    let now = convert::now_bson();
    let debt = Debt {
        id: None,
        user_id: ctx.user_id,
        portfolio_id,
        name: req.name,
        currency,
        status: crate::models::AssetStatus::Active,
        metadata: req.metadata,
        created_at: now,
        updated_at: now,
    };
    let id = repository::debt::create(state.db.as_ref(), debt)
        .await
        .map_err(AppError::internal)?;
    let d = repository::debt::get_by_id(state.db.as_ref(), ctx.user_id, id)
        .await
        .map_err(AppError::internal)?
        .ok_or_else(|| AppError::internal("created debt not found"))?;
    Ok(Json(dto::debt_to_dto(&d)))
}

/// GET /debts/:id – get one debt.
pub async fn get(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
) -> Result<Json<crate::models::DebtDto>, AppError> {
    let oid = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let d = repository::debt::get_by_id(state.db.as_ref(), ctx.user_id, oid)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::not_found())?;
    Ok(Json(dto::debt_to_dto(&d)))
}

/// GET /debts/:id/transactions – list transactions for this debt.
pub async fn list_transactions(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
) -> Result<Json<Vec<crate::models::DebtTransactionDto>>, AppError> {
    let debt_id = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let _ = repository::debt::get_by_id(state.db.as_ref(), ctx.user_id, debt_id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::not_found())?;
    let list = repository::debt_transaction::get_by_debt(state.db.as_ref(), ctx.user_id, debt_id)
        .await
        .map_err(AppError::internal)?;
    let dtos: Vec<_> = list.iter().map(dto::debt_transaction_to_dto).collect();
    Ok(Json(dtos))
}

/// POST /debts/:id/transactions – create a debt transaction (Borrow or Repay).
pub async fn create_transaction(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
    Json(req): Json<CreateDebtTransactionRequest>,
) -> Result<Json<crate::models::DebtTransactionDto>, AppError> {
    let debt_id = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let _ = repository::debt::get_by_id(state.db.as_ref(), ctx.user_id, debt_id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::not_found())?;
    let r#type = parse::parse_debt_transaction_type(&req.r#type).map_err(|_| AppError::bad_request())?;
    let date = convert::parse_iso_to_bson(&req.date).map_err(|_| AppError::bad_request())?;
    let now = convert::now_bson();
    let tx = DebtTransaction {
        id: None,
        user_id: ctx.user_id,
        debt_id,
        r#type,
        amount: req.amount,
        date,
        note: req.note,
        created_at: now,
        updated_at: now,
    };
    let _ = repository::debt_transaction::create(state.db.as_ref(), tx)
        .await
        .map_err(AppError::internal)?;
    let list = repository::debt_transaction::get_by_debt(state.db.as_ref(), ctx.user_id, debt_id)
        .await
        .map_err(AppError::internal)?;
    let t = list
        .last()
        .ok_or_else(|| AppError::internal("created transaction not found"))?;
    Ok(Json(dto::debt_transaction_to_dto(t)))
}

/// GET /debts/:id/performance?from_date=&to_date=
pub async fn performance(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
    Query(q): Query<PerformanceQuery>,
) -> Result<Json<crate::models::PerformanceResponseDto>, AppError> {
    let oid = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let out = performance::debt_performance(state.db.as_ref(), ctx.user_id, oid, &q.from_date, &q.to_date).await?;
    Ok(Json(out))
}

/// DELETE /debts/:id – delete debt and all its transactions.
pub async fn delete(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
) -> Result<axum::http::StatusCode, AppError> {
    use axum::http::StatusCode;
    let oid = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let _ = repository::debt::get_by_id(state.db.as_ref(), ctx.user_id, oid)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::not_found())?;
    let _ = repository::debt_transaction::delete_by_debt(state.db.as_ref(), ctx.user_id, oid)
        .await
        .map_err(AppError::internal)?;
    let deleted = repository::debt::delete(state.db.as_ref(), ctx.user_id, oid)
        .await
        .map_err(AppError::internal)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found())
    }
}
