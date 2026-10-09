use std::{
	fmt::Debug,
	fs::{self, OpenOptions},
	io::Write,
	path::Path,
};

const LOG_DIR: &str = "/Users/future/kb/project/crates/estate/log";
const AGENT_LOG: &str = "/Users/future/kb/project/crates/estate/log/agent.log";
const AGENT_EVENT_LOG: &str = "/Users/future/kb/project/crates/estate/log/agent-event.log";

/// Opens an absolute log path, creating its parent directory if necessary.
/// Always appends; never truncates existing content.
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

/// Writes diagnostic messages to the agent log.
pub fn write_agent_log(message: impl AsRef<str>) -> anyhow::Result<()> {
	let timestamp = chrono::Local::now().to_rfc3339();
	append_line(AGENT_LOG, &format!("[{timestamp}] {}", message.as_ref()))
}

/// Appends agent events exclusively to agent-event.log.
/// Each event occupies one line.
pub fn append_agent_event(event: &impl Debug) -> anyhow::Result<()> {
	let timestamp = chrono::Local::now().to_rfc3339();

	// Keep each event on one physical line.
	let event = format!("{event:?}")
		.replace('\\', "\\\\")
		.replace('\r', "\\r")
		.replace('\n', "\\n");

	append_line(AGENT_EVENT_LOG, &format!("[{timestamp}] {event}"))
}

/// Exposes the canonical log directory for inspection and diagnostics.
pub fn log_dir() -> &'static str {
	LOG_DIR
}
