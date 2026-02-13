mod config;
mod context;
mod convert;
mod error;
mod cron;
mod db;
mod models;
mod repository;
mod routes;
mod state;

use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let uri = config::mongodb_uri();
    let db = db::connect(&uri).await?;
    tracing::info!("Connected to MongoDB");

    db::create_indexes(db.as_ref()).await?;
    tracing::info!("Indexes ensured");

    let state = state::AppState { db: db.clone() };

    // Daily cron at 23:59:59 UTC: fetch today's price for CRON_COIN_IDS and upsert to MongoDB
    if !config::cron_coin_ids().is_empty() {
        tokio::spawn(cron::cron_loop(db));
        tracing::info!("Cron: daily price job scheduled (23:59:59 UTC)");
    }

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = axum::Router::new()
        .route("/health", axum::routing::get(|| async { "OK" }))
        .nest("/api", routes::router(state))
        .layer(cors);

    let port = config::port();
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Listening on http://{}", addr);

    axum::serve(
        tokio::net::TcpListener::bind(addr).await?,
        app,
    )
    .await?;

    Ok(())
}
