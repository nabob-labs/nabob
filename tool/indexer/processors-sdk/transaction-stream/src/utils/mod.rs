mod additional_headers;

pub use additional_headers::AdditionalHeaders;
use nabob_protos::util::timestamp::Timestamp;

// 9999-12-31 23:59:59, this is the max supported by Google BigQuery
pub const MAX_TIMESTAMP_SECS: i64 = 253_402_300_799;

pub fn parse_timestamp(ts: &Timestamp, version: i64) -> chrono::DateTime<chrono::Utc> {
    let final_ts = if ts.seconds >= MAX_TIMESTAMP_SECS {
        Timestamp {
            seconds: MAX_TIMESTAMP_SECS,
            nanos: 0,
        }
    } else {
        *ts
    };
    chrono::DateTime::from_timestamp(final_ts.seconds, final_ts.nanos as u32)
        .unwrap_or_else(|| panic!("Could not parse timestamp {:?} for version {}", ts, version))
}

/// Convert the protobuf timestamp to ISO format
pub fn timestamp_to_iso(timestamp: &Timestamp) -> String {
    let dt = parse_timestamp(timestamp, 0);
    dt.format("%Y-%m-%dT%H:%M:%S%.9fZ").to_string()
}

/// Convert the protobuf timestamp to unixtime
pub fn timestamp_to_unixtime(timestamp: &Timestamp) -> f64 {
    timestamp.seconds as f64 + timestamp.nanos as f64 * 1e-9
}
