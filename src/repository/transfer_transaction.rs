use mongodb::{
    bson::{doc, from_document, to_document},
    Database,
};
use mongodb::bson::oid::ObjectId;

use crate::models::TransferTransaction;

pub async fn create(db: &Database, transfer: TransferTransaction) -> Result<ObjectId, String> {
    let coll = db.collection::<mongodb::bson::Document>("transfer_transactions");
    let doc = to_document(&transfer).map_err(|e| e.to_string())?;
    let res = coll.insert_one(doc).await.map_err(|e| e.to_string())?;
    res.inserted_id
        .as_object_id()
        .ok_or_else(|| "missing inserted id".to_string())
        .map(|id| *id)
}

pub async fn get_all(
    db: &Database,
    user_id: ObjectId,
    from_portfolio_id: Option<ObjectId>,
    to_portfolio_id: Option<ObjectId>,
) -> Result<Vec<TransferTransaction>, String> {
    let coll = db.collection::<mongodb::bson::Document>("transfer_transactions");
    let mut filter = doc! { "user_id": user_id };
    if let Some(id) = from_portfolio_id {
        filter.insert("from_portfolio_id", id);
    }
    if let Some(id) = to_portfolio_id {
        filter.insert("to_portfolio_id", id);
    }
    let mut cursor = coll
        .find(filter)
        .sort(doc! { "date": -1 })
        .await
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while cursor.advance().await.map_err(|e| e.to_string())? {
        let doc = cursor.deserialize_current().map_err(|e| e.to_string())?;
        let t: TransferTransaction = from_document(doc).map_err(|e| e.to_string())?;
        out.push(t);
    }
    Ok(out)
}

pub async fn get_by_id(db: &Database, user_id: ObjectId, id: ObjectId) -> Result<Option<TransferTransaction>, String> {
    let coll = db.collection::<mongodb::bson::Document>("transfer_transactions");
    let doc = coll
        .find_one(doc! { "_id": id, "user_id": user_id })
        .await
        .map_err(|e| e.to_string())?;
    match doc {
        Some(d) => {
            let t: TransferTransaction = from_document(d).map_err(|e| e.to_string())?;
            Ok(Some(t))
        }
        None => Ok(None),
    }
}
