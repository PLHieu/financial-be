use mongodb::{
    bson::{doc, from_document, to_document},
    Database,
};
use mongodb::bson::oid::ObjectId;

use crate::models::DepreciatingItemTransaction;

pub async fn create(
    db: &Database,
    tx: DepreciatingItemTransaction,
) -> Result<ObjectId, String> {
    let coll = db.collection::<mongodb::bson::Document>("depreciating_item_transactions");
    let doc = to_document(&tx).map_err(|e| e.to_string())?;
    let res = coll.insert_one(doc).await.map_err(|e| e.to_string())?;
    res.inserted_id
        .as_object_id()
        .ok_or_else(|| "missing inserted id".to_string())
}

pub async fn get_by_item(
    db: &Database,
    user_id: ObjectId,
    item_id: ObjectId,
) -> Result<Vec<DepreciatingItemTransaction>, String> {
    let coll = db.collection::<mongodb::bson::Document>("depreciating_item_transactions");
    let mut cursor = coll
        .find(doc! { "user_id": user_id, "depreciatingItemId": item_id })
        .sort(doc! { "date": 1 })
        .await
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while cursor.advance().await.map_err(|e| e.to_string())? {
        let doc = cursor.deserialize_current().map_err(|e| e.to_string())?;
        let t: DepreciatingItemTransaction = from_document(doc).map_err(|e| e.to_string())?;
        out.push(t);
    }
    Ok(out)
}

pub async fn get_all_by_user(
    db: &Database,
    user_id: ObjectId,
) -> Result<Vec<DepreciatingItemTransaction>, String> {
    let coll = db.collection::<mongodb::bson::Document>("depreciating_item_transactions");
    let mut cursor = coll
        .find(doc! { "user_id": user_id })
        .sort(doc! { "date": 1 })
        .await
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while cursor.advance().await.map_err(|e| e.to_string())? {
        let doc = cursor.deserialize_current().map_err(|e| e.to_string())?;
        let t: DepreciatingItemTransaction = from_document(doc).map_err(|e| e.to_string())?;
        out.push(t);
    }
    Ok(out)
}

/// Delete all transactions for a depreciating item (before deleting the item).
pub async fn delete_by_item(db: &Database, user_id: ObjectId, item_id: ObjectId) -> Result<u64, String> {
    let coll = db.collection::<mongodb::bson::Document>("depreciating_item_transactions");
    let res = coll
        .delete_many(doc! { "user_id": user_id, "depreciatingItemId": item_id })
        .await
        .map_err(|e| e.to_string())?;
    Ok(res.deleted_count)
}
