use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use mongodb::bson::oid::ObjectId;

use crate::context::UserContext;
use crate::convert;
use crate::models::{CreateTransferRequest, TransferTransaction};
use crate::repository;
use crate::routes::dto;
use crate::state::AppState;

#[derive(serde::Deserialize)]
pub struct ListTransfersQuery {
    pub from_portfolio_id: Option<String>,
    pub to_portfolio_id: Option<String>,
}

pub async fn create(
    State(state): State<AppState>,
    ctx: UserContext,
    Json(req): Json<CreateTransferRequest>,
) -> Result<Json<crate::models::TransferTransactionDto>, StatusCode> {
    let from_portfolio_id = ObjectId::parse_str(&req.from_portfolio_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let to_portfolio_id = ObjectId::parse_str(&req.to_portfolio_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let date = convert::parse_iso_to_bson(&req.date).map_err(|_| StatusCode::BAD_REQUEST)?;
    let exchange_rate_id = req
        .exchange_rate_id
        .as_ref()
        .and_then(|s| ObjectId::parse_str(s).ok());
    let now = convert::now_bson();
    let transfer = TransferTransaction {
        id: None,
        user_id: ctx.user_id,
        from_portfolio_id,
        to_portfolio_id,
        amount: req.amount,
        from_currency: req.from_currency,
        to_currency: req.to_currency,
        exchange_rate_id,
        converted_amount: req.converted_amount,
        date,
        note: req.note,
        created_at: now,
    };
    let id = repository::transfer_transaction::create(&state.db, transfer)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let t = repository::transfer_transaction::get_by_id(&state.db, ctx.user_id, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(dto::transfer_transaction_to_dto(&t)))
}

pub async fn list(
    State(state): State<AppState>,
    ctx: UserContext,
    Query(q): Query<ListTransfersQuery>,
) -> Result<Json<Vec<crate::models::TransferTransactionDto>>, StatusCode> {
    let from_portfolio_id = q.from_portfolio_id.as_ref().and_then(|s| ObjectId::parse_str(s).ok());
    let to_portfolio_id = q.to_portfolio_id.as_ref().and_then(|s| ObjectId::parse_str(s).ok());
    let list = repository::transfer_transaction::get_all(&state.db, ctx.user_id, from_portfolio_id, to_portfolio_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let dtos: Vec<_> = list.iter().map(dto::transfer_transaction_to_dto).collect();
    Ok(Json(dtos))
}
