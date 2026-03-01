use mongodb::{
    bson::{doc, from_document, to_document, Bson},
    Database,
};
use mongodb::bson::oid::ObjectId;

use crate::models::{Asset, AssetMetadata, AssetStatus, AssetType, Currency};

pub async fn create(db: &Database, asset: Asset) -> Result<ObjectId, String> {
    let coll = db.collection::<mongodb::bson::Document>("assets");
    let doc = to_document(&asset).map_err(|e| e.to_string())?;
    let res = coll.insert_one(doc).await.map_err(|e| e.to_string())?;
    res.inserted_id
        .as_object_id()
        .ok_or_else(|| "missing inserted id".to_string())
}

/// Returns assets available for this portfolio: global (portfolio_id null) + scoped to this portfolio.
pub async fn get_by_portfolio(db: &Database, user_id: ObjectId, portfolio_id: ObjectId) -> Result<Vec<Asset>, String> {
    let coll = db.collection::<mongodb::bson::Document>("assets");
    let filter = doc! {
        "user_id": user_id,
        "$or": [
            { "portfolio_id": Bson::Null },
            { "portfolio_id": { "$exists": false } },
            { "portfolio_id": portfolio_id },
        ],
    };
    let mut cursor = coll
        .find(filter)
        .sort(doc! { "name": 1 })
        .await
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while cursor.advance().await.map_err(|e| e.to_string())? {
        let doc = cursor.deserialize_current().map_err(|e| e.to_string())?;
        let a: Asset = from_document(doc).map_err(|e| e.to_string())?;
        out.push(a);
    }
    Ok(out)
}

pub async fn get_by_id(db: &Database, user_id: ObjectId, id: ObjectId) -> Result<Option<Asset>, String> {
    let coll = db.collection::<mongodb::bson::Document>("assets");
    let doc = coll
        .find_one(doc! { "_id": id, "user_id": user_id })
        .await
        .map_err(|e| e.to_string())?;
    match doc {
        Some(d) => {
            let a: Asset = from_document(d).map_err(|e| e.to_string())?;
            Ok(Some(a))
        }
        None => Ok(None),
    }
}

pub async fn get_all(db: &Database, user_id: ObjectId) -> Result<Vec<Asset>, String> {
    let coll = db.collection::<mongodb::bson::Document>("assets");
    let mut cursor = coll
        .find(doc! { "user_id": user_id })
        .sort(doc! { "name": 1 })
        .await
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while cursor.advance().await.map_err(|e| e.to_string())? {
        let doc = cursor.deserialize_current().map_err(|e| e.to_string())?;
        let a: Asset = from_document(doc).map_err(|e| e.to_string())?;
        out.push(a);
    }
    Ok(out)
}

pub async fn update_status(
    db: &Database,
    user_id: ObjectId,
    id: ObjectId,
    status: &str,
    updated_at: mongodb::bson::DateTime,
) -> Result<bool, String> {
    let coll = db.collection::<mongodb::bson::Document>("assets");
    let res = coll
        .update_one(
            doc! { "_id": id, "user_id": user_id },
            doc! { "$set": { "status": status, "updated_at": updated_at } },
        )
        .await
        .map_err(|e| e.to_string())?;
    Ok(res.modified_count > 0)
}

pub async fn update(
    db: &Database,
    user_id: ObjectId,
    id: ObjectId,
    name: &str,
    r#type: &AssetType,
    currency: &Currency,
    status: &AssetStatus,
    metadata: Option<&AssetMetadata>,
    updated_at: mongodb::bson::DateTime,
) -> Result<bool, String> {
    use mongodb::bson::Bson;
    let coll = db.collection::<mongodb::bson::Document>("assets");
    let type_bson = mongodb::bson::to_bson(r#type).map_err(|e| e.to_string())?;
    let currency_bson = mongodb::bson::to_bson(currency).map_err(|e| e.to_string())?;
    let status_bson = mongodb::bson::to_bson(status).map_err(|e| e.to_string())?;
    let metadata_bson: Option<Bson> = metadata
        .and_then(|m| to_document(m).ok())
        .map(Bson::Document);
    let mut set = doc! {
        "name": name,
        "type": type_bson,
        "currency": currency_bson,
        "status": status_bson,
        "updated_at": updated_at,
    };
    set.insert(
        "metadata",
        metadata_bson.unwrap_or(Bson::Null),
    );
    let res = coll
        .update_one(
            doc! { "_id": id, "user_id": user_id },
            doc! { "$set": set },
        )
        .await
        .map_err(|e| e.to_string())?;
    Ok(res.modified_count > 0)
}

pub async fn delete(db: &Database, user_id: ObjectId, id: ObjectId) -> Result<bool, String> {
    let coll = db.collection::<mongodb::bson::Document>("assets");
    let res = coll
        .delete_one(doc! { "_id": id, "user_id": user_id })
        .await
        .map_err(|e| e.to_string())?;
    Ok(res.deleted_count > 0)
}
