use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct Session {
	pub start: Option<DateTime<Utc>>,
	pub end: Option<DateTime<Utc>>,

	#[serde(skip)]
	pub events: Vec<serde_json::Value>,

	#[serde(rename = "hotkey.triggers")]
	pub hotkey_triggers: Vec<serde_json::Value>,

	#[serde(rename = "file.participants")]
	pub file_participants: Vec<FileParticipant>,

	#[serde(rename = "index.current")]
	pub index_current: IndexSession,
}

impl Session {
	pub fn end(&mut self) {
		self.end = Some(Utc::now());
	}

	pub fn start_readable(&self) -> Option<String> {
		self
			.start
			.map(|dt| dt.format("%B %-d, %Y at %-I:%M:%S %p UTC").to_string())
	}

	pub fn end_readable(&self) -> Option<String> {
		self
			.end
			.map(|dt| dt.format("%B %-d, %Y at %-I:%M:%S %p UTC").to_string())
	}
	pub fn end_session(&mut self) -> serde_json::Value {
		self.end = Some(Utc::now());

		serde_json::json!({
			"start": self.start,
			"start_readable": self.start.map(|dt| {
				dt.format("%B %-d, %Y at %-I:%M:%S %p UTC").to_string()
			}),
			"end": self.end,
			"end_readable": self.end.map(|dt| {
				dt.format("%B %-d, %Y at %-I:%M:%S %p UTC").to_string()
			}),
			"events": self.events,
			"hotkey.triggers": self.hotkey_triggers,
			"file.participants": self.file_participants,
			"index.current": self.index_current,
		})
	}
}
impl Default for Session {
	fn default() -> Self {
		Self {
			start: Some(Utc::now()),
			end: None,
			events: Vec::new(),
			hotkey_triggers: Vec::new(),
			file_participants: Vec::new(),
			index_current: IndexSession::default(),
		}
	}
}

impl Default for IndexSession {
	fn default() -> Self {
		Self {
			start: None,
			end: None,
			active: false,
			status: "pending".into(),
		}
	}
}

#[derive(Debug, Hash, Clone, Serialize, Deserialize)]
pub struct FileParticipant {
	pub path: String,
	pub action: FileParticipantAction,
}

#[derive(Debug, Hash, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FileParticipantAction {
	OpenAndRead,
	Edited,
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct IndexSession {
	pub start: Option<String>,
	pub end: Option<String>,
	pub active: bool,
	pub status: String,
}
