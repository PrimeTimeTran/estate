use crate::app::*;
use serde::{Serialize, de::DeserializeOwned};
use std::path::{Path, PathBuf};

use crate::app::*;

#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct Session {
	pub start: Option<DateTime<Utc>>,
	pub end: Option<DateTime<Utc>>,

	pub start_readable: Option<String>,
	pub end_readable: Option<String>,

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
		self.end_readable = self.end_readable();
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
	pub fn end_session(&mut self) {
		self.end = Some(Utc::now());
		self.end_readable = self.end_readable();
	}
	pub fn as_json(&mut self) -> serde_json::Value {
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
			"jobs": self.events,
			"hotkey.triggers": self.hotkey_triggers,
			"file.participants": self.file_participants,
			"index.current": self.index_current,
		})
	}
}
impl Default for Session {
	fn default() -> Self {
		let start = Utc::now();
		Self {
			start: Some(start),
			end: None,
			start_readable: Some(start.format("%B %-d, %Y at %-I:%M:%S %p UTC").to_string()),
			end_readable: None,
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
			status: "pending".to_string(),
		}
	}
}
#[derive(Debug, Hash, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct FileParticipant {
	pub path: String,
	pub action: FileParticipantAction,
}

#[derive(Debug, Hash, Clone, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum FileParticipantAction {
	OpenAndRead,
	Edited,
}

#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct IndexSession {
	pub start: Option<DateTime<Utc>>,
	pub end: Option<DateTime<Utc>>,
	pub active: bool,
	pub status: String,
}
