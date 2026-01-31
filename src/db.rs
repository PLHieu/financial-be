//! MongoDB connection, collection helpers, and index creation.

use mongodb::{
    bson::doc,
    options::{IndexOptions},
    Client, Database, IndexModel,
};
use std::sync::Arc;

const DB_NAME: &str = "financial";

pub async fn connect(uri: &str) -> Result<Arc<Database>, mongodb::error::Error> {
    let client = Client::with_uri_str(uri).await?;
    let db = client.database(DB_NAME);
    Ok(Arc::new(db))
}

pub async fn create_indexes(db: &Database) -> Result<(), mongodb::error::Error> {
    // portfolios
    db.collection::<mongodb::bson::Document>("portfolios")
        .create_index(
            IndexModel::builder().keys(doc! { "user_id": 1 }).build(),
        )
        .await?;

    // assets
    db.collection::<mongodb::bson::Document>("assets")
        .create_index(
            IndexModel::builder().keys(doc! { "user_id": 1 }).build(),
        )
        .await?;
    db.collection::<mongodb::bson::Document>("assets")
        .create_index(
            IndexModel::builder().keys(doc! { "portfolio_id": 1 }).build(),
        )
        .await?;

    // transactions
    db.collection::<mongodb::bson::Document>("transactions")
        .create_index(
            IndexModel::builder().keys(doc! { "user_id": 1 }).build(),
        )
        .await?;
    db.collection::<mongodb::bson::Document>("transactions")
        .create_index(
            IndexModel::builder().keys(doc! { "asset_id": 1 }).build(),
        )
        .await?;

    // net_worth_snapshots
    db.collection::<mongodb::bson::Document>("net_worth_snapshots")
        .create_index(
            IndexModel::builder()
                .keys(doc! { "user_id": 1, "date": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        )
        .await?;

    // portfolio_snapshots
    db.collection::<mongodb::bson::Document>("portfolio_snapshots")
        .create_index(
            IndexModel::builder()
                .keys(doc! { "user_id": 1, "date": 1, "portfolio_id": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        )
        .await?;
    db.collection::<mongodb::bson::Document>("portfolio_snapshots")
        .create_index(
            IndexModel::builder()
                .keys(doc! { "user_id": 1, "portfolio_id": 1, "date": 1 })
                .build(),
        )
        .await?;

    // asset_price_history
    db.collection::<mongodb::bson::Document>("asset_price_history")
        .create_index(
            IndexModel::builder()
                .keys(doc! { "asset_id": 1, "date": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        )
        .await?;

    // transfer_transactions
    db.collection::<mongodb::bson::Document>("transfer_transactions")
        .create_index(
            IndexModel::builder().keys(doc! { "user_id": 1 }).build(),
        )
        .await?;
    db.collection::<mongodb::bson::Document>("transfer_transactions")
        .create_index(
            IndexModel::builder()
                .keys(doc! { "from_portfolio_id": 1, "date": 1 })
                .build(),
        )
        .await?;
    db.collection::<mongodb::bson::Document>("transfer_transactions")
        .create_index(
            IndexModel::builder()
                .keys(doc! { "to_portfolio_id": 1, "date": 1 })
                .build(),
        )
        .await?;

    // exchange_rates
    db.collection::<mongodb::bson::Document>("exchange_rates")
        .create_index(
            IndexModel::builder()
                .keys(doc! { "from_currency": 1, "to_currency": 1, "date": 1 })
                .options(IndexOptions::builder().unique(true).build())
                .build(),
        )
        .await?;
    db.collection::<mongodb::bson::Document>("exchange_rates")
        .create_index(
            IndexModel::builder().keys(doc! { "date": 1 }).build(),
        )
        .await?;

    Ok(())
}
