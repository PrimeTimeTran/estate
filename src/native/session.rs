use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, Serialize, Deserialize)]
pub struct Session {
	pub start: Option<DateTime<Utc>>,
	pub end: Option<DateTime<Utc>>,
	pub events: Vec<serde_json::Value>,
	#[serde(rename = "hotkey.triggers")]
	pub hotkey_triggers: Vec<serde_json::Value>,
	#[serde(rename = "file.participants")]
	pub file_participants: Vec<FileParticipant>,
	#[serde(rename = "index.current")]
	pub index_current: IndexSession,
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

impl Default for Session {
	fn default() -> Self {
		Self {
			start: Some(DateTime::default()),
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
