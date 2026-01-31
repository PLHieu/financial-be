use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use mongodb::bson::oid::ObjectId;

use crate::context::UserContext;
use crate::convert;
use crate::models::{CreateTransactionRequest, Transaction};
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
) -> Result<Json<Vec<crate::models::TransactionDto>>, StatusCode> {
    let aid = ObjectId::parse_str(&asset_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let list = repository::transaction::get_by_asset(&state.db, ctx.user_id, aid)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let dtos: Vec<_> = list.iter().map(dto::transaction_to_dto).collect();
    Ok(Json(dtos))
}

pub async fn create(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(asset_id): Path<String>,
    Json(req): Json<CreateTransactionRequest>,
) -> Result<Json<crate::models::TransactionDto>, StatusCode> {
    let aid = ObjectId::parse_str(&asset_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let _ = repository::asset::get_by_id(&state.db, ctx.user_id, aid)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let r#type = parse::parse_transaction_type(&req.r#type).map_err(|_| StatusCode::BAD_REQUEST)?;
    let date = convert::parse_iso_to_bson(&req.date).map_err(|_| StatusCode::BAD_REQUEST)?;
    let now = convert::now_bson();
    let transaction = Transaction {
        id: None,
        user_id: ctx.user_id,
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
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let t = repository::transaction::get_by_asset(&state.db, ctx.user_id, aid)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let t = t.into_iter().find(|x| x.id == Some(id)).ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(dto::transaction_to_dto(&t)))
}

pub async fn list_all(
    State(state): State<AppState>,
    ctx: UserContext,
    Query(q): Query<ListTransactionsQuery>,
) -> Result<Json<Vec<crate::models::TransactionDto>>, StatusCode> {
    let list = repository::transaction::get_all(&state.db, ctx.user_id, q.limit)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let dtos: Vec<_> = list.iter().map(dto::transaction_to_dto).collect();
    Ok(Json(dtos))
}
