//! Request context: user ID for multi-tenant scope.
//! No auth for now: use X-User-Id header if present, else default user.

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use mongodb::bson::oid::ObjectId;
use std::str::FromStr;

use crate::config;

#[derive(Clone)]
pub struct UserContext {
    pub user_id: ObjectId,
}

impl UserContext {
    pub fn from_header_or_default(header_value: Option<&str>) -> Result<Self, StatusCode> {
        let oid = match header_value {
            Some(h) => ObjectId::parse_str(h.trim()).map_err(|_| StatusCode::BAD_REQUEST)?,
            None => ObjectId::parse_str(config::default_user_id_hex().as_str())
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
        };
        Ok(UserContext { user_id: oid })
    }
}

impl<S> FromRequestParts<S> for UserContext
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let header_value = parts
            .headers
            .get("x-user-id")
            .and_then(|v| v.to_str().ok());
        UserContext::from_header_or_default(header_value)
    }
}
