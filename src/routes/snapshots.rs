use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};

use crate::context::UserContext;
use crate::convert;
use crate::error::AppError;
use crate::models::{NetWorthSnapshot, UpsertNetWorthSnapshotRequest};
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

pub async fn upsert_net_worth(
    State(state): State<AppState>,
    ctx: UserContext,
    Json(req): Json<UpsertNetWorthSnapshotRequest>,
) -> Result<Json<crate::models::NetWorthSnapshotDto>, AppError> {
    let date = convert::parse_iso_to_bson(&req.date).map_err(|_| AppError::bad_request())?;
    let now = convert::now_bson();
    let snapshot = NetWorthSnapshot {
        id: None,
        user_id: ctx.user_id,
        date,
        total_net_worth: req.total_net_worth,
        created_at: now,
    };
    let _ = repository::net_worth_snapshot::upsert(&state.db, snapshot)
        .await
        .map_err(AppError::internal)?;
    let list = repository::net_worth_snapshot::get_by_time_range(&state.db, ctx.user_id, Some(date))
        .await
        .map_err(AppError::internal)?;
    let s = list.into_iter().next().ok_or_else(|| AppError::internal("snapshot not found after upsert"))?;
    Ok(Json(dto::net_worth_snapshot_to_dto(&s)))
}

pub async fn list_net_worth(
    State(state): State<AppState>,
    ctx: UserContext,
    Query(q): Query<TimeRangeQuery>,
) -> Result<Json<Vec<crate::models::NetWorthSnapshotDto>>, AppError> {
    let start = q.time_range.as_deref().and_then(start_date_for_time_range);
    let list = repository::net_worth_snapshot::get_by_time_range(&state.db, ctx.user_id, start)
        .await
        .map_err(AppError::internal)?;
    let dtos: Vec<_> = list.iter().map(dto::net_worth_snapshot_to_dto).collect();
    Ok(Json(dtos))
}
