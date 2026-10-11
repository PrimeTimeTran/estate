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


pub fn append_event(event: &impl Debug) -> anyhow::Result<()> {
	let timestamp = chrono::Local::now().to_rfc3339();
	let event = format!("{event:?}");

	let event = if let Some(start) = event.find("prompt: \"") {
		let value_start = start + "prompt: \"".len();

		if let Some(end) = event[value_start..].find('"') {
			let prompt = &event[value_start..value_start + end];
			let truncated = truncate(prompt, 50);

			format!(
				"{}prompt: \"{}\"{}",
				&event[..value_start - "prompt: \"".len()],
				truncated,
				&event[value_start + end + 1..]
			)
		} else {
			event
		}
	} else {
		event
	};

	let event = event
		.replace('\\', "\\\\")
		.replace('\r', "\\r")
		.replace('\n', "\\n");

	append_line(AGENT_EVENT_LOG, &format!("[{timestamp}] {event}"))
}

pub fn truncate(prompt: &str, max_chars: usize) -> String {
	let prompt = prompt.replace('\n', "\\n").replace('\r', "\\r");

	if prompt.chars().count() <= max_chars {
		return prompt;
	}

	let truncated: String = prompt.chars().take(max_chars).collect();
	format!("{truncated}...")
}

fn write_agent_log(message: impl AsRef<str>) -> anyhow::Result<()> {
	let timestamp = chrono::Local::now().to_rfc3339();
	let message = truncate(message.as_ref(), 50);
	append_line(AGENT_LOG, &format!("[{timestamp}] {message}"))
}
