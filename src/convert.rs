//! Helpers for BSON DateTime <-> ISO string (API).

use chrono::{DateTime, TimeZone, Utc};
use mongodb::bson::DateTime as BsonDateTime;

pub fn bson_dt_to_rfc3339(dt: &BsonDateTime) -> String {
    let millis = dt.timestamp_millis();
    let chrono_dt: DateTime<Utc> = DateTime::from_timestamp_millis(millis).unwrap_or_default();
    chrono_dt.to_rfc3339()
}

pub fn parse_iso_to_bson(s: &str) -> Result<BsonDateTime, String> {
    let chrono_dt = if s.contains('T') {
        DateTime::parse_from_rfc3339(s)
            .map_err(|e| e.to_string())?
            .with_timezone(&Utc)
    } else {
        // Date-only "YYYY-MM-DD" from frontend: treat as midnight UTC
        let naive_date = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|e| e.to_string())?;
        let naive_dt = naive_date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| "invalid date".to_string())?;
        Utc.from_utc_datetime(&naive_dt)
    };
    Ok(BsonDateTime::from_millis(chrono_dt.timestamp_millis()))
}

pub fn now_bson() -> BsonDateTime {
    BsonDateTime::from_millis(Utc::now().timestamp_millis())
}
