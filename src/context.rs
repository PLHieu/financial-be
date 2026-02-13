//! Request context: user ID for multi-tenant scope.
//! No auth for now: use X-User-Id header if present, else default user.

use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use mongodb::bson::oid::ObjectId;
use std::future::Future;
use std::pin::Pin;

use crate::config;
use crate::error::AppError;

#[derive(Clone)]
pub struct UserContext {
    pub user_id: ObjectId,
}

impl UserContext {
    pub fn from_header_or_default(header_value: Option<&str>) -> Result<Self, AppError> {
        let oid = match header_value {
            Some(h) => ObjectId::parse_str(h.trim()).map_err(|_| AppError::bad_request())?,
            None => ObjectId::parse_str(config::default_user_id_hex().as_str())
                .map_err(|e| AppError::internal(format!("default user id invalid: {}", e)))?,
        };
        Ok(UserContext { user_id: oid })
    }
}

impl<S> FromRequestParts<S> for UserContext
where
    S: Send + Sync,
{
    type Rejection = AppError;

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut Parts,
        _state: &'life1 S,
    ) -> Pin<Box<dyn Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        let header_value = parts
            .headers
            .get("x-user-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        Box::pin(std::future::ready(UserContext::from_header_or_default(
            header_value.as_deref(),
        )))
    }
}
