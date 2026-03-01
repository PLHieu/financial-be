use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use mongodb::bson::oid::ObjectId;

use crate::context::UserContext;
use crate::convert;
use crate::error::AppError;
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
) -> Result<Json<crate::models::PortfolioSnapshotDto>, AppError> {
    let date = convert::parse_iso_to_bson(&req.date).map_err(|_| AppError::bad_request())?;
    let portfolio_id = ObjectId::parse_str(&req.portfolio_id).map_err(|_| AppError::bad_request())?;
    let now = convert::now_bson();
    let snapshot = PortfolioSnapshot {
        id: None,
        user_id: ctx.user_id,
        date,
        portfolio_id,
        total_deposit: req.total_deposit,
        inventory: req.inventory.clone(),
        total_value: req.total_value,
        created_at: now,
    };
    let _ = repository::portfolio_snapshot::upsert(&state.db, snapshot)
        .await
        .map_err(AppError::internal)?;
    let list = repository::portfolio_snapshot::get_by_time_range(
        &state.db,
        ctx.user_id,
        Some(portfolio_id),
        Some(date),
    )
    .await
    .map_err(AppError::internal)?;
    let s = list.into_iter().next().ok_or_else(|| AppError::internal("snapshot not found after upsert"))?;
    Ok(Json(dto::portfolio_snapshot_to_dto(&s)))
}

pub async fn list(
    State(state): State<AppState>,
    ctx: UserContext,
    Query(q): Query<PortfolioSnapshotsQuery>,
) -> Result<Json<Vec<crate::models::PortfolioSnapshotDto>>, AppError> {
    let portfolio_id = q
        .portfolio_id
        .as_ref()
        .and_then(|s| ObjectId::parse_str(s).ok());
    let start = q.time_range.as_deref().and_then(start_date_for_time_range);
    let dtos = crate::snapshot::compute_portfolio_snapshots(&state.db, ctx.user_id, portfolio_id, start)
        .await?;
    Ok(Json(dtos))
}
