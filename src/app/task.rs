use strum::IntoStaticStr;

use crate::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Hash, Serialize, IntoStaticStr)]
pub enum TaskKind {
	SessionStart,
	SessionStop,
	LoadMaster,
	IndexWorkspace,
	BuildEstatePrototype,
	GenerateView(String),
	RebuildIndex,
	SyncBookmarks,
}

#[derive(Debug, Clone, Deserialize, Hash, Serialize)]
pub enum TaskRequest {
	Create(TaskKind),
	Run(TaskId),
	Stop(TaskId),
	Delete(TaskId),
}

/// ## [TaskStatus]
///
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
	Pending,
	Running,
	Completed,
	Failed(String),
	Stopped,
	Interrupted,
}

impl TaskKind {
	pub fn name(&self) -> &'static str {
		self.into()
	}
}

pub type TaskId = Uuid;
