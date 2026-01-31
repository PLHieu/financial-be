use mongodb::{
    bson::{doc, from_document, to_document},
    Database,
};
use mongodb::bson::oid::ObjectId;

use crate::models::AssetPriceHistory;

pub async fn upsert(db: &Database, record: AssetPriceHistory) -> Result<ObjectId, String> {
    let coll = db.collection::<mongodb::bson::Document>("asset_price_history");
    let filter = doc! { "asset_id": record.asset_id, "date": record.date };
    let doc = to_document(&record).map_err(|e| e.to_string())?;
    let update = doc! { "$set": doc };
    let opts = mongodb::options::UpdateOptions::builder()
        .upsert(true)
        .build();
    let res = coll.update_one(filter.clone(), update).with_options(opts).await.map_err(|e| e.to_string())?;
    if let Some(id) = res.upserted_id.and_then(|v| v.as_object_id()) {
        Ok(id)
    } else {
        let existing = coll.find_one(filter).await.map_err(|e| e.to_string())?;
        existing
            .and_then(|d| d.get_object_id("_id").ok())
            .ok_or_else(|| "missing id after upsert".to_string())
    }
}

pub async fn get_by_asset(
    db: &Database,
    asset_id: ObjectId,
    start_date: Option<mongodb::bson::DateTime>,
) -> Result<Vec<AssetPriceHistory>, String> {
    let coll = db.collection::<mongodb::bson::Document>("asset_price_history");
    let mut filter = doc! { "asset_id": asset_id };
    if let Some(d) = start_date {
        filter.insert("date", doc! { "$gte": d });
    }
    let mut cursor = coll
        .find(filter)
        .sort(doc! { "date": 1 })
        .await
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while cursor.advance().await.map_err(|e| e.to_string())? {
        let doc = cursor.deserialize_current().map_err(|e| e.to_string())?;
        let r: AssetPriceHistory = from_document(doc).map_err(|e| e.to_string())?;
        out.push(r);
    }
    Ok(out)
}
