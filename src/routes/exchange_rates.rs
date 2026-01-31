use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};

use crate::context::UserContext;
use crate::convert;
use crate::models::{CreateExchangeRateRequest, ExchangeRate};
use crate::repository;
use crate::routes::dto;
use crate::state::AppState;

#[derive(serde::Deserialize)]
pub struct ListRatesQuery {
    pub date: Option<String>,
    pub from_currency: Option<String>,
    pub to_currency: Option<String>,
}

pub async fn create(
    State(state): State<AppState>,
    _ctx: UserContext,
    Json(req): Json<CreateExchangeRateRequest>,
) -> Result<Json<crate::models::ExchangeRateDto>, StatusCode> {
    let date = convert::parse_iso_to_bson(&req.date).map_err(|_| StatusCode::BAD_REQUEST)?;
    let now = convert::now_bson();
    let rate = ExchangeRate {
        id: None,
        from_currency: req.from_currency,
        to_currency: req.to_currency,
        rate: req.rate,
        date,
        source: req.source,
        created_at: now,
    };
    let id = repository::exchange_rate::create(&state.db, rate).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let list = repository::exchange_rate::get_by_date(&state.db, date, None, None)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let r = list.into_iter().find(|x| x.id == Some(id)).ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(dto::exchange_rate_to_dto(&r)))
}

pub async fn list(
    State(state): State<AppState>,
    _ctx: UserContext,
    Query(q): Query<ListRatesQuery>,
) -> Result<Json<Vec<crate::models::ExchangeRateDto>>, StatusCode> {
    let date = q
        .date
        .as_ref()
        .and_then(|s| convert::parse_iso_to_bson(s).ok());
    let from_currency = q.from_currency.as_deref();
    let to_currency = q.to_currency.as_deref();
    let list = if let Some(d) = date {
        repository::exchange_rate::get_by_date(&state.db, d, from_currency, to_currency)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        let start = None;
        repository::exchange_rate::get_all(&state.db, start)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };
    let dtos: Vec<_> = list.iter().map(dto::exchange_rate_to_dto).collect();
    Ok(Json(dtos))
}
