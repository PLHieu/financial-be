use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use mongodb::bson::oid::ObjectId;

use crate::context::UserContext;
use crate::convert;
use crate::models::{Asset, AssetMetadata, CreateAssetRequest};
use crate::repository;
use crate::routes::{dto, parse};
use crate::state::AppState;

pub async fn list_by_portfolio(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(portfolio_id): Path<String>,
) -> Result<Json<Vec<crate::models::AssetDto>>, StatusCode> {
    let pid = ObjectId::parse_str(&portfolio_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let list = repository::asset::get_by_portfolio(&state.db, ctx.user_id, pid)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let dtos: Vec<_> = list.iter().map(dto::asset_to_dto).collect();
    Ok(Json(dtos))
}

pub async fn create(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(portfolio_id): Path<String>,
    Json(req): Json<CreateAssetRequest>,
) -> Result<Json<crate::models::AssetDto>, StatusCode> {
    let pid = ObjectId::parse_str(&portfolio_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let _ = repository::portfolio::get_by_id(&state.db, ctx.user_id, pid)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let r#type = parse::parse_asset_type(&req.r#type).map_err(|_| StatusCode::BAD_REQUEST)?;
    let currency = parse::parse_currency(&req.currency).map_err(|_| StatusCode::BAD_REQUEST)?;
    let status = req
        .status
        .as_deref()
        .map(parse::parse_asset_status)
        .transpose()
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .unwrap_or(crate::models::AssetStatus::Active);
    let metadata: Option<AssetMetadata> = req
        .metadata
        .and_then(|v| serde_json::from_value(v).ok());
    let now = convert::now_bson();
    let asset = Asset {
        id: None,
        user_id: ctx.user_id,
        portfolio_id: pid,
        name: req.name,
        r#type,
        currency,
        status,
        metadata,
        created_at: now,
        updated_at: now,
    };
    let id = repository::asset::create(&state.db, asset).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let a = repository::asset::get_by_id(&state.db, ctx.user_id, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(dto::asset_to_dto(&a)))
}

pub async fn get(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
) -> Result<Json<crate::models::AssetDto>, StatusCode> {
    let oid = ObjectId::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let a = repository::asset::get_by_id(&state.db, ctx.user_id, oid)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(dto::asset_to_dto(&a)))
}

pub async fn update_status(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<crate::models::AssetDto>, StatusCode> {
    let oid = ObjectId::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let status = body
        .get("status")
        .and_then(|v| v.as_str())
        .ok_or(StatusCode::BAD_REQUEST)?;
    let status_enum = parse::parse_asset_status(status).map_err(|_| StatusCode::BAD_REQUEST)?;
    let status_str = match status_enum {
        crate::models::AssetStatus::Active => "Active",
        crate::models::AssetStatus::Closed => "Closed",
    };
    let now = convert::now_bson();
    repository::asset::update_status(&state.db, ctx.user_id, oid, status_str, now)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let a = repository::asset::get_by_id(&state.db, ctx.user_id, oid)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(dto::asset_to_dto(&a)))
}
