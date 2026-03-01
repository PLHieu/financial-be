use axum::{
    extract::{Path, Query, State},
    Json,
};
use mongodb::bson::oid::ObjectId;

use crate::context::UserContext;
use crate::convert;
use crate::error::AppError;
use crate::models::{CreateLoanRequest, CreateLoanTransactionRequest, Loan, LoanTransaction};
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
pub struct LoansQuery {
    pub portfolio_id: Option<String>,
}

/// GET /loans – list all loans for user, optionally filter by portfolio_id.
pub async fn list(
    State(state): State<AppState>,
    ctx: UserContext,
    Query(q): Query<LoansQuery>,
) -> Result<Json<Vec<crate::models::LoanDto>>, AppError> {
    let list = if let Some(pid_str) = &q.portfolio_id {
        let pid = ObjectId::parse_str(pid_str).map_err(|_| AppError::bad_request())?;
        repository::loan::get_by_portfolio(state.db.as_ref(), ctx.user_id, pid)
            .await
            .map_err(AppError::internal)?
    } else {
        repository::loan::get_all(state.db.as_ref(), ctx.user_id)
            .await
            .map_err(AppError::internal)?
    };
    let dtos: Vec<_> = list.iter().map(dto::loan_to_dto).collect();
    Ok(Json(dtos))
}

/// POST /loans – create a loan.
pub async fn create(
    State(state): State<AppState>,
    ctx: UserContext,
    Json(req): Json<CreateLoanRequest>,
) -> Result<Json<crate::models::LoanDto>, AppError> {
    let currency = parse::parse_currency(&req.currency).map_err(|_| AppError::bad_request())?;
    let portfolio_id = req
        .portfolio_id
        .as_ref()
        .and_then(|s| ObjectId::parse_str(s).ok());
    let now = convert::now_bson();
    let loan = Loan {
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
    let id = repository::loan::create(state.db.as_ref(), loan)
        .await
        .map_err(AppError::internal)?;
    let l = repository::loan::get_by_id(state.db.as_ref(), ctx.user_id, id)
        .await
        .map_err(AppError::internal)?
        .ok_or_else(|| AppError::internal("created loan not found"))?;
    Ok(Json(dto::loan_to_dto(&l)))
}

/// GET /loans/:id – get one loan.
pub async fn get(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
) -> Result<Json<crate::models::LoanDto>, AppError> {
    let oid = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let l = repository::loan::get_by_id(state.db.as_ref(), ctx.user_id, oid)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::not_found())?;
    Ok(Json(dto::loan_to_dto(&l)))
}

/// GET /loans/:id/transactions – list transactions for this loan.
pub async fn list_transactions(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
) -> Result<Json<Vec<crate::models::LoanTransactionDto>>, AppError> {
    let loan_id = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let _ = repository::loan::get_by_id(state.db.as_ref(), ctx.user_id, loan_id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::not_found())?;
    let list = repository::loan_transaction::get_by_loan(state.db.as_ref(), ctx.user_id, loan_id)
        .await
        .map_err(AppError::internal)?;
    let dtos: Vec<_> = list.iter().map(dto::loan_transaction_to_dto).collect();
    Ok(Json(dtos))
}

/// POST /loans/:id/transactions – create a loan transaction (Lend or Repay).
pub async fn create_transaction(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
    Json(req): Json<CreateLoanTransactionRequest>,
) -> Result<Json<crate::models::LoanTransactionDto>, AppError> {
    let loan_id = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let _ = repository::loan::get_by_id(state.db.as_ref(), ctx.user_id, loan_id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::not_found())?;
    let r#type = parse::parse_loan_transaction_type(&req.r#type).map_err(|_| AppError::bad_request())?;
    let date = convert::parse_iso_to_bson(&req.date).map_err(|_| AppError::bad_request())?;
    let now = convert::now_bson();
    let tx = LoanTransaction {
        id: None,
        user_id: ctx.user_id,
        loan_id,
        r#type,
        amount: req.amount,
        date,
        note: req.note,
        created_at: now,
        updated_at: now,
    };
    let _ = repository::loan_transaction::create(state.db.as_ref(), tx)
        .await
        .map_err(AppError::internal)?;
    let list = repository::loan_transaction::get_by_loan(state.db.as_ref(), ctx.user_id, loan_id)
        .await
        .map_err(AppError::internal)?;
    let t = list
        .last()
        .ok_or_else(|| AppError::internal("created transaction not found"))?;
    Ok(Json(dto::loan_transaction_to_dto(t)))
}

/// GET /loans/:id/performance?from_date=&to_date=
pub async fn performance(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
    Query(q): Query<PerformanceQuery>,
) -> Result<Json<crate::models::PerformanceResponseDto>, AppError> {
    let oid = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let out = performance::loan_performance(state.db.as_ref(), ctx.user_id, oid, &q.from_date, &q.to_date).await?;
    Ok(Json(out))
}

/// DELETE /loans/:id – delete loan and all its transactions.
pub async fn delete(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
) -> Result<axum::http::StatusCode, AppError> {
    use axum::http::StatusCode;
    let oid = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let _ = repository::loan::get_by_id(state.db.as_ref(), ctx.user_id, oid)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::not_found())?;
    let _ = repository::loan_transaction::delete_by_loan(state.db.as_ref(), ctx.user_id, oid)
        .await
        .map_err(AppError::internal)?;
    let deleted = repository::loan::delete(state.db.as_ref(), ctx.user_id, oid)
        .await
        .map_err(AppError::internal)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found())
    }
}
