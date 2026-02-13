use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use mongodb::bson::oid::ObjectId;

use crate::context::UserContext;
use crate::convert;
use crate::error::AppError;
use crate::models::{Asset, AssetMetadata, CreateAssetRequest, CreateGlobalAssetRequest, UpdateAssetRequest};
use crate::repository;
use crate::routes::{dto, parse};
use crate::state::AppState;

/// GET /assets – list all assets (global layer).
pub async fn list_all(
    State(state): State<AppState>,
    ctx: UserContext,
) -> Result<Json<Vec<crate::models::AssetDto>>, AppError> {
    let list: Vec<Asset> = repository::asset::get_all(state.db.as_ref(), ctx.user_id)
        .await
        .map_err(AppError::internal)?;
    let dtos: Vec<crate::models::AssetDto> = list.iter().map(dto::asset_to_dto).collect();
    Ok(Json(dtos))
}

/// POST /assets – create global asset (no portfolio).
pub async fn create_global(
    State(state): State<AppState>,
    ctx: UserContext,
    Json(req): Json<CreateGlobalAssetRequest>,
) -> Result<Json<crate::models::AssetDto>, AppError> {
    let r#type = parse::parse_asset_type(&req.r#type).map_err(|_| AppError::bad_request())?;
    let currency = parse::parse_currency(&req.currency).map_err(|_| AppError::bad_request())?;
    let status = req
        .status
        .as_deref()
        .map(parse::parse_asset_status)
        .transpose()
        .map_err(|_| AppError::bad_request())?
        .unwrap_or(crate::models::AssetStatus::Active);
    let metadata: Option<AssetMetadata> = req.metadata.and_then(|v| serde_json::from_value(v).ok());
    let now = convert::now_bson();
    let asset = Asset {
        id: None,
        user_id: ctx.user_id,
        portfolio_id: None,
        name: req.name,
        r#type,
        currency,
        status,
        metadata,
        created_at: now,
        updated_at: now,
    };
    let id = repository::asset::create(state.db.as_ref(), asset).await.map_err(AppError::internal)?;
    let a: Asset = repository::asset::get_by_id(state.db.as_ref(), ctx.user_id, id)
        .await
        .map_err(AppError::internal)?
        .ok_or_else(|| AppError::internal("created asset not found"))?;
    Ok(Json(dto::asset_to_dto(&a)))
}

pub async fn list_by_portfolio(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(portfolio_id): Path<String>,
) -> Result<Json<Vec<crate::models::AssetDto>>, AppError> {
    let pid = ObjectId::parse_str(&portfolio_id).map_err(|_| AppError::bad_request())?;
    let list: Vec<Asset> = repository::asset::get_by_portfolio(state.db.as_ref(), ctx.user_id, pid)
        .await
        .map_err(AppError::internal)?;
    let dtos: Vec<crate::models::AssetDto> = list.iter().map(dto::asset_to_dto).collect();
    Ok(Json(dtos))
}

pub async fn create(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(portfolio_id): Path<String>,
    Json(req): Json<CreateAssetRequest>,
) -> Result<Json<crate::models::AssetDto>, AppError> {
    let pid = ObjectId::parse_str(&portfolio_id).map_err(|_| AppError::bad_request())?;
    let _ = repository::portfolio::get_by_id(state.db.as_ref(), ctx.user_id, pid)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::not_found())?;
    let r#type = parse::parse_asset_type(&req.r#type).map_err(|_| AppError::bad_request())?;
    let currency = parse::parse_currency(&req.currency).map_err(|_| AppError::bad_request())?;
    let status = req
        .status
        .as_deref()
        .map(parse::parse_asset_status)
        .transpose()
        .map_err(|_| AppError::bad_request())?
        .unwrap_or(crate::models::AssetStatus::Active);
    let metadata: Option<AssetMetadata> = req
        .metadata
        .and_then(|v| serde_json::from_value(v).ok());
    let now = convert::now_bson();
    let asset = Asset {
        id: None,
        user_id: ctx.user_id,
        portfolio_id: Some(pid),
        name: req.name,
        r#type,
        currency,
        status,
        metadata,
        created_at: now,
        updated_at: now,
    };
    let id = repository::asset::create(state.db.as_ref(), asset).await.map_err(AppError::internal)?;
    let a: Asset = repository::asset::get_by_id(state.db.as_ref(), ctx.user_id, id)
        .await
        .map_err(AppError::internal)?
        .ok_or_else(|| AppError::internal("created asset not found"))?;
    Ok(Json(dto::asset_to_dto(&a)))
}

pub async fn get(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
) -> Result<Json<crate::models::AssetDto>, AppError> {
    let oid = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let a: Asset = repository::asset::get_by_id(state.db.as_ref(), ctx.user_id, oid)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::not_found())?;
    Ok(Json(dto::asset_to_dto(&a)))
}

pub async fn update(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
    Json(req): Json<UpdateAssetRequest>,
) -> Result<Json<crate::models::AssetDto>, AppError> {
    let oid = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let _existing: Asset = repository::asset::get_by_id(state.db.as_ref(), ctx.user_id, oid)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::not_found())?;
    let r#type = parse::parse_asset_type(&req.r#type).map_err(|_| AppError::bad_request())?;
    let currency = parse::parse_currency(&req.currency).map_err(|_| AppError::bad_request())?;
    let status = req
        .status
        .as_deref()
        .map(parse::parse_asset_status)
        .transpose()
        .map_err(|_| AppError::bad_request())?
        .unwrap_or(crate::models::AssetStatus::Active);
    let metadata: Option<AssetMetadata> = req
        .metadata
        .and_then(|v| serde_json::from_value(v).ok());
    let now = convert::now_bson();
    repository::asset::update(
        state.db.as_ref(),
        ctx.user_id,
        oid,
        &req.name,
        &r#type,
        &currency,
        &status,
        metadata.as_ref(),
        now,
    )
    .await
    .map_err(AppError::internal)?;
    let a: Asset = repository::asset::get_by_id(state.db.as_ref(), ctx.user_id, oid)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::not_found())?;
    Ok(Json(dto::asset_to_dto(&a)))
}

pub async fn update_status(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<crate::models::AssetDto>, AppError> {
    let oid = ObjectId::parse_str(&id).map_err(|_| AppError::bad_request())?;
    let status = body
        .get("status")
        .and_then(|v| v.as_str())
        .ok_or(AppError::bad_request())?;
    let status_enum = parse::parse_asset_status(status).map_err(|_| AppError::bad_request())?;
    let status_str = match status_enum {
        crate::models::AssetStatus::Active => "Active",
        crate::models::AssetStatus::Closed => "Closed",
    };
    let now = convert::now_bson();
    repository::asset::update_status(state.db.as_ref(), ctx.user_id, oid, status_str, now)
        .await
        .map_err(AppError::internal)?;
    let a: Asset = repository::asset::get_by_id(state.db.as_ref(), ctx.user_id, oid)
        .await
        .map_err(AppError::internal)?
        .ok_or(AppError::not_found())?;
    Ok(Json(dto::asset_to_dto(&a)))
}
