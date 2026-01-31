//! App config from environment. No auth for now; optional X-User-Id header
//! can be used. Default user id is used when header is missing (single-tenant mode).

pub fn mongodb_uri() -> String {
    std::env::var("MONGODB_URI").unwrap_or_else(|_| "mongodb://localhost:27017".to_string())
}

pub fn port() -> u16 {
    std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3001)
}

/// Default user ID when no auth / no X-User-Id. Use a fixed ObjectId so all
/// data is attributed to one "anonymous" user until auth is added.
pub fn default_user_id_hex() -> String {
    std::env::var("DEFAULT_USER_ID").unwrap_or_else(|_| "000000000000000000000001".to_string())
}

pub fn allowed_origin() -> Option<String> {
    std::env::var("ALLOWED_ORIGIN").ok()
}
