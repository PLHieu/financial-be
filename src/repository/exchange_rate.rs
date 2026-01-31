use mongodb::{
    bson::{doc, from_document, to_document},
    Database,
};
use mongodb::bson::oid::ObjectId;

use crate::models::ExchangeRate;

pub async fn create(db: &Database, rate: ExchangeRate) -> Result<ObjectId, String> {
    let coll = db.collection::<mongodb::bson::Document>("exchange_rates");
    let doc = to_document(&rate).map_err(|e| e.to_string())?;
    let res = coll.insert_one(doc).await.map_err(|e| e.to_string())?;
    res.inserted_id
        .as_object_id()
        .ok_or_else(|| "missing inserted id".to_string())
        .map(|id| *id)
}

pub async fn upsert(db: &Database, rate: ExchangeRate) -> Result<ObjectId, String> {
    let coll = db.collection::<mongodb::bson::Document>("exchange_rates");
    let filter = doc! {
        "from_currency": &rate.from_currency,
        "to_currency": &rate.to_currency,
        "date": rate.date,
    };
    let doc = to_document(&rate).map_err(|e| e.to_string())?;
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

pub async fn get_by_date(
    db: &Database,
    date: mongodb::bson::DateTime,
    from_currency: Option<&str>,
    to_currency: Option<&str>,
) -> Result<Vec<ExchangeRate>, String> {
    let coll = db.collection::<mongodb::bson::Document>("exchange_rates");
    let mut filter = doc! { "date": date };
    if let Some(c) = from_currency {
        filter.insert("from_currency", c);
    }
    if let Some(c) = to_currency {
        filter.insert("to_currency", c);
    }
    let mut cursor = coll
        .find(filter)
        .sort(doc! { "from_currency": 1, "to_currency": 1 })
        .await
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while cursor.advance().await.map_err(|e| e.to_string())? {
        let doc = cursor.deserialize_current().map_err(|e| e.to_string())?;
        let r: ExchangeRate = from_document(doc).map_err(|e| e.to_string())?;
        out.push(r);
    }
    Ok(out)
}

pub async fn get_all(db: &Database, start_date: Option<mongodb::bson::DateTime>) -> Result<Vec<ExchangeRate>, String> {
    let coll = db.collection::<mongodb::bson::Document>("exchange_rates");
    let filter = match start_date {
        Some(d) => doc! { "date": { "$gte": d } },
        None => doc! {},
    };
    let mut cursor = coll
        .find(filter)
        .sort(doc! { "date": 1 })
        .await
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    while cursor.advance().await.map_err(|e| e.to_string())? {
        let doc = cursor.deserialize_current().map_err(|e| e.to_string())?;
        let r: ExchangeRate = from_document(doc).map_err(|e| e.to_string())?;
        out.push(r);
    }
    Ok(out)
}
