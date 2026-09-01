use strum::IntoStaticStr;

use crate::prelude::*;

pub type TaskId = Uuid;

#[derive(Debug, Clone, Deserialize, Hash, Serialize)]
pub enum TaskRequest {
	Create(TaskKind),
	Run(TaskId),
	Stop(TaskId),
	Delete(TaskId),
}
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
impl TaskKind {
	pub fn name(&self) -> &'static str {
		self.into()
	}
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Inode;
