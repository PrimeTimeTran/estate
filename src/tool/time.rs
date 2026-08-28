pub fn format_timestamp(timestamp: f64) -> String {
	let timestamp = timestamp as u64;
	let datetime = std::time::UNIX_EPOCH + std::time::Duration::from_secs(timestamp);
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
