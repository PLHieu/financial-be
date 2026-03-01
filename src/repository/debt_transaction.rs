use mongodb::{
    bson::{doc, from_document, to_document},
    Database,
};
use mongodb::bson::oid::ObjectId;

use crate::models::DebtTransaction;

pub async fn create(db: &Database, tx: DebtTransaction) -> Result<ObjectId, String> {
    let coll = db.collection::<mongodb::bson::Document>("debt_transactions");
    let doc = to_document(&tx).map_err(|e| e.to_string())?;
    let res = coll.insert_one(doc).await.map_err(|e| e.to_string())?;
    res.inserted_id
        .as_object_id()
        .ok_or_else(|| "missing inserted id".to_string())
}

pub async fn get_by_debt(
    db: &Database,
    user_id: ObjectId,
    debt_id: ObjectId,
) -> Result<Vec<DebtTransaction>, String> {
    let coll = db.collection::<mongodb::bson::Document>("debt_transactions");
    let mut cursor = coll
        .find(doc! { "user_id": user_id, "debtId": debt_id })
        .sort(doc! { "date": 1 })
        .await
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while cursor.advance().await.map_err(|e| e.to_string())? {
        let doc = cursor.deserialize_current().map_err(|e| e.to_string())?;
        let t: DebtTransaction = from_document(doc).map_err(|e| e.to_string())?;
        out.push(t);
    }
    Ok(out)
}

/// All debt transactions for user, sorted by date asc (for snapshot balance computation).
pub async fn get_all_by_user(
    db: &Database,
    user_id: ObjectId,
) -> Result<Vec<DebtTransaction>, String> {
    let coll = db.collection::<mongodb::bson::Document>("debt_transactions");
    let mut cursor = coll
        .find(doc! { "user_id": user_id })
        .sort(doc! { "date": 1 })
        .await
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while cursor.advance().await.map_err(|e| e.to_string())? {
        let doc = cursor.deserialize_current().map_err(|e| e.to_string())?;
        let t: DebtTransaction = from_document(doc).map_err(|e| e.to_string())?;
        out.push(t);
    }
    Ok(out)
}

/// Delete all transactions for a debt (before deleting the debt).
pub async fn delete_by_debt(db: &Database, user_id: ObjectId, debt_id: ObjectId) -> Result<u64, String> {
    let coll = db.collection::<mongodb::bson::Document>("debt_transactions");
    let res = coll
        .delete_many(doc! { "user_id": user_id, "debtId": debt_id })
        .await
        .map_err(|e| e.to_string())?;
    Ok(res.deleted_count)
}
