use crate::{native::session::Session, prelude::*};

#[derive(Debug, Clone, Deserialize, Hash, Serialize)]
pub struct Event {
	pub id: u64,
	pub kind: EventKind,
	pub source: EventSource,
	pub timestamp: u64,
}
impl Event {
	pub fn new(source: EventSource, kind: EventKind) -> Self {
		tracing::info!("new Event {:?}", source);
		Self {
			id: EVENT_ID.fetch_add(1, Ordering::Relaxed),
			kind,
			source,
			timestamp: crate::util::now(),
		}
	}
	pub fn daemon(kind: EventKind) -> Self {
		Self::new(EventSource::Daemon, kind)
	}
	pub fn cli(kind: EventKind) -> Self {
		Self::new(EventSource::Cli, kind)
	}
	pub fn filesystem(kind: EventKind) -> Self {
		Self::new(EventSource::Filesystem, kind)
	}
	pub fn editor(kind: EventKind) -> Self {
		Self::new(EventSource::Editor, kind)
	}
	pub fn app(kind: EventKind) -> Self {
		Self::new(EventSource::App, kind)
	}
}
#[derive(Debug, Clone, Hash, Deserialize, Serialize)]
pub enum EventKind {
	SessionStart,
	StopSession { session: Session },
	WorkspaceIndexed { duration: u64 },
	DaemonStarted,
	DaemonStopped,
	StatusRequested,
	TaskRequested { request: TaskRequest },
	TaskCreated { task_id: Uuid, kind: TaskKind },
	TaskStarted { task_id: TaskId },
	TaskCompleted { task_id: TaskId },
	TaskFailed { task_id: TaskId, error: String },
	TaskStopped { task_id: TaskId },
	TaskDeleted { task_id: TaskId },
	TasksCleared,
	CommandExecuted { command: String },
	FileCreated { inode: Inode, path: String },
	FileModified { inode: Inode, path: String },
	FileDeleted { inode: Inode, path: String },
	EstateDiscovered { inode: Inode, path: String },
	EstateRemoved { inode: Inode, path: String },
	IndexUpdated { files_changed: u64 },
	CacheInvalidated { reason: String },
}
#[derive(Debug, Clone, Deserialize, Hash, Serialize)]
pub enum EventSource {
	App,
	Cli,
	Daemon,
	Editor,
	Filesystem,
}
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
	StartSesssion,
	StopSession,
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
			TaskKind::StartSesssion => "StartSesssion".into(),
			TaskKind::StopSession => "StopSession".into(),
			TaskKind::LoadMaster => "LoadMaster".into(),
			TaskKind::IndexWorkspace => "IndexWorkspace".into(),
			TaskKind::RebuildIndex => "RebuildIndex".into(),
			TaskKind::GenerateView(_) => "GenerateView".into(),
			TaskKind::SyncBookmarks => "SyncBookmarks".into(),
			TaskKind::BuildEstatePrototype => "Build Estate Prototype".into(),
		}
	}
}
#[derive(Debug)]
pub enum AppEvent {
	Shutdown,
	ModifiersChanged {
		alt: bool,
		command: bool,
		ctrl: bool,
		shift: bool,
	},
	CursorPosition {
		x: f64,
		y: f64,
	},
	TickClock(String),
}
#[derive(Clone, Debug, Serialize, Deserialize, Hash)]
pub struct Inode;
