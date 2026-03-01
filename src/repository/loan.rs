use mongodb::{
    bson::{doc, from_document, to_document, Bson},
    Database,
};
use mongodb::bson::oid::ObjectId;

use crate::models::Loan;

pub async fn create(db: &Database, loan: Loan) -> Result<ObjectId, String> {
    let coll = db.collection::<mongodb::bson::Document>("loans");
    let doc = to_document(&loan).map_err(|e| e.to_string())?;
    let res = coll.insert_one(doc).await.map_err(|e| e.to_string())?;
    res.inserted_id
        .as_object_id()
        .ok_or_else(|| "missing inserted id".to_string())
}

pub async fn get_by_id(db: &Database, user_id: ObjectId, id: ObjectId) -> Result<Option<Loan>, String> {
    let coll = db.collection::<mongodb::bson::Document>("loans");
    let doc = coll
        .find_one(doc! { "_id": id, "user_id": user_id })
        .await
        .map_err(|e| e.to_string())?;
    match doc {
        Some(d) => {
            let loan: Loan = from_document(d).map_err(|e| e.to_string())?;
            Ok(Some(loan))
        }
        None => Ok(None),
    }
}

pub async fn get_all(db: &Database, user_id: ObjectId) -> Result<Vec<Loan>, String> {
    let coll = db.collection::<mongodb::bson::Document>("loans");
    let mut cursor = coll
        .find(doc! { "user_id": user_id })
        .sort(doc! { "created_at": -1 })
        .await
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while cursor.advance().await.map_err(|e| e.to_string())? {
        let doc = cursor.deserialize_current().map_err(|e| e.to_string())?;
        let l: Loan = from_document(doc).map_err(|e| e.to_string())?;
        out.push(l);
    }
    Ok(out)
}

pub async fn get_by_portfolio(
    db: &Database,
    user_id: ObjectId,
    portfolio_id: ObjectId,
) -> Result<Vec<Loan>, String> {
    let coll = db.collection::<mongodb::bson::Document>("loans");
    let mut filter = doc! { "user_id": user_id };
    filter.insert("$or", vec![
        doc! { "portfolio_id": Bson::Null },
        doc! { "portfolio_id": { "$exists": false } },
        doc! { "portfolio_id": portfolio_id },
    ]);
    let mut cursor = coll
        .find(filter)
        .sort(doc! { "created_at": -1 })
        .await
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while cursor.advance().await.map_err(|e| e.to_string())? {
        let doc = cursor.deserialize_current().map_err(|e| e.to_string())?;
        let l: Loan = from_document(doc).map_err(|e| e.to_string())?;
        out.push(l);
    }
    Ok(out)
}

pub async fn delete(db: &Database, user_id: ObjectId, id: ObjectId) -> Result<bool, String> {
    let coll = db.collection::<mongodb::bson::Document>("loans");
    let res = coll
        .delete_one(doc! { "_id": id, "user_id": user_id })
        .await
        .map_err(|e| e.to_string())?;
    Ok(res.deleted_count > 0)
}
