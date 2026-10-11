use std::{
	fmt::Debug,
	fs::{self, OpenOptions},
	io::Write,
	path::Path,
};

use super::*;

fn append_line(path: &str, message: &str) -> anyhow::Result<()> {
	fs::create_dir_all(LOG_DIR)?;
	let mut file = OpenOptions::new()
		.create(true)
		.append(true)
		.open(Path::new(path))?;
	writeln!(file, "{message}")?;
	file.flush()?;

	Ok(())
}
pub fn write_agent_log(message: impl AsRef<str>) -> anyhow::Result<()> {
	let timestamp = chrono::Local::now().to_rfc3339();
	append_line(AGENT_LOG, &format!("[{timestamp}] {}", message.as_ref()))
}
pub fn append_event(event: &impl Debug) -> anyhow::Result<()> {
	let timestamp = chrono::Local::now().to_rfc3339();
	let event = format!("{event:?}")
		.replace('\\', "\\\\")
		.replace('\r', "\\r")
		.replace('\n', "\\n");
	append_line(AGENT_EVENT_LOG, &format!("[{timestamp}] {event}"))
}
pub fn log_dir() -> &'static str {
	LOG_DIR
}
