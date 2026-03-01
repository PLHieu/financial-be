use mongodb::{
    bson::{doc, from_document, to_document},
    Database,
};
use mongodb::bson::oid::ObjectId;

use crate::models::FundTransaction;

pub async fn create(db: &Database, tx: FundTransaction) -> Result<ObjectId, String> {
    let coll = db.collection::<mongodb::bson::Document>("fund_transactions");
    let doc = to_document(&tx).map_err(|e| e.to_string())?;
    let res = coll.insert_one(doc).await.map_err(|e| e.to_string())?;
    res.inserted_id
        .as_object_id()
        .ok_or_else(|| "missing inserted id".to_string())
}

pub async fn get_by_fund(
    db: &Database,
    user_id: ObjectId,
    fund_id: ObjectId,
) -> Result<Vec<FundTransaction>, String> {
    let coll = db.collection::<mongodb::bson::Document>("fund_transactions");
    let mut cursor = coll
        .find(doc! { "user_id": user_id, "fundId": fund_id })
        .sort(doc! { "date": 1 })
        .await
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while cursor.advance().await.map_err(|e| e.to_string())? {
        let doc = cursor.deserialize_current().map_err(|e| e.to_string())?;
        let t: FundTransaction = from_document(doc).map_err(|e| e.to_string())?;
        out.push(t);
    }
    Ok(out)
}

/// All fund transactions for user, sorted by date asc (for snapshot balance computation).
pub async fn get_all_by_user(
    db: &Database,
    user_id: ObjectId,
) -> Result<Vec<FundTransaction>, String> {
    let coll = db.collection::<mongodb::bson::Document>("fund_transactions");
    let mut cursor = coll
        .find(doc! { "user_id": user_id })
        .sort(doc! { "date": 1 })
        .await
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while cursor.advance().await.map_err(|e| e.to_string())? {
        let doc = cursor.deserialize_current().map_err(|e| e.to_string())?;
        let t: FundTransaction = from_document(doc).map_err(|e| e.to_string())?;
        out.push(t);
    }
    Ok(out)
}

/// Delete all transactions for a fund (before deleting the fund).
pub async fn delete_by_fund(db: &Database, user_id: ObjectId, fund_id: ObjectId) -> Result<u64, String> {
    let coll = db.collection::<mongodb::bson::Document>("fund_transactions");
    let res = coll
        .delete_many(doc! { "user_id": user_id, "fundId": fund_id })
        .await
        .map_err(|e| e.to_string())?;
    Ok(res.deleted_count)
}
