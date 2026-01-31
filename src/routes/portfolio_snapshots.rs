use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use mongodb::bson::oid::ObjectId;

use crate::context::UserContext;
use crate::convert;
use crate::models::{PortfolioSnapshot, UpsertPortfolioSnapshotRequest};
use crate::repository;
use crate::routes::dto;
use crate::state::AppState;

#[derive(serde::Deserialize)]
pub struct PortfolioSnapshotsQuery {
    pub time_range: Option<String>,
    pub portfolio_id: Option<String>,
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
    Json(req): Json<UpsertPortfolioSnapshotRequest>,
) -> Result<Json<crate::models::PortfolioSnapshotDto>, StatusCode> {
    let date = convert::parse_iso_to_bson(&req.date).map_err(|_| StatusCode::BAD_REQUEST)?;
    let portfolio_id = ObjectId::parse_str(&req.portfolio_id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let now = convert::now_bson();
    let snapshot = PortfolioSnapshot {
        id: None,
        user_id: ctx.user_id,
        date,
        portfolio_id,
        total_value: req.total_value,
        pnl: req.pnl,
        created_at: now,
    };
    let _ = repository::portfolio_snapshot::upsert(&state.db, snapshot)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let list = repository::portfolio_snapshot::get_by_time_range(
        &state.db,
        ctx.user_id,
        Some(portfolio_id),
        Some(date),
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let s = list.into_iter().next().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(dto::portfolio_snapshot_to_dto(&s)))
}

pub async fn list(
    State(state): State<AppState>,
    ctx: UserContext,
    Query(q): Query<PortfolioSnapshotsQuery>,
) -> Result<Json<Vec<crate::models::PortfolioSnapshotDto>>, StatusCode> {
    let portfolio_id = q
        .portfolio_id
        .as_ref()
        .and_then(|s| ObjectId::parse_str(s).ok());
    let start = q.time_range.as_deref().and_then(start_date_for_time_range);
    let list = repository::portfolio_snapshot::get_by_time_range(&state.db, ctx.user_id, portfolio_id, start)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let dtos: Vec<_> = list.iter().map(dto::portfolio_snapshot_to_dto).collect();
    Ok(Json(dtos))
}
