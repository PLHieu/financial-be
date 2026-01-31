use mongodb::{
    bson::{doc, from_document, to_document},
    Database,
};
use mongodb::bson::oid::ObjectId;

use crate::models::NetWorthSnapshot;

pub async fn upsert(db: &Database, snapshot: NetWorthSnapshot) -> Result<ObjectId, String> {
    let coll = db.collection::<mongodb::bson::Document>("net_worth_snapshots");
    let filter = doc! { "user_id": snapshot.user_id, "date": snapshot.date };
    let doc = to_document(&snapshot).map_err(|e| e.to_string())?;
    let update = doc! { "$set": doc };
    let opts = mongodb::options::UpdateOptions::builder()
        .upsert(true)
        .build();
    let res = coll.update_one(filter, update).with_options(opts).await.map_err(|e| e.to_string())?;
    if let Some(id) = res.upserted_id.and_then(|v| v.as_object_id()) {
        Ok(id)
    } else {
        let existing = coll
            .find_one(doc! { "user_id": snapshot.user_id, "date": snapshot.date })
            .await
            .map_err(|e| e.to_string())?;
        existing
            .and_then(|d| d.get_object_id("_id").ok())
            .ok_or_else(|| "missing id after upsert".to_string())
    }
}

pub async fn get_by_time_range(
    db: &Database,
    user_id: ObjectId,
    start_date: Option<mongodb::bson::DateTime>,
) -> Result<Vec<NetWorthSnapshot>, String> {
    let coll = db.collection::<mongodb::bson::Document>("net_worth_snapshots");
    let filter = match start_date {
        Some(d) => doc! { "user_id": user_id, "date": { "$gte": d } },
        None => doc! { "user_id": user_id },
    };
    let mut cursor = coll
        .find(filter)
        .sort(doc! { "date": 1 })
        .await
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while cursor.advance().await.map_err(|e| e.to_string())? {
        let doc = cursor.deserialize_current().map_err(|e| e.to_string())?;
        let s: NetWorthSnapshot = from_document(doc).map_err(|e| e.to_string())?;
        out.push(s);
    }
    Ok(out)
}
