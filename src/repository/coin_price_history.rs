use mongodb::{
    bson::{doc, from_document, to_document},
    Database,
};
use mongodb::bson::oid::ObjectId;

use crate::models::CoinPriceHistory;

pub async fn upsert(db: &Database, record: CoinPriceHistory) -> Result<ObjectId, String> {
    let coll = db.collection::<mongodb::bson::Document>("coin_price_history");
    let filter = doc! {
        "user_id": record.user_id,
        "coin_id": record.coin_id.as_str(),
        "date": record.date,
        "currency": record.currency.as_str(),
    };
    let doc = to_document(&record).map_err(|e| e.to_string())?;
    let update = doc! { "$set": doc };
    let opts = mongodb::options::UpdateOptions::builder()
        .upsert(true)
        .build();
    let res = coll
        .update_one(filter.clone(), update)
        .with_options(opts)
        .await
        .map_err(|e| e.to_string())?;
    if let Some(id) = res.upserted_id.and_then(|v| v.as_object_id()) {
        Ok(id)
    } else {
        let existing = coll.find_one(filter).await.map_err(|e| e.to_string())?;
        existing
            .and_then(|d| d.get_object_id("_id").ok())
            .ok_or_else(|| "missing id after upsert".to_string())
    }
}

pub async fn upsert_many(
    db: &Database,
    user_id: ObjectId,
    coin_id: &str,
    currency: &str,
    points: &[(mongodb::bson::DateTime, f64)],
    created_at: mongodb::bson::DateTime,
) -> Result<u64, String> {
    let coll = db.collection::<mongodb::bson::Document>("coin_price_history");
    let mut upserted = 0u64;
    for (date, price) in points {
        let record = CoinPriceHistory {
            id: None,
            user_id,
            coin_id: coin_id.to_string(),
            date: *date,
            price: *price,
            currency: currency.to_string(),
            created_at,
        };
        let filter = doc! {
            "user_id": user_id,
            "coin_id": coin_id,
            "date": date,
            "currency": currency,
        };
        let doc = to_document(&record).map_err(|e| e.to_string())?;
        let update = doc! { "$set": doc };
        let opts = mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build();
        let res = coll
            .update_one(filter, update)
            .with_options(opts)
            .await
            .map_err(|e| e.to_string())?;
        if res.upserted_id.is_some() || res.modified_count > 0 {
            upserted += 1;
        }
    }
    Ok(upserted)
}

pub async fn get_by_coin(
    db: &Database,
    user_id: ObjectId,
    coin_id: &str,
    currency: Option<&str>,
    start_date: Option<mongodb::bson::DateTime>,
) -> Result<Vec<CoinPriceHistory>, String> {
    let coll = db.collection::<mongodb::bson::Document>("coin_price_history");
    let mut filter = doc! { "user_id": user_id, "coin_id": coin_id };
    if let Some(c) = currency {
        filter.insert("currency", c);
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
        let r: CoinPriceHistory = from_document(doc).map_err(|e| e.to_string())?;
        out.push(r);
    }
    Ok(out)
}
