use axum::{
    extract::{Path, Query, State},
    Json,
};
use mongodb::bson::oid::ObjectId;

use crate::context::UserContext;
use crate::convert;
use crate::error::AppError;
use crate::models::{
    CreateDepreciatingItemRequest, CreateDepreciatingItemTransactionRequest, DepreciatingItem,
    DepreciatingItemTransaction,
};
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
pub struct DepreciatingItemsQuery {
    pub portfolio_id: Option<String>,
}

/// GET /depreciating-items – list all items for user, optionally filter by portfolio_id.
pub async fn list(
    State(state): State<AppState>,
    ctx: UserContext,
    Query(q): Query<DepreciatingItemsQuery>,
) -> Result<Json<Vec<crate::models::DepreciatingItemDto>>, AppError> {
    let list = if let Some(pid_str) = &q.portfolio_id {
        let pid = ObjectId::parse_str(pid_str).map_err(|_| AppError::bad_request())?;
        repository::depreciating_item::get_by_portfolio(state.db.as_ref(), ctx.user_id, pid)
            .await
            .map_err(AppError::internal)?
    } else {
        repository::depreciating_item::get_all(state.db.as_ref(), ctx.user_id)
            .await
            .map_err(AppError::internal)?
    };
    let dtos: Vec<_> = list.iter().map(dto::depreciating_item_to_dto).collect();
    Ok(Json(dtos))
}

/// POST /depreciating-items – create a depreciating item.
pub async fn create(
    State(state): State<AppState>,
    ctx: UserContext,
    Json(req): Json<CreateDepreciatingItemRequest>,
) -> Result<Json<crate::models::DepreciatingItemDto>, AppError> {
    let currency = parse::parse_currency(&req.currency).map_err(|_| AppError::bad_request())?;
    let portfolio_id = req
        .portfolio_id
        .as_ref()
        .and_then(|s| ObjectId::parse_str(s).ok());
    let now = convert::now_bson();
    let item = DepreciatingItem {
        id: None,
        user_id: ctx.user_id,
        portfolio_id,
        name: req.name,
        currency,
        purchase_date: req.purchase_date,
        depreciation_curve: req.depreciation_curve,
        status: crate::models::AssetStatus::Active,
        metadata: req.metadata,
        created_at: now,
        updated_at: now,
    };
    let id = repository::depreciating_item::create(state.db.as_ref(), item)
        .await
        .map_err(AppError::internal)?;
    let i = repository::depreciating_item::get_by_id(state.db.as_ref(), ctx.user_id, id)
        .await
        .map_err(AppError::internal)?
        .ok_or_else(|| AppError::internal("created item not found"))?;
    Ok(Json(dto::depreciating_item_to_dto(&i)))
}

/// GET /depreciating-items/:id – get one item.
pub async fn get(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
) -> Result<Json<crate::models::DepreciatingItemDto>, AppError> {
    let oid = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let i = repository::depreciating_item::get_by_id(state.db.as_ref(), ctx.user_id, oid)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::not_found())?;
    Ok(Json(dto::depreciating_item_to_dto(&i)))
}

/// GET /depreciating-items/:id/transactions – list transactions for this item.
pub async fn list_transactions(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
) -> Result<Json<Vec<crate::models::DepreciatingItemTransactionDto>>, AppError> {
    let item_id = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let _ = repository::depreciating_item::get_by_id(state.db.as_ref(), ctx.user_id, item_id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::not_found())?;
    let list =
        repository::depreciating_item_transaction::get_by_item(state.db.as_ref(), ctx.user_id, item_id)
            .await
            .map_err(AppError::internal)?;
    let dtos: Vec<_> = list.iter().map(dto::depreciating_item_transaction_to_dto).collect();
    Ok(Json(dtos))
}

/// POST /depreciating-items/:id/transactions – create a transaction (Deposit or Withdraw).
pub async fn create_transaction(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
    Json(req): Json<CreateDepreciatingItemTransactionRequest>,
) -> Result<Json<crate::models::DepreciatingItemTransactionDto>, AppError> {
    let item_id = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let _ = repository::depreciating_item::get_by_id(state.db.as_ref(), ctx.user_id, item_id)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::not_found())?;
    let r#type = parse::parse_depreciating_item_transaction_type(&req.r#type)
        .map_err(|_| AppError::bad_request())?;
    let date = convert::parse_iso_to_bson(&req.date).map_err(|_| AppError::bad_request())?;
    let now = convert::now_bson();
    let tx = DepreciatingItemTransaction {
        id: None,
        user_id: ctx.user_id,
        depreciating_item_id: item_id,
        r#type,
        amount: req.amount,
        date,
        note: req.note,
        created_at: now,
        updated_at: now,
    };
    let _ = repository::depreciating_item_transaction::create(state.db.as_ref(), tx)
        .await
        .map_err(AppError::internal)?;
    let list =
        repository::depreciating_item_transaction::get_by_item(state.db.as_ref(), ctx.user_id, item_id)
            .await
            .map_err(AppError::internal)?;
    let t = list
        .last()
        .ok_or_else(|| AppError::internal("created transaction not found"))?;
    Ok(Json(dto::depreciating_item_transaction_to_dto(t)))
}

/// GET /depreciating-items/:id/performance?from_date=&to_date=
pub async fn performance(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
    Query(q): Query<PerformanceQuery>,
) -> Result<Json<crate::models::PerformanceResponseDto>, AppError> {
    let oid = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let out = performance::depreciating_performance(state.db.as_ref(), ctx.user_id, oid, &q.from_date, &q.to_date).await?;
    Ok(Json(out))
}

/// DELETE /depreciating-items/:id – delete item and all its transactions.
pub async fn delete(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
) -> Result<axum::http::StatusCode, AppError> {
    use axum::http::StatusCode;
    let oid = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let _ = repository::depreciating_item::get_by_id(state.db.as_ref(), ctx.user_id, oid)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::not_found())?;
    let _ = repository::depreciating_item_transaction::delete_by_item(state.db.as_ref(), ctx.user_id, oid)
        .await
        .map_err(AppError::internal)?;
    let deleted = repository::depreciating_item::delete(state.db.as_ref(), ctx.user_id, oid)
        .await
        .map_err(AppError::internal)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found())
    }
}
