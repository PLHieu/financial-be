use mongodb::{
    bson::{doc, from_document, to_document, Bson},
    Database,
};
use mongodb::bson::oid::ObjectId;

use crate::models::Transaction;

pub async fn create(db: &Database, transaction: Transaction) -> Result<ObjectId, String> {
    let coll = db.collection::<mongodb::bson::Document>("transactions");
    let doc = to_document(&transaction).map_err(|e| e.to_string())?;
    let res = coll.insert_one(doc).await.map_err(|e| e.to_string())?;
    res.inserted_id
        .as_object_id()
        .ok_or_else(|| "missing inserted id".to_string())
}

pub async fn get_by_asset(db: &Database, user_id: ObjectId, asset_id: ObjectId) -> Result<Vec<Transaction>, String> {
    let coll = db.collection::<mongodb::bson::Document>("transactions");
    let mut cursor = coll
        .find(doc! { "user_id": user_id, "asset_id": asset_id })
        .sort(doc! { "date": -1, "created_at": -1 })
        .await
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while cursor.advance().await.map_err(|e| e.to_string())? {
        let doc = cursor.deserialize_current().map_err(|e| e.to_string())?;
        let t: Transaction = from_document(doc).map_err(|e| e.to_string())?;
        out.push(t);
    }
    Ok(out)
}

pub async fn get_by_portfolio(
    db: &Database,
    user_id: ObjectId,
    portfolio_id: ObjectId,
) -> Result<Vec<Transaction>, String> {
    let coll = db.collection::<mongodb::bson::Document>("transactions");
    let mut cursor = coll
        .find(doc! { "user_id": user_id, "portfolio_id": portfolio_id })
        .sort(doc! { "date": -1, "created_at": -1 })
        .await
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while cursor.advance().await.map_err(|e| e.to_string())? {
        let doc = cursor.deserialize_current().map_err(|e| e.to_string())?;
        let t: Transaction = from_document(doc).map_err(|e| e.to_string())?;
        out.push(t);
    }
    Ok(out)
}

/// Transactions with no portfolio (global assets).
pub async fn get_by_portfolio_global(
    db: &Database,
    user_id: ObjectId,
) -> Result<Vec<Transaction>, String> {
    let coll = db.collection::<mongodb::bson::Document>("transactions");
    let mut cursor = coll
        .find(doc! { "user_id": user_id, "portfolio_id": Bson::Null })
        .sort(doc! { "date": -1, "created_at": -1 })
        .await
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while cursor.advance().await.map_err(|e| e.to_string())? {
        let doc = cursor.deserialize_current().map_err(|e| e.to_string())?;
        let t: Transaction = from_document(doc).map_err(|e| e.to_string())?;
        out.push(t);
    }
    Ok(out)
}

pub async fn get_all(db: &Database, user_id: ObjectId, limit: Option<u64>) -> Result<Vec<Transaction>, String> {
    let coll = db.collection::<mongodb::bson::Document>("transactions");
    let cursor_builder = coll
        .find(doc! { "user_id": user_id })
        .sort(doc! { "date": -1, "created_at": -1 });
    let mut cursor = match limit {
        Some(l) => cursor_builder.limit(l as i64).await,
        None => cursor_builder.await,
    }
    .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while cursor.advance().await.map_err(|e| e.to_string())? {
        let doc = cursor.deserialize_current().map_err(|e| e.to_string())?;
        let t: Transaction = from_document(doc).map_err(|e| e.to_string())?;
        out.push(t);
    }
    Ok(out)
}

pub async fn delete_by_id(db: &Database, user_id: ObjectId, id: ObjectId) -> Result<bool, String> {
    let coll = db.collection::<mongodb::bson::Document>("transactions");
    let res = coll
        .delete_one(doc! { "_id": id, "user_id": user_id })
        .await
        .map_err(|e| e.to_string())?;
    Ok(res.deleted_count > 0)
}

/// Delete all transactions for an asset (before deleting the asset).
pub async fn delete_by_asset(db: &Database, user_id: ObjectId, asset_id: ObjectId) -> Result<u64, String> {
    let coll = db.collection::<mongodb::bson::Document>("transactions");
    let res = coll
        .delete_many(doc! { "user_id": user_id, "asset_id": asset_id })
        .await
        .map_err(|e| e.to_string())?;
    Ok(res.deleted_count)
}
