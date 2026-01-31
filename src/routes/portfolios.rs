use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use mongodb::bson::oid::ObjectId;

use crate::context::UserContext;
use crate::convert;
use crate::models::{CreatePortfolioRequest, Portfolio};
use crate::repository;
use crate::routes::{dto, parse};
use crate::state::AppState;

pub async fn list(State(state): State<AppState>, ctx: UserContext) -> Result<Json<Vec<crate::models::PortfolioDto>>, StatusCode> {
    let list = repository::portfolio::get_all(&state.db, ctx.user_id).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let dtos: Vec<_> = list.iter().map(dto::portfolio_to_dto).collect();
    Ok(Json(dtos))
}

pub async fn create(
    State(state): State<AppState>,
    ctx: UserContext,
    Json(req): Json<CreatePortfolioRequest>,
) -> Result<Json<crate::models::PortfolioDto>, StatusCode> {
    let r#type = parse::parse_portfolio_type(&req.r#type).map_err(|_| StatusCode::BAD_REQUEST)?;
    let base_currency = parse::parse_currency(&req.base_currency).map_err(|_| StatusCode::BAD_REQUEST)?;
    let now = convert::now_bson();
    let portfolio = Portfolio {
        id: None,
        user_id: ctx.user_id,
        name: req.name,
        r#type,
        base_currency,
        description: req.description,
        color: req.color,
        created_at: now,
        updated_at: now,
    };
    let id = repository::portfolio::create(&state.db, portfolio).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let p = repository::portfolio::get_by_id(&state.db, ctx.user_id, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(dto::portfolio_to_dto(&p)))
}

pub async fn get(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
) -> Result<Json<crate::models::PortfolioDto>, StatusCode> {
    let oid = ObjectId::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let p = repository::get_by_id(&state.db, ctx.user_id, oid)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(dto::portfolio_to_dto(&p)))
}

pub async fn update(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
    Json(req): Json<CreatePortfolioRequest>,
) -> Result<Json<crate::models::PortfolioDto>, StatusCode> {
    let oid = ObjectId::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let existing = repository::portfolio::get_by_id(&state.db, ctx.user_id, oid)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let r#type = parse::parse_portfolio_type(&req.r#type).map_err(|_| StatusCode::BAD_REQUEST)?;
    let base_currency = parse::parse_currency(&req.base_currency).map_err(|_| StatusCode::BAD_REQUEST)?;
    let now = convert::now_bson();
    let portfolio = Portfolio {
        id: Some(oid),
        user_id: ctx.user_id,
        name: req.name,
        r#type,
        base_currency,
        description: req.description,
        color: req.color,
        created_at: existing.created_at,
        updated_at: now,
    };
    repository::portfolio::update(&state.db, ctx.user_id, oid, portfolio)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let p = repository::portfolio::get_by_id(&state.db, ctx.user_id, oid)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(dto::portfolio_to_dto(&p)))
}

pub async fn delete(State(state): State<AppState>, ctx: UserContext, Path(id): Path<String>) -> Result<StatusCode, StatusCode> {
    let oid = ObjectId::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let deleted = repository::portfolio::delete(&state.db, ctx.user_id, oid).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
