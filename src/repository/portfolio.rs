use mongodb::{
    bson::{doc, from_document, to_document},
    Database,
};
use mongodb::bson::oid::ObjectId;

use crate::models::Portfolio;

pub async fn create(db: &Database, portfolio: Portfolio) -> Result<ObjectId, String> {
    let coll = db.collection::<mongodb::bson::Document>("portfolios");
    let doc = to_document(&portfolio).map_err(|e| e.to_string())?;
    let res = coll.insert_one(doc).await.map_err(|e| e.to_string())?;
    res.inserted_id
        .as_object_id()
        .ok_or_else(|| "missing inserted id".to_string())
        .map(|id| *id)
}

pub async fn get_all(db: &Database, user_id: ObjectId) -> Result<Vec<Portfolio>, String> {
    let coll = db.collection::<mongodb::bson::Document>("portfolios");
    let mut cursor = coll
        .find(doc! { "user_id": user_id })
        .sort(doc! { "created_at": -1 })
        .await
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while cursor.advance().await.map_err(|e| e.to_string())? {
        let doc = cursor.deserialize_current().map_err(|e| e.to_string())?;
        let p: Portfolio = from_document(doc).map_err(|e| e.to_string())?;
        out.push(p);
    }
    Ok(out)
}

pub async fn get_by_id(db: &Database, user_id: ObjectId, id: ObjectId) -> Result<Option<Portfolio>, String> {
    let coll = db.collection::<mongodb::bson::Document>("portfolios");
    let doc = coll
        .find_one(doc! { "_id": id, "user_id": user_id })
        .await
        .map_err(|e| e.to_string())?;
    match doc {
        Some(d) => {
            let p: Portfolio = from_document(d).map_err(|e| e.to_string())?;
            Ok(Some(p))
        }
        None => Ok(None),
    }
}

pub async fn update(db: &Database, user_id: ObjectId, id: ObjectId, portfolio: Portfolio) -> Result<bool, String> {
    let coll = db.collection::<mongodb::bson::Document>("portfolios");
    let doc = to_document(&portfolio).map_err(|e| e.to_string())?;
    let res = coll
        .update_one(
            doc! { "_id": id, "user_id": user_id },
            doc! { "$set": doc },
        )
        .await
        .map_err(|e| e.to_string())?;
    Ok(res.modified_count > 0)
}

pub async fn delete(db: &Database, user_id: ObjectId, id: ObjectId) -> Result<bool, String> {
    let coll = db.collection::<mongodb::bson::Document>("portfolios");
    let res = coll
        .delete_one(doc! { "_id": id, "user_id": user_id })
        .await
        .map_err(|e| e.to_string())?;
    Ok(res.deleted_count > 0)
}
