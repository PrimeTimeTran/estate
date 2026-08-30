use crate::prelude::*;

pub type TaskId = Uuid;
#[derive(Debug, Clone, Deserialize, Hash, Serialize)]
pub enum TaskRequest {
	Create(TaskKind),
	Run(TaskId),
	Stop(TaskId),
	Delete(TaskId),
}
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Hash, Serialize)]
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
	pub fn name(&self) -> String {
		match self {
			TaskKind::SessionStart => "SessionStart".into(),
			TaskKind::SessionStop => "SessionStop".into(),
			TaskKind::LoadMaster => "LoadMaster".into(),
			TaskKind::IndexWorkspace => "IndexWorkspace".into(),
			TaskKind::RebuildIndex => "RebuildIndex".into(),
			TaskKind::GenerateView(_) => "GenerateView".into(),
			TaskKind::SyncBookmarks => "SyncBookmarks".into(),
			TaskKind::BuildEstatePrototype => "Build Estate Prototype".into(),
		}
	}
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Inode;
