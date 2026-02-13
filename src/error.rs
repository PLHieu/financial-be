//! Application error type that logs the cause when returning 500.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::fmt;

/// Application error; internal errors are logged when converted to a response.
#[derive(Debug)]
pub enum AppError {
    BadRequest,
    NotFound,
    /// Internal error — logged at ERROR level when returning 500.
    Internal(String),
    /// Upstream/gateway error — logged at WARN, returns 502.
    BadGateway(String),
}

impl AppError {
    pub fn bad_request() -> Self {
        AppError::BadRequest
    }

    pub fn not_found() -> Self {
        AppError::NotFound
    }

    /// Wrap an internal error; the message is logged when the response is sent.
    pub fn internal(e: impl fmt::Display) -> Self {
        AppError::Internal(e.to_string())
    }

    pub fn bad_gateway(e: impl fmt::Display) -> Self {
        AppError::BadGateway(e.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match &self {
            AppError::BadRequest => (StatusCode::BAD_REQUEST, ()).into_response(),
            AppError::NotFound => (StatusCode::NOT_FOUND, ()).into_response(),
            AppError::Internal(msg) => {
                tracing::error!("Internal server error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, ()).into_response()
            }
            AppError::BadGateway(msg) => {
                tracing::warn!("Bad gateway: {}", msg);
                (StatusCode::BAD_GATEWAY, ()).into_response()
            }
        }
    }
}

impl From<StatusCode> for AppError {
    fn from(s: StatusCode) -> Self {
        match s {
            StatusCode::BAD_REQUEST => AppError::BadRequest,
            StatusCode::NOT_FOUND => AppError::NotFound,
            StatusCode::BAD_GATEWAY => AppError::BadGateway(s.to_string()),
            _ => AppError::Internal(s.to_string()),
        }
    }
}
