use chrono::{DateTime, Utc};
use std::time::Duration;

pub fn format_timestamp(timestamp: f64) -> String {
	let timestamp = timestamp as u64;
	let datetime = std::time::UNIX_EPOCH + Duration::from_secs(timestamp);
	let datetime: chrono::DateTime<chrono::Local> = datetime.into();
	datetime.format("%H:%M:%S").to_string()
}
pub fn format_duration_ms(ms: u64) -> String {
	let total_seconds = ms / 1_000;
	let hours = total_seconds / 3_600;
	let minutes = (total_seconds % 3_600) / 60;
	let seconds = total_seconds % 60;
	let millis = ms % 1_000;

	if hours > 0 {
		format!("{hours}h {minutes:02}m {seconds:02}s")
	} else if minutes > 0 {
		format!("{minutes}m {seconds:02}s")
	} else {
		format!("{seconds}.{millis:03}s")
	}
}

pub fn parse_timestamp(value: Option<String>) -> Option<prost_types::Timestamp> {
	value.and_then(|value| {
		value
			.parse::<chrono::DateTime<chrono::Utc>>()
			.ok()
			.map(|dt| prost_types::Timestamp {
				seconds: dt.timestamp(),
				nanos: dt.timestamp_subsec_nanos() as i32,
			})
	})
}

pub fn timestamp(value: Option<String>) -> Option<prost_types::Timestamp> {
	value.and_then(|value| {
		chrono::DateTime::parse_from_rfc3339(&value)
			.ok()
			.map(|dt| prost_types::Timestamp {
				seconds: dt.timestamp(),
				nanos: dt.timestamp_subsec_nanos() as i32,
			})
	})
}
pub fn to_timestamp(value: Option<&DateTime<Utc>>) -> Option<prost_types::Timestamp> {
	value.map(|dt| prost_types::Timestamp {
		seconds: dt.timestamp(),
		nanos: dt.timestamp_subsec_nanos() as i32,
	})
}
