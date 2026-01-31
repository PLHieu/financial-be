use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use mongodb::bson::oid::ObjectId;

use crate::context::UserContext;
use crate::convert;
use crate::models::{AssetPriceHistory, UpsertAssetPriceRequest};
use crate::repository;
use crate::routes::dto;
use crate::state::AppState;

#[derive(serde::Deserialize)]
pub struct TimeRangeQuery {
    pub time_range: Option<String>,
}

fn start_date_for_time_range(time_range: &str) -> Option<mongodb::bson::DateTime> {
    let now = chrono::Utc::now();
    let start = match time_range {
        "1M" => now - chrono::Duration::days(30),
        "3M" => now - chrono::Duration::days(90),
        "6M" => now - chrono::Duration::days(180),
        "1Y" => now - chrono::Duration::days(365),
        _ => return None,
    };
    Some(mongodb::bson::DateTime::from_millis(start.timestamp_millis()))
}

pub async fn upsert(
    State(state): State<AppState>,
    ctx: UserContext,
    Json(req): Json<UpsertAssetPriceRequest>,
) -> Result<Json<crate::models::AssetPriceHistoryDto>, StatusCode> {
    let asset_id = ObjectId::parse_str(&req.asset_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let _ = repository::asset::get_by_id(&state.db, ctx.user_id, asset_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let date = convert::parse_iso_to_bson(&req.date).map_err(|_| StatusCode::BAD_REQUEST)?;
    let now = convert::now_bson();
    let record = AssetPriceHistory {
        id: None,
        asset_id,
        date,
        price: req.price,
        currency: req.currency,
        created_at: now,
    };
    let _ = repository::asset_price_history::upsert(&state.db, record)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let list = repository::asset_price_history::get_by_asset(&state.db, asset_id, Some(date))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let r = list.into_iter().next().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(dto::asset_price_history_to_dto(&r)))
}

pub async fn list_by_asset(
    State(state): State<AppState>,
    ctx: UserContext,
    Path(asset_id): Path<String>,
    Query(q): Query<TimeRangeQuery>,
) -> Result<Json<Vec<crate::models::AssetPriceHistoryDto>>, StatusCode> {
    let aid = ObjectId::parse_str(&asset_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let _ = repository::asset::get_by_id(&state.db, ctx.user_id, aid)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;
    let start = q.time_range.as_deref().and_then(start_date_for_time_range);
    let list = repository::asset_price_history::get_by_asset(&state.db, aid, start)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let dtos: Vec<_> = list.iter().map(dto::asset_price_history_to_dto).collect();
    Ok(Json(dtos))
}
