use mongodb::{
    bson::{doc, from_document, to_document},
    Database,
};
use mongodb::bson::oid::ObjectId;

use crate::models::PortfolioSnapshot;

pub async fn upsert(db: &Database, snapshot: PortfolioSnapshot) -> Result<ObjectId, String> {
    let coll = db.collection::<mongodb::bson::Document>("portfolio_snapshots");
    let filter = doc! {
        "user_id": snapshot.user_id,
        "date": snapshot.date,
        "portfolio_id": snapshot.portfolio_id,
    };
    let doc = to_document(&snapshot).map_err(|e| e.to_string())?;
    let update = doc! { "$set": doc };
    let opts = mongodb::options::UpdateOptions::builder()
        .upsert(true)
        .build();
    let res = coll.update_one(filter, update).with_options(opts).await.map_err(|e| e.to_string())?;
    if let Some(id) = res.upserted_id.and_then(|v| v.as_object_id().copied()) {
        Ok(id)
    } else {
        let existing = coll.find_one(filter).await.map_err(|e| e.to_string())?;
        existing
            .and_then(|d| d.get_object_id("_id").ok().copied())
            .ok_or_else(|| "missing id after upsert".to_string())
    }
}

pub async fn get_by_time_range(
    db: &Database,
    user_id: ObjectId,
    portfolio_id: Option<ObjectId>,
    start_date: Option<mongodb::bson::DateTime>,
) -> Result<Vec<PortfolioSnapshot>, String> {
    let coll = db.collection::<mongodb::bson::Document>("portfolio_snapshots");
    let mut filter = doc! { "user_id": user_id };
    if let Some(pid) = portfolio_id {
        filter.insert("portfolio_id", pid);
    }
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
        let s: PortfolioSnapshot = from_document(doc).map_err(|e| e.to_string())?;
        out.push(s);
    }
    Ok(out)
}
