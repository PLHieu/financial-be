use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use mongodb::bson::oid::ObjectId;

use crate::context::UserContext;
use crate::convert;
use crate::error::AppError;
use crate::models::{CreatePortfolioRequest, Portfolio};
use crate::performance;
use crate::repository;
use crate::routes::{dto, parse};
use crate::state::AppState;

#[derive(serde::Deserialize)]
pub struct PerformanceQuery {
    pub from_date: String,
    pub to_date: String,
}

pub async fn list(State(state): State<AppState>, ctx: UserContext) -> Result<Json<Vec<crate::models::PortfolioDto>>, AppError> {
    let list = repository::portfolio::get_all(&state.db, ctx.user_id).await.map_err(AppError::internal)?;
    let dtos: Vec<_> = list.iter().map(dto::portfolio_to_dto).collect();
    Ok(Json(dtos))
}

pub async fn create(
    State(state): State<AppState>,
    ctx: UserContext,
    Json(req): Json<CreatePortfolioRequest>,
) -> Result<Json<crate::models::PortfolioDto>, AppError> {
    let r#type = parse::parse_portfolio_type(&req.r#type).map_err(|_| AppError::bad_request())?;
    let base_currency = parse::parse_currency(&req.base_currency).map_err(|_| AppError::bad_request())?;
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
    let id = repository::portfolio::create(&state.db, portfolio).await.map_err(AppError::internal)?;
    let p = repository::portfolio::get_by_id(&state.db, ctx.user_id, id)
        .await
        .map_err(AppError::internal)?
        .ok_or_else(|| AppError::internal("created portfolio not found"))?;
    Ok(Json(dto::portfolio_to_dto(&p)))
}

pub async fn get(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
) -> Result<Json<crate::models::PortfolioDto>, AppError> {
    let oid = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let p = repository::portfolio::get_by_id(&state.db, ctx.user_id, oid)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::not_found())?;
    Ok(Json(dto::portfolio_to_dto(&p)))
}

pub async fn update(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
    Json(req): Json<CreatePortfolioRequest>,
) -> Result<Json<crate::models::PortfolioDto>, AppError> {
    let oid = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let existing = repository::portfolio::get_by_id(&state.db, ctx.user_id, oid)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::not_found())?;
    let r#type = parse::parse_portfolio_type(&req.r#type).map_err(|_| AppError::bad_request())?;
    let base_currency = parse::parse_currency(&req.base_currency).map_err(|_| AppError::bad_request())?;
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
        .map_err(AppError::internal)?;
    let p = repository::portfolio::get_by_id(&state.db, ctx.user_id, oid)
        .await
        .map_err(AppError::internal)?
        .ok_or_else(|| AppError::internal("portfolio not found after update"))?;
    Ok(Json(dto::portfolio_to_dto(&p)))
}

pub async fn delete(State(state): State<AppState>, ctx: UserContext, Path(id): Path<String>) -> Result<StatusCode, AppError> {
    let oid = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let deleted = repository::portfolio::delete(&state.db, ctx.user_id, oid).await.map_err(AppError::internal)?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::not_found())
    }
}

/// GET /portfolios/:id/performance?from_date=&to_date=
pub async fn performance(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
    Query(q): Query<PerformanceQuery>,
) -> Result<Json<crate::models::PerformanceResponseDto>, AppError> {
    let oid = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let out = performance::portfolio_performance(&state.db, ctx.user_id, oid, &q.from_date, &q.to_date).await?;
    Ok(Json(out))
}
